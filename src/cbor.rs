use std::rc::Rc;
use bc_components::{tags_registry, Digest, EncryptedMessage, Compressed};
use bc_ur::{UREncodable, URDecodable, URCodable};
use dcbor::{CBORTagged, CBOREncodable, CBORDecodable, CBORError, CBOR, CBORCodable, CBORTaggedEncodable, CBORTaggedDecodable, CBORTaggedCodable, Tag};
use crate::{Envelope, Assertion, KnownValue};

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
    const CBOR_TAG: Tag = tags_registry::ENVELOPE;
}

impl CBOREncodable for Envelope {
    fn cbor(&self) -> CBOR {
        self.tagged_cbor()
    }
}

impl CBORDecodable for Envelope {
    fn from_cbor(cbor: &CBOR) -> Result<Rc<Self>, CBORError> {
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
            Envelope::Leaf { cbor, digest: _ } => CBOR::Tagged(tags_registry::LEAF, Rc::new(cbor.clone())),
            Envelope::Wrapped { envelope, digest: _ } => CBOR::Tagged(tags_registry::WRAPPED_ENVELOPE, Rc::new(envelope.cbor())),
            Envelope::KnownValue { value, digest: _ } => value.tagged_cbor(),
            Envelope::Assertion(assertion) => assertion.tagged_cbor(),
            Envelope::Encrypted(encrypted_message) => encrypted_message.tagged_cbor(),
            Envelope::Compressed(compressed) => compressed.tagged_cbor(),
            Envelope::Elided(digest) => digest.tagged_cbor(),
        }
    }
}

impl CBORTaggedDecodable for Envelope {
    fn from_untagged_cbor(cbor: &CBOR) -> Result<Rc<Self>, CBORError> {
        match cbor {
            CBOR::Tagged(tag, item) => {
                match tag.value() {
                    tags_registry::LEAF_VALUE => {
                        let cbor = item.as_ref().clone();
                        let envelope = Envelope::new_leaf(cbor);
                        Ok(envelope)
                    },
                    tags_registry::KNOWN_VALUE_VALUE => {
                        let known_value = KnownValue::from_untagged_cbor(item)?.as_ref().clone();
                        let envelope = Envelope::new_with_known_value(known_value);
                        Ok(envelope)
                    },
                    tags_registry::WRAPPED_ENVELOPE_VALUE => {
                        let inner_envelope = Envelope::from_untagged_cbor(item)?;
                        let envelope = Envelope::new_wrapped(inner_envelope);
                        Ok(envelope)
                    },
                    tags_registry::ASSERTION_VALUE => {
                        let assertion = Assertion::from_untagged_cbor(item)?.as_ref().clone();
                        let envelope = Envelope::new_with_assertion(assertion);
                        Ok(envelope)
                    },
                    tags_registry::ENCRYPTED_VALUE => {
                        let encrypted = EncryptedMessage::from_untagged_cbor(item)?.as_ref().clone();
                        let envelope = Envelope::new_with_encrypted(encrypted).map_err(|_| CBORError::InvalidFormat)?;
                        Ok(envelope)
                    },
                    tags_registry::COMPRESSED_VALUE => {
                        let compressed = Compressed::from_untagged_cbor(item)?.as_ref().clone();
                        let envelope = Envelope::new_with_compressed(compressed).map_err(|_| CBORError::InvalidFormat)?;
                        Ok(envelope)
                    },
                    tags_registry::DIGEST_VALUE => {
                        let digest = Digest::from_untagged_cbor(item)?.as_ref().clone();
                        let envelope = Envelope::new_elided(digest);
                        Ok(envelope)
                    },
                    _ => Err(CBORError::InvalidFormat),
                }
            }
            CBOR::Array(elements) => {
                if elements.len() < 2 {
                    return Err(CBORError::InvalidFormat);
                }
                let subject = Envelope::from_tagged_cbor(&elements[0])?;
                let assertions = elements[1..].iter().map(|item| Envelope::from_tagged_cbor(item)).collect::<Result<Vec<Rc<Envelope>>, CBORError>>()?;
                Ok(Envelope::new_with_assertions(subject, assertions).map_err(|_| CBORError::InvalidFormat)?)
            }
            _ => Err(CBORError::InvalidFormat),
        }
    }
}

impl CBORTaggedCodable for Envelope { }

impl UREncodable for Envelope { }

impl URDecodable for Envelope { }

impl URCodable for Envelope { }

/*
```swift
public extension Envelope {
    /// Used by test suite to check round-trip encoding of ``Envelope``.
    ///
    /// Not needed in production code.
    @discardableResult
    func checkEncoding(knownTags: KnownTags? = nil) throws -> Envelope {
        do {
            let cbor = taggedCBOR
            let restored = try Envelope(taggedCBOR: cbor)
            guard self.digest == restored.digest else {
                print("=== EXPECTED")
                print(self.format)
                print("=== GOT")
                print(restored.format)
                print("===")
                throw EnvelopeError.invalidFormat
            }
            return self
        } catch {
            print("===")
            print(format())
            print("===")
            print(cbor.diagnostic(annotate: true, knownTags: knownTags))
            print("===")
            throw error
        }
    }
}
```
 */

#[cfg(test)]
impl Envelope {
    /// Used by test suite to check round-trip encoding of `Envelope`.
    ///
    /// Not needed in production code.
    pub fn check_encoding(&self) -> Result<Rc<Self>, CBORError> {
        let cbor = self.tagged_cbor();
        let restored = Envelope::from_tagged_cbor(&cbor);
        let restored = restored.map_err(|_| {
            println!("=== EXPECTED");
            println!("{}", self.format());
            println!("=== GOT");
            println!("{}", cbor.diagnostic());
            println!("===");
            CBORError::InvalidFormat
        })?;
        if self.digest_ref() != restored.digest_ref() {
            println!("=== EXPECTED");
            println!("{}", self.format());
            println!("=== GOT");
            println!("{}", restored.format());
            println!("===");
            return Err(CBORError::InvalidFormat);
        }
        Ok(Rc::new(self.clone()))
    }
}
