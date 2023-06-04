use std::{rc::Rc, borrow::Cow};

use bc_components::{SymmetricKey, Nonce, Digest, DigestProvider};
use dcbor::{CBOREncodable, CBORTaggedEncodable};

use crate::{Envelope, Error, envelope::new_envelope_with_unchecked_assertions};

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
    pub fn encrypt_subject(self: Rc<Self>, key: &SymmetricKey) -> Result<Rc<Envelope>, Error> {
        self.encrypt_subject_opt(key, None)
    }

    pub fn encrypt_subject_opt(self: Rc<Self>, key: &SymmetricKey, test_nonce: Option<Nonce>) -> Result<Rc<Envelope>, Error> {
        let result: Rc<Envelope>;
        let original_digest: Cow<Digest>;

        match &*self {
            Envelope::Node { subject, assertions, .. } => {
                if subject.is_encrypted() {
                    return Err(Error::AlreadyEncrypted);
                }
                let encoded_cbor = subject.cbor_data();
                let digest = subject.digest();
                let encrypted_message = key.encrypt_with_digest(encoded_cbor, &digest, test_nonce);
                let encrypted_subject = Self::new_with_encrypted(encrypted_message)?;
                result = new_envelope_with_unchecked_assertions(encrypted_subject, assertions.clone());
                original_digest = digest;
            }
            Envelope::Leaf { cbor, digest } => {
                let encoded_cbor = cbor.cbor_data();
                let encrypted_message = key.encrypt_with_digest(encoded_cbor, digest, test_nonce);
                result = Self::new_with_encrypted(encrypted_message)?;
                original_digest = Cow::Borrowed(digest);
            }
            Envelope::Wrapped { digest, .. } => {
                let encoded_cbor = self.untagged_cbor().cbor_data();
                let encrypted_message = key.encrypt_with_digest(encoded_cbor, digest, test_nonce);
                result = Self::new_with_encrypted(encrypted_message)?;
                original_digest = Cow::Borrowed(digest);
            }
            Envelope::KnownValue { value, digest } => {
                let encoded_cbor = value.cbor_data();
                let encrypted_message = key.encrypt_with_digest(encoded_cbor, digest, test_nonce);
                result = Self::new_with_encrypted(encrypted_message)?;
                original_digest = Cow::Borrowed(digest);
            }
            Envelope::Assertion(assertion) => {
                let digest = assertion.digest();
                let encoded_cbor = assertion.cbor_data();
                let encrypted_message = key.encrypt_with_digest(encoded_cbor, &digest, test_nonce);
                result = Self::new_with_encrypted(encrypted_message)?;
                original_digest = digest;
            }
            Envelope::Encrypted { .. } => {
                return Err(Error::AlreadyEncrypted);
            }
            Envelope::Compressed(compressed) => {
                let digest = compressed.digest();
                let encoded_cbor = compressed.tagged_cbor().cbor_data();
                let encrypted_message = key.encrypt_with_digest(encoded_cbor, &digest, test_nonce);
                result = Self::new_with_encrypted(encrypted_message)?;
                original_digest = digest;
            }
            Envelope::Elided { .. } => {
                return Err(Error::AlreadyElided);
            }
        }
        assert_eq!(result.digest(), original_digest);
        Ok(result)
    }
/*
```swift
    /// Returns a new envelope with its subject decrypted.
    ///
    /// - Parameter key: The `SymmetricKey` to use to decrypt the subject.
    ///
    /// - Returns: The decrypted envelope.
    ///
    /// - Throws: If the envelope is not encrypted or if the `SymmetricKey` is not correct.
    func decryptSubject(with key: SymmetricKey) throws -> Envelope {
        guard case .encrypted(let message) = subject else {
            throw EnvelopeError.notEncrypted
        }

        let encodedCBOR = try key.decrypt(message: message)

        guard let subjectDigest = message.digest else {
            throw EnvelopeError.missingDigest
        }

        let cbor = try CBOR(encodedCBOR)
        let resultSubject = try Envelope(untaggedCBOR: cbor).subject

        guard resultSubject.digest == subjectDigest else {
            throw EnvelopeError.invalidDigest
        }

        switch self {
        case .node(subject: _, assertions: let assertions, digest: let originalDigest):
            let result = Envelope(subject: resultSubject, uncheckedAssertions: assertions)
            guard result.digest == originalDigest else {
                throw EnvelopeError.invalidDigest
            }
            return result
        default:
            return resultSubject
        }
    }
```
 */

    pub fn decrypt_subject(self: Rc<Self>, key: &SymmetricKey) -> Result<Envelope, Error> {
        let a: Rc<Envelope> = self.subject();
        todo!();
        // let message = self.subject().encrypted_message()?.ok_or(Error::NotEncrypted)?;
        // if let Envelope::Encrypted(message) = self {
        //     let subject_digest = message.opt_digest().ok_or(Error::MissingDigest)?;
        //     let encoded_cbor = key.decrypt(&message)?;
        // } else {
        //     Err(Error::NotEncrypted)
        // }
    }
}
