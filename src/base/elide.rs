use std::collections::HashSet;

use bc_components::{DigestProvider, Digest};
#[cfg(feature = "encrypt")]
use bc_components::{SymmetricKey, Nonce};
#[cfg(feature = "encrypt")]
use dcbor::prelude::*;

use crate::{Envelope, Assertion, EnvelopeError};

use super::envelope::EnvelopeCase;

/// An action to perform on a target set in an envelope.
pub enum ObscureAction {
    /// Elide the target, leaving only its digest.
    Elide,

    /// Encrypt the target using the specified key.
    #[cfg(feature = "encrypt")]
    Encrypt(SymmetricKey),

    /// Compress the target.
    #[cfg(feature = "compress")]
    Compress,
}

/// Support for eliding elements from envelopes.
///
/// This includes eliding, encrypting and compressing (obscuring) elements.
impl Envelope {
    /// Returns the elided variant of this envelope.
    ///
    /// Returns the same envelope if it is already elided.
    pub fn elide(&self) -> Self {
        match self.case() {
            EnvelopeCase::Elided(_) => self.clone(),
            _ => Self::new_elided(self.digest().into_owned())
        }
    }

    /// Returns a version of this envelope with elements in the `target` set elided.
    ///
    /// - Parameters:
    ///   - target: The target set of digests.
    ///   - action: Perform the specified action (elision, encryption or compression).
    ///
    /// - Returns: The elided envelope.
    pub fn elide_removing_set_with_action(&self, target: &HashSet<Digest>, action: &ObscureAction) -> Self {
        self.elide_set_with_action(target, false, action)
    }

    /// Returns a version of this envelope with elements in the `target` set elided.
    ///
    /// - Parameters:
    ///   - target: The target set of digests.
    ///   - action: Perform the specified action (elision, encryption or compression).
    ///
    /// - Returns: The elided envelope.
    pub fn elide_removing_set(&self, target: &HashSet<Digest>) -> Self {
        self.elide_set(target, false)
    }

    /// Returns a version of this envelope with elements in the `target` set elided.
    ///
    /// - Parameters:
    ///   - target: An array of `DigestProvider`s.
    ///   - action: Perform the specified action (elision, encryption or compression).
    ///
    /// - Returns: The elided envelope.
    pub fn elide_removing_array_with_action(&self, target: &[&dyn DigestProvider], action: &ObscureAction) -> Self {
        self.elide_array_with_action(target, false, action)
    }

    /// Returns a version of this envelope with elements in the `target` set elided.
    ///
    /// - Parameters:
    ///   - target: An array of `DigestProvider`s.
    ///   - action: Perform the specified action (elision, encryption or compression).
    ///
    /// - Returns: The elided envelope.
    pub fn elide_removing_array(&self, target: &[&dyn DigestProvider]) -> Self {
        self.elide_array(target, false)
    }

