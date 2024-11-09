use std::collections::HashSet;

use anyhow::{ bail, Error, Result };
use bc_components::{ tags, PrivateKeyBase, PublicKeyBase, Signer, XID };
use dcbor::CBOREncodable;
use bc_ur::prelude::*;

use crate::{
    base::format::{ EnvelopeFormat, EnvelopeFormatItem },
    extension::{ ALLOW, ALLOW_RAW, DENY, DENY_RAW, KEY, KEY_RAW },
    Envelope,
    EnvelopeEncodable,
    FormatContext,
    KnownValue,
};

use super::XIDFunction;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct XIDDocument {
    xid: XID,
    genesis_key: Option<PublicKeyBase>,
    keys: HashSet<PublicKeyBase>,
    allow: HashSet<XIDFunction>,
    deny: HashSet<XIDFunction>,
}

impl XIDDocument {
    pub fn new(genesis_key: PublicKeyBase) -> Self {
        let xid = XID::new(genesis_key.signing_public_key());
        let mut keys = HashSet::new();
        keys.insert(genesis_key.clone());
        let mut allow = HashSet::new();
        allow.insert(XIDFunction::All);
        Self {
            xid,
            genesis_key: Some(genesis_key.clone()),
            keys,
            allow,
            deny: HashSet::new(),
        }
    }

    pub fn from_xid(xid: XID) -> Self {
        Self {
            xid,
            genesis_key: None,
            keys: HashSet::new(),
            allow: HashSet::new(),
            deny: HashSet::new(),
        }
    }

    pub fn xid(&self) -> &XID {
        &self.xid
    }

    pub fn genesis_key(&self) -> Option<&PublicKeyBase> {
        self.genesis_key.as_ref()
    }

    pub fn is_empty(&self) -> bool {
        self.genesis_key.is_none() && self.allow.is_empty() && self.deny.is_empty()
    }

    fn to_unsigned_envelope(&self) -> crate::Envelope {
        let mut envelope = Envelope::new(self.xid.clone());

        // Add an assertion for each key in the set.
        for key in self.keys.clone() {
            envelope = envelope.add_assertion(KEY, key);
        }

        // Add an assertion for each function in the allow set.
        for function in self.allow.clone() {
            envelope = envelope.add_assertion(ALLOW, KnownValue::from(&function));
        }

        // Add an assertion for each function in the deny set.
        for function in self.deny.clone() {
            envelope = envelope.add_assertion(DENY, KnownValue::from(&function));
        }

        envelope
    }

    fn from_unsigned_envelope(envelope: &Envelope) -> Result<Self> {
        let xid: XID = envelope.subject().try_leaf()?.try_into()?;
        let mut genesis_key = None;
        let mut keys = HashSet::new();
        let mut allow = HashSet::new();
        let mut deny = HashSet::new();

        for assertion in envelope.assertions() {
            let predicate = assertion.try_predicate()?;
            let object = assertion.try_object()?;

            if let Some(known_value) = predicate.as_known_value() {
                let object_subject = object.subject();
                match known_value.value() {
                    KEY_RAW => {
                        let public_key_base = PublicKeyBase::try_from(object_subject.try_leaf()?)?;
                        // If this is the key that generated the XID, then it is the genesis key.
                        if xid.validate(public_key_base.signing_public_key()) {
                            genesis_key = Some(public_key_base.clone());
                        }
                        keys.insert(public_key_base);
                    }
                    ALLOW_RAW => {
                        let function = XIDFunction::try_from(object_subject.try_known_value()?)?;
                        allow.insert(function);
                    }
                    DENY_RAW => {
                        let function = XIDFunction::try_from(object_subject.try_known_value()?)?;
                        deny.insert(function);
                    }
                    _ => bail!("Unknown assertion"),
                }
            }
        }

        Ok(Self {
            xid,
            genesis_key,
            keys,
            allow,
            deny,
        })
    }

