//! Public key encryption extension for Gordian Envelope.
//!
//! This module implements public key encryption for Gordian Envelope, allowing
//! encrypted content to be selectively shared with one or more recipients. Each
//! recipient needs their own public/private key pair, and only recipients with
//! the corresponding private key can decrypt the envelope's content.
//!
//! The recipient extension builds on the basic envelope encryption capabilities
//! by adding:
//!
//! - **Multiple Recipients** - A single envelope can be encrypted to multiple
//!   recipients
//! - **Content Key Distribution** - Uses public key cryptography to securely
//!   distribute the symmetric key that encrypts the actual content
//! - **Privacy** - Recipients can decrypt the envelope independently without
//!   revealing their identity or access to other recipients
//!
//! # How It Works
//!
//! The envelope's subject is encrypted using a random symmetric key (the
//! "content key"), and then this content key is encrypted to each recipient's
//! public key using a `SealedMessage`. Each encrypted content key is attached
//! to the envelope with a `hasRecipient` assertion.
//!
//! When recipients want to decrypt the envelope, they use their private key to
//! decrypt the content key from the appropriate `SealedMessage`, and then use
//! that content key to decrypt the envelope's subject.
//!
//! # Basic Usage
//!
//! ```
//! # #[cfg(feature = "recipient")]
//! # {
//! use bc_envelope::prelude::*;
//! use bc_components::{Encrypter, SymmetricKey, PublicKeysProvider};
//!
//! // Create some test keys (in a real application, these would be proper asymmetric keys)
//! let alice_keys = bc_components::PrivateKeyBase::new();
//! let bob_keys = bc_components::PrivateKeyBase::new();
//!
//! // Create an envelope with some content
//! let envelope = Envelope::new("Confidential message");
//!
//! // Encrypt to Bob (generates a random content key internally)
//! let encrypted = envelope
//!     .encrypt_subject_to_recipient(&bob_keys.public_keys())
//!     .unwrap();
//!
//! // Bob can decrypt it with his private key
//! let decrypted = encrypted
//!     .decrypt_subject_to_recipient(&bob_keys)
//!     .unwrap();
//!
//! assert_eq!(decrypted.extract_subject::<String>().unwrap(), "Confidential message");
//!
//! // Alice can't decrypt it because it wasn't encrypted to her
//! assert!(encrypted.clone()
//!     .decrypt_subject_to_recipient(&alice_keys)
//!     .is_err());
//! # }
//! ```
//!
//! # Multiple Recipients
//!
//! You can encrypt an envelope to multiple recipients, and each can
//! independently decrypt it:
//!
//! ```
//! # #[cfg(feature = "recipient")]
//! # {
//! use bc_components::{PublicKeysProvider, SymmetricKey};
//! use bc_envelope::prelude::*;
//!
//! // Create keys for multiple recipients
//! let alice_keys = bc_components::PrivateKeyBase::new();
//! let bob_keys = bc_components::PrivateKeyBase::new();
//! let carol_keys = bc_components::PrivateKeyBase::new();
//!
//! // Create and encrypt the envelope to both Bob and Carol
//! let envelope = Envelope::new("Shared secret");
//! let encrypted = envelope
//!     .encrypt_subject_to_recipients(&[
//!         &bob_keys.public_keys(),
//!         &carol_keys.public_keys(),
//!     ])
//!     .unwrap();
//!
//! // Bob can decrypt it
//! let bob_decrypted = encrypted
//!     .clone()
//!     .decrypt_subject_to_recipient(&bob_keys)
//!     .unwrap();
//!
//! // Carol can decrypt it
//! let carol_decrypted = encrypted
//!     .clone()
//!     .decrypt_subject_to_recipient(&carol_keys)
//!     .unwrap();
//!
//! // Alice can't decrypt it
//! assert!(encrypted.decrypt_subject_to_recipient(&alice_keys).is_err());
//! # }
//! ```
//!
//! # Usage with Custom Keys
//!
//! If you need more control over the encryption process, you can use the
//! lower-level methods to encrypt with a specific content key:
//!
//! ```
//! # #[cfg(feature = "recipient")]
//! # {
//! use bc_components::{PublicKeysProvider, SymmetricKey};
//! use bc_envelope::prelude::*;
//!
//! // Create keys for the recipient
//! let bob_keys = bc_components::PrivateKeyBase::new();
//!
//! // Create the envelope and a specific content key
//! let envelope = Envelope::new("Secret message");
//! let content_key = SymmetricKey::new();
//!
//! // Encrypt the envelope's subject with the content key
//! let encrypted = envelope
//!     .encrypt_subject(&content_key)
//!     .unwrap()
//!     .add_recipient(&bob_keys.public_keys(), &content_key);
//!
//! // Bob can decrypt it
//! let decrypted = encrypted.decrypt_subject_to_recipient(&bob_keys).unwrap();
//! # }
//! ```
//!
//! # Combining with Signatures
//!
//! Recipient encryption works well with envelope signatures, allowing you to
//! create authenticated encrypted messages:
//!
//! ```
//! # #[cfg(all(feature = "recipient", feature = "signature"))]
//! # {
//! use bc_components::PublicKeysProvider;
//! use bc_envelope::prelude::*;
//!
//! // Create keys for sender and recipient
//! let alice_keys = bc_components::PrivateKeyBase::new();
//! let bob_keys = bc_components::PrivateKeyBase::new();
//!
//! // Alice signs and encrypts a message to Bob
//! let envelope = Envelope::new("From Alice to Bob")
//!     .add_signature(&alice_keys)
//!     .encrypt_subject_to_recipient(&bob_keys.public_keys())
//!     .unwrap();
//!
//! // Bob receives it, verifies the signature, and decrypts it
//! let message = envelope
//!     .verify_signature_from(&alice_keys.public_keys())
//!     .unwrap()
//!     .decrypt_subject_to_recipient(&bob_keys)
//!     .unwrap()
//!     .extract_subject::<String>()
//!     .unwrap();
//!
//! assert_eq!(message, "From Alice to Bob");
//! # }
//! ```

