#[cfg(not(feature = "multithreaded"))]
use std::rc::Rc as RefCounted;
#[cfg(feature = "multithreaded")]
use std::sync::Arc as RefCounted;

use anyhow::{Result, bail};
#[cfg(feature = "compress")]
use bc_components::Compressed;
#[cfg(feature = "encrypt")]
use bc_components::EncryptedMessage;
use bc_components::{Digest, DigestProvider};
use dcbor::prelude::*;

#[cfg(feature = "known_value")]
use crate::extension::KnownValue;
use crate::{EnvelopeEncodable, Error, base::Assertion};

/// A flexible container for structured data with built-in integrity
/// verification.
///
/// Gordian Envelope is the primary data structure of this crate. It provides a
/// way to encapsulate and organize data with cryptographic integrity, privacy
/// features, and selective disclosure capabilities.
///
/// Key characteristics of envelopes:
///
/// - **Immutability**: Envelopes are immutable. Operations that appear to
///   "modify" an envelope actually create a new envelope. This immutability is
///   fundamental to maintaining the integrity of the envelope's digest tree.
///
/// - **Semantic Structure**: Envelopes can represent various semantic
///   relationships through subjects, predicates, and objects (similar to RDF
///   triples).
///
/// - **Digest Tree**: Each envelope maintains a Merkle-like digest tree that
///   ensures the integrity of its contents and enables verification of
///   individual parts.
///
/// - **Privacy Features**: Envelopes support selective disclosure through
///   elision, encryption, and compression of specific parts, while maintaining
///   the overall integrity of the structure.
///
/// - **Deterministic Representation**: Envelopes use deterministic CBOR
///   encoding to ensure consistent serialization across platforms.
///
/// The Gordian Envelope specification is defined in an IETF Internet Draft, and
/// this implementation closely follows that specification.
///
/// # Example
///
/// ```
/// use bc_envelope::prelude::*;
///
/// // Create an envelope representing a person
/// let person = Envelope::new("person")
///     .add_assertion("name", "Alice")
///     .add_assertion("age", 30)
///     .add_assertion("email", "alice@example.com");
///
/// // Create a partially redacted version by eliding the email
/// let redacted = person.elide_removing_target(
///     &person.assertion_with_predicate("email").unwrap(),
/// );
///
/// // The digest of both envelopes remains the same
/// assert_eq!(person.digest(), redacted.digest());
/// ```
#[derive(Debug, Clone)]
pub struct Envelope(RefCounted<EnvelopeCase>);

impl Envelope {
    /// Returns a reference to the underlying envelope case.
    ///
    /// The `EnvelopeCase` enum represents the specific structural variant of
    /// this envelope. This method provides access to that underlying
    /// variant for operations that need to differentiate between the
    /// different envelope types.
    ///
    /// # Returns
    ///
    /// A reference to the `EnvelopeCase` that defines this envelope's
    /// structure.
    pub fn case(&self) -> &EnvelopeCase { &self.0 }
}

/// Conversion from `EnvelopeCase` to `Envelope`.
///
/// This allows creating an envelope directly from an envelope case variant.
impl From<EnvelopeCase> for Envelope {
    fn from(case: EnvelopeCase) -> Self { Self(RefCounted::new(case)) }
}

/// Conversion from `&Envelope` to `Envelope`.
///
/// This creates a clone of the envelope. Since envelopes use reference
/// counting, this is a relatively inexpensive operation.
impl From<&Envelope> for Envelope {
    fn from(envelope: &Envelope) -> Self { envelope.clone() }
}

/// The core structural variants of a Gordian Envelope.
///
/// Each variant of this enum represents a different structural form that an
/// envelope can take, as defined in the Gordian Envelope IETF Internet Draft.
/// The different cases provide different capabilities and serve different
/// purposes in the envelope ecosystem.
///
/// The `EnvelopeCase` is the internal representation of an envelope's
/// structure. While each case has unique properties, they all maintain a digest
/// that ensures the integrity of the envelope.
#[derive(Debug)]
pub enum EnvelopeCase {
    /// Represents an envelope with a subject and one or more assertions.
    ///
    /// A node is the fundamental structural component for building complex data
    /// structures with Gordian Envelope. It consists of a subject and a set of
    /// assertions about that subject.
    ///
    /// The digest of a node is derived from the digests of its subject and all
    /// assertions, ensuring that any change to the node or its components would
    /// result in a different digest.
    Node {
        /// The subject of the node
        subject: Envelope,
        /// The assertions attached to the subject
        assertions: Vec<Envelope>,
        /// The digest of the node
        digest: Digest,
    },

    /// Represents an envelope containing a primitive CBOR value.
    ///
    /// A leaf is the simplest form of envelope, containing a single CBOR value
    /// such as a string, number, or boolean. Leaves are the terminal nodes in
    /// the envelope structure.
    ///
    /// The digest of a leaf is derived directly from its CBOR representation.
    Leaf {
        /// The CBOR value contained in the leaf
        cbor: CBOR,
        /// The digest of the leaf
        digest: Digest,
    },

