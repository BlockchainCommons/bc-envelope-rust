use std::{rc::Rc, collections::HashSet};

use bc_components::{SymmetricKey, DigestProvider, Digest, Nonce};
use dcbor::prelude::*;

use crate::{Envelope, Assertion, EnvelopeError};

/// An action to perform on a target set in an envelope.
pub enum ObscureAction {
    /// Elide the target, leaving only its digest.
    Elide,

    /// Encrypt the target using the specified key.
    Encrypt(SymmetricKey),

    #[cfg(feature = "compress")]
    /// Compress the target.
    Compress,
}

/// Support for eliding elements from envelopes.
///
/// This includes eliding, encrypting and compressing (obscuring) elements.
impl Envelope {
    /// Returns the elided variant of this envelope.
    ///
    /// Returns the same envelope if it is already elided.
    pub fn elide(self: Rc<Self>) -> Rc<Self> {
        match *self {
            Self::Elided(_) => self.clone(),
            _ => Rc::new(Self::new_elided(self.digest().into_owned()))
        }
    }

    /// Returns a version of this envelope with elements in the `target` set elided.
    ///
    /// - Parameters:
    ///   - target: The target set of digests.
    ///   - action: Perform the specified action (elision, encryption or compression).
    ///
    /// - Returns: The elided envelope.
    pub fn elide_removing_set_with_action(self: Rc<Self>, target: &HashSet<Digest>, action: &ObscureAction) -> Rc<Self> {
        self.elide_set_with_action(target, false, action)
    }

    /// Returns a version of this envelope with elements in the `target` set elided.
    ///
    /// - Parameters:
    ///   - target: The target set of digests.
    ///   - action: Perform the specified action (elision, encryption or compression).
    ///
    /// - Returns: The elided envelope.
    pub fn elide_removing_set(self: Rc<Self>, target: &HashSet<Digest>) -> Rc<Self> {
        self.elide_set(target, false)
    }

    /// Returns a version of this envelope with elements in the `target` set elided.
    ///
    /// - Parameters:
    ///   - target: An array of `DigestProvider`s.
    ///   - action: Perform the specified action (elision, encryption or compression).
    ///
    /// - Returns: The elided envelope.
    pub fn elide_removing_array_with_action(self: Rc<Self>, target: &[&dyn DigestProvider], action: &ObscureAction) -> Rc<Self> {
        self.elide_array_with_action(target, false, action)
    }

    /// Returns a version of this envelope with elements in the `target` set elided.
    ///
    /// - Parameters:
    ///   - target: An array of `DigestProvider`s.
    ///   - action: Perform the specified action (elision, encryption or compression).
    ///
    /// - Returns: The elided envelope.
    pub fn elide_removing_array(self: Rc<Self>, target: &[&dyn DigestProvider]) -> Rc<Self> {
        self.elide_array(target, false)
    }

    /// Returns a version of this envelope with the target element elided.
    ///
    /// - Parameters:
    ///   - target: A `DigestProvider`.
    ///   - action: Perform the specified action (elision, encryption or compression).
    ///
    /// - Returns: The elided envelope.
    pub fn elide_removing_target_with_action(self: Rc<Self>, target: &dyn DigestProvider, action: &ObscureAction) -> Rc<Self> {
        self.elide_target_with_action(target, false, action)
    }

    /// Returns a version of this envelope with the target element elided.
    ///
    /// - Parameters:
    ///   - target: A `DigestProvider`.
    ///
    /// - Returns: The elided envelope.
    pub fn elide_removing_target(self: Rc<Self>, target: &dyn DigestProvider) -> Rc<Self> {
        self.elide_target(target, false)
    }

    /// Returns a version of this envelope with elements *not* in the `target` set elided.
    ///
    /// - Parameters:
    ///   - target: The target set of digests.
    ///   - action: Perform the specified action (elision, encryption or compression).
    ///
    /// - Returns: The elided envelope.
    pub fn elide_revealing_set_with_action(self: Rc<Self>, target: &HashSet<Digest>, action: &ObscureAction) -> Rc<Self> {
        self.elide_set_with_action(target, true, action)
    }

    /// Returns a version of this envelope with elements *not* in the `target` set elided.
    ///
    /// - Parameters:
    ///   - target: The target set of digests.
    ///
    /// - Returns: The elided envelope.
    pub fn elide_revealing_set(self: Rc<Self>, target: &HashSet<Digest>) -> Rc<Self> {
        self.elide_set(target, true)
    }