use anyhow::{Result, bail};
#[cfg(feature = "encrypt")]
use bc_components::Decrypter;
use bc_components::{Encrypter, Nonce, SealedMessage, SymmetricKey};
use dcbor::prelude::*;
#[cfg(feature = "known_value")]
use known_values;

use crate::{Envelope, Error};

/// Support for public key encryption.
impl Envelope {
    /// Returns a new envelope with an added `hasRecipient: SealedMessage`
    /// assertion.
    ///
    /// This method adds a recipient to an already-encrypted envelope. It
    /// creates a `hasRecipient` assertion containing a `SealedMessage` that
    /// holds the content key encrypted to the recipient's public key.
    ///
    /// # Parameters
    /// * `recipient` - The public keys of the recipient that will be able to
    ///   decrypt the envelope
    /// * `content_key` - The symmetric key that was used to encrypt the
    ///   envelope's subject
    ///
    /// # Returns
    /// A new envelope with the recipient assertion added
    ///
    /// # Example
    ///
    /// ```
    /// # #[cfg(feature = "recipient")]
    /// # {
    /// use bc_components::{PublicKeysProvider, SymmetricKey};
    /// use bc_envelope::prelude::*;
    ///
    /// let bob_keys = bc_components::PrivateKeyBase::new();
    ///
    /// // Create and encrypt the envelope
    /// let content_key = SymmetricKey::new();
    /// let envelope = Envelope::new("Secret message")
    ///     .encrypt_subject(&content_key)
    ///     .unwrap()
    ///     .add_recipient(&bob_keys.public_keys(), &content_key);
    ///
    /// // Format of the envelope shows the hasRecipient assertion
    /// assert!(envelope.format().contains("hasRecipient"));
    /// # }
    /// ```
    pub fn add_recipient(
        &self,
        recipient: &dyn Encrypter,
        content_key: &SymmetricKey,
    ) -> Self {
        self.add_recipient_opt(recipient, content_key, None::<&Nonce>)
    }

