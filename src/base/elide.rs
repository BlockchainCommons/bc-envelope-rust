use std::collections::HashSet;

use bc_components::{Digest, DigestProvider};
#[cfg(feature = "encrypt")]
use bc_components::{Nonce, SymmetricKey};
#[cfg(feature = "encrypt")]
use dcbor::prelude::*;

use super::envelope::EnvelopeCase;
use crate::{Assertion, Envelope, Error, Result};

/// Types of obscuration that can be applied to envelope elements.
///
/// This enum identifies the different ways an envelope element can be obscured.
/// Unlike `ObscureAction` which is used to perform obscuration operations,
/// `ObscureType` is used to identify and filter elements based on their
/// obscuration state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ObscureType {
    /// The element has been elided, showing only its digest.
    Elided,

    /// The element has been encrypted using symmetric encryption.
    ///
    /// This variant is only available when the `encrypt` feature is enabled.
    #[cfg(feature = "encrypt")]
    Encrypted,

    /// The element has been compressed to reduce its size.
    ///
    /// This variant is only available when the `compress` feature is enabled.
    #[cfg(feature = "compress")]
    Compressed,
}

/// Actions that can be performed on parts of an envelope to obscure them.
///
/// Gordian Envelope supports several ways to obscure parts of an envelope while
/// maintaining its semantic integrity and digest tree. This enum defines the
/// possible actions that can be taken when obscuring envelope elements.
///
/// Obscuring parts of an envelope is a key feature for privacy and selective
/// disclosure, allowing the holder of an envelope to share only specific parts
/// while hiding, encrypting, or compressing others.
pub enum ObscureAction {
    /// Elide the target, leaving only its digest.
    ///
    /// Elision replaces the targeted envelope element with just its digest,
    /// hiding its actual content while maintaining the integrity of the
    /// envelope's digest tree. This is the most basic form of selective
    /// disclosure.
    ///
    /// Elided elements can be revealed later by providing the original unelided
    /// envelope to the recipient, who can verify that the revealed content
    /// matches the digest in the elided version.
    Elide,

    /// Encrypt the target using the specified symmetric key.
    ///
    /// This encrypts the targeted envelope element using authenticated
    /// encryption with the provided key. The encrypted content can only be
    /// accessed by those who possess the symmetric key.
    ///
    /// This action is only available when the `encrypt` feature is enabled.
    #[cfg(feature = "encrypt")]
    Encrypt(SymmetricKey),

    /// Compress the target using a compression algorithm.
    ///
    /// This compresses the targeted envelope element to reduce its size while
    /// still allowing it to be decompressed by any recipient. Unlike elision or
    /// encryption, compression doesn't provide privacy but can reduce the size
    /// of large envelope components.
    ///
    /// This action is only available when the `compress` feature is enabled.
    #[cfg(feature = "compress")]
    Compress,
}

/// Support for eliding elements from envelopes.
///
/// This includes eliding, encrypting and compressing (obscuring) elements.
impl Envelope {
    /// Returns the elided variant of this envelope.
    ///
    /// Elision replaces an envelope with just its digest, hiding its content
    /// while maintaining the integrity of the envelope's digest tree. This
    /// is a fundamental privacy feature of Gordian Envelope that enables
    /// selective disclosure.
    ///
    /// Returns the same envelope if it is already elided.
    ///
    /// # Examples
    ///
    /// ```
    /// # use bc_envelope::prelude::*;
    /// # use indoc::indoc;
    /// let envelope = Envelope::new("Hello.");
    /// let elided = envelope.elide();
    ///
    /// // The elided envelope shows only "ELIDED" in formatting
    /// assert_eq!(elided.format_flat(), "ELIDED");
    ///
    /// // But it maintains the same digest as the original
    /// assert!(envelope.is_equivalent_to(&elided));
    /// ```
    pub fn elide(&self) -> Self {
        match self.case() {
            EnvelopeCase::Elided(_) => self.clone(),
            _ => Self::new_elided(self.digest().into_owned()),
        }
    }

