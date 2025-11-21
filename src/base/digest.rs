use std::{cell::RefCell, collections::HashSet};

use bc_components::{Digest, DigestProvider};

use super::{envelope::EnvelopeCase, walk::EdgeType};
use crate::Envelope;

/// Support for calculating the digests associated with `Envelope`.
///
/// Envelopes implement the `DigestProvider` trait, which means they can provide
/// cryptographic digests of their contents. This is a fundamental feature of
/// Gordian Envelope that enables privacy-enhancing features like selective
/// disclosure through elision while maintaining verifiable integrity.
impl DigestProvider for Envelope {
    /// Returns the envelope's digest.
    ///
    /// The digest of an envelope uniquely identifies its semantic content,
    /// regardless of whether parts of it are elided, encrypted, or
    /// compressed. This is a key property that enables privacy features
    /// while maintaining integrity.
    ///
    /// Two envelopes with the same digest are considered semantically
    /// equivalent - they represent the same information, even if parts of
    /// one envelope are elided or obscured.
    ///
    /// # Returns
    ///
    /// A borrowed reference to the digest if it's already computed and stored
    /// in the envelope, or a newly computed digest if needed.
    ///
    /// # Examples
    ///
    /// ```
    /// # use bc_envelope::prelude::*;
    /// # use bc_components::DigestProvider;
    /// let envelope = Envelope::new("Hello.");
    /// let elided = envelope.elide();
    ///
    /// // Even though the content is elided, the digests match
    /// assert_eq!(envelope.digest(), elided.digest());
    /// ```
    fn digest(&self) -> Digest {
        match self.case() {
            EnvelopeCase::Node { digest, .. } => *digest,
            EnvelopeCase::Leaf { digest, .. } => *digest,
            EnvelopeCase::Wrapped { digest, .. } => *digest,
            EnvelopeCase::Assertion(assertion) => assertion.digest(),
            EnvelopeCase::Elided(digest) => *digest,
            #[cfg(feature = "known_value")]
            EnvelopeCase::KnownValue { digest, .. } => *digest,
            #[cfg(feature = "encrypt")]
            EnvelopeCase::Encrypted(encrypted_message) => {
                encrypted_message.digest()
            }
            #[cfg(feature = "compress")]
            EnvelopeCase::Compressed(compressed) => compressed.digest(),
        }
    }
}

/// Support for working with the digest tree of an `Envelope`.
///
/// Gordian Envelope's structure includes a Merkle-like digest tree where each
/// node has a cryptographic digest representing its content. These methods help
/// navigate and utilize this digest tree.
impl Envelope {
    /// Returns the set of digests contained in the envelope's elements, down to
    /// the specified level.
    ///
    /// This method collects all digests from the envelope's structure up to a
    /// certain depth. It's extremely useful for selective elision or
    /// revelation, allowing you to gather digests of specific parts of the
    /// structure to target them for operations.
    ///
    /// # Parameters
    ///
    /// * `level_limit` - Maximum level depth to include (0 means no digests,
    ///   `usize::MAX` means all digests)
    ///
    /// # Returns
    ///
    /// A `HashSet` containing all the digests in the envelope structure up to
    /// the specified level. The digest of the envelope itself and its
    /// subject are both included (if they differ).
    ///
    /// # Examples
    ///
    /// ```
    /// # use bc_envelope::prelude::*;
    /// let envelope = Envelope::new("Alice")
    ///     .add_assertion("name", "Alice Smith")
    ///     .add_assertion("age", 30);
    ///
    /// // Level 0 returns an empty set
    /// assert_eq!(envelope.digests(0).len(), 0);
    ///
    /// // Level 1 returns just the envelope's digest
    /// assert_eq!(envelope.digests(1).len(), 2); // Envelope and subject digests
    ///
    /// // A deeper level includes assertion digests
    /// assert!(envelope.digests(3).len() > 4); // Includes assertions plus their predicates and objects
    /// ```
    pub fn digests(&self, level_limit: usize) -> HashSet<Digest> {
        let result = RefCell::new(HashSet::new());
        let visitor = |envelope: &Envelope,
                       level: usize,
                       _: EdgeType,
                       _: ()|
         -> (_, bool) {
            if level < level_limit {
                let mut result = result.borrow_mut();
                result.insert(envelope.digest());
                result.insert(envelope.subject().digest());
            }
            ((), false) // Continue walking
        };
        self.walk(false, (), &visitor);
        result.into_inner()
    }

