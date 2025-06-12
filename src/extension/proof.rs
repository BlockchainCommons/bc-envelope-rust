use std::{
    collections::{HashSet, hash_map::RandomState},
    iter,
};

use bc_components::{Digest, DigestProvider};

use crate::{Envelope, base::envelope::EnvelopeCase};

/// # Inclusion Proofs
///
/// Inclusion proofs allow a holder of an envelope to prove that specific
/// elements exist within the envelope without revealing the entire contents.
/// This is particularly useful for selective disclosure of information in
/// privacy-preserving scenarios.
///
/// The inclusion proof mechanism leverages the Merkle-like digest tree
/// structure of envelopes:
/// - The holder creates a minimal structure containing only the digests
///   necessary to validate the proof
/// - A verifier with a trusted root digest can confirm that the specific
///   elements exist in the original envelope
/// - All other content can remain elided, preserving privacy
///
/// For enhanced privacy, elements can be salted to prevent correlation attacks.
///
/// ## Examples
///
/// ### Basic Inclusion Proof
///
/// ```
/// use std::collections::HashSet;
///
/// use bc_envelope::prelude::*;
///
/// // Create an envelope with multiple assertions
/// let alice_friends = Envelope::new("Alice")
///     .add_assertion("knows", "Bob")
///     .add_assertion("knows", "Carol")
///     .add_assertion("knows", "Dan");
///
/// // Create a representation of just the root digest
/// let alice_friends_root = alice_friends.elide_revealing_set(&HashSet::new());
///
/// // Create the target we want to prove exists
/// let knows_bob_assertion = Envelope::new_assertion("knows", "Bob");
///
/// // Generate a proof that Alice knows Bob
/// let alice_knows_bob_proof = alice_friends
///     .proof_contains_target(&knows_bob_assertion)
///     .unwrap();
///
/// // A third party can verify the proof against the trusted root
/// assert!(
///     alice_friends_root.confirm_contains_target(
///         &knows_bob_assertion,
///         &alice_knows_bob_proof
///     )
/// );
/// ```
///
/// ### Enhanced Privacy with Salting
///
/// ```
/// use std::collections::HashSet;
///
/// use bc_envelope::prelude::*;
///
/// // Create an envelope with salted assertions for enhanced privacy
/// let alice_friends = Envelope::new("Alice")
///     .add_assertion_salted("knows", "Bob", true)
///     .add_assertion_salted("knows", "Carol", true)
///     .add_assertion_salted("knows", "Dan", true);
///
/// // Create a representation of just the root digest
/// let alice_friends_root = alice_friends.elide_revealing_set(&HashSet::new());
///
/// // Create the target we want to prove exists
/// let knows_bob_assertion = Envelope::new_assertion("knows", "Bob");
///
/// // Generate a proof that Alice knows Bob
/// let alice_knows_bob_proof = alice_friends
///     .proof_contains_target(&knows_bob_assertion)
///     .unwrap();
///
/// // A third party can verify the proof against the trusted root
/// // Note: The salting prevents the third party from guessing other friends
/// // by simple correlation attacks
/// assert!(
///     alice_friends_root.confirm_contains_target(
///         &knows_bob_assertion,
///         &alice_knows_bob_proof
///     )
/// );
/// ```
impl Envelope {
    /// Creates a proof that this envelope includes every element in the target
    /// set.
    ///
    /// An inclusion proof is a specially constructed envelope that:
    /// - Has the same digest as the original envelope (or an elided version of
    ///   it)
    /// - Contains the minimal structure needed to prove the existence of target
    ///   elements
    /// - Keeps all other content elided to preserve privacy
    ///
    /// # Parameters
    /// - `target`: The set of digests representing elements that the proof must
    ///   include.
    ///
    /// # Returns
    /// - `Some(Envelope)`: A proof envelope if all targets can be proven to
    ///   exist
    /// - `None`: If it cannot be proven that the envelope contains every
    ///   element in the target set
    ///
    /// # Example
    ///
    /// ```
    /// use std::collections::HashSet;
    ///
    /// use bc_envelope::prelude::*;
    ///
    /// // Create a document with multiple assertions
    /// let document = Envelope::new("Document")
    ///     .add_assertion("title", "Important Report")
    ///     .add_assertion("author", "Alice")
    ///     .add_assertion("confidential", true);
    ///
    /// // Create a set of elements we want to prove exist
    /// let title_assertion = Envelope::new_assertion("title", "Important Report");
    /// let author_assertion = Envelope::new_assertion("author", "Alice");
    ///
    /// let mut target_set = HashSet::new();
    /// target_set.insert(title_assertion.digest().into_owned());
    /// target_set.insert(author_assertion.digest().into_owned());
    ///
    /// // Generate a proof for multiple elements
    /// let proof = document.proof_contains_set(&target_set).unwrap();
    ///
    /// // The proof can be verified against the document's root digest
    /// let document_root = document.elide_revealing_set(&HashSet::new());
    /// assert!(document_root.confirm_contains_set(&target_set, &proof));
    /// ```
    pub fn proof_contains_set(
        &self,
        target: &HashSet<Digest, RandomState>,
    ) -> Option<Envelope> {
        let reveal_set = self.reveal_set_of_set(target);
        if !target.is_subset(&reveal_set) {
            return None;
        }
        Some(
            self.elide_revealing_set(&reveal_set)
                .elide_removing_set(target),
        )
    }