    /// Returns a version of this envelope with elements in the `target` set
    /// elided.
    ///
    /// This function obscures elements in the envelope whose digests are in the
    /// provided target set, applying the specified action (elision,
    /// encryption, or compression) to those elements while leaving other
    /// elements intact.
    ///
    /// # Parameters
    ///
    /// * `target` - The set of digests that identify elements to be obscured
    /// * `action` - The action to perform on the targeted elements (elide,
    ///   encrypt, or compress)
    ///
    /// # Examples
    ///
    /// ```
    /// # use bc_envelope::prelude::*;
    /// # use std::collections::HashSet;
    /// let envelope = Envelope::new("Alice")
    ///     .add_assertion("knows", "Bob")
    ///     .add_assertion("livesAt", "123 Main St.");
    ///
    /// // Create a set of digests targeting the "livesAt" assertion
    /// let mut target = HashSet::new();
    /// let livesAt_assertion = envelope.assertion_with_predicate("livesAt").unwrap();
    /// target.insert(livesAt_assertion.digest().into_owned());
    ///
    /// // Elide that specific assertion
    /// let elided = envelope.elide_removing_set_with_action(&target, &ObscureAction::Elide);
    ///
    /// // The result will have the "livesAt" assertion elided but "knows" still visible
    /// ```
    pub fn elide_removing_set_with_action(
        &self,
        target: &HashSet<Digest>,
        action: &ObscureAction,
    ) -> Self {
        self.elide_set_with_action(target, false, action)
    }

    /// Returns a version of this envelope with elements in the `target` set
    /// elided.
    ///
    /// This is a convenience function that calls `elide_set` with
    /// `is_revealing` set to `false`, using the standard elision action.
    /// Use this when you want to simply elide elements rather than encrypt
    /// or compress them.
    ///
    /// # Parameters
    ///
    /// * `target` - The set of digests that identify elements to be elided
    ///
    /// # Examples
    ///
    /// ```
    /// # use bc_envelope::prelude::*;
    /// # use std::collections::HashSet;
    /// let envelope = Envelope::new("Alice")
    ///     .add_assertion("knows", "Bob")
    ///     .add_assertion("email", "alice@example.com");
    ///
    /// // Create a set of digests targeting the email assertion
    /// let mut target = HashSet::new();
    /// let email_assertion = envelope.assertion_with_predicate("email").unwrap();
    /// target.insert(email_assertion.digest().into_owned());
    ///
    /// // Elide the email assertion for privacy
    /// let redacted = envelope.elide_removing_set(&target);
    /// ```
    pub fn elide_removing_set(&self, target: &HashSet<Digest>) -> Self {
        self.elide_set(target, false)
    }

    /// Returns a version of this envelope with elements in the `target` set
    /// elided.
    ///
    /// - Parameters:
    ///   - target: An array of `DigestProvider`s.
    ///   - action: Perform the specified action (elision, encryption or
    ///     compression).
    ///
    /// - Returns: The elided envelope.
    pub fn elide_removing_array_with_action(
        &self,
        target: &[&dyn DigestProvider],
        action: &ObscureAction,
    ) -> Self {
        self.elide_array_with_action(target, false, action)
    }

    /// Returns a version of this envelope with elements in the `target` set
    /// elided.
    ///
    /// - Parameters:
    ///   - target: An array of `DigestProvider`s.
    ///   - action: Perform the specified action (elision, encryption or
    ///     compression).
    ///
    /// - Returns: The elided envelope.
    pub fn elide_removing_array(&self, target: &[&dyn DigestProvider]) -> Self {
        self.elide_array(target, false)
    }

    /// Returns a version of this envelope with the target element elided.
    ///
    /// - Parameters:
    ///   - target: A `DigestProvider`.
    ///   - action: Perform the specified action (elision, encryption or
    ///     compression).
    ///
    /// - Returns: The elided envelope.
    pub fn elide_removing_target_with_action(
        &self,
        target: &dyn DigestProvider,
        action: &ObscureAction,
    ) -> Self {
        self.elide_target_with_action(target, false, action)
    }

    /// Returns a version of this envelope with the target element elided.
    ///
    /// - Parameters:
    ///   - target: A `DigestProvider`.
    ///
    /// - Returns: The elided envelope.
    pub fn elide_removing_target(&self, target: &dyn DigestProvider) -> Self {
        self.elide_target(target, false)
    }

