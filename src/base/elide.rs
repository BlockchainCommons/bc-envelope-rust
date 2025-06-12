use std::collections::HashSet;

use anyhow::{Result, bail};
use bc_components::{Digest, DigestProvider};
#[cfg(feature = "encrypt")]
use bc_components::{Nonce, SymmetricKey};
#[cfg(feature = "encrypt")]
use dcbor::prelude::*;

use super::envelope::EnvelopeCase;
use crate::{Assertion, Envelope, Error};

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
            bail!(Error::InvalidDigest)
        }
    }
}
