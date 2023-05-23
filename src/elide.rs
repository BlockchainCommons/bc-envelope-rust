use std::{rc::Rc, collections::HashSet};

use bc_components::{SymmetricKey, DigestProvider, Digest};
use dcbor::{CBORTaggedEncodable, CBOREncodable};

use crate::Envelope;

/*
```swift
// MARK: - High-Level Elision Functions

// An action which obscures (elides, encrypts, or compresses) an envelope.
public enum ObscureAction {
    case elide
    case encrypt(SymmetricKey)
    case compress
}

public extension Envelope {
    /// Returns the elided variant of this envelope.
    ///
    /// Returns the same envelope if it is already elided.
    func elide() -> Envelope {
        switch self {
        case .elided:
            return self
        default:
            return Envelope(elided: self.digest)
        }
    }
}

public extension Envelope {
    /// Returns a version of this envelope with elements in the `target` set elided.
    ///
    /// - Parameters:
    ///   - target: The target set of digests.
    ///   - action: If provided, perform the specified action (encryption or compression) instead of elision.
    ///
    /// - Returns: The elided envelope.
    func elideRemoving(_ target: Set<Digest>, action: ObscureAction = .elide) throws -> Envelope {
        try elide(target, isRevealing: false, action: action)
    }

    /// Returns a version of this envelope with elements in the `target` set elided.
    ///
    /// - Parameters:
    ///   - target: An array of `DigestProvider`s.
    ///   - action: If provided, perform the specified action (encryption or compression) instead of elision.
    ///
    /// - Returns: The elided envelope.
    func elideRemoving(_ target: [DigestProvider], action: ObscureAction = .elide) throws -> Envelope {
        try elide(target, isRevealing: false, action: action)
    }

    /// Returns a version of this envelope with the target element elided.
    ///
    /// - Parameters:
    ///   - target: A `DigestProvider`.
    ///   - key: If provided, encrypt the targeted element using the `SymmetricKey` instead of eliding it.
    ///
    /// - Returns: The elided envelope.
    func elideRemoving(_ target: DigestProvider, action: ObscureAction = .elide) throws -> Envelope {
        try elide(target, isRevealing: false, action: action)
    }

    /// Returns a version of this envelope with elements *not* in the `target` set elided.
    ///
    /// - Parameters:
    ///   - target: The target set of digests.
    ///   - action: If provided, perform the specified action (encryption or compression) instead of elision.
    ///
    /// - Returns: The elided envelope.
    func elideRevealing(_ target: Set<Digest>, action: ObscureAction = .elide) throws -> Envelope {
        try elide(target, isRevealing: true, action: action)
    }

    /// Returns a version of this envelope with elements *not* in the `target` set elided.
    ///
    /// - Parameters:
    ///   - target: An array of `DigestProvider`s.
    ///   - action: If provided, perform the specified action (encryption or compression) instead of elision.
    ///
    /// - Returns: The elided envelope.
    func elideRevealing(_ target: [DigestProvider], action: ObscureAction = .elide) throws -> Envelope {
        try elide(target, isRevealing: true, action: action)
    }

    /// Returns a version of this envelope with all elements *except* the target element elided.
    ///
    /// - Parameters:
    ///   - target: A `DigestProvider`.
    ///   - key: If provided, encrypt the targeted element using the `SymmetricKey` instead of eliding it.
    ///
    /// - Returns: The elided envelope.
    func elideRevealing(_ target: DigestProvider, action: ObscureAction = .elide) throws -> Envelope {
        try elide(target, isRevealing: true, action: action)
    }
}

// MARK: - Utility Elision Functions

public extension Envelope {
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
    ///   - action: If provided, perform the specified action (encryption or compression) instead of elision.
    ///
    /// - Returns: The elided envelope.
    func elide(_ target: Set<Digest>, isRevealing: Bool, action: ObscureAction = .elide) throws -> Envelope {
        let result: Envelope
        if target.contains(digest) != isRevealing {
            switch action {
            case .elide:
                result = elide()
            case .encrypt(let key):
                let message = key.encrypt(plaintext: self.taggedCBOR.cborData, digest: self.digest)
                result = try Envelope(encryptedMessage: message)
            case .compress:
                result = try compress()
            }
        } else if case .assertion(let assertion) = self {
            let predicate = try assertion.predicate.elide(target, isRevealing: isRevealing, action: action)
            let object = try assertion.object.elide(target, isRevealing: isRevealing, action: action)
            let elidedAssertion = Assertion(predicate: predicate, object: object)
            assert(elidedAssertion == assertion)
            result = Envelope(assertion: elidedAssertion)
        } else if case .node(let subject, let assertions, _) = self {
            let elidedSubject = try subject.elide(target, isRevealing: isRevealing, action: action)
            assert(elidedSubject.digest == subject.digest)
            let elidedAssertions = try assertions.map { assertion in
                let elidedAssertion = try assertion.elide(target, isRevealing: isRevealing, action: action)
                assert(elidedAssertion.digest == assertion.digest)
                return elidedAssertion
            }
            result = Envelope(subject: elidedSubject, uncheckedAssertions: elidedAssertions)
        } else if case .wrapped(let envelope, _) = self {
            let elidedEnvelope = try envelope.elide(target, isRevealing: isRevealing, action: action)
            assert(elidedEnvelope.digest == envelope.digest)
            result = Envelope(wrapped: elidedEnvelope)
        } else {
            result = self
        }
        assert(result.digest == digest)
        return result
    }

    /// Returns an elided version of this envelope.
    ///
    /// - Parameters:
    ///   - target: An array of `DigestProvider`s.
    ///   - isRevealing: If `true`, the target set contains the digests of the elements to
    ///   leave revealed. If it is `false`, the target set contains the digests of the
    ///   elements to elide.
    ///   - action: If provided, perform the specified action (encryption or compression) instead of elision.
    ///
    /// - Returns: The elided envelope.
    func elide(_ target: [DigestProvider], isRevealing: Bool, action: ObscureAction = .elide) throws -> Envelope {
        try elide(Set(target.map { $0.digest }), isRevealing: isRevealing, action: action)
    }

    /// Returns an elided version of this envelope.
    ///
    /// - Parameters:
    ///   - target: A `DigestProvider`.
    ///   - isRevealing: If `true`, the target is the element to leave revealed, eliding
    ///   all others. If it is `false`, the target is the element to elide, leaving all
    ///   others revealed.
    ///   - action: If provided, perform the specified action (encryption or compression) instead of elision.
    ///
    /// - Returns: The elided envelope.
    func elide(_ target: DigestProvider, isRevealing: Bool, action: ObscureAction = .elide) throws -> Envelope {
        try elide([target], isRevealing: isRevealing, action: action)
    }
}

// MARK: - Uneliding an Envelope

public extension Envelope {
    /// Returns the unelided variant of this envelope.
    ///
    /// Throws an exception if the digest of the unelided version does not match.
    func unelide(_ envelope: Envelope) throws -> Envelope {
        guard digest == envelope.digest else {
            throw EnvelopeError.invalidDigest
        }
        return envelope
    }
}
```
 */