    /// Returns a version of this envelope with only elements in the `target`
    /// set revealed, and all other elements elided.
    ///
    /// This function performs the opposite operation of
    /// `elide_removing_set_with_action`. Instead of specifying which
    /// elements to obscure, you specify which elements to reveal,
    /// and everything else will be obscured using the specified action.
    ///
    /// This is particularly useful for selective disclosure where you want to
    /// reveal only specific portions of an envelope while keeping the rest
    /// private.
    ///
    /// # Parameters
    ///
    /// * `target` - The set of digests that identify elements to be revealed
    /// * `action` - The action to perform on all other elements (elide,
    ///   encrypt, or compress)
    ///
    /// # Examples
    ///
    /// ```
    /// # use bc_envelope::prelude::*;
    /// # use std::collections::HashSet;
    /// let envelope = Envelope::new("Alice")
    ///     .add_assertion("name", "Alice Smith")
    ///     .add_assertion("age", 30)
    ///     .add_assertion("ssn", "123-45-6789");
    ///
    /// // Create a set of digests for elements we want to reveal
    /// let mut reveal_set = HashSet::new();
    ///
    /// // Add the subject and the name assertion to the set to reveal
    /// reveal_set.insert(envelope.subject().digest().into_owned());
    /// reveal_set.insert(
    ///     envelope
    ///         .assertion_with_predicate("name")
    ///         .unwrap()
    ///         .digest()
    ///         .into_owned(),
    /// );
    ///
    /// // Create an envelope that only reveals name and hides age and SSN
    /// let selective = envelope
    ///     .elide_revealing_set_with_action(&reveal_set, &ObscureAction::Elide);
    /// ```
    pub fn elide_revealing_set_with_action(
        &self,
        target: &HashSet<Digest>,
        action: &ObscureAction,
    ) -> Self {
        self.elide_set_with_action(target, true, action)
    }

    /// Returns a version of this envelope with elements *not* in the `target`
    /// set elided.
    ///
    /// - Parameters:
    ///   - target: The target set of digests.
    ///
    /// - Returns: The elided envelope.
    pub fn elide_revealing_set(&self, target: &HashSet<Digest>) -> Self {
        self.elide_set(target, true)
    }

    /// Returns a version of this envelope with elements *not* in the `target`
    /// set elided.
    ///
    /// - Parameters:
    ///   - target: An array of `DigestProvider`s.
    ///   - action: Perform the specified action (elision, encryption or
    ///     compression).
    ///
    /// - Returns: The elided envelope.
    pub fn elide_revealing_array_with_action(
        &self,
        target: &[&dyn DigestProvider],
        action: &ObscureAction,
    ) -> Self {
        self.elide_array_with_action(target, true, action)
    }

    /// Returns a version of this envelope with elements *not* in the `target`
    /// set elided.
    ///
    /// - Parameters:
    ///   - target: An array of `DigestProvider`s.
    ///
    /// - Returns: The elided envelope.
    pub fn elide_revealing_array(
        &self,
        target: &[&dyn DigestProvider],
    ) -> Self {
        self.elide_array(target, true)
    }

    /// Returns a version of this envelope with all elements *except* the target
    /// element elided.
    ///
    /// - Parameters:
    ///   - target: A `DigestProvider`.
    ///   - action: Perform the specified action (elision, encryption or
    ///     compression).
    ///
    /// - Returns: The elided envelope.
    pub fn elide_revealing_target_with_action(
        &self,
        target: &dyn DigestProvider,
        action: &ObscureAction,
    ) -> Self {
        self.elide_target_with_action(target, true, action)
    }

    /// Returns a version of this envelope with all elements *except* the target
    /// element elided.
    ///
    /// - Parameters:
    ///   - target: A `DigestProvider`.
    ///
    /// - Returns: The elided envelope.
    pub fn elide_revealing_target(&self, target: &dyn DigestProvider) -> Self {
        self.elide_target(target, true)
    }

    // Target Matches   isRevealing     elide
    // ----------------------------------------
    //     false           false        false
    //     false           true         true
    //     true            false        true
    //     true            true         false