    /// Version of `add_recipient` that accepts an optional test nonce for
    /// deterministic testing.
    ///
    /// This is an internal method primarily used for testing. In production
    /// code, use `add_recipient` instead, which will generate a
    /// cryptographically secure random nonce.
    #[doc(hidden)]
    pub fn add_recipient_opt(
        &self,
        recipient: &dyn Encrypter,
        content_key: &SymmetricKey,
        test_nonce: Option<&Nonce>,
    ) -> Self {
        let assertion =
            Self::make_has_recipient(recipient, content_key, test_nonce);
        self.add_assertion_envelope(assertion).unwrap()
    }

    /// Returns all `SealedMessage`s from the envelope's `hasRecipient`
    /// assertions.
    ///
    /// This method extracts all the `SealedMessage` objects attached to the
    /// envelope as `hasRecipient` assertions. Each `SealedMessage` contains
    /// the content key encrypted to a particular recipient's public key.
    ///
    /// # Returns
    /// A vector of `SealedMessage` objects, one for each recipient
    ///
    /// # Errors
    /// Returns an error if any `hasRecipient` assertion does not have a
    /// `SealedMessage` as its object, or if the object is obscured (elided
    /// or encrypted).
    ///
    /// # Example
    ///
    /// ```
    /// # #[cfg(feature = "recipient")]
    /// # {
    /// use bc_components::{PublicKeysProvider, SymmetricKey};
    /// use bc_envelope::prelude::*;
    ///
    /// let bob_keys = bc_components::PrivateKeyBase::new();
    /// let carol_keys = bc_components::PrivateKeyBase::new();
    ///
    /// // Create envelope with multiple recipients
    /// let content_key = SymmetricKey::new();
    /// let envelope = Envelope::new("Secret message")
    ///     .encrypt_subject(&content_key)
    ///     .unwrap()
    ///     .add_recipient(&bob_keys.public_keys(), &content_key)
    ///     .add_recipient(&carol_keys.public_keys(), &content_key);
    ///
    /// // Get all recipient sealed messages
    /// let sealed_messages = envelope.recipients().unwrap();
    /// assert_eq!(sealed_messages.len(), 2);
    /// # }
    /// ```
    pub fn recipients(&self) -> Result<Vec<SealedMessage>> {
        self.assertions_with_predicate(known_values::HAS_RECIPIENT)
            .into_iter()
            .filter(|assertion| !assertion.as_object().unwrap().is_obscured())
            .map(|assertion| {
                assertion
                    .as_object()
                    .unwrap()
                    .extract_subject::<SealedMessage>()
            })
            .collect()
    }

    /// Encrypts the envelope's subject and adds recipient assertions for
    /// multiple recipients.
    ///
    /// This is a convenience method that handles the complete process of:
    /// 1. Generating a random symmetric key (the content key)
    /// 2. Encrypting the envelope's subject with this key
    /// 3. Encrypting the content key to each recipient's public key
    /// 4. Adding a `hasRecipient` assertion for each recipient
    ///
    /// # Parameters
    /// * `recipients` - An array of public keys, one for each potential
    ///   recipient
    ///
    /// # Returns
    /// A new envelope with encrypted subject and recipient assertions
    ///
    /// # Errors
    /// Returns an error if the envelope's subject is already encrypted or
    /// cannot be encrypted for any other reason.
    ///
    /// # Example
    ///
    /// ```
    /// # #[cfg(feature = "encrypt")]
    /// # {
    /// use bc_envelope::prelude::*;
    /// use bc_components::PublicKeysProvider;
    ///
    /// let bob_keys = bc_components::PrivateKeyBase::new();
    /// let carol_keys = bc_components::PrivateKeyBase::new();
    ///
    /// // Create and encrypt the envelope to both Bob and Carol
    /// let envelope = Envelope::new("Shared secret");
    /// let encrypted = envelope.encrypt_subject_to_recipients(
    ///     &[&bob_keys.public_keys(), &carol_keys.public_keys()]
    /// ).unwrap();
    ///
    /// // The envelope is now encrypted
    /// assert_eq!(encrypted.format(), "ENCRYPTED [\n    'hasRecipient': SealedMessage\n    'hasRecipient': SealedMessage\n]");
    /// # }
    /// ```
    #[cfg(feature = "encrypt")]
    pub fn encrypt_subject_to_recipients(
        &self,
        recipients: &[&dyn Encrypter],
    ) -> Result<Self> {
        self.encrypt_subject_to_recipients_opt(recipients, None::<&Nonce>)
    }

