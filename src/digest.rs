use std::{collections::HashSet, cell::RefCell, rc::Rc, borrow::Cow};

use bc_components::{Digest, DigestProvider};

use crate::{envelope::Envelope, EdgeType};

/// Support for calculating the digests associated with `Envelope`.

impl DigestProvider for Envelope {
    /// The envelope's digest.
    ///
    /// This digest can be used to compare two envelopes for semantic equivalence, that
    /// is, the two envelopes would contain the same information in their unencrypted
    /// and unelided forms. See <doc:Diffing> for more information.
    fn digest(&self) -> Cow<Digest> {
        match self {
            Self::Node { digest, .. } => Cow::Borrowed(digest),
            Self::Leaf { digest, .. } => Cow::Borrowed(digest),
            Self::Wrapped { digest, .. } => Cow::Borrowed(digest),
            Self::KnownValue { digest, .. } => Cow::Borrowed(digest),
            Self::Assertion(assertion) => assertion.digest(),
            Self::Encrypted(encrypted_message) => encrypted_message.digest(),
            Self::Compressed(compressed) => compressed.digest(),
            Self::Elided(digest) => Cow::Borrowed(digest),
        }
    }
}

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
    pub fn digests(self: Rc<Self>, level_limit: usize) -> HashSet<Digest> {
        let result = RefCell::new(HashSet::new());
        let visitor = |envelope: Rc<Self>, level: usize, _: EdgeType, _: Option<&()>| -> _ {
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

    /// The set of all digests in the envelope.
    pub fn deep_digests(self: Rc<Self>) -> HashSet<Digest> {
        self.digests(usize::MAX)
    }

    /// The set of all digests in the envelope, down to its second level.
    pub fn shallow_digests(self: Rc<Self>) -> HashSet<Digest> {
        self.digests(2)
    }
}

impl Envelope {
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
    pub fn structural_digest(self: Rc<Self>) -> Digest {
        let image = RefCell::new(Vec::new());
        let visitor = |envelope: Rc<Self>, _: usize, _: EdgeType, _: Option<&()>| -> _ {
            // Add a discriminator to the image for the obscured cases.
            match &*envelope {
                Self::Encrypted(_) => image.borrow_mut().push(0),
                Self::Elided(_) => image.borrow_mut().push(1),
                Self::Compressed(_) => image.borrow_mut().push(2),
                _ => {}
            }
            image.borrow_mut().extend_from_slice(envelope.digest().data());
            None
        };
        self.walk(false, &visitor);
        Digest::from_image(&image.into_inner())
    }
}

impl Envelope {
    /// Tests two envelopes for semantic equivalence.
    ///
    /// Calling `e1.is_equivalent_to(e2)` has a complexity of `O(1)` and simply compares
    /// the two envelope's digests. The means that two envelopes with certain structural
    /// differences (e.g., one envelope is partially elided and the other is not) will
    /// still test as equivalent.
    pub fn is_equivalent_to(self: Rc<Self>, other: Rc<Self>) -> bool {
        self.digest() == other.digest()
    }

    /// Tests two envelopes for structural equality.
    ///
    /// Calling `e1.is_identical_to(e2)` has a complexity of `O(1)` if the envelopes are
    /// not semantically equivalent (that is, their top-level digests are different, and
    /// thus they *must* have different structures) and a complexity of `O(m + n)` where
    /// `m` and `n` are the number of elements in each of the two envelopes when they
    /// *are* semantically equivalent.
    pub fn is_identical_to(self: Rc<Self>, other: Rc<Self>) -> bool {
        if !self.clone().is_equivalent_to(other.clone()) {
            return true;
        }
        self.structural_digest() == other.structural_digest()
    }
}