    /// Returns a version of this envelope with the target element elided.
    ///
    /// - Parameters:
    ///   - target: A `DigestProvider`.
    ///   - action: Perform the specified action (elision, encryption or compression).
    ///
    /// - Returns: The elided envelope.
    pub fn elide_removing_target_with_action(&self, target: &dyn DigestProvider, action: &ObscureAction) -> Self {
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

    /// Returns a version of this envelope with elements *not* in the `target` set elided.
    ///
    /// - Parameters:
    ///   - target: The target set of digests.
    ///   - action: Perform the specified action (elision, encryption or compression).
    ///
    /// - Returns: The elided envelope.
    pub fn elide_revealing_set_with_action(&self, target: &HashSet<Digest>, action: &ObscureAction) -> Self {
        self.elide_set_with_action(target, true, action)
    }

    /// Returns a version of this envelope with elements *not* in the `target` set elided.
    ///
    /// - Parameters:
    ///   - target: The target set of digests.
    ///
    /// - Returns: The elided envelope.
    pub fn elide_revealing_set(&self, target: &HashSet<Digest>) -> Self {
        self.elide_set(target, true)
    }

    /// Returns a version of this envelope with elements *not* in the `target` set elided.
    ///
    /// - Parameters:
    ///   - target: An array of `DigestProvider`s.
    ///   - action: Perform the specified action (elision, encryption or compression).
    ///
    /// - Returns: The elided envelope.
    pub fn elide_revealing_array_with_action(&self, target: &[&dyn DigestProvider], action: &ObscureAction) -> Self {
        self.elide_array_with_action(target, true, action)
    }

    /// Returns a version of this envelope with elements *not* in the `target` set elided.
    ///
    /// - Parameters:
    ///   - target: An array of `DigestProvider`s.
    ///
    /// - Returns: The elided envelope.
    pub fn elide_revealing_array(&self, target: &[&dyn DigestProvider]) -> Self {
        self.elide_array(target, true)
    }

    /// Returns a version of this envelope with all elements *except* the target element elided.
    ///
    /// - Parameters:
    ///   - target: A `DigestProvider`.
    ///   - action: Perform the specified action (elision, encryption or compression).
    ///
    /// - Returns: The elided envelope.
    pub fn elide_revealing_target_with_action(&self, target: &dyn DigestProvider, action: &ObscureAction) -> Self {
        self.elide_target_with_action(target, true, action)
    }

    /// Returns a version of this envelope with all elements *except* the target element elided.
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
    ///   - isRevealing: If `true`, the target set contains the digests of the elements to
    ///   leave revealed. If it is `false`, the target set contains the digests of the
    ///   elements to elide.
    ///   - action: Perform the specified action (elision, encryption or compression).
    ///
    /// - Returns: The elided envelope.
    pub fn elide_set_with_action(&self, target: &HashSet<Digest>, is_revealing: bool, action: &ObscureAction) -> Self {
        let self_digest = self.digest().into_owned();
        if target.contains(&self_digest) != is_revealing {
            match action {
                ObscureAction::Elide => self.elide(),
                #[cfg(feature = "encrypt")]
                ObscureAction::Encrypt(key) => {
                    let message = key.encrypt(self.tagged_cbor().cbor_data(), Some(self_digest), None::<Nonce>);
                    Self::new_with_encrypted(message).unwrap()
                },
                #[cfg(feature = "compress")]
                ObscureAction::Compress => self.compress().unwrap(),
            }
        } else if let EnvelopeCase::Assertion(assertion) = self.case() {
            let predicate = assertion.predicate().elide_set_with_action(target, is_revealing, action);
            let object = assertion.object().elide_set_with_action(target, is_revealing, action);
            let elided_assertion = Assertion::new(predicate, object);
            assert!(&elided_assertion == assertion);
            Self::new_with_assertion(elided_assertion)
        } else if let EnvelopeCase::Node { subject, assertions, ..} = self.case() {
            let elided_subject = subject.clone().elide_set_with_action(target, is_revealing, action);
            assert!(elided_subject.digest() == subject.digest());
            let elided_assertions = assertions.iter().map(|assertion| {
                let elided_assertion = assertion.clone().elide_set_with_action(target, is_revealing, action);
                assert!(elided_assertion.digest() == assertion.digest());
                elided_assertion
            }).collect();
            Self::new_with_unchecked_assertions(elided_subject, elided_assertions)
        } else if let EnvelopeCase::Wrapped { envelope, .. } = self.case() {
            let elided_envelope = envelope.clone().elide_set_with_action(target, is_revealing, action);
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
    ///   - isRevealing: If `true`, the target set contains the digests of the elements to
    ///   leave revealed. If it is `false`, the target set contains the digests of the
    ///   elements to elide.
    ///
    /// - Returns: The elided envelope.
    pub fn elide_set(&self, target: &HashSet<Digest>, is_revealing: bool) -> Self {
        self.elide_set_with_action(target, is_revealing, &ObscureAction::Elide)
    }

    /// Returns an elided version of this envelope.
    ///
    /// - Parameters:
    ///   - target: An array of `DigestProvider`s.
    ///   - isRevealing: If `true`, the target set contains the digests of the elements to
    ///   leave revealed. If it is `false`, the target set contains the digests of the
    ///   elements to elide.
    ///   - action: Perform the specified action (elision, encryption or compression).
    ///
    /// - Returns: The elided envelope.
    pub fn elide_array_with_action(&self, target: &[&dyn DigestProvider], is_revealing: bool, action: &ObscureAction) -> Self {
        self.elide_set_with_action(&target.iter().map(|provider| provider.digest().into_owned()).collect(), is_revealing, action)
    }

    /// Returns an elided version of this envelope.
    ///
    /// - Parameters:
    ///   - target: An array of `DigestProvider`s.
    ///   - isRevealing: If `true`, the target set contains the digests of the elements to
    ///   leave revealed. If it is `false`, the target set contains the digests of the
    ///   elements to elide.
    ///
    /// - Returns: The elided envelope.
    pub fn elide_array(&self, target: &[&dyn DigestProvider], is_revealing: bool) -> Self {
        self.elide_array_with_action(target, is_revealing, &ObscureAction::Elide)
    }

    /// Returns an elided version of this envelope.
    ///
    /// - Parameters:
    ///   - target: A `DigestProvider`.
    ///   - isRevealing: If `true`, the target is the element to leave revealed, eliding
    ///   all others. If it is `false`, the target is the element to elide, leaving all
    ///   others revealed.
    ///   - action: Perform the specified action (elision, encryption or compression).
    ///
    /// - Returns: The elided envelope.
    pub fn elide_target_with_action(&self, target: &dyn DigestProvider, is_revealing: bool, action: &ObscureAction) -> Self {
        self.elide_array_with_action(&[target], is_revealing, action)
    }

    /// Returns an elided version of this envelope.
    ///
    /// - Parameters:
    ///   - target: A `DigestProvider`.
    ///   - isRevealing: If `true`, the target is the element to leave revealed, eliding
    ///   all others. If it is `false`, the target is the element to elide, leaving all
    ///   others revealed.
    ///
    /// - Returns: The elided envelope.
    pub fn elide_target(&self, target: &dyn DigestProvider, is_revealing: bool) -> Self {
        self.elide_target_with_action(target, is_revealing, &ObscureAction::Elide)
    }

    /// Returns the unelided variant of this envelope.
    ///
    /// Returns the same envelope if it is already unelided.
    pub fn unelide(&self, envelope: Self) -> Result<Self, EnvelopeError> {
        if self.digest() == envelope.digest() {
            Ok(envelope)
        } else {
            Err(EnvelopeError::InvalidDigest)
        }
    }
}
