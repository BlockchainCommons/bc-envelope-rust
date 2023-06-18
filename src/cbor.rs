use std::rc::Rc;
use bc_components::{tags, Digest, EncryptedMessage, Compressed};
use bc_ur::{UREncodable, URDecodable, URCodable};
use dcbor::{CBORTagged, CBOREncodable, CBORDecodable, CBOR, CBORCodable, CBORTaggedEncodable, CBORTaggedDecodable, CBORTaggedCodable, Tag};
use crate::{Envelope, Assertion, known_values::KnownValue};

/// Support for CBOR encoding and decoding of ``Envelope``.

/// All envelopes are tagged with the `envelope` tag. Within that tag, each of
/// the seven cases has a unique CBOR signature:
///
/// * `.node` contains a CBOR array, the first element of which is the subject,
/// followed by one or more assertions.
/// * `.leaf` is tagged #6.24, which is the IANA tag for embedded CBOR.
/// * `.wrapped` is tagged with the `wrapped-envelope` tag.
/// * `.knownValue` is tagged with the `known-value` tag.
/// * `.assertion` is tagged with the `assertion` tag.
/// * `.encrypted` is tagged with the `crypto-msg` tag.
/// * `.elided` is tagged with the `crypto-digest` tag.

impl CBORTagged for Envelope {
    const CBOR_TAG: Tag = tags::ENVELOPE;
}

impl CBOREncodable for Envelope {
    fn cbor(&self) -> CBOR {
        self.tagged_cbor()
    }
}

impl CBORDecodable for Envelope {
    fn from_cbor(cbor: &CBOR) -> Result<Self, dcbor::Error> {
        Self::from_tagged_cbor(cbor)
    }
}

impl CBORCodable for Envelope { }

impl CBORTaggedEncodable for Envelope {
    fn untagged_cbor(&self) -> CBOR {
        match self {
            Envelope::Node { subject, assertions, digest: _ } => {
                let mut result = vec![subject.tagged_cbor()];
                for assertion in assertions {
                    result.push(assertion.tagged_cbor());
                }
                CBOR::Array(result)
            }
            Envelope::Leaf { cbor, digest: _ } => CBOR::tagged_value(tags::LEAF, cbor.clone()),
            Envelope::Wrapped { envelope, digest: _ } => CBOR::tagged_value(tags::WRAPPED_ENVELOPE, envelope.untagged_cbor()),
            Envelope::KnownValue { value, digest: _ } => value.tagged_cbor(),
            Envelope::Assertion(assertion) => assertion.tagged_cbor(),
            Envelope::Encrypted(encrypted_message) => encrypted_message.tagged_cbor(),
            Envelope::Compressed(compressed) => compressed.tagged_cbor(),
            Envelope::Elided(digest) => digest.tagged_cbor(),
        }
    }
}

impl CBORTaggedDecodable for Envelope {
    fn from_untagged_cbor(cbor: &CBOR) -> Result<Self, dcbor::Error> {
        match cbor {
            CBOR::Tagged(tag, item) => {
                match tag.value() {
                    tags::LEAF_VALUE => {
                        let cbor = item.as_ref().clone();
                        Ok(Envelope::new_leaf(cbor))
                    },
                    tags::KNOWN_VALUE_VALUE => {
                        let known_value = KnownValue::from_untagged_cbor(item)?;
                        Ok(Envelope::new_with_known_value(known_value))
                    },
                    tags::WRAPPED_ENVELOPE_VALUE => {
                        let inner_envelope = Rc::new(Envelope::from_untagged_cbor(item)?);
                        Ok(Envelope::new_wrapped(inner_envelope))
                    },
                    tags::ASSERTION_VALUE => {
                        let assertion = Assertion::from_untagged_cbor(item)?;
                        Ok(Envelope::new_with_assertion(assertion))
                    },
                    tags::ENVELOPE_VALUE => {
                        let envelope = Envelope::from_untagged_cbor(item)?;
                        Ok(envelope)
                    },
                    tags::ENCRYPTED_VALUE => {
                        let encrypted = EncryptedMessage::from_untagged_cbor(item)?;
                        let envelope = Envelope::new_with_encrypted(encrypted)?;
                        Ok(envelope)
                    },
                    tags::COMPRESSED_VALUE => {
                        let compressed = Compressed::from_untagged_cbor(item)?;
                        let envelope = Envelope::new_with_compressed(compressed)?;
                        Ok(envelope)
                    },
                    tags::DIGEST_VALUE => {
                        let digest = Digest::from_untagged_cbor(item)?;
                        let envelope = Envelope::new_elided(digest);
                        Ok(envelope)
                    },
                    _ => Err(dcbor::Error::InvalidFormat),
                }
            }
            CBOR::Array(elements) => {
                if elements.len() < 2 {
                    return Err(dcbor::Error::InvalidFormat);
                }
                let subject = Rc::new(Envelope::from_tagged_cbor(&elements[0])?);
                // let assertions = elements[1..].iter().map(Envelope::from_tagged_cbor).collect::<Result<Vec<Self>, dcbor::Error>>()?;
                // let assertions: Vec<Rc<Envelope>> = assertions.into_iter().map(Rc::new).collect();

                // The above two lines as a single line:
                let assertions: Vec<Rc<Envelope>> = elements[1..]
                    .iter()
                    .map(Envelope::from_tagged_cbor)
                    .collect::<Result<Vec<Self>, dcbor::Error>>()?
                    .into_iter
                    ().map(Rc::new
                    ).collect();
                Ok(Envelope::new_with_assertions(subject, assertions)?)
            }
            _ => Err(dcbor::Error::InvalidFormat),
        }
    }
}

impl CBORTaggedCodable for Envelope { }

impl UREncodable for Envelope { }

impl URDecodable for Envelope { }

impl URCodable for Envelope { }