    /// Returns a version of this envelope with elements *not* in the `target` set elided.
    ///
    /// - Parameters:
    ///   - target: An array of `DigestProvider`s.
    ///   - action: Perform the specified action (elision, encryption or compression).
    ///
    /// - Returns: The elided envelope.
    pub fn elide_revealing_array_with_action(self: Rc<Self>, target: &[&dyn DigestProvider], action: &ObscureAction) -> Rc<Self> {
        self.elide_array_with_action(target, true, action)
    }

    /// Returns a version of this envelope with elements *not* in the `target` set elided.
    ///
    /// - Parameters:
    ///   - target: An array of `DigestProvider`s.
    ///
    /// - Returns: The elided envelope.
    pub fn elide_revealing_array(self: Rc<Self>, target: &[&dyn DigestProvider]) -> Rc<Self> {
        self.elide_array(target, true)
    }

    /// Returns a version of this envelope with all elements *except* the target element elided.
    ///
    /// - Parameters:
    ///   - target: A `DigestProvider`.
    ///   - action: Perform the specified action (elision, encryption or compression).
    ///
    /// - Returns: The elided envelope.
    pub fn elide_revealing_target_with_action(self: Rc<Self>, target: &dyn DigestProvider, action: &ObscureAction) -> Rc<Self> {
        self.elide_target_with_action(target, true, action)
    }

    /// Returns a version of this envelope with all elements *except* the target element elided.
    ///
    /// - Parameters:
    ///   - target: A `DigestProvider`.
    ///
    /// - Returns: The elided envelope.
    pub fn elide_revealing_target(self: Rc<Self>, target: &dyn DigestProvider) -> Rc<Self> {
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
    pub fn elide_set_with_action(self: Rc<Self>, target: &HashSet<Digest>, is_revealing: bool, action: &ObscureAction) -> Rc<Self> {
        let self_digest = self.digest().into_owned();
        if target.contains(&self_digest) != is_revealing {
            match action {
                ObscureAction::Elide => self.elide(),
                ObscureAction::Encrypt(key) => {
                    let message = key.encrypt(self.tagged_cbor().cbor_data(), Some((&self_digest).into()), None::<Nonce>);
                    Rc::new(Self::new_with_encrypted(message).unwrap())
                },
                #[cfg(feature = "compress")]
                ObscureAction::Compress => self.compress().unwrap(),
            }
        } else if let Self::Assertion(assertion) = &*self {
            let predicate = assertion.predicate().elide_set_with_action(target, is_revealing, action);
            let object = assertion.object().elide_set_with_action(target, is_revealing, action);
            let elided_assertion = Assertion::new(predicate, object);
            assert!(&elided_assertion == assertion);
            Rc::new(Self::new_with_assertion(elided_assertion))
        } else if let Self::Node { subject, assertions, ..} = &*self {
            let elided_subject = subject.clone().elide_set_with_action(target, is_revealing, action);
            assert!(elided_subject.digest() == subject.digest());
            let elided_assertions = assertions.iter().map(|assertion| {
                let elided_assertion = assertion.clone().elide_set_with_action(target, is_revealing, action);
                assert!(elided_assertion.digest() == assertion.digest());
                elided_assertion
            }).collect();
            Rc::new(Self::new_with_unchecked_assertions(elided_subject, elided_assertions))
        } else if let Self::Wrapped { envelope, .. } = &*self {
            let elided_envelope = envelope.clone().elide_set_with_action(target, is_revealing, action);
            assert!(elided_envelope.digest() == envelope.digest());
            Rc::new(Self::new_wrapped(elided_envelope))
        } else {
            self
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
    pub fn elide_set(self: Rc<Self>, target: &HashSet<Digest>, is_revealing: bool) -> Rc<Self> {
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
    pub fn elide_array_with_action(self: Rc<Self>, target: &[&dyn DigestProvider], is_revealing: bool, action: &ObscureAction) -> Rc<Self> {
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
    pub fn elide_array(self: Rc<Self>, target: &[&dyn DigestProvider], is_revealing: bool) -> Rc<Self> {
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
    pub fn elide_target_with_action(self: Rc<Self>, target: &dyn DigestProvider, is_revealing: bool, action: &ObscureAction) -> Rc<Self> {
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
    pub fn elide_target(self: Rc<Self>, target: &dyn DigestProvider, is_revealing: bool) -> Rc<Self> {
        self.elide_target_with_action(target, is_revealing, &ObscureAction::Elide)
    }

    /// Returns the unelided variant of this envelope.
    ///
    /// Returns the same envelope if it is already unelided.
    pub fn unelide(self: Rc<Self>, envelope: Rc<Self>) -> Result<Rc<Self>, EnvelopeError> {
        if self.digest() == envelope.digest() {
            Ok(envelope)
        } else {
            Err(EnvelopeError::InvalidDigest)
        }
    }
}
