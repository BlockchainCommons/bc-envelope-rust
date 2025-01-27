use std::{collections::HashSet, cell::RefCell, borrow::Cow};

use bc_components::{Digest, DigestProvider};

use crate::Envelope;

use super::{walk::EdgeType, envelope::EnvelopeCase};

/// Support for calculating the digests associated with `Envelope`.
impl DigestProvider for Envelope {
    /// The envelope's digest.
    ///
    /// This digest can be used to compare two envelopes for semantic equivalence, that
    /// is, the two envelopes would contain the same information in their unencrypted
    /// and unelided forms. See <doc:Diffing> for more information.
    fn digest(&self) -> Cow<'_, Digest> {
        match self.case() {
            EnvelopeCase::Node { digest, .. } => Cow::Borrowed(digest),
            EnvelopeCase::Leaf { digest, .. } => Cow::Borrowed(digest),
            EnvelopeCase::Wrapped { digest, .. } => Cow::Borrowed(digest),
            EnvelopeCase::Assertion(assertion) => assertion.digest(),
            EnvelopeCase::Elided(digest) => Cow::Borrowed(digest),
            #[cfg(feature = "known_value")]
            EnvelopeCase::KnownValue { digest, .. } => Cow::Borrowed(digest),
            #[cfg(feature = "encrypt")]
            EnvelopeCase::Encrypted(encrypted_message) => encrypted_message.digest(),
            #[cfg(feature = "compress")]
            EnvelopeCase::Compressed(compressed) => compressed.digest(),
        }
    }
}

/// Support for working with the digest tree of an `Envelope`.
impl Envelope {
    /// Returns the set of digests contained in the envelope's elements, down to the
    /// specified level.
    ///
    /// The digest of the envelope is included as well as the digest of the envelope's
    /// subject (if it is different).
    ///
    /// If no `levelLimit` is provided, all digests in the envelope will be returned.
    ///
    /// A `levelLimit` of zero will return no digests.
    ///
    /// # Arguments
    ///
    /// * `levelLimit` - Return digests at levels below this value.
    ///
    /// # Returns
    ///
    /// * A set of digests down to `levelLimit`.
    pub fn digests(&self, level_limit: usize) -> HashSet<Digest> {
        let result = RefCell::new(HashSet::new());
        let visitor = |envelope: Self, level: usize, _: EdgeType, _: Option<&()>| -> _ {
            if level < level_limit {
                let mut result = result.borrow_mut();
                result.insert(envelope.digest().into_owned());
                result.insert(envelope.subject().digest().into_owned());
            }
            None
        };
        self.walk(false, &visitor);
        result.into_inner()
    }

    /// Returns the set of all digests in the envelope.
    pub fn deep_digests(&self) -> HashSet<Digest> {
        self.digests(usize::MAX)
    }

    /// Returns the set of all digests in the envelope, down to its second level.
    pub fn shallow_digests(&self) -> HashSet<Digest> {
        self.digests(2)
    }

    /// Produce a value that will necessarily be different if two envelopes differ
    /// structurally, even if they are semantically equivalent.
    ///
    /// Comparing the `digest` field of two envelopes (or calling `isEquivalent(to:)`) tests
    /// whether two envelopes are *semantically equivalent*. This is accomplished by
    /// simply comparing the top level digests of the envelopes for equality, and has a
    /// complexity of `O(1)`.
    ///
    /// This means that two envelopes are considered equivalent if they contain
    /// identical information in their completely unencrypted and unelided form.
    ///
    /// Some applications need to determine whether two envelopes are not only
    /// semantically equivalent, but also structurally identical. Two envelopes that are
    /// not semantically equivalent cannot be structurally identical, but two envelopes
    /// that *are* semantically equivalent *may or may not* be structurally identical.
    ///
    /// The `structural_digest` attribute is used to produce a value that will
    /// necessarily be different if two envelopes differ structurally, even if they are
    /// semantically equivalent. It has a complexity of `O(m + n)` where `m` and `n` are
    /// the number of elements in each of the two envelopes when they *are* semantically
    /// equivalent. It is recommended that envelopes be compared for structural equality
    /// by calling `isIdentical(to:)` as this short-circuits to `false` in cases where
    /// the compared envelopes are not semantically equivalent.
    pub fn structural_digest(&self) -> Digest {
        let image = RefCell::new(Vec::new());
        let visitor = |envelope: Self, _: usize, _: EdgeType, _: Option<&()>| -> _ {
            // Add a discriminator to the image for the obscured cases.
            match envelope.case() {
                EnvelopeCase::Elided(_) => image.borrow_mut().push(1),
                #[cfg(feature = "encrypt")]
                EnvelopeCase::Encrypted(_) => image.borrow_mut().push(0),
                #[cfg(feature = "compress")]
                EnvelopeCase::Compressed(_) => image.borrow_mut().push(2),
                _ => {}
            }
            image.borrow_mut().extend_from_slice(envelope.digest().data());
            None
        };
        self.walk(false, &visitor);
        Digest::from_image(image.into_inner())
    }

    /// Tests two envelopes for semantic equivalence.
    ///
    /// Calling `e1.is_equivalent_to(e2)` has a complexity of `O(1)` and simply compares
    /// the two envelope's digests. The means that two envelopes with certain structural
    /// differences (e.g., one envelope is partially elided and the other is not) will
    /// still test as equivalent.
    pub fn is_equivalent_to(&self, other: &Self) -> bool {
        self.digest() == other.digest()
    }

    /// Tests two envelopes for structural equality.
    ///
    /// Calling `e1.is_identical_to(e2)` has a complexity of `O(1)` if the envelopes are
    /// not semantically equivalent (that is, their top-level digests are different, and
    /// thus they *must* have different structures) and a complexity of `O(m + n)` where
    /// `m` and `n` are the number of elements in each of the two envelopes when they
    /// *are* semantically equivalent.
    pub fn is_identical_to(&self, other: &Self) -> bool {
        if !self.is_equivalent_to(other) {
            return true;
        }
        self.structural_digest() == other.structural_digest()
    }
}

/// Implement `PartialEq` for `Envelope` to allow for structural comparison.
///
/// Note that we deliberately do *not* also implement `Eq` as this comparison
/// for identicality is potentially expensive, and structures like `HashMap`
/// require that `Eq` be implemented for its keys. This should be a fast/cheap
/// operation.
///
/// If you want to use envelopes as keys in such structures, you can pre-compute
/// an envelope's `structural_digest` and use that as the key.
impl PartialEq for Envelope {
    fn eq(&self, other: &Self) -> bool {
        self.is_identical_to(other)
    }
}
