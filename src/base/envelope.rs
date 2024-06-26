use anyhow::{bail, Result};
use bc_components::{Digest, DigestProvider};
#[cfg(feature = "encrypt")]
use bc_components::EncryptedMessage;
#[cfg(feature = "compress")]
use bc_components::Compressed;
use dcbor::prelude::*;
use crate::{base::Assertion, EnvelopeEncodable, EnvelopeError};
#[cfg(feature = "known_value")]
use crate::extension::KnownValue;

#[cfg(feature = "multithreaded")]
use std::sync::Arc as RefCounted;

#[cfg(not(feature = "multithreaded"))]
use std::rc::Rc as RefCounted;

/// A flexible container for structured data.
///
/// Envelopes are immutable. You create "mutations" by creating new envelopes from old envelopes.
#[derive(Debug, Clone)]
pub struct Envelope(RefCounted<EnvelopeCase>);

impl Envelope {
    pub fn case(&self) -> &EnvelopeCase {
        &self.0
    }
}

impl From<EnvelopeCase> for Envelope {
    fn from(case: EnvelopeCase) -> Self {
        Self(RefCounted::new(case))
    }
}

impl From<&Envelope> for Envelope {
    fn from(envelope: &Envelope) -> Self {
        envelope.clone()
    }
}

#[derive(Debug)]
pub enum EnvelopeCase {
    /// Represents an envelope with one or more assertions.
    Node { subject: Envelope, assertions: Vec<Envelope>, digest: Digest },

    /// Represents an envelope with encoded CBOR data.
    Leaf { cbor: CBOR, digest: Digest },

    /// Represents an envelope that wraps another envelope.
    Wrapped { envelope: Envelope, digest: Digest },

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

impl Envelope {
    pub fn r#false() -> Self {
        Self::new_leaf(false)
    }

    pub fn r#true() -> Self {
        Self::new_leaf(true)
    }

    pub fn null() -> Self {
        Self::new_leaf(dcbor::Simple::Null)
    }

    pub fn is_false(&self) -> bool {
        self.extract_subject().ok() == Some(false)
    }

    pub fn is_true(&self) -> bool {
        self.extract_subject().ok() == Some(true)
    }

    pub fn is_null(&self) -> bool {
        self.extract_subject().ok() == Some(dcbor::Simple::Null)
    }
}

/// Support for basic envelope creation.
impl Envelope {
    /// Creates an envelope with a `subject`, which
    /// can be any instance that implements ``EnvelopeEncodable``.
    pub fn new(subject: impl EnvelopeEncodable) -> Self {
        subject.into_envelope()
    }

    /// Creates an envelope with a `subject`, which
    /// can be any instance that implements ``EnvelopeEncodable``.
    ///
    /// If `subject` is `None`, returns a null envelope.
    pub fn new_or_null(subject: Option<impl EnvelopeEncodable>) -> Self {
        subject.map_or_else(Self::null, Self::new)
    }

    /// Creates an envelope with a `subject`, which
    /// can be any instance that implements ``EnvelopeEncodable``.
    ///
    /// If `subject` is `None`, returns `None`.
    pub fn new_or_none(subject: Option<impl EnvelopeEncodable>) -> Option<Self> {
        subject.map(Self::new)
    }

    /// Creates an assertion envelope with a `predicate` and `object`,
    /// each of which can be any instance that implements ``EnvelopeEncodable``.
    pub fn new_assertion(predicate: impl EnvelopeEncodable, object: impl EnvelopeEncodable) -> Self {
        Self::new_with_assertion(Assertion::new(predicate, object))
    }
}

/// Internal constructors
impl Envelope {
    pub(crate) fn new_with_unchecked_assertions(subject: Self, unchecked_assertions: Vec<Self>) -> Self {
        assert!(!unchecked_assertions.is_empty());
        let mut sorted_assertions = unchecked_assertions;
        sorted_assertions.sort_by(|a, b| a.digest().cmp(&b.digest()));
        let mut digests = vec![subject.digest().into_owned()];
        digests.extend(sorted_assertions.iter().map(|a| a.digest().into_owned()));
        let digest = Digest::from_digests(&digests);
        (EnvelopeCase::Node { subject, assertions: sorted_assertions, digest }).into()
    }

    pub(crate) fn new_with_assertions(subject: Self, assertions: Vec<Self>) -> Result<Self> {
        if !assertions.iter().all(|a| a.is_subject_assertion() || a.is_subject_obscured()) {
            bail!(EnvelopeError::InvalidFormat);
        }
        Ok(Self::new_with_unchecked_assertions(subject, assertions))
    }

    pub(crate) fn new_with_assertion(assertion: Assertion) -> Self {
        EnvelopeCase::Assertion(assertion).into()
    }

    #[cfg(feature = "known_value")]
    pub(crate) fn new_with_known_value(value: KnownValue) -> Self {
        let digest = value.digest().into_owned();
        (EnvelopeCase::KnownValue { value, digest }).into()
    }

    #[cfg(feature = "encrypt")]
    pub(crate) fn new_with_encrypted(encrypted_message: EncryptedMessage) -> Result<Self> {
        if !encrypted_message.has_digest() {
            bail!(EnvelopeError::MissingDigest);
        }
        Ok(EnvelopeCase::Encrypted(encrypted_message).into())
    }

    #[cfg(feature = "compress")]
    pub(crate) fn new_with_compressed(compressed: Compressed) -> Result<Self> {
        if !compressed.has_digest() {
            bail!(EnvelopeError::MissingDigest);
        }
        Ok(EnvelopeCase::Compressed(compressed).into())
    }

    pub(crate) fn new_elided(digest: Digest) -> Self {
        EnvelopeCase::Elided(digest).into()
    }

    pub(crate) fn new_leaf(value: impl Into<CBOR>) -> Self {
        let cbor: CBOR = value.into();
        let digest = Digest::from_image(cbor.to_cbor_data());
        (EnvelopeCase::Leaf { cbor, digest }).into()
    }

    pub(crate) fn new_wrapped(envelope: Self) -> Self {
        let digest = Digest::from_digests(&[envelope.digest().into_owned()]);
        (EnvelopeCase::Wrapped { envelope, digest }).into()
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
        let e2 = Envelope::try_from(compressed).unwrap();
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
