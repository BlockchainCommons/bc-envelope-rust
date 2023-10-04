use std::rc::Rc;

use crate::{Envelope, known_values, EnvelopeError};

use bc_components::{SealedMessage, PublicKeyBase, SymmetricKey, Nonce, PrivateKeyBase};
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
    pub fn add_recipient(self: Rc<Self>, recipient: &PublicKeyBase, content_key: &SymmetricKey) -> Rc<Self> {
        self.add_recipient_opt(recipient, content_key, None, None::<&Nonce>)
    }

    #[doc(hidden)]
    pub fn add_recipient_opt(self: Rc<Self>, recipient: &PublicKeyBase, content_key: &SymmetricKey, test_key_material: Option<&[u8]>, test_nonce: Option<&Nonce>) -> Rc<Self> {
        let assertion = Self::make_has_recipient(recipient, content_key, test_key_material, test_nonce);
        self.add_assertion_envelope(assertion).unwrap()
    }

    /// Returns an array of `SealedMessage`s from all of the envelope's `hasRecipient` assertions.
    ///
    /// - Throws: Throws an exception if any `hasRecipient` assertions do not have a `SealedMessage` as their object.
    pub fn recipients(self: Rc<Self>) -> anyhow::Result<Vec<Rc<SealedMessage>>> {
        self
            .assertions_with_predicate(known_values::HAS_RECIPIENT)
            .into_iter()
            .filter(|assertion| {
                !assertion.clone().object().unwrap().is_obscured()
            })
            .map(|assertion| {
                assertion.object().unwrap().extract_subject::<SealedMessage>()
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
    /// recipient.
    ///
    /// - Returns: The encrypted envelope.
    ///
    /// - Throws: If the envelope is already encrypted.
    pub fn encrypt_subject_to_recipients<T>(
        self: Rc<Self>,
        recipients: &[T]
    ) -> Result<Rc<Self>, EnvelopeError>
    where
        T: AsRef<PublicKeyBase>
    {
        self.encrypt_subject_to_recipients_opt(recipients, None, None::<&Nonce>)
    }

    #[doc(hidden)]
    pub fn encrypt_subject_to_recipients_opt<T>(
        self: Rc<Self>,
        recipients: &[T],
        test_key_material: Option<&[u8]>,
        test_nonce: Option<&Nonce>
    ) -> Result<Rc<Self>, EnvelopeError>
    where
        T: AsRef<PublicKeyBase>
    {
        let content_key = SymmetricKey::new();
        let mut e = self.encrypt_subject(&content_key)?;
        for recipient in recipients {
            e = e.add_recipient_opt(recipient.as_ref(), &content_key, test_key_material, test_nonce);
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
    pub fn encrypt_subject_to_recipient(self: Rc<Self>, recipient: &PublicKeyBase) -> Result<Rc<Self>, EnvelopeError> {
        self.encrypt_subject_to_recipient_opt(recipient, None, None::<&Nonce>)
    }

    #[doc(hidden)]
    pub fn encrypt_subject_to_recipient_opt(self: Rc<Self>, recipient: &PublicKeyBase, test_key_material: Option<&[u8]>, test_nonce: Option<&Nonce>) -> Result<Rc<Self>, EnvelopeError> {
        self.encrypt_subject_to_recipients_opt(&[recipient], test_key_material, test_nonce)
    }

    fn first_plaintext_in_sealed_messages(sealed_messages: &[Rc<SealedMessage>], private_keys: &PrivateKeyBase) -> Result<Vec<u8>, EnvelopeError> {
        for sealed_message in sealed_messages {
            let a = sealed_message.decrypt(private_keys).ok();
            if let Some(plaintext) = a {
                return Ok(plaintext);
            }
        }
        Err(EnvelopeError::InvalidRecipient)
    }

    /// Returns a new envelope with its subject decrypted using the recipient's
    /// `PrivateKeyBase`.
    ///
    /// - Parameter recipient: The recipient's `PrivateKeyBase`
    ///
    /// - Returns: The decryptedEnvelope.
    ///
    /// - Throws: If a `SealedMessage` for `recipient` is not found among the
    /// `hasRecipient` assertions on the envelope.
    pub fn decrypt_to_recipient(self: Rc<Self>, recipient: &PrivateKeyBase) -> anyhow::Result<Rc<Self>> {
        let sealed_messages = self.clone().recipients()?;
        let content_key_data = Self::first_plaintext_in_sealed_messages(&sealed_messages, recipient)?;
        let content_key = SymmetricKey::from_tagged_cbor_data(&content_key_data)?;
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
    fn make_has_recipient(recipient: &PublicKeyBase, content_key: &SymmetricKey, test_key_material: Option<&[u8]>, test_nonce: Option<&Nonce>) -> Rc<Self>
    {
        let sealed_message = SealedMessage::new_opt(content_key.cbor_data(), recipient, None, test_key_material, test_nonce);
        Self::new_assertion(known_values::HAS_RECIPIENT, sealed_message)
    }
}
