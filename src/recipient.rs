use std::rc::Rc;

use crate::{Envelope, known_values, Error};

use bc_components::{SealedMessage, PublicKeyBase, SymmetricKey, Nonce, PrivateKeyBase};
use dcbor::{CBOREncodable, CBORTaggedDecodable};

impl Envelope {
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

    pub fn add_recipient(self: Rc<Self>, recipient: &PublicKeyBase, content_key: &SymmetricKey) -> Rc<Self> {
        self.add_recipient_opt(recipient, content_key, None, None::<&Nonce>)
    }

    /// Returns a new envelope with an added `hasRecipient: SealedMessage` assertion.
    ///
    /// The `SealedMessage` contains the `contentKey` encrypted to the recipient's `PublicKeyBase`.
    ///
    /// - Parameters:
    ///   - recipient: The `PublicKeyBase` of the recipient.
    ///   - contentKey: The `SymmetricKey` that was used to encrypt the subject.
    ///
    /// - Returns: The new envelope.
    pub fn add_recipient_opt(self: Rc<Self>, recipient: &PublicKeyBase, content_key: &SymmetricKey, test_key_material: Option<&[u8]>, test_nonce: Option<&Nonce>) -> Rc<Self> {
        let assertion = Self::make_has_recipient(recipient, content_key, test_key_material, test_nonce);
        self.add_assertion_envelope(assertion).unwrap()
    }

    /// Returns an array of `SealedMessage`s from all of the envelope's `hasRecipient` assertions.
    ///
    /// - Throws: Throws an exception if any `hasRecipient` assertions do not have a `SealedMessage` as their object.
    pub fn recipients(self: Rc<Self>) -> Result<Vec<Rc<SealedMessage>>, Error> {
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
}

impl Envelope {
    pub fn encrypt_subject_to_recipients(self: Rc<Self>, recipients: &[&PublicKeyBase]) -> Result<Rc<Self>, Error> {
        self.encrypt_subject_to_recipients_opt(recipients, None, None::<&Nonce>)
    }

    /// Returns an new envelope with its subject encrypted and a `hasReceipient`
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
    pub fn encrypt_subject_to_recipients_opt(self: Rc<Self>, recipients: &[&PublicKeyBase], test_key_material: Option<&[u8]>, test_nonce: Option<&Nonce>) -> Result<Rc<Self>, Error> {
        let content_key = SymmetricKey::new();
        let mut e = self.encrypt_subject(&content_key)?;
        for recipient in recipients {
            e = e.add_recipient_opt(recipient, &content_key, test_key_material, test_nonce);
        }
        Ok(e)
    }

    pub fn encrypt_subject_to_recipient(self: Rc<Self>, recipient: &PublicKeyBase) -> Result<Rc<Self>, Error> {
        self.encrypt_subject_to_recipient_opt(recipient, None, None::<&Nonce>)
    }

    pub fn encrypt_subject_to_recipient_opt(self: Rc<Self>, recipient: &PublicKeyBase, test_key_material: Option<&[u8]>, test_nonce: Option<&Nonce>) -> Result<Rc<Self>, Error> {
        self.encrypt_subject_to_recipients_opt(&[recipient], test_key_material, test_nonce)
    }
}

impl Envelope {
    fn first_plaintext_in_sealed_messages(sealed_messages: &[Rc<SealedMessage>], private_keys: &PrivateKeyBase) -> Result<Vec<u8>, Error> {
        for sealed_message in sealed_messages {
            let a = sealed_message.decrypt(private_keys).ok();
            if let Some(plaintext) = a {
                return Ok(plaintext);
            }
        }
        Err(Error::InvalidRecipient)
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
    pub fn decrypt_to_recipient(self: Rc<Self>, recipient: &PrivateKeyBase) -> Result<Rc<Self>, Error> {
        let sealed_messages = self.clone().recipients()?;
        let content_key_data = Self::first_plaintext_in_sealed_messages(&sealed_messages, recipient)?;
        let content_key = SymmetricKey::from_tagged_cbor_data(&content_key_data)?;
        self.decrypt_subject(&content_key)
    }
}