    /// Version of `encrypt_subject_to_recipients` that accepts an optional test
    /// nonce.
    ///
    /// This is an internal method primarily used for testing. In production
    /// code, use `encrypt_subject_to_recipients` instead, which will
    /// generate a cryptographically secure random nonce for the content key
    /// encryption.
    #[cfg(feature = "encrypt")]
    #[doc(hidden)]
    pub fn encrypt_subject_to_recipients_opt(
        &self,
        recipients: &[&dyn Encrypter],
        test_nonce: Option<&Nonce>,
    ) -> Result<Self> {
        let content_key = SymmetricKey::new();
        let mut e = self.encrypt_subject(&content_key)?;
        for recipient in recipients {
            e = e.add_recipient_opt(*recipient, &content_key, test_nonce);
        }
        Ok(e)
    }

    /// Encrypts the envelope's subject and adds a recipient assertion for a
    /// single recipient.
    ///
    /// This is a convenience method that handles the complete process of:
    /// 1. Generating a random symmetric key (the content key)
    /// 2. Encrypting the envelope's subject with this key
    /// 3. Encrypting the content key to the recipient's public key
    /// 4. Adding a `hasRecipient` assertion for the recipient
    ///
    /// # Parameters
    /// * `recipient` - The public keys of the recipient who will be able to
    ///   decrypt the envelope
    ///
    /// # Returns
    /// A new envelope with encrypted subject and recipient assertion
    ///
    /// # Errors
    /// Returns an error if the envelope's subject is already encrypted or
    /// cannot be encrypted for any other reason.
    ///
    /// # Example
    ///
    /// ```
    /// # #[cfg(feature = "encrypt")]
    /// # {
    /// use bc_components::PublicKeysProvider;
    /// use bc_envelope::prelude::*;
    ///
    /// let bob_keys = bc_components::PrivateKeyBase::new();
    ///
    /// // Create and encrypt the envelope to Bob
    /// let envelope = Envelope::new("Secret message");
    /// let encrypted = envelope
    ///     .encrypt_subject_to_recipient(&bob_keys.public_keys())
    ///     .unwrap();
    ///
    /// // The envelope is now encrypted with a recipient
    /// assert!(encrypted.format().contains("hasRecipient"));
    /// # }
    /// ```
    #[cfg(feature = "encrypt")]
    pub fn encrypt_subject_to_recipient(
        &self,
        recipient: &dyn Encrypter,
    ) -> Result<Self> {
        self.encrypt_subject_to_recipient_opt(recipient, None::<&Nonce>)
    }

    /// Version of `encrypt_subject_to_recipient` that accepts an optional test
    /// nonce.
    ///
    /// This is an internal method primarily used for testing. In production
    /// code, use `encrypt_subject_to_recipient` instead, which will
    /// generate a cryptographically secure random nonce for the content key
    /// encryption.
    #[cfg(feature = "encrypt")]
    #[doc(hidden)]
    pub fn encrypt_subject_to_recipient_opt(
        &self,
        recipient: &dyn Encrypter,
        test_nonce: Option<&Nonce>,
    ) -> Result<Self> {
        self.encrypt_subject_to_recipients_opt(&[recipient], test_nonce)
    }

