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
            Envelope::Node { digest, .. } => Cow::Borrowed(digest),
            Envelope::Leaf { digest, .. } => Cow::Borrowed(digest),
            Envelope::Wrapped { digest, .. } => Cow::Borrowed(digest),
            Envelope::KnownValue { digest, .. } => Cow::Borrowed(digest),
            Envelope::Assertion(assertion) => assertion.digest(),
            Envelope::Encrypted(encrypted_message) => encrypted_message.digest(),
            Envelope::Compressed(compressed) => compressed.digest(),
            Envelope::Elided(digest) => Cow::Borrowed(digest),
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
    pub fn digests(self: Rc<Envelope>, level_limit: usize) -> HashSet<Digest> {
        let result = RefCell::new(HashSet::new());
        let visitor = |envelope: Rc<Envelope>, level: usize, _: EdgeType, _: Option<&()>| -> _ {
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
    pub fn deep_digests(self: Rc<Envelope>) -> HashSet<Digest> {
        self.digests(usize::MAX)
    }

    /// The set of all digests in the envelope, down to its second level.
    pub fn shallow_digests(self: Rc<Envelope>) -> HashSet<Digest> {
        self.digests(2)
    }
}

impl Envelope {
    pub fn structural_digest(self: Rc<Envelope>) -> Digest {
        todo!();
    }
}

impl Envelope {
    /// Tests two envelopes for semantic equivalence.
    ///
    /// Calling `e1.is_equivalent_to(e2)` has a complexity of `O(1)` and simply compares
    /// the two envelope's digests. The means that two envelopes with certain structural
    /// differences (e.g., one envelope is partially elided and the other is not) will
    /// still test as equivalent.
    pub fn is_equivalent_to(self: Rc<Envelope>, other: Rc<Envelope>) -> bool {
        self.digest() == other.digest()
    }

    /// Tests two envelopes for structural equality.
    ///
    /// Calling `e1.is_identical_to(e2)` has a complexity of `O(1)` if the envelopes are
    /// not semantically equivalent (that is, their top-level digests are different, and
    /// thus they *must* have different structures) and a complexity of `O(m + n)` where
    /// `m` and `n` are the number of elements in each of the two envelopes when they
    /// *are* semantically equivalent.
    pub fn is_identical_to(self: Rc<Envelope>, other: Rc<Envelope>) -> bool {
        if self.clone().is_equivalent_to(other.clone()) {
            return true;
        }
        self.structural_digest() == other.structural_digest()
    }
}
