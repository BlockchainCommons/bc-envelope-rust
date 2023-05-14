use bc_components::{tags, Digest, EncryptedMessage, Compressed};
use bc_ur::{UREncodable, URDecodable, URCodable};
use dcbor::{CBORTagged, CBOREncodable, CBORDecodable, CBORError, CBOR, CBORCodable, CBORTaggedEncodable, CBORTaggedDecodable, CBORTaggedCodable, Tag};

use crate::{Envelope, KnownValue, Assertion};

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
    fn from_cbor(cbor: &CBOR) -> Result<Box<Self>, CBORError> {
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
            Envelope::Leaf { cbor, digest: _ } => CBOR::Tagged(tags::LEAF, Box::new(cbor.clone())),
            Envelope::Wrapped { envelope, digest: _ } => CBOR::Tagged(tags::WRAPPED_ENVELOPE, Box::new(envelope.cbor())),
            Envelope::KnownValue { value, digest: _ } => value.tagged_cbor(),
            Envelope::Assertion(assertion) => assertion.tagged_cbor(),
            Envelope::Encrypted(encrypted_message) => encrypted_message.tagged_cbor(),
            Envelope::Compressed(compressed) => compressed.tagged_cbor(),
            Envelope::Elided(digest) => digest.tagged_cbor(),
        }
    }
}

impl CBORTaggedDecodable for Envelope {
    fn from_untagged_cbor(cbor: &CBOR) -> Result<Box<Self>, CBORError> {
        match cbor {
            CBOR::Tagged(tag, item) => {
                match tag.value() {
                    tags::LEAF_VALUE => Ok(Box::new(Envelope::new_leaf(*item.clone()))),
                    tags::KNOWN_VALUE_VALUE => Ok(Box::new(Envelope::new_with_known_value(*KnownValue::from_untagged_cbor(item)?))),
                    tags::WRAPPED_ENVELOPE_VALUE => Ok(Box::new(Envelope::new_wrapped(*Envelope::from_untagged_cbor(item)?))),
                    tags::ASSERTION_VALUE => Ok(Box::new(Envelope::new_with_assertion(*Assertion::from_untagged_cbor(item)?))),
                    tags::ENCRYPTED_VALUE => Ok(Box::new(Envelope::new_with_encrypted(*EncryptedMessage::from_untagged_cbor(item)?).map_err(|_| CBORError::InvalidFormat)?)),
                    tags::COMPRESSED_VALUE => Ok(Box::new(Envelope::new_with_compressed(*Compressed::from_untagged_cbor(item)?).map_err(|_| CBORError::InvalidFormat)?)),
                    tags::DIGEST_VALUE => Ok(Box::new(Envelope::new_elided(*Digest::from_untagged_cbor(item)?))),
                    _ => Err(CBORError::InvalidFormat),
                }
            }
            CBOR::Array(elements) => {
                if elements.len() < 2 {
                    return Err(CBORError::InvalidFormat);
                }
                let subject = Envelope::from_tagged_cbor(&elements[0])?;
                let assertions = elements[1..].iter().map(|item| Envelope::from_tagged_cbor(item)).collect::<Result<Vec<Box<Envelope>>, CBORError>>()?;
                Ok(Box::new(Envelope::new_with_assertions(subject, assertions).map_err(|_| CBORError::InvalidFormat)?))
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
