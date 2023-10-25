use std::{collections::{HashSet, hash_map::RandomState}, iter};

use bc_components::{DigestProvider, Digest};

use crate::{Envelope, base::envelope::EnvelopeCase};

/// Support for inclusions proofs.
impl Envelope {
    /// Returns a proof that this envelope includes every element in the target set.
    ///
    /// # Parameters
    /// - `target`: The elements if this envelope that the proof must include.
    /// # Returns
    /// The proof, of `None` if it cannot be proven that the envelope contains every element in the target set.
    pub fn proof_contains_set(&self, target: &HashSet<Digest, RandomState>) -> Option<Envelope> {
        let reveal_set = self.reveal_set_of_set(target);
        if !target.is_subset(&reveal_set) {
            return None;
        }
        Some(self.elide_revealing_set(&reveal_set).elide_removing_set(target))
    }

    /// Returns a proof that this envelope includes the target element.
    ///
    /// # Parameters
    /// - `target`: The element of this envelope that the proof must include.
    /// # Returns
    /// The proof, of `None` if it cannot be proven that the envelope contains the targeted element.
    pub fn proof_contains_target(&self, target: &dyn DigestProvider) -> Option<Envelope> {
        let set = HashSet::from_iter(iter::once(target.digest().into_owned()));
        self.proof_contains_set(&set)
    }

    /// Confirms whether or not this envelope contains the target set using the given inclusion proof.
    ///
    /// # Parameters
    /// - `target`: The target elements that need to be proven exist somewhere in this envelope, even if they were elided or encrypted.
    /// - `proof`: The inclusion proof to use.
    /// # Returns
    /// `true` if every element of `target` is in this envelope as shown by `proof`, `false` otherwise.
    pub fn confirm_contains_set(&self, target: &HashSet<Digest, RandomState>, proof: &Envelope) -> bool {
        self.digest() == proof.digest() && proof.contains_all(target)
    }

    /// Confirms whether or not this envelope contains the target element using the given inclusion proof.
    ///
    /// # Parameters
    /// - `target`: The target element that needs to be proven to exist somewhere in this envelope, even if it was elided or encrypted.
    /// - `proof`: The inclusion proof to use.
    /// # Returns
    /// `true` if `target` is in this envelope as shown by `proof`, `false` otherwise.
    pub fn confirm_contains_target(&self, target: &dyn DigestProvider, proof: &Envelope) -> bool {
        let set = HashSet::from_iter(iter::once(target.digest().into_owned()));
        self.confirm_contains_set(&set, proof)
    }
}

impl Envelope {
    fn reveal_set_of_set(&self, target: &HashSet<Digest>) -> HashSet<Digest> {
        let mut result = HashSet::new();
        self.reveal_sets(target, &HashSet::new(), &mut result);
        result
    }

    fn contains_all(&self, target: &HashSet<Digest>) -> bool {
        let mut target = target.clone();
        self.remove_all_found(&mut target);
        target.is_empty()
    }

    fn reveal_sets(&self, target: &HashSet<Digest>, current: &HashSet<Digest>, result: &mut HashSet<Digest>) {
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
