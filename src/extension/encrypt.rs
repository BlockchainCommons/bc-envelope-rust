use std::borrow::Cow;

use anyhow::{ bail, Result };
use bc_components::{ SymmetricKey, Nonce, Digest, DigestProvider, tags };
use dcbor::prelude::*;

use crate::{ Envelope, Error, base::envelope::EnvelopeCase };

/// Support for encrypting and decrypting envelopes using symmetric encryption.
///
/// This module extends Gordian Envelope with functions for symmetric encryption and decryption
/// using the IETF-ChaCha20-Poly1305 construct. It enables privacy-enhancing operations
/// by allowing envelope elements to be encrypted without changing the envelope's digest,
/// similar to elision.
///
/// The encryption process preserves the envelope's digest tree structure, which means
/// signatures, proofs, and other cryptographic artifacts remain valid even when parts of
/// the envelope are encrypted.
///
/// # Examples
///
/// ```
/// use bc_envelope::prelude::*;
/// use bc_components::SymmetricKey;
///
/// // Create an envelope
/// let envelope = Envelope::new("Hello world");
///
/// // Generate a symmetric key for encryption
/// let key = SymmetricKey::new();
///
/// // Encrypt the envelope's subject
/// let encrypted = envelope.encrypt_subject(&key).unwrap();
///
/// // The encrypted envelope has the same digest as the original
/// assert_eq!(envelope.digest(), encrypted.digest());
///
/// // The subject is now encrypted
/// assert!(encrypted.subject().is_encrypted());
///
/// // Decrypt the envelope
/// let decrypted = encrypted.decrypt_subject(&key).unwrap();
///
/// // The decrypted envelope is equivalent to the original
/// assert!(envelope.is_equivalent_to(&decrypted));
/// ```
///
/// For encrypting the entire envelope including its assertions, you must first wrap the envelope:
///
/// ```
/// use bc_envelope::prelude::*;
/// use bc_components::SymmetricKey;
///
/// // Create an envelope with assertions
/// let envelope = Envelope::new("Alice")
///     .add_assertion("knows", "Bob")
///     .add_assertion("knows", "Carol");
///
/// // Generate a symmetric key
/// let key = SymmetricKey::new();
///
/// // Encrypt the entire envelope (wrapper method does the wrapping for you)
/// let encrypted = envelope.encrypt(&key);
///
/// // Decrypt the entire envelope
/// let decrypted = encrypted.decrypt(&key).unwrap();
///
/// // The decrypted envelope is equivalent to the original
/// assert!(envelope.is_equivalent_to(&decrypted));
/// ```
impl Envelope {
    /// Returns a new envelope with its subject encrypted.
    ///
    /// Encrypts only the subject of the envelope, leaving assertions unencrypted.
    /// To encrypt an entire envelope including its assertions, it must first be wrapped
    /// using the `wrap_envelope()` method, or you can use the `encrypt()` convenience method.
    ///
    /// The encryption uses ChaCha20-Poly1305 and preserves the envelope's digest, allowing
    /// for features like selective disclosure and signature verification to work even on
    /// encrypted envelopes.
    ///
    /// # Parameters
    ///
    /// * `key` - The `SymmetricKey` to use for encryption
    ///
    /// # Returns
    ///
    /// A new envelope with its subject encrypted
    ///
    /// # Errors
    ///
    /// Returns an error if the envelope is already encrypted or elided
    pub fn encrypt_subject(&self, key: &SymmetricKey) -> Result<Self> {
        self.encrypt_subject_opt(key, None)
    }

    #[doc(hidden)]
    /// Internal function for encrypting with an optional test nonce
    pub fn encrypt_subject_opt(
        &self,
        key: &SymmetricKey,
        test_nonce: Option<Nonce>
    ) -> Result<Self> {
        let result: Self;
        let original_digest: Cow<'_, Digest>;

        match self.case() {
            EnvelopeCase::Node { subject, assertions, digest: envelope_digest } => {
                if subject.is_encrypted() {
                    bail!(Error::AlreadyEncrypted);
                }
                let encoded_cbor = subject.tagged_cbor().to_cbor_data();
                let digest = subject.digest();
                let encrypted_message = key.encrypt_with_digest(encoded_cbor, digest, test_nonce);
                let encrypted_subject = Self::new_with_encrypted(encrypted_message).unwrap();
                result = Self::new_with_unchecked_assertions(encrypted_subject, assertions.clone());
                original_digest = Cow::Borrowed(envelope_digest);
            }
            EnvelopeCase::Leaf { cbor, digest } => {
                let encoded_cbor = CBOR::to_tagged_value(
                    tags::TAG_ENVELOPE,
                    CBOR::to_tagged_value(tags::TAG_LEAF, cbor.clone())
                ).to_cbor_data();
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
                let encoded_cbor = CBOR::to_tagged_value(
                    tags::TAG_ENVELOPE,
                    value.untagged_cbor()
                ).to_cbor_data();
                let encrypted_message = key.encrypt_with_digest(encoded_cbor, digest, test_nonce);
                result = Self::new_with_encrypted(encrypted_message).unwrap();
                original_digest = Cow::Borrowed(digest);
            }
            EnvelopeCase::Assertion(assertion) => {
                let digest = assertion.digest();
                let encoded_cbor = CBOR::to_tagged_value(
                    tags::TAG_ENVELOPE,
                    assertion.clone()
                ).to_cbor_data();
                let encrypted_message = key.encrypt_with_digest(encoded_cbor, &digest, test_nonce);
                result = Self::new_with_encrypted(encrypted_message).unwrap();
                original_digest = digest;
            }
            EnvelopeCase::Encrypted { .. } => {
                bail!(Error::AlreadyEncrypted);
            }
            #[cfg(feature = "compress")]
            EnvelopeCase::Compressed(compressed) => {
                let digest = compressed.digest();
                let encoded_cbor = CBOR::to_tagged_value(
                    tags::TAG_ENVELOPE,
                    compressed.tagged_cbor()
                ).to_cbor_data();
                let encrypted_message = key.encrypt_with_digest(encoded_cbor, &digest, test_nonce);
                result = Self::new_with_encrypted(encrypted_message).unwrap();
                original_digest = digest;
            }
            EnvelopeCase::Elided { .. } => {
                bail!(Error::AlreadyElided);
            }
        }
        assert_eq!(result.digest(), original_digest);
        Ok(result)
    }