    /// Returns an elided version of this envelope.
    ///
    /// - Parameters:
    ///   - target: The target set of digests.
    ///   - isRevealing: If `true`, the target set contains the digests of the
    ///     elements to leave revealed. If it is `false`, the target set
    ///     contains the digests of the elements to elide.
    ///   - action: Perform the specified action (elision, encryption or
    ///     compression).
    ///
    /// - Returns: The elided envelope.
    pub fn elide_set_with_action(
        &self,
        target: &HashSet<Digest>,
        is_revealing: bool,
        action: &ObscureAction,
    ) -> Self {
        let self_digest = self.digest().into_owned();
        if target.contains(&self_digest) != is_revealing {
            match action {
                ObscureAction::Elide => self.elide(),
                #[cfg(feature = "encrypt")]
                ObscureAction::Encrypt(key) => {
                    let message = key.encrypt_with_digest(
                        self.tagged_cbor().to_cbor_data(),
                        self_digest,
                        None::<Nonce>,
                    );
                    Self::new_with_encrypted(message).unwrap()
                }
                #[cfg(feature = "compress")]
                ObscureAction::Compress => self.compress().unwrap(),
            }
        } else if let EnvelopeCase::Assertion(assertion) = self.case() {
            let predicate = assertion.predicate().elide_set_with_action(
                target,
                is_revealing,
                action,
            );
            let object = assertion.object().elide_set_with_action(
                target,
                is_revealing,
                action,
            );
            let elided_assertion = Assertion::new(predicate, object);
            assert!(&elided_assertion == assertion);
            Self::new_with_assertion(elided_assertion)
        } else if let EnvelopeCase::Node { subject, assertions, .. } =
            self.case()
        {
            let elided_subject =
                subject.elide_set_with_action(target, is_revealing, action);
            assert!(elided_subject.digest() == subject.digest());
            let elided_assertions = assertions
                .iter()
                .map(|assertion| {
                    let elided_assertion = assertion.elide_set_with_action(
                        target,
                        is_revealing,
                        action,
                    );
                    assert!(elided_assertion.digest() == assertion.digest());
                    elided_assertion
                })
                .collect();
            Self::new_with_unchecked_assertions(
                elided_subject,
                elided_assertions,
            )
        } else if let EnvelopeCase::Wrapped { envelope, .. } = self.case() {
            let elided_envelope =
                envelope.elide_set_with_action(target, is_revealing, action);
            assert!(elided_envelope.digest() == envelope.digest());
            Self::new_wrapped(elided_envelope)
        } else {
            self.clone()
        }
    }

    /// Returns an elided version of this envelope.
    ///
    /// - Parameters:
    ///   - target: The target set of digests.
    ///   - isRevealing: If `true`, the target set contains the digests of the
    ///     elements to leave revealed. If it is `false`, the target set
    ///     contains the digests of the elements to elide.
    ///
    /// - Returns: The elided envelope.
    pub fn elide_set(
        &self,
        target: &HashSet<Digest>,
        is_revealing: bool,
    ) -> Self {
        self.elide_set_with_action(target, is_revealing, &ObscureAction::Elide)
    }

    /// Returns an elided version of this envelope.
    ///
    /// - Parameters:
    ///   - target: An array of `DigestProvider`s.
    ///   - isRevealing: If `true`, the target set contains the digests of the
    ///     elements to leave revealed. If it is `false`, the target set
    ///     contains the digests of the elements to elide.
    ///   - action: Perform the specified action (elision, encryption or
    ///     compression).
    ///
    /// - Returns: The elided envelope.
    pub fn elide_array_with_action(
        &self,
        target: &[&dyn DigestProvider],
        is_revealing: bool,
        action: &ObscureAction,
    ) -> Self {
        self.elide_set_with_action(
            &target
                .iter()
                .map(|provider| provider.digest().into_owned())
                .collect(),
            is_revealing,
            action,
        )
    }

    /// Returns an elided version of this envelope.
    ///
    /// - Parameters:
    ///   - target: An array of `DigestProvider`s.
    ///   - isRevealing: If `true`, the target set contains the digests of the
    ///     elements to leave revealed. If it is `false`, the target set
    ///     contains the digests of the elements to elide.
    ///
    /// - Returns: The elided envelope.
    pub fn elide_array(
        &self,
        target: &[&dyn DigestProvider],
        is_revealing: bool,
    ) -> Self {
        self.elide_array_with_action(
            target,
            is_revealing,
            &ObscureAction::Elide,
        )
    }

