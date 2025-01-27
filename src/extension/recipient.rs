use crate::{Envelope, EnvelopeError};
#[cfg(feature = "known_value")]
use crate::extension::known_values;

#[cfg(feature = "encrypt")]
use bc_components::Decrypter;

use anyhow::{bail, Result};
use bc_components::{SealedMessage, SymmetricKey, Nonce, Encrypter};
use dcbor::prelude::*;

/// Support for public key encryption.
impl Envelope {
    /// Returns a new envelope with an added `hasRecipient: SealedMessage` assertion.
    ///
    /// The `SealedMessage` contains the `contentKey` encrypted to the recipient's `PublicKeyBase`.
    ///
    /// - Parameters:
    ///   - recipient: The `PublicKeyBase` of the recipient.
    ///   - contentKey: The `SymmetricKey` that was used to encrypt the subject.
    ///
    /// - Returns: The new envelope.
    pub fn add_recipient(&self, recipient: &dyn Encrypter, content_key: &SymmetricKey) -> Self {
        self.add_recipient_opt(recipient, content_key, None::<&Nonce>)
    }

    #[doc(hidden)]
    pub fn add_recipient_opt(&self, recipient: &dyn Encrypter, content_key: &SymmetricKey, test_nonce: Option<&Nonce>) -> Self {
        let assertion = Self::make_has_recipient(recipient, content_key, test_nonce);
        self.add_assertion_envelope(assertion).unwrap()
    }

    /// Returns an array of `SealedMessage`s from all of the envelope's `hasRecipient` assertions.
    ///
    /// - Throws: Throws an exception if any `hasRecipient` assertions do not have a `SealedMessage` as their object.
    pub fn recipients(&self) -> Result<Vec<SealedMessage>> {
        self
            .assertions_with_predicate(known_values::HAS_RECIPIENT)
            .into_iter()
            .filter(|assertion| {
                !assertion.as_object().unwrap().is_obscured()
            })
            .map(|assertion| {
                assertion.as_object().unwrap().extract_subject::<SealedMessage>()
            })
            .collect()
    }

    /// Returns an new envelope with its subject encrypted and a `hasRecipient`
    /// assertion added for each of the `recipients`.
    ///
    /// Generates an ephemeral symmetric key which is used to encrypt the subject and
    /// which is then encrypted to each recipient's public key.
    ///
    /// - Parameter recipients: An array of `PublicKeyBase`, one for each potential
    ///     recipient.
    ///
    /// - Returns: The encrypted envelope.
    ///
    /// - Throws: If the envelope is already encrypted.
    #[cfg(feature = "encrypt")]
    pub fn encrypt_subject_to_recipients(
        &self,
        recipients: &[&dyn Encrypter]
    ) -> Result<Self>
    {
        self.encrypt_subject_to_recipients_opt(recipients, None::<&Nonce>)
    }

    #[cfg(feature = "encrypt")]
    #[doc(hidden)]
    pub fn encrypt_subject_to_recipients_opt(
        &self,
        recipients: &[&dyn Encrypter],
        test_nonce: Option<&Nonce>
    ) -> Result<Self>
    {
        let content_key = SymmetricKey::new();
        let mut e = self.encrypt_subject(&content_key)?;
        for recipient in recipients {
            e = e.add_recipient_opt(*recipient, &content_key, test_nonce);
        }
        Ok(e)
    }

    /// Returns a new envelope with its subject encrypted and a `hasRecipient`
    /// assertion added for the `recipient`.
    ///
    /// Generates an ephemeral symmetric key which is used to encrypt the subject and
    /// which is then encrypted to the recipient's public key.
    ///
    /// - Parameter recipient: The recipient's `PublicKeyBase`.
    ///
    /// - Returns: The encrypted envelope.
    #[cfg(feature = "encrypt")]
    pub fn encrypt_subject_to_recipient(&self, recipient: &dyn Encrypter) -> Result<Self> {
        self.encrypt_subject_to_recipient_opt(recipient, None::<&Nonce>)
    }

    #[cfg(feature = "encrypt")]
    #[doc(hidden)]
    pub fn encrypt_subject_to_recipient_opt(&self, recipient: &dyn Encrypter, test_nonce: Option<&Nonce>) -> Result<Self> {
        self.encrypt_subject_to_recipients_opt(&[recipient], test_nonce)
    }

    #[cfg(feature = "encrypt")]
    fn first_plaintext_in_sealed_messages(sealed_messages: &[SealedMessage], private_key: &dyn Decrypter) -> Result<Vec<u8>> {
        for sealed_message in sealed_messages {
            let a = sealed_message.decrypt(private_key).ok();
            if let Some(plaintext) = a {
                return Ok(plaintext);
            }
        }
        bail!(EnvelopeError::UnknownRecipient)
    }

    /// Returns a new envelope with its subject decrypted using the recipient's
    /// `Decrypter`.
    ///
    /// - Parameter recipient: The recipient's `Decrypter`
    ///
    /// - Returns: The decryptedEnvelope.
    ///
    /// - Throws: If a `SealedMessage` for `recipient` is not found among the
    ///     `hasRecipient` assertions on the envelope.
    #[cfg(feature = "encrypt")]
    pub fn decrypt_subject_to_recipient(&self, recipient: &dyn Decrypter) -> Result<Self> {
        let sealed_messages = self.clone().recipients()?;
        let content_key_data = Self::first_plaintext_in_sealed_messages(&sealed_messages, recipient)?;
        let content_key = SymmetricKey::from_tagged_cbor_data(content_key_data)?;
        self.decrypt_subject(&content_key)
    }

    /// Convenience constructor for a `hasRecipient: SealedMessage` assertion.
    ///
    /// The `SealedMessage` contains the `contentKey` encrypted to the recipient's `PublicKeyBase`.
    ///
    /// - Parameters:
    ///   - recipient: The `PublicKeyBase` of the recipient.
    ///   - contentKey: The `SymmetricKey` that was used to encrypt the subject.
    ///
    /// - Returns: The assertion envelope.
    fn make_has_recipient(recipient: &dyn Encrypter, content_key: &SymmetricKey, test_nonce: Option<&Nonce>) -> Self
    {
        let sealed_message = SealedMessage::new_opt(content_key.to_cbor_data(), recipient, None::<Vec<u8>>, test_nonce);
        Self::new_assertion(known_values::HAS_RECIPIENT, sealed_message)
    }
}

impl Envelope {
    pub fn encrypt_to_recipient(&self, recipient: &dyn Encrypter) -> Envelope {
        self
            .wrap_envelope()
            .encrypt_subject_to_recipient(recipient)
            .unwrap()
    }

    pub fn decrypt_to_recipient(&self, recipient: &dyn Decrypter) -> Result<Envelope> {
        self
            .decrypt_subject_to_recipient(recipient)?
            .unwrap_envelope()
    }
}