    /// Creates a proof that this envelope includes the single target element.
    ///
    /// This is a convenience method that wraps `proof_contains_set()` for the
    /// common case of proving the existence of just one element.
    ///
    /// # Parameters
    /// - `target`: The element that the proof must demonstrate exists in this
    ///   envelope.
    ///
    /// # Returns
    /// - `Some(Envelope)`: A proof envelope if the target can be proven to
    ///   exist
    /// - `None`: If it cannot be proven that the envelope contains the target
    ///   element
    ///
    /// # Example
    ///
    /// ```
    /// use std::collections::HashSet;
    ///
    /// use bc_envelope::prelude::*;
    ///
    /// // Create a credential with various attributes
    /// let credential = Envelope::new("Credential")
    ///     .add_assertion("firstName", "John")
    ///     .add_assertion("lastName", "Smith")
    ///     .add_assertion("birthDate", "1990-01-01")
    ///     .add_assertion("address", "123 Main St")
    ///     .add_assertion("documentNumber", "ABC123456");
    ///
    /// // Create a trusted root digest
    /// let credential_root = credential.elide_revealing_set(&HashSet::new());
    ///
    /// // The holder wants to prove just their name without revealing other details
    /// let first_name = Envelope::new_assertion("firstName", "John");
    ///
    /// // Generate a proof for the first name
    /// let proof = credential.proof_contains_target(&first_name).unwrap();
    ///
    /// // A verifier can confirm the proof is valid
    /// assert!(credential_root.confirm_contains_target(&first_name, &proof));
    /// ```
    pub fn proof_contains_target(
        &self,
        target: &dyn DigestProvider,
    ) -> Option<Envelope> {
        let set = HashSet::from_iter(iter::once(target.digest().into_owned()));
        self.proof_contains_set(&set)
    }

    /// Verifies whether this envelope contains all elements in the target set
    /// using the given inclusion proof.
    ///
    /// This method is used by a verifier to check if a proof demonstrates the
    /// existence of all target elements within this envelope. The
    /// verification succeeds only if:
    /// 1. The proof's digest matches this envelope's digest
    /// 2. The proof contains all the target elements
    ///
    /// # Parameters
    /// - `target`: The set of digests representing elements that need to be
    ///   proven to exist.
    /// - `proof`: The inclusion proof envelope to verify.
    ///
    /// # Returns
    /// - `true`: If all target elements are proven to exist in this envelope by
    ///   the proof
    /// - `false`: Otherwise
    ///
    /// # Example
    ///
    /// ```
    /// use std::collections::HashSet;
    ///
    /// use bc_envelope::prelude::*;
    ///
    /// // A verifier has a trusted root digest of a document
    /// let document = Envelope::new("Document")
    ///     .add_assertion("title", "Report")
    ///     .add_assertion("author", "Alice");
    /// let document_root = document.elide_revealing_set(&HashSet::new());
    ///
    /// // Create a set of elements we want to verify
    /// let author_assertion = Envelope::new_assertion("author", "Alice");
    /// let mut target_set = HashSet::new();
    /// target_set.insert(author_assertion.digest().into_owned());
    ///
    /// // The holder provides a proof
    /// let proof = document.proof_contains_set(&target_set).unwrap();
    ///
    /// // The verifier confirms the proof is valid
    /// assert!(document_root.confirm_contains_set(&target_set, &proof));
    /// ```
    pub fn confirm_contains_set(
        &self,
        target: &HashSet<Digest, RandomState>,
        proof: &Envelope,
    ) -> bool {
        self.digest() == proof.digest() && proof.contains_all(target)
    }