    /// Returns an elided version of this envelope.
    ///
    /// - Parameters:
    ///   - target: A `DigestProvider`.
    ///   - isRevealing: If `true`, the target is the element to leave revealed,
    ///     eliding all others. If it is `false`, the target is the element to
    ///     elide, leaving all others revealed.
    ///   - action: Perform the specified action (elision, encryption or
    ///     compression).
    ///
    /// - Returns: The elided envelope.
    pub fn elide_target_with_action(
        &self,
        target: &dyn DigestProvider,
        is_revealing: bool,
        action: &ObscureAction,
    ) -> Self {
        self.elide_array_with_action(&[target], is_revealing, action)
    }

    /// Returns an elided version of this envelope.
    ///
    /// - Parameters:
    ///   - target: A `DigestProvider`.
    ///   - isRevealing: If `true`, the target is the element to leave revealed,
    ///     eliding all others. If it is `false`, the target is the element to
    ///     elide, leaving all others revealed.
    ///
    /// - Returns: The elided envelope.
    pub fn elide_target(
        &self,
        target: &dyn DigestProvider,
        is_revealing: bool,
    ) -> Self {
        self.elide_target_with_action(
            target,
            is_revealing,
            &ObscureAction::Elide,
        )
    }

    /// Returns the unelided variant of this envelope by revealing the original
    /// content.
    ///
    /// This function allows restoring an elided envelope to its original form,
    /// but only if the provided envelope's digest matches the elided
    /// envelope's digest. This ensures the integrity of the revealed
    /// content.
    ///
    /// Returns the same envelope if it is already unelided.
    ///
    /// # Errors
    ///
    /// Returns `EnvelopeError::InvalidDigest` if the provided envelope's digest
    /// doesn't match the current envelope's digest.
    ///
    /// # Examples
    ///
    /// ```
    /// # use bc_envelope::prelude::*;
    /// let original = Envelope::new("Hello.");
    /// let elided = original.elide();
    ///
    /// // Later, we can unelide the envelope if we have the original
    /// let revealed = elided.unelide(&original).unwrap();
    /// assert_eq!(revealed.format(), "\"Hello.\"");
    ///
    /// // Attempting to unelide with a different envelope will fail
    /// let different = Envelope::new("Different");
    /// assert!(elided.unelide(&different).is_err());
    /// ```
    pub fn unelide(&self, envelope: impl Into<Envelope>) -> Result<Self> {
        let envelope = envelope.into();
        if self.digest() == envelope.digest() {
            Ok(envelope)
        } else {
            Err(Error::InvalidDigest)
        }
    }