pub enum ObscureAction {
    Elide,
    Encrypt(SymmetricKey),
    Compress,
}

impl Envelope {
    /// Returns the elided variant of this envelope.
    ///
    /// Returns the same envelope if it is already elided.
    pub fn elide(self: Rc<Self>) -> Rc<Self> {
        match *self {
            Envelope::Elided(_) => self.clone(),
            _ => Envelope::new_elided(self.digest())
        }
    }
}

impl Envelope {
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
    ///   - action: If provided, perform the specified action (encryption or compression) instead of elision.
    ///
    /// - Returns: The elided envelope.
    pub fn elide_target_set(self: Rc<Self>, target: &HashSet<Digest>, is_revealing: bool, action: &ObscureAction) -> Rc<Envelope> {
        if target.contains(self.digest_ref()) != is_revealing {
            match action {
                ObscureAction::Elide => self.elide(),
                ObscureAction::Encrypt(key) => {
                    let message = key.encrypt(&self.tagged_cbor().cbor_data(), Some(self.digest_ref()));
                    Envelope::new_encrypted(message)
                },
                ObscureAction::Compress => self.compress(),
            }
        } else if let Envelope::Assertion(assertion) = self {
            let predicate = assertion.predicate.elide(target, is_revealing, action);
            let object = assertion.object.elide(target, is_revealing, action);
            let elided_assertion = Assertion::new(predicate, object);
            assert!(elided_assertion == assertion);
            Envelope::new_assertion(elided_assertion)
        } else if let Envelope::Node(subject, assertions, _) = self {
            let elided_subject = subject.elide(target, is_revealing, action);
            assert!(elided_subject.digest() == subject.digest());
            let elided_assertions = assertions.iter().map(|assertion| {
                let elided_assertion = assertion.elide(target, is_revealing, action);
                assert!(elided_assertion.digest() == assertion.digest());
                elided_assertion
            }).collect();
            Envelope::new_node(elided_subject, elided_assertions)
        } else if let Envelope::Wrapped(envelope, _) = self {
            let elided_envelope = envelope.elide(target, is_revealing, action);
            assert!(elided_envelope.digest() == envelope.digest());
            Envelope::new_wrapped(elided_envelope)
        } else {
            self
        }
        todo!()
    }
}
