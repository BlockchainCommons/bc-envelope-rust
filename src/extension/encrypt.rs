use std::borrow::Cow;

use anyhow::{bail, Result};
use bc_components::{SymmetricKey, Nonce, Digest, DigestProvider, tags};
use dcbor::prelude::*;

use crate::{Envelope, EnvelopeError, base::envelope::EnvelopeCase};

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
    pub fn encrypt_subject(&self, key: &SymmetricKey) -> Result<Self> {
        self.encrypt_subject_opt(key, None)
    }

    #[doc(hidden)]
    pub fn encrypt_subject_opt(&self, key: &SymmetricKey, test_nonce: Option<Nonce>) -> Result<Self> {
        let result: Self;
        let original_digest: Cow<'_, Digest>;

        match self.case() {
            EnvelopeCase::Node { subject, assertions, digest: envelope_digest } => {
                if subject.is_encrypted() {
                    bail!(EnvelopeError::AlreadyEncrypted);
                }
                let encoded_cbor = subject.tagged_cbor().to_cbor_data();
                let digest = subject.digest();
                let encrypted_message = key.encrypt_with_digest(encoded_cbor, digest, test_nonce);
                let encrypted_subject = Self::new_with_encrypted(encrypted_message).unwrap();
                result = Self::new_with_unchecked_assertions(encrypted_subject, assertions.clone());
                original_digest = Cow::Borrowed(envelope_digest);
            }
            EnvelopeCase::Leaf { cbor, digest } => {
                let encoded_cbor = CBOR::to_tagged_value(tags::TAG_ENVELOPE, CBOR::to_tagged_value(tags::TAG_LEAF, cbor.clone())).to_cbor_data();
                let encrypted_message = key.encrypt_with_digest(encoded_cbor, digest, test_nonce);
                result = Self::new_with_encrypted(encrypted_message).unwrap();
                original_digest = Cow::Borrowed(digest);
            }
            EnvelopeCase::Wrapped { digest, .. } => {
                let encoded_cbor = self.tagged_cbor().to_cbor_data();
                let encrypted_message = key.encrypt_with_digest(encoded_cbor, digest, test_nonce);
                result = Self::new_with_encrypted(encrypted_message).unwrap();
                original_digest = Cow::Borrowed(digest);
            }
            EnvelopeCase::KnownValue { value, digest } => {
                let encoded_cbor = CBOR::to_tagged_value(tags::TAG_ENVELOPE, value.untagged_cbor()).to_cbor_data();
                let encrypted_message = key.encrypt_with_digest(encoded_cbor, digest, test_nonce);
                result = Self::new_with_encrypted(encrypted_message).unwrap();
                original_digest = Cow::Borrowed(digest);
            }
            EnvelopeCase::Assertion(assertion) => {
                let digest = assertion.digest();
                let encoded_cbor = CBOR::to_tagged_value(tags::TAG_ENVELOPE, assertion.clone()).to_cbor_data();
                let encrypted_message = key.encrypt_with_digest(encoded_cbor, &digest, test_nonce);
                result = Self::new_with_encrypted(encrypted_message).unwrap();
                original_digest = digest;
            }
            EnvelopeCase::Encrypted { .. } => {
                bail!(EnvelopeError::AlreadyEncrypted);
            }
            #[cfg(feature = "compress")]
            EnvelopeCase::Compressed(compressed) => {
                let digest = compressed.digest();
                let encoded_cbor = CBOR::to_tagged_value(tags::TAG_ENVELOPE, compressed.tagged_cbor()).to_cbor_data();
                let encrypted_message = key.encrypt_with_digest(encoded_cbor, &digest, test_nonce);
                result = Self::new_with_encrypted(encrypted_message).unwrap();
                original_digest = digest;
            }
            EnvelopeCase::Elided { .. } => {
                bail!(EnvelopeError::AlreadyElided);
            }
        }
        assert_eq!(result.digest(), original_digest);
        Ok(result)
    }

    /// Returns a new envelope with its subject decrypted.
    pub fn decrypt_subject(&self, key: &SymmetricKey) -> Result<Self> {
        match self.subject().case() {
            EnvelopeCase::Encrypted(message) => {
                let encoded_cbor = key.decrypt(message)?;
                let subject_digest = message.opt_digest().ok_or(EnvelopeError::MissingDigest)?;
                let cbor = CBOR::try_from_data(encoded_cbor)?;
                let result_subject = Self::from_tagged_cbor(cbor)?;
                if *result_subject.digest() != subject_digest {
                    bail!(EnvelopeError::InvalidDigest);
                }
                match self.case() {
                    EnvelopeCase::Node { assertions, digest, .. } => {
                        let result = Self::new_with_unchecked_assertions(result_subject, assertions.clone());
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
