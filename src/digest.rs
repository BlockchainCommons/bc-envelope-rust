use std::collections::HashSet;

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

/*
```swift
public extension Envelope {
    /// Returns the set of digests contained in the envelope's elements, down to the
    /// specified level.
    ///
    /// - Parameter levelLimit: Return digests at levels below this value.
    /// - Returns: The set of digests down to `levelLimit`.
    ///
    /// The digest of the envelope is included as well as the digest of the envelope's
    /// subject (if it is different).
    ///
    /// If no `levelLimit` is provided, all digests in the envelope will be returned.
    ///
    /// A `levelLimit` of zero will return no digests.
    func digests(levelLimit: Int = .max) -> Set<Digest> {
        var result: Set<Digest> = []
        walk { (envelope, level, incomingEdge, _) -> Int? in
            guard level < levelLimit else {
                return nil
            }
            result.insert(envelope)
            result.insert(envelope.subject)
            return nil
        }
        return result
    }

    /// The set of all digests in the envelope.
    var deepDigests: Set<Digest> {
        digests()
    }

    /// The set of all digests in the envelope, down to its second level.
    var shallowDigests: Set<Digest> {
        digests(levelLimit: 2)
    }
}
```
 */

// The above Swift code translated to Rust:

impl Envelope {
    // / Returns the set of digests contained in the envelope's elements, down to the
    // / specified level.
    // /
    // / The digest of the envelope is included as well as the digest of the envelope's
    // / subject (if it is different).
    // /
    // / If no `levelLimit` is provided, all digests in the envelope will be returned.
    // /
    // / A `levelLimit` of zero will return no digests.
    // /
    // / # Arguments
    // /
    // / * `levelLimit` - Return digests at levels below this value.
    // /
    // / # Returns
    // /
    // / * A set of digests down to `levelLimit`.
    // pub fn digests<'a>(&'a self, level_limit: usize) -> HashSet<Digest> {
    //     let mut result = HashSet::new();

    //     let visitor: Visitor<_> = &|envelope: &Envelope, level: usize, incoming_edge: EdgeType, _: Option<&()>| {
    //         if level < level_limit {
    //             result.insert(envelope.digest());
    //             result.insert(envelope.subject().digest());
    //         }
    //         None
    //     };
    //     self.walk(visitor);
    //     result
    // }

    // /// The set of all digests in the envelope.
    // pub fn deep_digests(&self) -> HashSet<Digest> {
    //     self.digests(usize::MAX)
    // }

    // /// The set of all digests in the envelope, down to its second level.
    // pub fn shallow_digests(&self) -> HashSet<Digest> {
    //     self.digests(2)
    // }
}