    /// Verifies whether this envelope contains the single target element using
    /// the given inclusion proof.
    ///
    /// This is a convenience method that wraps `confirm_contains_set()` for the
    /// common case of verifying just one element.
    ///
    /// # Parameters
    /// - `target`: The element that needs to be proven to exist in this
    ///   envelope.
    /// - `proof`: The inclusion proof envelope to verify.
    ///
    /// # Returns
    /// - `true`: If the target element is proven to exist in this envelope by
    ///   the proof
    /// - `false`: Otherwise
    ///
    /// # Example
    ///
    /// ```
    /// use std::collections::HashSet;
    ///
    /// use bc_envelope::prelude::*;
    ///
    /// // A verifier has a trusted root digest of a credential
    /// let credential = Envelope::new("Credential")
    ///     .add_assertion("isOver21", true)
    ///     .add_assertion("licenseNumber", "DL12345678");
    /// let credential_root = credential.elide_revealing_set(&HashSet::new());
    ///
    /// // The element to be verified
    /// let is_over_21 = Envelope::new_assertion("isOver21", true);
    ///
    /// // The holder provides a proof
    /// let proof = credential.proof_contains_target(&is_over_21).unwrap();
    ///
    /// // The verifier confirms the proof shows the person is over 21
    /// // without revealing their license number
    /// assert!(credential_root.confirm_contains_target(&is_over_21, &proof));
    /// ```
    pub fn confirm_contains_target(
        &self,
        target: &dyn DigestProvider,
        proof: &Envelope,
    ) -> bool {
        let set = HashSet::from_iter(iter::once(target.digest().into_owned()));
        self.confirm_contains_set(&set, proof)
    }
}

/// Internal implementation methods for inclusion proofs
impl Envelope {
    /// Builds a set of all digests needed to reveal the target set.
    ///
    /// This collects all digests in the path from the envelope's root to each
    /// target element.
    fn reveal_set_of_set(&self, target: &HashSet<Digest>) -> HashSet<Digest> {
        let mut result = HashSet::new();
        self.reveal_sets(target, &HashSet::new(), &mut result);
        result
    }

    /// Checks if this envelope contains all elements in the target set.
    ///
    /// Used during proof verification to confirm all target elements exist in
    /// the proof.
    fn contains_all(&self, target: &HashSet<Digest>) -> bool {
        let mut target = target.clone();
        self.remove_all_found(&mut target);
        target.is_empty()
    }

    /// Recursively traverses the envelope to collect all digests needed to
    /// reveal the target set.
    ///
    /// Builds the set of digests forming the path from the root to each target
    /// element.
    fn reveal_sets(
        &self,
        target: &HashSet<Digest>,
        current: &HashSet<Digest>,
        result: &mut HashSet<Digest>,
    ) {
        let mut current = current.clone();
        current.insert(self.digest().into_owned());

        if target.contains(&self.digest()) {
            result.extend(current.iter().cloned());
        }

        match self.case() {
            EnvelopeCase::Node { subject, assertions, .. } => {
                subject.reveal_sets(target, &current, result);
                for assertion in assertions {
                    assertion.reveal_sets(target, &current, result);
                }
            }
            EnvelopeCase::Wrapped { envelope, .. } => {
                envelope.reveal_sets(target, &current, result);
            }
            EnvelopeCase::Assertion(assertion) => {
                assertion.predicate().reveal_sets(target, &current, result);
                assertion.object().reveal_sets(target, &current, result);
            }
            _ => {}
        }
    }

    /// Recursively traverses the envelope and removes found target elements
    /// from the set.
    ///
    /// Used during proof verification to confirm all target elements are
    /// present.
    fn remove_all_found(&self, target: &mut HashSet<Digest>) {
        if target.contains(&self.digest()) {
            target.remove(&self.digest());
        }
        if target.is_empty() {
            return;
        }

        match self.case() {
            EnvelopeCase::Node { subject, assertions, .. } => {
                subject.remove_all_found(target);
                for assertion in assertions {
                    assertion.remove_all_found(target);
                }
            }
            EnvelopeCase::Wrapped { envelope, .. } => {
                envelope.remove_all_found(target);
            }
            EnvelopeCase::Assertion(assertion) => {
                assertion.predicate().remove_all_found(target);
                assertion.object().remove_all_found(target);
            }
            _ => {}
        }
    }
}