    /// Returns the set of digests of nodes matching the specified criteria.
    ///
    /// This function walks the envelope hierarchy and returns digests of nodes
    /// that match both:
    /// - The optional target digest set (if provided; otherwise all nodes
    ///   match)
    /// - Any of the specified obscuration types
    ///
    /// If no obscuration types are provided, all nodes in the target set (or
    /// all nodes if no target set) are returned.
    ///
    /// # Parameters
    ///
    /// * `target_digests` - Optional set of digests to filter by. If `None`,
    ///   all nodes are considered for matching.
    /// * `obscure_types` - Slice of `ObscureType` values to match against. Only
    ///   nodes obscured in one of these ways will be included.
    ///
    /// # Returns
    ///
    /// A `HashSet` of digests for nodes matching the specified criteria.
    ///
    /// # Examples
    ///
    /// ```
    /// # use bc_envelope::prelude::*;
    /// # use std::collections::HashSet;
    /// let envelope = Envelope::new("Alice")
    ///     .add_assertion("knows", "Bob")
    ///     .add_assertion("age", 30);
    ///
    /// // Elide one assertion
    /// let knows_digest = envelope
    ///     .assertion_with_predicate("knows")
    ///     .unwrap()
    ///     .digest()
    ///     .into_owned();
    /// let mut target = HashSet::new();
    /// target.insert(knows_digest.clone());
    ///
    /// let elided = envelope.elide_removing_set(&target);
    ///
    /// // Find all elided nodes
    /// let elided_digests = elided.nodes_matching(None, &[ObscureType::Elided]);
    /// assert!(elided_digests.contains(&knows_digest));
    /// ```
    pub fn nodes_matching(
        &self,
        target_digests: Option<&HashSet<Digest>>,
        obscure_types: &[ObscureType],
    ) -> HashSet<Digest> {
        use std::cell::RefCell;

        use super::walk::EdgeType;

        let result = RefCell::new(HashSet::new());

        let visitor = |envelope: &Envelope,
                       _level: usize,
                       _edge: EdgeType,
                       _state: ()|
         -> ((), bool) {
            // Check if this node matches the target digests (or if no target
            // specified)
            let digest_matches = target_digests
                .map(|targets| targets.contains(envelope.digest().as_ref()))
                .unwrap_or(true);

            if !digest_matches {
                return ((), false);
            }

            // If no obscure types specified, include all nodes
            if obscure_types.is_empty() {
                result.borrow_mut().insert(envelope.digest().into_owned());
                return ((), false);
            }

            // Check if this node matches any of the specified obscure types
            let type_matches =
                obscure_types.iter().any(|obscure_type| {
                    match (obscure_type, envelope.case()) {
                        (ObscureType::Elided, EnvelopeCase::Elided(_)) => true,
                        #[cfg(feature = "encrypt")]
                        (
                            ObscureType::Encrypted,
                            EnvelopeCase::Encrypted(_),
                        ) => true,
                        #[cfg(feature = "compress")]
                        (
                            ObscureType::Compressed,
                            EnvelopeCase::Compressed(_),
                        ) => true,
                        _ => false,
                    }
                });

            if type_matches {
                result.borrow_mut().insert(envelope.digest().into_owned());
            }

            ((), false)
        };

        self.walk(false, (), &visitor);
        result.into_inner()
    }

    /// Returns a new envelope with elided nodes restored from the provided set.
    ///
    /// This function walks the envelope hierarchy and attempts to restore any
    /// elided nodes by matching their digests against the provided set of
    /// envelopes. If a match is found, the elided node is replaced with the
    /// matching envelope.
    ///
    /// If no matches are found, the original envelope is returned unchanged.
    ///
    /// # Parameters
    ///
    /// * `envelopes` - A slice of envelopes that may match elided nodes in
    ///   `self`
    ///
    /// # Returns
    ///
    /// A new envelope with elided nodes restored where possible.
    ///
    /// # Examples
    ///
    /// ```
    /// # use bc_envelope::prelude::*;
    /// let alice = Envelope::new("Alice");
    /// let bob = Envelope::new("Bob");
    /// let envelope = Envelope::new("Alice").add_assertion("knows", "Bob");
    ///
    /// // Elide both the subject and an assertion
    /// let elided = envelope
    ///     .elide_removing_target(&alice)
    ///     .elide_removing_target(&bob);
    ///
    /// // Restore the elided nodes
    /// let restored = elided.walk_unelide(&[alice, bob]);
    ///
    /// // The restored envelope should match the original
    /// assert_eq!(restored.format(), envelope.format());
    /// ```
    pub fn walk_unelide(&self, envelopes: &[Envelope]) -> Self {
        use std::collections::HashMap;

        // Build a lookup map of digest -> envelope
        let mut envelope_map = HashMap::new();
        for envelope in envelopes {
            envelope_map
                .insert(envelope.digest().into_owned(), envelope.clone());
        }

        self.walk_unelide_with_map(&envelope_map)
    }

