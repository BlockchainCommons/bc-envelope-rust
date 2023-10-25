use std::rc::Rc;
use bc_components::{Digest, DigestProvider};
#[cfg(feature = "encrypt")]
use bc_components::EncryptedMessage;
#[cfg(feature = "compress")]
use bc_components::Compressed;
use dcbor::prelude::*;
use crate::{base::Assertion, EnvelopeError, EnvelopeEncodable};
#[cfg(feature = "known_value")]
use crate::extension::KnownValue;

/// A flexible container for structured data.
///
/// Envelopes are immutable. You create "mutations" by creating new envelopes from old envelopes.
#[derive(Clone, Debug)]
pub enum Envelope {
    /// Represents an envelope with one or more assertions.
    Node { subject: Rc<Self>, assertions: Vec<Rc<Self>>, digest: Digest },

    /// Represents an envelope with encoded CBOR data.
    Leaf { cbor: CBOR, digest: Digest },

    /// Represents an envelope that wraps another envelope.
    Wrapped { envelope: Rc<Self>, digest: Digest },

    /// Represents an assertion.
    ///
    /// An assertion is a predicate-object pair, each of which is itself an ``Envelope``.
    Assertion(Assertion),

    /// Represents an elided envelope.
    Elided(Digest),

    /// Represents a value from a namespace of unsigned integers.
    #[cfg(feature = "known_value")]
    KnownValue { value: KnownValue, digest: Digest },

    /// Represents an encrypted envelope.
    #[cfg(feature = "encrypt")]
    Encrypted(EncryptedMessage),

    /// Represents a compressed envelope.
    #[cfg(feature = "compress")]
    Compressed(Compressed),
}

/// Support for basic envelope creation.
impl Envelope {
    /// Creates an envelope with a `subject`, which
    /// can be any instance that implements ``IntoEnvelope``.
    pub fn new<E>(subject: E) -> Rc<Self>
    where
        E: EnvelopeEncodable,
    {
        subject.envelope()
    }

    /// Creates an assertion envelope with a `predicate` and `object`,
    /// each of which can be any instance that implements ``IntoEnvelope``.
    pub fn new_assertion<P, O>(predicate: P, object: O) -> Rc<Self>
    where
        P: EnvelopeEncodable,
        O: EnvelopeEncodable,
    {
        Rc::new(Self::new_with_assertion(Assertion::new(predicate, object)))
    }
}

/// Internal constructors
impl Envelope {
    pub(crate) fn new_with_unchecked_assertions(subject: Rc<Self>, unchecked_assertions: Vec<Rc<Self>>) -> Self {
        assert!(!unchecked_assertions.is_empty());
        let mut sorted_assertions = unchecked_assertions;
        sorted_assertions.sort_by(|a, b| a.digest().cmp(&b.digest()));
        let mut digests = vec![subject.digest().into_owned()];
        digests.extend(sorted_assertions.iter().map(|a| a.digest().into_owned()));
        let digest = Digest::from_digests(&digests);
        Self::Node { subject, assertions: sorted_assertions, digest }
    }

    pub(crate) fn new_with_assertions(subject: Rc<Self>, assertions: Vec<Rc<Self>>) -> Result<Self, EnvelopeError> {
        if !assertions.iter().all(|a| a.is_subject_assertion() || a.is_subject_obscured()) {
            return Err(EnvelopeError::InvalidFormat);
        }
        Ok(Self::new_with_unchecked_assertions(subject, assertions))
    }

    pub(crate) fn new_with_assertion(assertion: Assertion) -> Self {
        Self::Assertion(assertion)
    }

    #[cfg(feature = "known_value")]
    pub(crate) fn new_with_known_value(value: KnownValue) -> Self {
        let digest = value.digest().into_owned();
        Self::KnownValue { value, digest }
    }

    #[cfg(feature = "encrypt")]
    pub(crate) fn new_with_encrypted(encrypted_message: EncryptedMessage) -> Result<Self, EnvelopeError> {
        if !encrypted_message.has_digest() {
            return Err(EnvelopeError::MissingDigest);
        }
        Ok(Self::Encrypted(encrypted_message))
    }

    #[cfg(feature = "compress")]
    pub(crate) fn new_with_compressed(compressed: Compressed) -> Result<Self, EnvelopeError> {
        if !compressed.has_digest() {
            return Err(EnvelopeError::MissingDigest);
        }
        Ok(Self::Compressed(compressed))
    }

    pub(crate) fn new_elided(digest: Digest) -> Self {
        Self::Elided(digest)
    }

    pub(crate) fn new_leaf<T: CBOREncodable>(cbor: T) -> Self {
        let cbor = cbor.cbor();
        let digest = Digest::from_image(&cbor.cbor_data());
        Self::Leaf { cbor, digest }
    }

    pub(crate) fn new_wrapped(envelope: Rc<Self>) -> Self {
        let digest = Digest::from_digests(&[envelope.digest().into_owned()]);
        Self::Wrapped { envelope, digest }
    }
}

#[cfg(test)]
mod tests {
    use bc_components::DigestProvider;
    #[cfg(feature = "compress")]
    use bc_components::Compressed;
    use crate::{Envelope, Assertion};
    #[cfg(feature = "known_value")]
    use crate::extension::KnownValue;

    #[test]
    fn test_any_envelope() {
        let e1 = Envelope::new_leaf("Hello");
        let e2 = Envelope::new("Hello");
        assert_eq!(e1.format(), e2.format());
        assert_eq!(e1.digest(), e2.digest());
    }

    #[cfg(feature = "known_value")]
    #[test]
    fn test_any_known_value() {
        let known_value = KnownValue::new(100);
        let e1 = Envelope::new_with_known_value(known_value.clone());
        let e2 = Envelope::new(known_value);
        assert_eq!(e1.format(), e2.format());
        assert_eq!(e1.digest(), e2.digest());
    }

    #[test]
    fn test_any_assertion() {
        let assertion = Assertion::new("knows", "Bob");
        let e1 = Envelope::new_with_assertion(assertion.clone());
        let e2 = Envelope::new(assertion);
        assert_eq!(e1.format(), e2.format());
        assert_eq!(e1.digest(), e2.digest());
    }

    #[test]
    fn test_any_encrypted() {
        //todo!()
    }

    #[cfg(feature = "compress")]
    #[test]
    fn test_any_compressed() {
        let data = "Hello".as_bytes();
        let digest = data.digest().into_owned();
        let compressed = Compressed::from_uncompressed_data(data, Some(digest));
        let e1 = Envelope::new_with_compressed(compressed.clone()).unwrap();
        let e2 = Envelope::new(compressed);
        assert_eq!(e1.format(), e2.format());
        assert_eq!(e1.digest(), e2.digest());
    }

    #[test]
    fn test_any_cbor_encodable() {
        let e1 = Envelope::new_leaf(1);
        let e2 = Envelope::new(1);
        assert_eq!(e1.format(), e2.format());
        assert_eq!(e1.digest(), e2.digest());
    }
}