    /// Find and decrypt the first sealed message that can be decrypted with the
    /// given private key.
    ///
    /// This internal helper method tries to decrypt each sealed message with
    /// the provided private key, returning the first successful decryption.
    /// It's used during the recipient decryption process to find the
    /// content key that was encrypted for this particular recipient.
    #[cfg(feature = "encrypt")]
    fn first_plaintext_in_sealed_messages(
        sealed_messages: &[SealedMessage],
        private_key: &dyn Decrypter,
    ) -> Result<Vec<u8>> {
        for sealed_message in sealed_messages {
            let a = sealed_message.decrypt(private_key).ok();
            if let Some(plaintext) = a {
                return Ok(plaintext);
            }
        }
        bail!(Error::UnknownRecipient)
    }

    /// Decrypts an envelope's subject using the recipient's private key.
    ///
    /// This method:
    /// 1. Finds and extracts all `SealedMessage` objects from `hasRecipient`
    ///    assertions
    /// 2. Tries to decrypt each one with the provided private key until
    ///    successful
    /// 3. Extracts the content key from the decrypted message
    /// 4. Uses the content key to decrypt the envelope's subject
    ///
    /// # Parameters
    /// * `recipient` - The private key of the recipient trying to decrypt the
    ///   envelope
    ///
    /// # Returns
    /// A new envelope with decrypted subject
    ///
    /// # Errors
    /// Returns an error if:
    /// - No `hasRecipient` assertions containing `SealedMessage` objects are
    ///   found
    /// - None of the sealed messages can be decrypted with the provided private
    ///   key
    /// - The envelope's subject cannot be decrypted with the extracted content
    ///   key
    ///
    /// # Example
    ///
    /// ```
    /// # #[cfg(feature = "encrypt")]
    /// # {
    /// use bc_components::PublicKeysProvider;
    /// use bc_envelope::prelude::*;
    ///
    /// let bob_keys = bc_components::PrivateKeyBase::new();
    ///
    /// // Create and encrypt the envelope to Bob
    /// let envelope = Envelope::new("Secret message")
    ///     .encrypt_subject_to_recipient(&bob_keys.public_keys())
    ///     .unwrap();
    ///
    /// // Bob decrypts it with his private key
    /// let decrypted = envelope.decrypt_subject_to_recipient(&bob_keys).unwrap();
    ///
    /// assert_eq!(
    ///     decrypted.extract_subject::<String>().unwrap(),
    ///     "Secret message"
    /// );
    /// # }
    /// ```
    #[cfg(feature = "encrypt")]
    pub fn decrypt_subject_to_recipient(
        &self,
        recipient: &dyn Decrypter,
    ) -> Result<Self> {
        let sealed_messages = self.clone().recipients()?;
        let content_key_data = Self::first_plaintext_in_sealed_messages(
            &sealed_messages,
            recipient,
        )?;
        let content_key =
            SymmetricKey::from_tagged_cbor_data(content_key_data)?;
        self.decrypt_subject(&content_key)
    }

    /// Creates a `hasRecipient: SealedMessage` assertion envelope.
    ///
    /// This internal helper method creates an assertion envelope with:
    /// - The predicate set to the known value `hasRecipient`
    /// - The object set to a `SealedMessage` containing the content key
    ///   encrypted to the recipient's public key
    ///
    /// # Parameters
    /// * `recipient` - The public keys of the recipient
    /// * `content_key` - The symmetric key that was used to encrypt the subject
    /// * `test_nonce` - Optional nonce for deterministic testing
    ///
    /// # Returns
    /// An assertion envelope containing the sealed message
    fn make_has_recipient(
        recipient: &dyn Encrypter,
        content_key: &SymmetricKey,
        test_nonce: Option<&Nonce>,
    ) -> Self {
        let sealed_message = SealedMessage::new_opt(
            content_key.to_cbor_data(),
            recipient,
            None::<Vec<u8>>,
            test_nonce,
        );
        Self::new_assertion(known_values::HAS_RECIPIENT, sealed_message)
    }
}

