use std::rc::Rc;

use bc_components::{Digest, Compressed, EncryptedMessage, DigestProvider};
use dcbor::{CBOR, CBOREncodable};
use crate::{assertion::Assertion, KnownValue, EnvelopeError};

/// A flexible container for structured data.
///
/// Envelopes are immutable. You create "mutations" by creating new envelopes from old envelopes.
#[derive(Clone, Debug)]
pub enum Envelope {
    /// Represents an envelope with one or more assertions.
    Node { subject: Rc<Envelope>, assertions: Vec<Rc<Envelope>>, digest: Digest },

    /// Represents an envelope with encoded CBOR data.
    Leaf { cbor: CBOR, digest: Digest },

    /// Represents an envelope that wraps another envelope.
    Wrapped { envelope: Rc<Envelope>, digest: Digest },

    /// Represents a value from a namespace of unsigned integers.
    KnownValue { value: KnownValue, digest: Digest },

    /// Represents an assertion.
    ///
    /// An assertion is a predicate-object pair, each of which is itself an ``Envelope``.
    Assertion(Assertion),

    /// Represents an encrypted envelope.
    Encrypted(EncryptedMessage),

    /// Represents a compressed envelope.
    Compressed(Compressed),

    /// Represents an elided envelope.
    Elided(Digest),
}

impl Envelope {
    /// Create an Envelope from a &dyn CBOREncodable
    pub fn from_cbor_encodable(cbor_encodable: &dyn CBOREncodable) -> Rc<Self> {
        let cbor = cbor_encodable.cbor();
        let digest = Digest::from_image(&cbor.cbor_data());
        Rc::new(Envelope::Leaf {
            cbor,
            digest,
        })
    }
}

// Conversion from &CBOREncodable to Envelope
impl<T> From<&T> for Envelope
where
    T: CBOREncodable,
{
    fn from(t: &T) -> Self {
        let cbor = t.cbor();
        let digest = Digest::from_image(&cbor.cbor_data());
        Envelope::Leaf {
            cbor,
            digest,
        }
    }
}

pub fn new_envelope_with_unchecked_assertions(subject: Rc<Envelope>, unchecked_assertions: Vec<Rc<Envelope>>) -> Rc<Envelope> {
    assert!(!unchecked_assertions.is_empty());
    let mut sorted_assertions = unchecked_assertions;
    sorted_assertions.sort_by(|a, b| a.digest_ref().cmp(b.digest_ref()));
    let mut digests = vec![subject.digest()];
    digests.extend(sorted_assertions.iter().map(|a| a.digest()));
    let digest = Digest::from_digests(&digests);
    Rc::new(Envelope::Node { subject: subject, assertions: sorted_assertions, digest })
}

impl Envelope {
    pub fn new_with_assertions(subject: Rc<Envelope>, assertions: Vec<Rc<Envelope>>) -> Result<Rc<Self>, EnvelopeError> {
        if !assertions.iter().all(|a| a.is_subject_assertion() || a.is_subject_obscured()) {
            return Err(EnvelopeError::InvalidFormat);
        }
        Ok(new_envelope_with_unchecked_assertions(subject, assertions))
    }

    pub fn new_with_known_value(value: KnownValue) -> Rc<Self> {
        let digest = value.digest();
        Rc::new(Envelope::KnownValue { value, digest })
    }

    pub fn new_with_assertion(assertion: Assertion) -> Rc<Self> {
        Rc::new(Envelope::Assertion(assertion))
    }

    pub fn new_with_encrypted(encrypted_message: EncryptedMessage) -> Result<Rc<Self>, EnvelopeError> {
        if !encrypted_message.has_digest() {
            return Err(EnvelopeError::MissingDigest);
        }
        Ok(Rc::new(Envelope::Encrypted(encrypted_message)))
    }

    pub fn new_with_compressed(compressed: Compressed) -> Result<Rc<Self>, EnvelopeError> {
        if !compressed.has_digest() {
            return Err(EnvelopeError::MissingDigest);
        }
        Ok(Rc::new(Envelope::Compressed(compressed)))
    }

    pub fn new_elided(digest: Digest) -> Rc<Self> {
        Rc::new(Envelope::Elided(digest))
    }

    pub fn new_leaf<T: CBOREncodable>(cbor: T) -> Rc<Self> {
        let cbor = cbor.cbor();
        let digest = Digest::from_image(&cbor.cbor_data());
        Rc::new(Envelope::Leaf { cbor, digest })
    }

    pub fn new_wrapped(envelope: Rc<Envelope>) -> Rc<Self> {
        let digest = Digest::from_digests(&[envelope.digest()]);
        Rc::new(Envelope::Wrapped { envelope, digest })
    }
}
