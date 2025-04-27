use std::borrow::Cow;
use anyhow::{ bail, Error, Result };
use bc_components::{ Digest, DigestProvider };
use dcbor::prelude::*;

use crate::{ Envelope, EnvelopeEncodable, EnvelopeError };

/// A predicate-object relationship representing an assertion about a subject.
///
/// In Gordian Envelope, assertions are the basic building blocks for attaching
/// information to a subject. An assertion consists of a predicate (which states
/// what is being asserted) and an object (which provides the assertion's value).
///
/// Assertions can be attached to envelope subjects to form semantic statements like:
/// "subject hasAttribute value" or "document signedBy signature".
///
/// Assertions are equivalent to RDF (Resource Description Framework) triples,
/// where:
/// - The envelope's subject is the subject of the triple
/// - The assertion's predicate is the predicate of the triple
/// - The assertion's object is the object of the triple
///
/// Generally you do not create an instance of this type directly, but
/// instead use [`Envelope::new_assertion`], or the various functions
/// on `Envelope` that create assertions.
#[derive(Clone, Debug)]
pub struct Assertion {
    predicate: Envelope,
    object: Envelope,
    digest: Digest,
}

impl Assertion {
    /// Creates a new assertion and calculates its digest.
    ///
    /// This constructor takes a predicate and object, both of which are converted to
    /// envelopes using the `EnvelopeEncodable` trait. It then calculates the assertion's
    /// digest by combining the digests of the predicate and object.
    ///
    /// The digest is calculated according to the Gordian Envelope specification, which
    /// ensures that semantically equivalent assertions always produce the same digest.
    ///
    /// # Parameters
    ///
    /// * `predicate` - The predicate of the assertion, which states what is being asserted
    /// * `object` - The object of the assertion, which provides the assertion's value
    ///
    /// # Returns
    ///
    /// A new assertion with the specified predicate, object, and calculated digest.
    ///
    /// # Example
    ///
    /// ```
    /// # use bc_envelope::prelude::*;
    /// // Direct method - create an assertion envelope
    /// let assertion_envelope = Envelope::new_assertion("name", "Alice");
    ///
    /// // Or create and add an assertion to a subject
    /// let person = Envelope::new("person")
    ///     .add_assertion("name", "Alice");
    /// ```
    pub fn new(predicate: impl EnvelopeEncodable, object: impl EnvelopeEncodable) -> Self {
        let predicate = predicate.into_envelope();
        let object = object.into_envelope();
        let digest = Digest::from_digests(
            &[predicate.digest().into_owned(), object.digest().into_owned()]
        );
        Self {
            predicate,
            object,
            digest,
        }
    }

    /// Returns the predicate of the assertion.
    ///
    /// The predicate states what is being asserted about the subject. It is typically
    /// a string or known value, but can be any envelope.
    ///
    /// # Returns
    ///
    /// A clone of the assertion's predicate envelope.
    pub fn predicate(&self) -> Envelope {
        self.predicate.clone()
    }

    /// Returns the object of the assertion.
    ///
    /// The object provides the value or content of the assertion. It can be any
    /// type that can be represented as an envelope.
    ///
    /// # Returns
    ///
    /// A clone of the assertion's object envelope.
    pub fn object(&self) -> Envelope {
        self.object.clone()
    }

    /// Returns a reference to the digest of the assertion.
    ///
    /// The digest is calculated when the assertion is created and is used for
    /// verification and deduplication. The digest calculation follows the rules
    /// specified in the Gordian Envelope IETF draft, Section 4.4.
    ///
    /// # Returns
    ///
    /// A reference to the assertion's digest.
    pub fn digest_ref(&self) -> &Digest {
        &self.digest
    }
}

/// Equality is based on digest equality, not structural equality.
///
/// Two assertions are considered equal if they have the same digest,
/// regardless of how they were constructed.
impl PartialEq for Assertion {
    fn eq(&self, other: &Self) -> bool {
        self.digest_ref() == other.digest_ref()
    }
}

/// Assertion implements full equality.
impl Eq for Assertion {}

/// Implementation of `DigestProvider` for `Assertion`.
///
/// This allows an assertion to provide its digest for calculation of
/// higher-level digests in the envelope digest tree.
impl DigestProvider for Assertion {
    /// Returns a reference to the assertion's digest.
    ///
    /// This is used in the envelope digest tree calculation.
    fn digest(&self) -> Cow<'_, Digest> {
        Cow::Borrowed(&self.digest)
    }
}

/// Converts an assertion to its CBOR representation.
///
/// The CBOR representation of an assertion is a map with a single key-value pair,
/// where the key is the predicate's CBOR and the value is the object's CBOR.
impl From<Assertion> for CBOR {
    fn from(value: Assertion) -> Self {
        let mut map = Map::new();
        map.insert(value.predicate.untagged_cbor(), value.object.untagged_cbor());
        map.into()
    }
}

/// Attempts to convert a CBOR value to an assertion.
///
/// The CBOR must be a map with exactly one entry, where the key represents
/// the predicate and the value represents the object.
impl TryFrom<CBOR> for Assertion {
    type Error = Error;

    fn try_from(value: CBOR) -> Result<Self> {
        if let CBORCase::Map(map) = value.as_case() {
            return map.clone().try_into();
        }
        bail!(EnvelopeError::InvalidAssertion)
    }
}

/// Attempts to convert a CBOR map to an assertion.
///
/// The map must have exactly one entry, where the key represents the
/// predicate and the value represents the object. This is used in
/// the deserialization process.
impl TryFrom<Map> for Assertion {
    type Error = Error;

    fn try_from(map: Map) -> Result<Self> {
        if map.len() != 1 {
            bail!(EnvelopeError::InvalidAssertion)
        }
        let elem = map.iter().next().unwrap();
        let predicate = Envelope::from_untagged_cbor(elem.0.clone())?;
        let object = Envelope::from_untagged_cbor(elem.1.clone())?;
        Ok(Self::new(predicate, object))
    }
}
