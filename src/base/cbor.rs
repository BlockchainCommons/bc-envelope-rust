use anyhow::{bail, Error, Result};
use dcbor::prelude::*;
use bc_components::{tags, Digest};
#[cfg(feature = "encrypt")]
use bc_components::EncryptedMessage;
#[cfg(feature = "compress")]
use bc_components::Compressed;
use crate::{Assertion, Envelope};
#[cfg(feature = "known_value")]
use crate::extension::KnownValue;

use super::envelope::EnvelopeCase;

/// Support for CBOR encoding and decoding of ``Envelope``.
///
/// All envelopes are tagged with the `envelope` tag. Within that tag, each of
/// the seven cases has a unique CBOR signature:
///
/// * `.node` contains a CBOR array, the first element of which is the subject,
///     followed by one or more assertions.
/// * `.leaf` is tagged #6.24, which is the IANA tag for embedded CBOR.
/// * `.wrapped` is tagged with the `envelope` tag.
/// * `.assertion` is a single-element map `{predicate: object}`.
/// * `.knownValue` is an unsigned 64-bit integer.
/// * `.encrypted` is tagged with the `crypto-msg` tag.
/// * `.elided` is a byte string of length 32.
impl CBORTagged for Envelope {
    fn cbor_tags() -> Vec<Tag> {
        tags_for_values(&[tags::TAG_ENVELOPE])
    }
}

impl From<Envelope> for CBOR {
    fn from(value: Envelope) -> Self {
        value.tagged_cbor()
    }
}

impl TryFrom<CBOR> for Envelope {
    type Error = Error;

    fn try_from(value: CBOR) -> Result<Self> {
        Self::from_tagged_cbor(value)
    }
}

impl CBORTaggedEncodable for Envelope {
    fn untagged_cbor(&self) -> CBOR {
        match self.case() {
            EnvelopeCase::Node { subject, assertions, digest: _ } => {
                let mut result = vec![subject.untagged_cbor()];
                for assertion in assertions {
                    result.push(assertion.untagged_cbor());
                }
                CBORCase::Array(result).into()
            }
            EnvelopeCase::Leaf { cbor, digest: _ } => CBOR::to_tagged_value(tags::TAG_LEAF, cbor.clone()),
            EnvelopeCase::Wrapped { envelope, digest: _ } => envelope.tagged_cbor(),
            EnvelopeCase::Assertion(assertion) => assertion.clone().into(),
            EnvelopeCase::Elided(digest) => digest.untagged_cbor(),
            #[cfg(feature = "known_value")]
            EnvelopeCase::KnownValue { value, digest: _ } => value.untagged_cbor(),
            #[cfg(feature = "encrypt")]
            EnvelopeCase::Encrypted(encrypted_message) => encrypted_message.tagged_cbor(),
            #[cfg(feature = "compress")]
            EnvelopeCase::Compressed(compressed) => compressed.tagged_cbor(),
        }
    }
}

impl CBORTaggedDecodable for Envelope {
    fn from_untagged_cbor(cbor: CBOR) -> Result<Self> {
        match cbor.as_case() {
            CBORCase::Tagged(tag, item) => {
                match tag.value() {
                    tags::TAG_LEAF | tags::TAG_ENCODED_CBOR => {
                        Ok(Self::new_leaf(item.clone()))
                    },
                    tags::TAG_ENVELOPE => {
                        let envelope = Envelope::try_from(cbor)?;
                        Ok(Self::new_wrapped(envelope))
                    },
                    #[cfg(feature = "encrypt")]
                    tags::TAG_ENCRYPTED => {
                        let encrypted = EncryptedMessage::from_untagged_cbor(item.clone())?;
                        let envelope = Self::new_with_encrypted(encrypted)?;
                        Ok(envelope)
                    },
                    #[cfg(feature = "compress")]
                    tags::TAG_COMPRESSED => {
                        let compressed = Compressed::from_untagged_cbor(item.clone())?;
                        let envelope = Self::new_with_compressed(compressed)?;
                        Ok(envelope)
                    },
                    _ => bail!("unknown envelope tag: {}", tag.value()),
                }
            }
            CBORCase::ByteString(bytes) => {
                Ok(Self::new_elided(Digest::from_data_ref(bytes)?))
            }
            CBORCase::Array(elements) => {
                if elements.len() < 2 {
                    bail!("node must have at least two elements")
                }
                let subject = Self::from_untagged_cbor(elements[0].clone())?;
                let assertions: Vec<Envelope> = elements[1..]
                    .iter()
                    .cloned()
                    .map(Self::from_untagged_cbor)
                    .collect::<Result<Vec<Self>, Error>>()?;
                Ok(Self::new_with_assertions(subject, assertions)?)
            }
            CBORCase::Map(_) => {
                let assertion = Assertion::try_from(cbor)?;
                Ok(Self::new_with_assertion(assertion))
            }
            #[cfg(feature = "known_value")]
            CBORCase::Unsigned(value) => {
                let known_value = KnownValue::new(*value);
                Ok(Self::new_with_known_value(known_value))
            }
            _ => bail!("invalid envelope"),
        }
    }
}
