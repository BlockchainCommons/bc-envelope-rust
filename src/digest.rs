use std::{collections::HashSet, cell::RefCell, rc::Rc};

use bc_components::{Digest, DigestProvider};

use crate::{envelope::Envelope, EdgeType};

/// Support for calculating the digests associated with `Envelope`.

impl<'a> Envelope {
    /// The envelope's digest.
    ///
    /// This digest can be used to compare two envelopes for semantic equivalence, that
    /// is, the two envelopes would contain the same information in their unencrypted
    /// and unelided forms. See <doc:Diffing> for more information.
    pub fn digest_ref(&'a self) -> &'a Digest {
        match self {
            Envelope::Node { digest, .. } => digest,
            Envelope::Leaf { digest, .. } => digest,
            Envelope::Wrapped { digest, .. } => digest,
            Envelope::KnownValue { digest, .. } => digest,
            Envelope::Assertion(assertion) => assertion.digest_ref(),
            Envelope::Encrypted(encrypted_message) => encrypted_message.digest_ref(),
            Envelope::Compressed(compressed) => compressed.digest_ref(),
            Envelope::Elided(digest) => digest,
        }
    }
}

impl DigestProvider for Envelope {
    fn digest(&self) -> Digest {
        self.digest_ref().clone()
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
                result.insert(envelope.digest());
                result.insert(envelope.subject().digest());
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