    /// Returns a new envelope with its subject decrypted.
    ///
    /// Decrypts the subject of an envelope that was previously encrypted using `encrypt_subject()`.
    /// The symmetric key used must be the same one used for encryption.
    ///
    /// # Parameters
    ///
    /// * `key` - The `SymmetricKey` to use for decryption
    ///
    /// # Returns
    ///
    /// A new envelope with its subject decrypted
    ///
    /// # Errors
    ///
    /// * Returns an error if the envelope's subject is not encrypted
    /// * Returns an error if the key is incorrect
    /// * Returns an error if the digest of the decrypted envelope doesn't match the expected digest
    pub fn decrypt_subject(&self, key: &SymmetricKey) -> Result<Self> {
        match self.subject().case() {
            EnvelopeCase::Encrypted(message) => {
                let encoded_cbor = key.decrypt(message)?;
                let subject_digest = message.opt_digest().ok_or(Error::MissingDigest)?;
                let cbor = CBOR::try_from_data(encoded_cbor)?;
                let result_subject = Self::from_tagged_cbor(cbor)?;
                if *result_subject.digest() != subject_digest {
                    bail!(Error::InvalidDigest);
                }
                match self.case() {
                    EnvelopeCase::Node { assertions, digest, .. } => {
                        let result = Self::new_with_unchecked_assertions(
                            result_subject,
                            assertions.clone()
                        );
                        if *result.digest() != *digest {
                            bail!(Error::InvalidDigest);
                        }
                        Ok(result)
                    }
                    _ => Ok(result_subject),
                }
            }
            _ => bail!(Error::NotEncrypted),
        }
    }
}

impl Envelope {
    /// Convenience method to encrypt an entire envelope including its assertions.
    ///
    /// This method wraps the envelope and then encrypts its subject, which has the effect
    /// of encrypting the entire original envelope including all its assertions.
    ///
    /// # Parameters
    ///
    /// * `key` - The `SymmetricKey` to use for encryption
    ///
    /// # Returns
    ///
    /// A new envelope with the entire original envelope encrypted as its subject
    ///
    /// # Examples
    ///
    /// ```
    /// use bc_envelope::prelude::*;
    /// use bc_components::SymmetricKey;
    ///
    /// // Create an envelope with assertions
    /// let envelope = Envelope::new("Alice")
    ///     .add_assertion("knows", "Bob");
    ///
    /// // Generate a symmetric key
    /// let key = SymmetricKey::new();
    ///
    /// // Encrypt the entire envelope
    /// let encrypted = envelope.encrypt(&key);
    /// ```
    pub fn encrypt(&self, key: &SymmetricKey) -> Envelope {
        self.wrap_envelope().encrypt_subject(key).unwrap()
    }

    /// Convenience method to decrypt an entire envelope that was encrypted using the `encrypt()` method.
    ///
    /// This method decrypts the subject and then unwraps the resulting envelope,
    /// returning the original envelope with all its assertions.
    ///
    /// # Parameters
    ///
    /// * `key` - The `SymmetricKey` to use for decryption
    ///
    /// # Returns
    ///
    /// The original decrypted envelope
    ///
    /// # Errors
    ///
    /// * Returns an error if the envelope is not encrypted
    /// * Returns an error if the key is incorrect
    /// * Returns an error if the digest of the decrypted envelope doesn't match the expected digest
    /// * Returns an error if the decrypted envelope cannot be unwrapped
    ///
    /// # Examples
    ///
    /// ```
    /// use bc_envelope::prelude::*;
    /// use bc_components::SymmetricKey;
    ///
    /// // Create an envelope with assertions
    /// let envelope = Envelope::new("Alice")
    ///     .add_assertion("knows", "Bob");
    ///
    /// // Generate a symmetric key
    /// let key = SymmetricKey::new();
    ///
    /// // Encrypt and then decrypt the entire envelope
    /// let encrypted = envelope.encrypt(&key);
    /// let decrypted = encrypted.decrypt(&key).unwrap();
    ///
    /// // The decrypted envelope is equivalent to the original
    /// assert!(envelope.is_equivalent_to(&decrypted));
    /// ```
    pub fn decrypt(&self, key: &SymmetricKey) -> Result<Envelope> {
        self.decrypt_subject(key)?.unwrap_envelope()
    }
}