    pub fn to_signed_envelope(&self, signing_key: &impl Signer) -> Envelope {
        self.to_unsigned_envelope().sign(signing_key)
    }

    pub fn try_from_signed_envelope(signed_envelope: &Envelope) -> Result<Self> {
        // Unwrap the envelope and construct a provisional XIDDocument.
        let xid_document = XIDDocument::try_from(&signed_envelope.unwrap_envelope()?)?;
        // Extract the genesis key from the provisional XIDDocument, throwing an error if it is missing.
        let genesis_key = xid_document.genesis_key().ok_or_else(|| Error::msg("Missing genesis key"))?;
        // Verify the signature on the envelope using the genesis key.
        signed_envelope.verify(genesis_key)?;
        // Extract the XID from the provisional XIDDocument.
        let xid = xid_document.xid();
        // Verify that the genesis key is the one that generated the XID.
        if xid.validate(genesis_key.signing_public_key()) {
            // If the genesis key is valid return the XIDDocument, now verified.
            Ok(xid_document)
        } else {
            bail!("Invalid XID")
        }
    }
}

impl From<XIDDocument> for XID {
    fn from(doc: XIDDocument) -> Self {
        doc.xid
    }
}

impl From<XID> for XIDDocument {
    fn from(xid: XID) -> Self {
        XIDDocument::from_xid(xid)
    }
}

impl From<&XID> for XIDDocument {
    fn from(xid: &XID) -> Self {
        XIDDocument::from_xid(xid.clone())
    }
}

impl From<PublicKeyBase> for XIDDocument {
    fn from(genesis_key: PublicKeyBase) -> Self {
        XIDDocument::new(genesis_key)
    }
}

impl From<&PrivateKeyBase> for XIDDocument {
    fn from(genesis_key: &PrivateKeyBase) -> Self {
        XIDDocument::new(genesis_key.schnorr_public_key_base())
    }
}

impl EnvelopeEncodable for XID {
    fn into_envelope(self) -> crate::Envelope {
        Envelope::new(self.to_cbor())
    }
}

impl EnvelopeEncodable for XIDDocument {
    fn into_envelope(self) -> crate::Envelope {
        self.to_unsigned_envelope()
    }
}

impl TryFrom<&Envelope> for XIDDocument {
    type Error = Error;

    fn try_from(envelope: &Envelope) -> Result<Self> {
        Self::from_unsigned_envelope(envelope)
    }
}

impl TryFrom<Envelope> for XIDDocument {
    type Error = Error;

    fn try_from(envelope: Envelope) -> Result<Self> {
        XIDDocument::try_from(&envelope)
    }
}

impl EnvelopeFormat for XID {
    fn format_item(&self, _context: &FormatContext) -> EnvelopeFormatItem {
        EnvelopeFormatItem::Item(hex::encode(self.data()))
    }
}

impl CBORTagged for XIDDocument {
    fn cbor_tags() -> Vec<Tag> {
        vec![tags::XID]
    }
}

impl From<XIDDocument> for CBOR {
    fn from(value: XIDDocument) -> Self {
        value.tagged_cbor()
    }
}

impl CBORTaggedEncodable for XIDDocument {
    fn untagged_cbor(&self) -> CBOR {
        if self.is_empty() {
            return self.xid.untagged_cbor();
        }
        self.to_envelope().to_cbor()
    }
}

impl TryFrom<CBOR> for XIDDocument {
    type Error = Error;

    fn try_from(cbor: CBOR) -> Result<Self> {
        Self::from_tagged_cbor(cbor)
    }
}

impl CBORTaggedDecodable for XIDDocument {
    fn from_untagged_cbor(cbor: CBOR) -> Result<Self> {
        if let Some(byte_string) = cbor.clone().into_byte_string() {
            let xid = XID::from_data_ref(byte_string)?;
            return Ok(Self::from_xid(xid));
        }

        Envelope::try_from(cbor)?.try_into()
    }
}