    /// Returns the set of all digests in the envelope, at all levels.
    ///
    /// This is a convenience method that retrieves every digest in the envelope
    /// structure, no matter how deeply nested. It's useful when you need to
    /// work with the complete digest tree, such as when verifying integrity
    /// or preparing for comprehensive elision.
    ///
    /// # Returns
    ///
    /// A `HashSet` containing all digests in the envelope structure.
    ///
    /// # Examples
    ///
    /// ```
    /// # use bc_envelope::prelude::*;
    /// let envelope = Envelope::new("Alice")
    ///     .add_assertion("name", "Alice Smith")
    ///     .add_assertion("age", 30)
    ///     .add_assertion(
    ///         "address",
    ///         Envelope::new("Address")
    ///             .add_assertion("city", "Boston")
    ///             .add_assertion("zip", "02108"),
    ///     );
    ///
    /// // Gets all digests from the entire structure
    /// let digests = envelope.deep_digests();
    ///
    /// // This includes nested assertions digests
    /// assert!(digests.len() > 10);
    /// ```
    pub fn deep_digests(&self) -> HashSet<Digest> { self.digests(usize::MAX) }

    /// Returns the set of digests in the envelope, down to its second level
    /// only.
    ///
    /// This is a convenience method that retrieves just the top-level digests
    /// in the envelope, including the envelope itself, its subject, and its
    /// immediate assertions, but not deeper nested content. It's useful
    /// when you want to work with just the top-level structure without
    /// getting into nested details.
    ///
    /// # Returns
    ///
    /// A `HashSet` containing digests from the envelope's first two levels.
    ///
    /// # Examples
    ///
    /// ```
    /// # use bc_envelope::prelude::*;
    /// let envelope = Envelope::new("Alice")
    ///     .add_assertion("name", "Alice Smith")
    ///     .add_assertion(
    ///         "address",
    ///         Envelope::new("Address")
    ///             .add_assertion("city", "Boston")
    ///             .add_assertion("zip", "02108"),
    ///     );
    ///
    /// // Gets just the shallow digests
    /// let shallow = envelope.shallow_digests();
    ///
    /// // Gets all digests
    /// let deep = envelope.deep_digests();
    ///
    /// // The shallow set is smaller than the deep set
    /// assert!(shallow.len() < deep.len());
    /// ```
    pub fn shallow_digests(&self) -> HashSet<Digest> { self.digests(2) }

    /// Returns a digest that captures the structural form of an envelope, not
    /// just its semantic content.
    ///
    /// While the regular `digest()` method provides a value for comparing
    /// semantic equivalence (whether two envelopes contain the same
    /// information), this method produces a digest that additionally
    /// captures the structural form of the envelope, including its
    /// elision patterns, encryption, and compression.
    ///
    /// This allows distinguishing between envelopes that contain the same
    /// information but differ in how that information is structured or
    /// obscured.
    ///
    /// # Semantic vs. Structural Comparison
    ///
    /// - **Semantic equivalence** (using `digest()` or `is_equivalent_to()`) -
    ///   Two envelopes contain the same information in their unobscured form,
    ///   complexity O(1)
    ///
    /// - **Structural identity** (using `structural_digest()` or
    ///   `is_identical_to()`) - Two envelopes not only contain the same
    ///   information but also have the same structure, including elision
    ///   patterns, encryption, compression, etc., complexity O(m + n) where m
    ///   and n are the number of elements in each envelope
    ///
    /// # Returns
    ///
    /// A `Digest` that uniquely identifies the envelope's structure as well as
    /// its content.
    ///
    /// # Examples
    ///
    /// ```
    /// # use bc_envelope::prelude::*;
    /// # use bc_components::DigestProvider;
    /// // Two envelopes with the same content but different structure
    /// let envelope1 = Envelope::new("Alice").add_assertion("name", "Alice Smith");
    ///
    /// let envelope2 = Envelope::new("Alice")
    ///     .add_assertion("name", "Alice Smith")
    ///     .elide_removing_target(&Envelope::new("Alice"));
    ///
    /// // They are semantically equivalent
    /// assert!(envelope1.is_equivalent_to(&envelope2));
    /// assert_eq!(envelope1.digest(), envelope2.digest());
    ///
    /// // But they are structurally different
    /// assert!(!envelope1.is_identical_to(&envelope2));
    /// assert_ne!(envelope1.structural_digest(), envelope2.structural_digest());
    /// ```
    pub fn structural_digest(&self) -> Digest {
        let image = RefCell::new(Vec::new());
        let visitor =
            |envelope: &Envelope, _: usize, _: EdgeType, _: ()| -> (_, bool) {
                // Add a discriminator to the image for the obscured cases.
                match envelope.case() {
                    EnvelopeCase::Elided(_) => image.borrow_mut().push(1),
                    #[cfg(feature = "encrypt")]
                    EnvelopeCase::Encrypted(_) => image.borrow_mut().push(0),
                    #[cfg(feature = "compress")]
                    EnvelopeCase::Compressed(_) => image.borrow_mut().push(2),
                    _ => {}
                }
                image
                    .borrow_mut()
                    .extend_from_slice(envelope.digest().data());
                ((), false) // Continue walking
            };
        self.walk(false, (), &visitor);
        Digest::from_image(image.into_inner())
    }

