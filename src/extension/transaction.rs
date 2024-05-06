use bc_components::{PrivateKeyBase, PublicKeyBase, ARID};

use crate::{known_values, Envelope, Function};

/// Transaction Request Construction
impl Envelope {
    /// Creates an envelope with an `ARID` subject, a `body: «function»`
    /// assertion, and a `'senderPublicKey'` assertion.
    ///
    /// Also adds a `'note'` assertion if `note` is not empty, and a
    /// `'date'` assertion if `date` is not `nil`.
    pub fn into_transaction_request_with_metadata(
        self,
        id: impl AsRef<ARID>,
        sender: impl AsRef<PublicKeyBase>,
        note: impl Into<String>,
        date: Option<dcbor::Date>,
    ) -> Self {
        self.into_request_with_metadata(id, note, date)
            .add_assertion(known_values::SENDER_PUBLIC_KEY, sender.as_ref().clone())
    }

    /// Creates an envelope with an `ARID` subject, a `body: «function»`
    /// assertion, and a `'senderPublicKey'` assertion, then wraps and signs
    /// the envelope with the sender's private key.
    ///
    /// Also adds a `'note'` assertion if `note` is not empty, and a
    /// `'date'` assertion if `date` is not `nil`.
    pub fn into_signed_transaction_request_with_metadata(
        self,
        id: impl AsRef<ARID>,
        sender: &PrivateKeyBase,
        note: impl Into<String>,
        date: Option<dcbor::Date>,
    ) -> Self {
        self.into_transaction_request_with_metadata(id, sender.public_key(), note, date)
            .sign(sender)
    }

    /// Creates an envelope with an `ARID` subject, a `body: «function»`
    /// assertion, and a `'senderPublicKey'` assertion, then wraps and signs the
    /// envelope with the sender's private key, and finally wraps and encrypts
    /// the envelope with the recipient's public key.
    ///
    /// Also adds a `'note'` assertion if `note` is not empty, and a
    /// `'date'` assertion if `date` is not `nil`.
    pub fn into_sealed_transaction_request_with_metadata(
        self,
        id: impl AsRef<ARID>,
        sender: &PrivateKeyBase,
        recipient: &PublicKeyBase,
        note: impl Into<String>,
        date: Option<dcbor::Date>,
    ) -> anyhow::Result<Envelope> {
        self.into_transaction_request_with_metadata(id, sender.public_key(), note, date)
            .seal(sender, recipient)
    }

    /// Creates an envelope with an `ARID` subject, a `body: «function»`, and a
    /// `'senderPublicKey'` assertion.
    pub fn into_transaction_request(
        self,
        id: impl AsRef<ARID>,
        sender: impl AsRef<PublicKeyBase>,
    ) -> Self {
        self.into_transaction_request_with_metadata(id, sender, "", None)
    }

    /// Creates an envelope with an `ARID` subject, a `body: «function»`, and a
    /// `'senderPublicKey'` assertion, then wraps and signs the envelope with the
    /// sender's private key.
    pub fn into_signed_transaction_request(
        self,
        id: impl AsRef<ARID>,
        sender: &PrivateKeyBase,
    ) -> Self {
        self.into_signed_transaction_request_with_metadata(id, sender, "", None)
    }

    /// Creates an envelope with an `ARID` subject, a `body: «function»`, and a
    /// `'senderPublicKey'` assertion, then wraps and signs the envelope with the
    /// sender's private key, and finally wraps and encrypts the envelope with the
    /// recipient's public key.
    pub fn into_sealed_transaction_request(
        self,
        id: impl AsRef<ARID>,
        sender: &PrivateKeyBase,
        recipient: &PublicKeyBase,
    ) -> anyhow::Result<Envelope> {
        self.into_sealed_transaction_request_with_metadata(id, sender, recipient, "", None)
    }
}

/// Transaction Request Parsing
impl Envelope {
    /// Parses the transaction request envelope and returns the id, sender,
    /// body, note, and date.
    pub fn from_transaction_request_with_metadata(&self, expected_function: Option<&Function>) -> anyhow::Result<(ARID, PublicKeyBase, Envelope, Function, String, Option<dcbor::Date>)> {
        let (id, body, function, note, date) = self.from_request_with_metadata(expected_function)?;
        let sender: PublicKeyBase = self.extract_object_for_predicate(known_values::SENDER_PUBLIC_KEY)?;
        Ok((id, sender, body, function, note, date))
    }

    /// Parses the signed transaction request envelope and verifies the
    /// signature from the sender, then returns the id, sender, body, note, and
    /// date.
    pub fn from_signed_transaction_request_with_metadata(&self, expected_function: Option<&Function>) -> anyhow::Result<(ARID, PublicKeyBase, Envelope, Function, String, Option<dcbor::Date>)> {
        let inner = self.unwrap_envelope()?;
        let (id, sender, body, function, note, date) = inner.from_transaction_request_with_metadata(expected_function)?;
        self.verify_signature_from(&sender)?;
        Ok((id, sender, body, function, note, date))
    }

    /// Decrypts the sealed transaction request envelope to the recipient,
    /// verifies the signature from the sender, and returns the id, sender,
    /// body, note, and date.
    pub fn from_sealed_transaction_request_with_metadata(&self, expected_function: Option<&Function>, recipient: &PrivateKeyBase) -> anyhow::Result<(ARID, PublicKeyBase, Envelope, Function, String, Option<dcbor::Date>)> {
        let signed = self.decrypt(recipient)?;
        let (id, sender, body, function, note, date) = signed.from_transaction_request_with_metadata(expected_function)?;
        Ok((id, sender, body, function, note, date))
    }

    /// Parses the transaction request envelope and returns the id, sender, and
    /// body.
    pub fn from_transaction_request(&self, expected_function: Option<&Function>) -> anyhow::Result<(ARID, PublicKeyBase, Envelope, Function)> {
        let (id, body, function) = self.from_request(expected_function)?;
        let sender: PublicKeyBase = self.extract_object_for_predicate(known_values::SENDER_PUBLIC_KEY)?;
        Ok((id, sender, body, function))
    }

    /// Parses the signed transaction request envelope and verifies the
    /// signature from the sender, then returns the id, sender, and body.
    pub fn from_signed_transaction_request(&self, expected_function: Option<&Function>) -> anyhow::Result<(ARID, PublicKeyBase, Envelope, Function)> {
        let inner = self.unwrap_envelope()?;
        let (id, sender, body, function) = inner.from_transaction_request(expected_function)?;
        self.verify_signature_from(&sender)?;
        Ok((id, sender, body, function))
    }

    /// Decrypts the sealed transaction request envelope to the recipient,
    /// verifies the signature from the sender, and returns the id, sender, and
    /// body.
    pub fn from_sealed_transaction_request(&self, expected_function: Option<&Function>, recipient: &PrivateKeyBase) -> anyhow::Result<(ARID, PublicKeyBase, Envelope, Function)> {
        let signed = self.decrypt(recipient)?;
        let (id, sender, body, function) = signed.from_signed_transaction_request(expected_function)?;
        Ok((id, sender, body, function))
    }
}