    /// Represents an envelope that wraps another envelope.
    ///
    /// Wrapping provides a way to encapsulate an entire envelope as the subject
    /// of another envelope, enabling hierarchical structures and metadata
    /// attachment.
    ///
    /// The digest of a wrapped envelope is derived from the digest of the
    /// envelope it wraps.
    Wrapped {
        /// The envelope being wrapped
        envelope: Envelope,
        /// The digest of the wrapped envelope
        digest: Digest,
    },

    /// Represents a predicate-object assertion.
    ///
    /// An assertion is a statement about a subject, consisting of a predicate
    /// (what is being asserted) and an object (the value of the assertion).
    /// Assertions are attached to envelope subjects to form semantic
    /// statements.
    ///
    /// For example, in the statement "Alice hasEmail alice@example.com":
    /// - The subject is "Alice"
    /// - The predicate is "hasEmail"
    /// - The object is "alice@example.com"
    Assertion(Assertion),

    /// Represents an envelope that has been elided, leaving only its digest.
    ///
    /// Elision is a key privacy feature of Gordian Envelope, allowing parts of
    /// an envelope to be removed while maintaining the integrity of the digest
    /// tree. This enables selective disclosure of information.
    Elided(Digest),

    /// Represents a value from a namespace of unsigned integers used for
    /// ontological concepts.
    ///
    /// Known Values are 64-bit unsigned integers used to represent stand-alone
    /// ontological concepts like relationships (`isA`, `containedIn`),
    /// classes (`Seed`, `PrivateKey`), or enumerated values (`MainNet`,
    /// `OK`). They provide a compact, deterministic alternative to URIs for
    /// representing common predicates and values.
    ///
    /// Using Known Values instead of strings for common predicates offers
    /// several advantages:
    /// - More compact representation (integers vs. long strings/URIs)
    /// - Standardized semantics across implementations
    /// - Deterministic encoding for cryptographic operations
    /// - Resistance to manipulation attacks that target string representations
    ///
    /// Known Values are displayed with single quotes, e.g., `'isA'` or by their
    /// numeric value like `'1'` (when no name is assigned).
    ///
    /// This variant is only available when the `known_value` feature is
    /// enabled.
    #[cfg(feature = "known_value")]
    KnownValue {
        /// The Known Value instance containing the integer value and optional
        /// name
        value: KnownValue,
        /// The digest of the known value
        digest: Digest,
    },

    /// Represents an envelope that has been encrypted.
    ///
    /// Encryption is a privacy feature that allows parts of an envelope to be
    /// encrypted while maintaining the integrity of the digest tree. The
    /// encrypted content can only be accessed by those with the appropriate
    /// key.
    ///
    /// This variant is only available when the `encrypt` feature is enabled.
    #[cfg(feature = "encrypt")]
    Encrypted(EncryptedMessage),

    /// Represents an envelope that has been compressed.
    ///
    /// Compression reduces the size of an envelope while maintaining its full
    /// content and digest integrity. Unlike elision or encryption, compression
    /// doesn't restrict access to the content, but simply makes it more
    /// compact.
    ///
    /// This variant is only available when the `compress` feature is enabled.
    #[cfg(feature = "compress")]
    Compressed(Compressed),
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
    pub fn new_or_none(
        subject: Option<impl EnvelopeEncodable>,
    ) -> Option<Self> {
        subject.map(Self::new)
    }

    /// Creates an assertion envelope with a `predicate` and `object`,
    /// each of which can be any instance that implements ``EnvelopeEncodable``.
    pub fn new_assertion(
        predicate: impl EnvelopeEncodable,
        object: impl EnvelopeEncodable,
    ) -> Self {
        Self::new_with_assertion(Assertion::new(predicate, object))
    }
}

/// Internal constructors
impl Envelope {
    pub(crate) fn new_with_unchecked_assertions(
        subject: Self,
        unchecked_assertions: Vec<Self>,
    ) -> Self {
        assert!(!unchecked_assertions.is_empty());
        let mut sorted_assertions = unchecked_assertions;
        sorted_assertions.sort_by(|a, b| a.digest().cmp(&b.digest()));
        let mut digests = vec![subject.digest().into_owned()];
        digests
            .extend(sorted_assertions.iter().map(|a| a.digest().into_owned()));
        let digest = Digest::from_digests(&digests);
        (EnvelopeCase::Node { subject, assertions: sorted_assertions, digest })
            .into()
    }

    pub(crate) fn new_with_assertions(
        subject: Self,
        assertions: Vec<Self>,
    ) -> Result<Self> {
        if !assertions
            .iter()
            .all(|a| (a.is_subject_assertion() || a.is_subject_obscured()))
        {
            bail!(Error::InvalidFormat);
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
    pub(crate) fn new_with_encrypted(
        encrypted_message: EncryptedMessage,
    ) -> Result<Self> {
        if !encrypted_message.has_digest() {
            bail!(Error::MissingDigest);
        }
        Ok(EnvelopeCase::Encrypted(encrypted_message).into())
    }

    #[cfg(feature = "compress")]
    pub(crate) fn new_with_compressed(compressed: Compressed) -> Result<Self> {
        if !compressed.has_digest() {
            bail!(Error::MissingDigest);
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
    #[cfg(feature = "compress")]
    use bc_components::Compressed;
    use bc_components::DigestProvider;

    #[cfg(feature = "known_value")]
    use crate::extension::KnownValue;
    use crate::{Assertion, Envelope};

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