    fn walk_unelide_with_map(
        &self,
        envelope_map: &std::collections::HashMap<Digest, Envelope>,
    ) -> Self {
        match self.case() {
            EnvelopeCase::Elided(_) => {
                // Try to find a matching envelope to restore
                if let Some(replacement) =
                    envelope_map.get(self.digest().as_ref())
                {
                    replacement.clone()
                } else {
                    self.clone()
                }
            }
            EnvelopeCase::Node { subject, assertions, .. } => {
                // Recursively unelide subject and assertions
                let new_subject = subject.walk_unelide_with_map(envelope_map);
                let new_assertions: Vec<_> = assertions
                    .iter()
                    .map(|a| a.walk_unelide_with_map(envelope_map))
                    .collect();

                if new_subject.is_identical_to(subject)
                    && new_assertions
                        .iter()
                        .zip(assertions.iter())
                        .all(|(a, b)| a.is_identical_to(b))
                {
                    self.clone()
                } else {
                    Self::new_with_unchecked_assertions(
                        new_subject,
                        new_assertions,
                    )
                }
            }
            EnvelopeCase::Wrapped { envelope, .. } => {
                let new_envelope = envelope.walk_unelide_with_map(envelope_map);
                if new_envelope.is_identical_to(envelope) {
                    self.clone()
                } else {
                    new_envelope.wrap()
                }
            }
            EnvelopeCase::Assertion(assertion) => {
                // Recursively unelide predicate and object
                let new_predicate =
                    assertion.predicate().walk_unelide_with_map(envelope_map);
                let new_object =
                    assertion.object().walk_unelide_with_map(envelope_map);

                if new_predicate.is_identical_to(&assertion.predicate())
                    && new_object.is_identical_to(&assertion.object())
                {
                    self.clone()
                } else {
                    Envelope::new_assertion(new_predicate, new_object)
                }
            }
            _ => self.clone(),
        }
    }

    /// Returns a new envelope with encrypted nodes decrypted using the
    /// provided keys.
    ///
    /// This function walks the envelope hierarchy and attempts to decrypt any
    /// encrypted nodes using the provided set of symmetric keys. Each key is
    /// tried in sequence until one succeeds or all fail.
    ///
    /// If no nodes can be decrypted, the original envelope is returned
    /// unchanged.
    ///
    /// This function is only available when the `encrypt` feature is enabled.
    ///
    /// # Parameters
    ///
    /// * `keys` - A slice of `SymmetricKey` values to use for decryption
    ///
    /// # Returns
    ///
    /// A new envelope with encrypted nodes decrypted where possible.
    ///
    /// # Examples
    ///
    /// ```
    /// # use bc_envelope::prelude::*;
    /// # use bc_components::SymmetricKey;
    /// let key1 = SymmetricKey::new();
    /// let key2 = SymmetricKey::new();
    ///
    /// let envelope = Envelope::new("Alice")
    ///     .add_assertion("knows", "Bob")
    ///     .add_assertion("age", 30);
    ///
    /// // Encrypt different parts with different keys
    /// let encrypted = envelope.elide_removing_set_with_action(
    ///     &std::collections::HashSet::from([envelope
    ///         .assertion_with_predicate("knows")
    ///         .unwrap()
    ///         .digest()
    ///         .into_owned()]),
    ///     &ObscureAction::Encrypt(key1.clone()),
    /// );
    ///
    /// // Decrypt with a set of keys
    /// let decrypted = encrypted.walk_decrypt(&[key1, key2]);
    ///
    /// // The decrypted envelope should match the original
    /// assert!(decrypted.is_equivalent_to(&envelope));
    /// ```
    #[cfg(feature = "encrypt")]
    pub fn walk_decrypt(&self, keys: &[SymmetricKey]) -> Self {
        match self.case() {
            EnvelopeCase::Encrypted(_) => {
                // Try each key until one works
                for key in keys {
                    if let Ok(decrypted) = self.decrypt_subject(key) {
                        return decrypted.walk_decrypt(keys);
                    }
                }
                // No key worked, return unchanged
                self.clone()
            }
            EnvelopeCase::Node { subject, assertions, .. } => {
                // Recursively decrypt subject and assertions
                let new_subject = subject.walk_decrypt(keys);
                let new_assertions: Vec<_> =
                    assertions.iter().map(|a| a.walk_decrypt(keys)).collect();

                if new_subject.is_identical_to(subject)
                    && new_assertions
                        .iter()
                        .zip(assertions.iter())
                        .all(|(a, b)| a.is_identical_to(b))
                {
                    self.clone()
                } else {
                    Self::new_with_unchecked_assertions(
                        new_subject,
                        new_assertions,
                    )
                }
            }
            EnvelopeCase::Wrapped { envelope, .. } => {
                let new_envelope = envelope.walk_decrypt(keys);
                if new_envelope.is_identical_to(envelope) {
                    self.clone()
                } else {
                    new_envelope.wrap()
                }
            }
            EnvelopeCase::Assertion(assertion) => {
                // Recursively decrypt predicate and object
                let new_predicate = assertion.predicate().walk_decrypt(keys);
                let new_object = assertion.object().walk_decrypt(keys);

                if new_predicate.is_identical_to(&assertion.predicate())
                    && new_object.is_identical_to(&assertion.object())
                {
                    self.clone()
                } else {
                    Envelope::new_assertion(new_predicate, new_object)
                }
            }
            _ => self.clone(),
        }
    }