/// Convenience methods for recipient-based encryption and decryption.
///
/// These methods provide simplified versions of the recipient encryption and
/// decryption operations with a more intuitive API for common use cases.
#[cfg(feature = "recipient")]
impl Envelope {
    /// Wraps and encrypts an envelope to a single recipient.
    ///
    /// This is a convenience method that:
    /// 1. Wraps the envelope (preserving its assertions in the wrap)
    /// 2. Encrypts the resulting envelope to the recipient
    ///
    /// This method is simpler than calling `wrap()` and then
    /// `encrypt_subject_to_recipient()` separately, and it handles error
    /// unwrapping.
    ///
    /// # Parameters
    /// * `recipient` - The public keys of the recipient who will be able to
    ///   decrypt the envelope
    ///
    /// # Returns
    /// A new envelope that wraps and encrypts the original envelope to the
    /// recipient
    ///
    /// # Example
    ///
    /// ```
    /// # #[cfg(feature = "recipient")]
    /// # {
    /// use bc_components::PublicKeysProvider;
    /// use bc_envelope::prelude::*;
    ///
    /// let bob_keys = bc_components::PrivateKeyBase::new();
    ///
    /// // Create an envelope with some assertions
    /// let envelope = Envelope::new("Alice")
    ///     .add_assertion("knows", "Bob")
    ///     .add_assertion("age", 30);
    ///
    /// // Wrap and encrypt it to Bob in one step
    /// let encrypted = envelope.encrypt_to_recipient(&bob_keys.public_keys());
    ///
    /// // The format shows it's encrypted but doesn't reveal the content
    /// assert_eq!(
    ///     encrypted.format(),
    ///     "ENCRYPTED [\n    'hasRecipient': SealedMessage\n]"
    /// );
    /// # }
    /// ```
    pub fn encrypt_to_recipient(&self, recipient: &dyn Encrypter) -> Envelope {
        self.wrap().encrypt_subject_to_recipient(recipient).unwrap()
    }

    /// Decrypts an envelope that was encrypted to a recipient and unwraps it.
    ///
    /// This is a convenience method that:
    /// 1. Decrypts the envelope using the recipient's private key
    /// 2. Unwraps the resulting envelope to reveal the original content
    ///
    /// This method is simpler than calling `decrypt_subject_to_recipient()` and
    /// then `try_unwrap()` separately.
    ///
    /// # Parameters
    /// * `recipient` - The private key of the recipient trying to decrypt the
    ///   envelope
    ///
    /// # Returns
    /// The original, unwrapped envelope
    ///
    /// # Errors
    /// Returns an error if:
    /// - The envelope cannot be decrypted with the recipient's private key
    /// - The decrypted envelope is not a wrapped envelope
    ///
    /// # Example
    ///
    /// ```
    /// # #[cfg(feature = "recipient")]
    /// # {
    /// use bc_components::PublicKeysProvider;
    /// use bc_envelope::prelude::*;
    ///
    /// let bob_keys = bc_components::PrivateKeyBase::new();
    ///
    /// // Create an envelope with assertions and encrypt it to Bob
    /// let original = Envelope::new("Alice")
    ///     .add_assertion("knows", "Bob")
    ///     .add_assertion("age", 30);
    ///
    /// let encrypted = original.encrypt_to_recipient(&bob_keys.public_keys());
    ///
    /// // Bob decrypts it with his private key and unwraps it
    /// let decrypted = encrypted.decrypt_to_recipient(&bob_keys).unwrap();
    ///
    /// // The decrypted envelope should match the original
    /// assert!(decrypted.is_identical_to(&original));
    /// # }
    /// ```
    pub fn decrypt_to_recipient(
        &self,
        recipient: &dyn Decrypter,
    ) -> Result<Envelope> {
        self.decrypt_subject_to_recipient(recipient)?.try_unwrap()
    }
}
