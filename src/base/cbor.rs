use anyhow::bail;
use bc_components::{tags, Digest};
#[cfg(feature = "encrypt")]
use bc_components::EncryptedMessage;
#[cfg(feature = "compress")]
use bc_components::Compressed;
use bc_ur::prelude::*;
use crate::{Envelope, Assertion};
#[cfg(feature = "known_value")]
use crate::extension::KnownValue;

use super::envelope::EnvelopeCase;

/// Support for CBOR encoding and decoding of ``Envelope``.

/// All envelopes are tagged with the `envelope` tag. Within that tag, each of
/// the seven cases has a unique CBOR signature:
///
/// * `.node` contains a CBOR array, the first element of which is the subject,
/// followed by one or more assertions.
/// * `.leaf` is tagged #6.24, which is the IANA tag for embedded CBOR.
/// * `.wrapped` is tagged with the `envelope` tag.
/// * `.assertion` is a single-element map `{predicate: object}`.
/// * `.knownValue` is an unsigned 64-bit integer.
/// * `.encrypted` is tagged with the `crypto-msg` tag.
/// * `.elided` is a byte string of length 32.

impl CBORTagged for Envelope {
    const CBOR_TAG: Tag = tags::ENVELOPE;
}

impl CBOREncodable for Envelope {
    fn cbor(&self) -> CBOR {
        self.untagged_cbor()
    }
}

impl CBORDecodable for Envelope {
    fn from_cbor(cbor: &CBOR) -> anyhow::Result<Self> {
        Self::from_untagged_cbor(cbor)
    }
}

impl CBORCodable for Envelope { }

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
            EnvelopeCase::Leaf { cbor, digest: _ } => CBOR::tagged_value(tags::LEAF, cbor),
            EnvelopeCase::Wrapped { envelope, digest: _ } => envelope.tagged_cbor(),
            EnvelopeCase::Assertion(assertion) => assertion.cbor(),
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
    fn from_untagged_cbor(cbor: &CBOR) -> anyhow::Result<Self> {
        match cbor.case() {
            CBORCase::Tagged(tag, item) => {
                match tag.value() {
                    tags::LEAF_VALUE | tags::ENCODED_CBOR_VALUE => {
                        let cbor = item.as_ref();
                        Ok(Self::new_leaf(cbor))
                    },
                    tags::ENVELOPE_VALUE => {
                        let envelope = Self::from_tagged_cbor(cbor)?;
                        Ok(Self::new_wrapped(envelope))
                    },
                    #[cfg(feature = "encrypt")]
                    tags::ENCRYPTED_VALUE => {
                        let encrypted = EncryptedMessage::from_untagged_cbor(item)?;
                        let envelope = Self::new_with_encrypted(encrypted)?;
                        Ok(envelope)
                    },
                    #[cfg(feature = "compress")]
                    tags::COMPRESSED_VALUE => {
                        let compressed = Compressed::from_untagged_cbor(item)?;
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
                let subject = Self::from_untagged_cbor(&elements[0])?;
                // let assertions = elements[1..].iter().map(Self::from_tagged_cbor).collect::<Result<Vec<Self>, dcbor::Error>>()?;
                // let assertions: Vec<Envelope> = assertions.into_iter().collect();

                // The above two lines as a single line:
                let assertions: Vec<Envelope> = elements[1..]
                    .iter()
                    .map(Self::from_untagged_cbor)
                    .collect::<Result<Vec<Self>, anyhow::Error>>()?
                    .into_iter()
                    .collect();
                Ok(Self::new_with_assertions(subject, assertions)?)
            }
            CBORCase::Map(_) => {
                let assertion = Assertion::from_cbor(cbor)?;
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

impl CBORTaggedCodable for Envelope { }

impl UREncodable for Envelope { }

impl URDecodable for Envelope { }

impl URCodable for Envelope { }

impl From<Envelope> for CBOR {
    fn from(value: Envelope) -> Self {
        value.tagged_cbor()
    }
}