    /// Returns a new envelope with compressed nodes uncompressed.
    ///
    /// This function walks the envelope hierarchy and uncompresses nodes that:
    /// - Are compressed, AND
    /// - Match the target digest set (if provided), OR all compressed nodes if
    ///   no target set is provided
    ///
    /// If no nodes can be uncompressed, the original envelope is returned
    /// unchanged.
    ///
    /// This function is only available when the `compress` feature is enabled.
    ///
    /// # Parameters
    ///
    /// * `target_digests` - Optional set of digests to filter by. If `None`,
    ///   all compressed nodes will be uncompressed.
    ///
    /// # Returns
    ///
    /// A new envelope with compressed nodes uncompressed where they match the
    /// criteria.
    ///
    /// # Examples
    ///
    /// ```
    /// # use bc_envelope::prelude::*;
    /// # use std::collections::HashSet;
    /// let envelope = Envelope::new("Alice")
    ///     .add_assertion("knows", "Bob")
    ///     .add_assertion("bio", "A".repeat(1000));
    ///
    /// // Compress one assertion
    /// let bio_assertion = envelope.assertion_with_predicate("bio").unwrap();
    /// let bio_digest = bio_assertion.digest().into_owned();
    /// let mut target = HashSet::new();
    /// target.insert(bio_digest);
    ///
    /// let compressed = envelope
    ///     .elide_removing_set_with_action(&target, &ObscureAction::Compress);
    ///
    /// // Uncompress just the targeted node
    /// let uncompressed = compressed.walk_uncompress(Some(&target));
    ///
    /// // The uncompressed envelope should match the original
    /// assert!(uncompressed.is_equivalent_to(&envelope));
    /// ```
    #[cfg(feature = "compress")]
    pub fn walk_uncompress(
        &self,
        target_digests: Option<&HashSet<Digest>>,
    ) -> Self {
        match self.case() {
            EnvelopeCase::Compressed(_) => {
                // Check if this node matches the target (if target specified)
                let matches_target = target_digests
                    .map(|targets| targets.contains(self.digest().as_ref()))
                    .unwrap_or(true);

                if matches_target {
                    // Try to uncompress
                    if let Ok(uncompressed) = self.uncompress() {
                        return uncompressed.walk_uncompress(target_digests);
                    }
                }
                // Either doesn't match target or uncompress failed
                self.clone()
            }
            EnvelopeCase::Node { subject, assertions, .. } => {
                // Recursively uncompress subject and assertions
                let new_subject = subject.walk_uncompress(target_digests);
                let new_assertions: Vec<_> = assertions
                    .iter()
                    .map(|a| a.walk_uncompress(target_digests))
                    .collect();

                if new_subject.is_identical_to(subject)
                    && new_assertions
                        .iter()
                        .zip(assertions.iter())
                        .all(|(a, b)| a.is_identical_to(b))
                {
                    self.clone()
                } else {
                    Self::new_with_unchecked_assertions(
                        new_subject,
                        new_assertions,
                    )
                }
            }
            EnvelopeCase::Wrapped { envelope, .. } => {
                let new_envelope = envelope.walk_uncompress(target_digests);
                if new_envelope.is_identical_to(envelope) {
                    self.clone()
                } else {
                    new_envelope.wrap()
                }
            }
            EnvelopeCase::Assertion(assertion) => {
                // Recursively uncompress predicate and object
                let new_predicate =
                    assertion.predicate().walk_uncompress(target_digests);
                let new_object =
                    assertion.object().walk_uncompress(target_digests);

                if new_predicate.is_identical_to(&assertion.predicate())
                    && new_object.is_identical_to(&assertion.object())
                {
                    self.clone()
                } else {
                    Envelope::new_assertion(new_predicate, new_object)
                }
            }
            _ => self.clone(),
        }
    }
}