    /// Tests if this envelope is semantically equivalent to another envelope.
    ///
    /// Two envelopes are semantically equivalent if they contain the same
    /// information in their unobscured form, even if they have different
    /// structures (e.g., one is partially elided and the other is not).
    ///
    /// This comparison has a complexity of O(1) as it simply compares the
    /// top-level digests.
    ///
    /// # Parameters
    ///
    /// * `other` - The other envelope to compare with
    ///
    /// # Returns
    ///
    /// `true` if the envelopes are semantically equivalent, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// # use bc_envelope::prelude::*;
    /// // Create two envelopes with the same semantic content but different structure
    /// let original = Envelope::new("Hello world");
    /// let elided = original.elide();
    ///
    /// // They are semantically equivalent (contain the same information)
    /// assert!(original.is_equivalent_to(&elided));
    ///
    /// // But they have different structures
    /// assert!(!original.is_identical_to(&elided));
    /// ```
    pub fn is_equivalent_to(&self, other: &Self) -> bool {
        self.digest() == other.digest()
    }

    /// Tests if two envelopes are structurally identical.
    ///
    /// This function determines whether two envelopes are structurally
    /// identical. The comparison follows these rules:
    ///
    /// 1. If the envelopes are *not* semantically equivalent (different
    ///    digests), returns `false` - different content means they cannot be
    ///    identical
    ///
    /// 2. If the envelopes are semantically equivalent (same digests), then it
    ///    compares their structural digests to check if they have the same
    ///    structure (elision patterns, encryption, etc.)
    ///
    /// The comparison has a complexity of O(1) if the envelopes are not
    /// semantically equivalent, and O(m + n) if they are, where m and n are
    /// the number of elements in each envelope.
    ///
    /// # Parameters
    ///
    /// * `other` - The other envelope to compare with
    ///
    /// # Returns
    ///
    /// `true` if and only if the envelopes have both the same content
    /// (semantically equivalent) AND the same structure, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// # use bc_envelope::prelude::*;
    /// // Two envelopes with identical structure
    /// let env1 = Envelope::new("Alice");
    /// let env2 = Envelope::new("Alice");
    /// let env3 = Envelope::new("Bob");
    ///
    /// // Semantically different envelopes are not identical
    /// assert!(!env1.is_identical_to(&env3));
    ///
    /// // Two envelopes with same content and structure are identical
    /// assert!(env1.is_identical_to(&env2));
    ///
    /// // Envelopes with same content but different structure (one elided) are not identical
    /// let elided = env1.elide();
    /// assert!(env1.is_equivalent_to(&elided));  // semantically the same
    /// assert!(!env1.is_identical_to(&elided));  // but structurally different
    /// ```
    pub fn is_identical_to(&self, other: &Self) -> bool {
        if !self.is_equivalent_to(other) {
            return false; // Different content means not identical
        }
        self.structural_digest() == other.structural_digest()
    }
}

/// Implementation of `PartialEq` for `Envelope` to allow for structural
/// comparison.
///
/// This implements the `==` operator for envelopes using the
/// `is_identical_to()` method, which checks for both semantic equivalence and
/// structural identity. Following the corrected behavior of that method:
///
/// 1. Envelopes with different content (not semantically equivalent) are not
///    equal (!=)
/// 2. Envelopes with the same content but different structure are not equal
///    (!=)
/// 3. Only envelopes with both the same content and structure are equal (==)
///
/// Note that we deliberately do *not* also implement `Eq` as this comparison
/// for structural identity is potentially expensive (O(m + n) in the worst
/// case), and data structures like `HashMap` expect `Eq` to be a fast
/// operation.
///
/// # Usage with Hash Maps and Sets
///
/// If you want to use envelopes as keys in hash-based collections like
/// `HashMap` or `HashSet`, you should pre-compute the envelope's
/// `structural_digest()` and use that as the key instead, as this will be more
/// efficient.
///
/// # Examples
///
/// ```
/// # use bc_envelope::prelude::*;
/// let env1 = Envelope::new("Alice");
/// let env2 = Envelope::new("Alice");
/// let env3 = Envelope::new("Bob");
///
/// // Same content and structure are equal
/// assert!(env1 == env2);
///
/// // Different content means not equal
/// assert!(env1 != env3);
///
/// // Same content but different structure (one elided) are not equal
/// assert!(env1 != env1.elide());
/// ```
impl PartialEq for Envelope {
    /// Compares two envelopes for structural identity.
    ///
    /// This is equivalent to calling `self.is_identical_to(other)`.
    fn eq(&self, other: &Self) -> bool { self.is_identical_to(other) }
}
