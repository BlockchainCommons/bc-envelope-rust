use std::{rc::Rc, borrow::Cow};

use anyhow::bail;
use bc_components::{SymmetricKey, Nonce, Digest, DigestProvider, tags::{LEAF, ENVELOPE}};
use dcbor::prelude::*;

use crate::{Envelope, EnvelopeError};

/// Support for encrypting and decrypting envelopes.
impl Envelope {
    /// Returns a new envelope with its subject encrypted.
    ///
    /// Assertions are not encrypted. To encrypt an entire envelope including its
    /// assertions it must first be wrapped using the ``wrap()`` method.
    ///
    /// - Parameters:
    ///   - key: The `SymmetricKey` to be used to encrypt the subject.
    ///
    /// - Returns: The encrypted envelope.
    ///
    /// - Throws: If the envelope is already encrypted.
    pub fn encrypt_subject(self: Rc<Self>, key: &SymmetricKey) -> Result<Rc<Self>, EnvelopeError> {
        self.encrypt_subject_opt(key, None)
    }

    #[doc(hidden)]
    pub fn encrypt_subject_opt(self: Rc<Self>, key: &SymmetricKey, test_nonce: Option<Nonce>) -> Result<Rc<Self>, EnvelopeError> {
        let result: Rc<Self>;
        let original_digest: Cow<'_, Digest>;

        match &*self {
            Self::Node { subject, assertions, digest: envelope_digest } => {
                if subject.is_encrypted() {
                    return Err(EnvelopeError::AlreadyEncrypted);
                }
                let encoded_cbor = subject.tagged_cbor().cbor_data();
                let digest = subject.digest();
                let encrypted_message = key.encrypt_with_digest(encoded_cbor, digest, test_nonce);
                let encrypted_subject = Rc::new(Self::new_with_encrypted(encrypted_message)?);
                result = Rc::new(Self::new_with_unchecked_assertions(encrypted_subject, assertions.clone()));
                original_digest = Cow::Borrowed(envelope_digest);
            }
            Self::Leaf { cbor, digest } => {
                let encoded_cbor = CBOR::tagged_value(ENVELOPE, CBOR::tagged_value(LEAF, cbor.clone())).cbor_data();
                let encrypted_message = key.encrypt_with_digest(encoded_cbor, digest, test_nonce);
                result = Rc::new(Self::new_with_encrypted(encrypted_message)?);
                original_digest = Cow::Borrowed(digest);
            }
            Self::Wrapped { digest, .. } => {
                let encoded_cbor = self.tagged_cbor().cbor_data();
                let encrypted_message = key.encrypt_with_digest(encoded_cbor, digest, test_nonce);
                result = Rc::new(Self::new_with_encrypted(encrypted_message)?);
                original_digest = Cow::Borrowed(digest);
            }
            Self::KnownValue { value, digest } => {
                let encoded_cbor = CBOR::tagged_value(ENVELOPE, value.untagged_cbor()).cbor_data();
                let encrypted_message = key.encrypt_with_digest(encoded_cbor, digest, test_nonce);
                result = Rc::new(Self::new_with_encrypted(encrypted_message)?);
                original_digest = Cow::Borrowed(digest);
            }
            Self::Assertion(assertion) => {
                let digest = assertion.digest();
                let encoded_cbor = CBOR::tagged_value(ENVELOPE, assertion.cbor()).cbor_data();
                let encrypted_message = key.encrypt_with_digest(encoded_cbor, &digest, test_nonce);
                result = Rc::new(Self::new_with_encrypted(encrypted_message)?);
                original_digest = digest;
            }
            Self::Encrypted { .. } => {
                return Err(EnvelopeError::AlreadyEncrypted);
            }
            #[cfg(feature = "compress")]
            Self::Compressed(compressed) => {
                let digest = compressed.digest();
                let encoded_cbor = CBOR::tagged_value(ENVELOPE, compressed.tagged_cbor()).cbor_data();
                let encrypted_message = key.encrypt_with_digest(encoded_cbor, &digest, test_nonce);
                result = Rc::new(Self::new_with_encrypted(encrypted_message)?);
                original_digest = digest;
            }
            Self::Elided { .. } => {
                return Err(EnvelopeError::AlreadyElided);
            }
        }
        assert_eq!(result.digest(), original_digest);
        Ok(result)
    }

    /// Returns a new envelope with its subject decrypted.
    pub fn decrypt_subject(self: Rc<Self>, key: &SymmetricKey) -> anyhow::Result<Rc<Self>> {
        match &*self.clone().subject() {
            Self::Encrypted(message) => {
                let encoded_cbor = key.decrypt(message)?;
                let subject_digest = message.opt_digest().ok_or(EnvelopeError::MissingDigest)?;
                let cbor = CBOR::from_data(&encoded_cbor)?;
                let result_subject = Rc::new(Self::from_tagged_cbor(&cbor)?);
                if *result_subject.digest() != subject_digest {
                    bail!(EnvelopeError::InvalidDigest);
                }
                match &*self {
                    Self::Node { assertions, digest, .. } => {
                        let result = Rc::new(Self::new_with_unchecked_assertions(result_subject, assertions.clone()));
                        if *result.digest() != *digest {
                            bail!(EnvelopeError::InvalidDigest);
                        }
                        Ok(result)
                    }
                    _ => Ok(result_subject)
                }
            },
            _ => bail!(EnvelopeError::NotEncrypted)
        }
    }
}
