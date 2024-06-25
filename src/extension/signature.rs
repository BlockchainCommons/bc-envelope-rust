use anyhow::{bail, Result};
use bc_components::{DigestProvider, Signature, Signer, SigningOptions, Verifier};

use crate::{Envelope, EnvelopeError};
#[cfg(feature = "known_value")]
use crate::extension::known_values;

/// Support for signing envelopes and verifying signatures.
impl Envelope {
    /// Creates a signature for the envelope's subject and returns a new envelope with a `verifiedBy: Signature` assertion.
    ///
    /// - Parameters:
    ///   - private_key: The signer's `SigningPrivateKey`
    ///
    /// - Returns: The signed envelope.
    pub fn add_signature(&self, private_key: &dyn Signer) -> Self {
        self.add_signature_opt(private_key, None, None)
    }

    #[doc(hidden)]
    /// Creates a signature for the envelope's subject and returns a new envelope with a `verifiedBy: Signature` assertion.
    ///
    /// - Parameters:
    ///   - private_key: A signer's `PrivateKeyBase` or `SigningPrivateKey`.
    ///   - options: Optional signing options.
    ///   - note: Optional text note to add to the `Signature`
    ///
    /// - Returns: The signed envelope.
    pub fn add_signature_opt(
        &self,
        private_key: &dyn Signer,
        options: Option<SigningOptions>,
        note: Option<&str>,
    ) -> Self {
        let mut assertions: Vec<Envelope> = vec![];
        if let Some(note) = note {
            assertions.push(Self::new_assertion(known_values::NOTE, note));
        }
        self.add_signatures_with_uncovered_assertions(&assertions, private_key, options)
    }

    #[doc(hidden)]
    /// Creates several signatures for the envelope's subject and returns a new envelope with additional `verifiedBy: Signature` assertions.
    ///
    /// - Parameters:
    ///  - private_keys: An array of signers' `SigningPrivateKey`s.
    ///
    /// - Returns: The signed envelope.
    pub fn add_signatures(&self, private_keys: &[&dyn Signer]) -> Self {
        private_keys.iter().fold(self.clone(), |envelope, private_key| {
            envelope.add_signature(*private_key)
        })
    }

    #[doc(hidden)]
    /// Creates several signatures for the envelope's subject and returns a new envelope with additional `verifiedBy: Signature` assertions.
    ///
    /// - Parameters:
    ///   - private_keys: An array of signers' `SigningPrivateKey`s and optional `SigningOptions`.
    ///
    /// - Returns: The signed envelope.
    pub fn add_signatures_opt(&self, private_keys: &[(&dyn Signer, Option<SigningOptions>)]) -> Self {
        private_keys.iter().fold(self.clone(), |envelope, (private_key, options)| {
            envelope.add_signature_opt(*private_key, options.clone(), None)
        })
    }

    #[doc(hidden)]
    /// Creates a signature for the envelope's subject and returns a new envelope with a `verifiedBy: Signature` assertion.
    ///
    /// - Parameters:
    ///   - private_key: The signer's `PrivateKeyBase`
    ///   - uncoveredAssertions: Assertions to add to the `Signature`.
    ///   - rng: The random number generator to use.
    ///
    /// - Returns: The signed envelope.
    fn add_signatures_with_uncovered_assertions(
        &self,
        uncovered_assertions: &[Self],
        private_key: &dyn Signer,
        options: Option<SigningOptions>,
    ) -> Self
    {
        let digest = *self.subject().digest().data();
        let signature = Envelope::new(private_key.sign_with_options(&digest as &dyn AsRef<[u8]>, options).unwrap())
            .add_assertion_envelopes(uncovered_assertions)
            .unwrap();
        self.add_assertion(known_values::VERIFIED_BY, signature)
    }

    /// Convenience constructor for a `verifiedBy: Signature` assertion envelope.
    ///
    /// - Parameters:
    ///   - signature: The `Signature` for the object.
    ///   - note: An optional note to be added to the `Signature`.
    ///
    /// - Returns: The new assertion envelope.
    pub fn make_verified_by_signature(
        &self,
        signature: &Signature,
        note: Option<&str>
    ) -> Self {
        let verified_by = known_values::VERIFIED_BY;
        let mut envelope = Envelope::new_assertion(verified_by, signature.clone());
        if let Some(note) = note {
            envelope = envelope.add_assertion(known_values::NOTE, note);
        }
        envelope
    }

    /// Returns an array of `Signature`s from all of the envelope's `verifiedBy` predicates.
    ///
    /// - Throws: Throws an exception if any `verifiedBy` assertion doesn't contain a
    /// valid `Signature` as its object.
    pub fn signatures(&self) -> Result<Vec<Signature>> {
        let verified_by = known_values::VERIFIED_BY;
        self
            .assertions_with_predicate(verified_by).into_iter()
            .map(|assertion| assertion.as_object().unwrap().extract_subject::<Signature>())
            .collect()
    }

    /// Returns whether the given signature is valid.
    ///
    /// - Parameters:
    ///   - signature: The `Signature` to be checked.
    ///   - public_key: The potential signer's `Verifier`.
    ///
    /// - Returns: `true` if the signature is valid for this envelope's subject, `false` otherwise.
    pub fn is_verified_signature(
        &self,
        signature: &Signature,
        public_key: &dyn Verifier,
    ) -> bool {
        self.is_signature_from_key(signature, public_key)
    }

    /// Checks whether the given signature is valid for the given public key.
    ///
    /// Used for chaining a series of operations that include validating signatures.
    ///
    /// - Parameters:
    ///   - signature: The `Signature` to be checked.
    ///   - public_key: The potential signer's `Verifier`.
    ///
    /// - Returns: This envelope.
    ///
    /// - Throws: Throws `EnvelopeError.unverifiedSignature` if the signature is not valid.
    /// valid.
    pub fn verify_signature(
        &self,
        signature: &Signature,
        public_key: &dyn Verifier,
    ) -> Result<Self> {
        self.verify_signature_from_key(signature, public_key)
    }

    /// Returns whether the envelope's subject has a valid signature from the given public key.
    ///
    /// - Parameters:
    ///   - public_key: The potential signer's `Verifier`.
    ///
    /// - Returns: `true` if the signature is valid for this envelope's subject, `false` otherwise.
    ///
    /// - Throws: Throws an exception if any `verifiedBy` assertion doesn't contain a
    /// valid `Signature` as its object.
    pub fn has_signature_from(
        &self,
        public_key: &dyn Verifier,
    ) -> Result<bool> {
        self.has_some_signature_from_key(public_key)
    }

    /// Returns whether the envelope's subject has a valid signature from the given public key.
    ///
    /// Used for chaining a series of operations that include validating signatures.
    ///
    /// - Parameters:
    ///   - public_key: The potential signer's `Verifier`.
    ///
    /// - Returns: This envelope.
    ///
    /// - Throws: Throws `EnvelopeError.unverifiedSignature` if the signature is not valid.
    /// valid.
    pub fn verify_signature_from(
        &self,
        public_key: &dyn Verifier,
    ) -> Result<Self> {
        self.verify_has_some_signature_from_key(public_key)
    }

    /// Checks whether the envelope's subject has a set of signatures.
    pub fn has_signatures_from(
        &self,
        public_keys: &[&dyn Verifier],
    ) -> Result<bool> {
        self.has_signatures_from_threshold(public_keys, None)
    }

    /// Returns whether the envelope's subject has some threshold of signatures.
    ///
    /// If `threshold` is `nil`, then *all* signers in `public_keys` must have
    /// signed. If `threshold` is `1`, then at least one signer must have signed.
    ///
    /// - Parameters:
    ///   - public_keys: An array of potential signers' `Verifier`s.
    ///   - threshold: Optional minimum number of signers.
    ///
    /// - Returns: `true` if the threshold of valid signatures is met, `false` otherwise.
    ///
    /// - Throws: Throws an exception if any `verifiedBy` assertion doesn't contain a
    /// valid `Signature` as its object.
    pub fn has_signatures_from_threshold(
        &self,
        public_keys: &[&dyn Verifier],
        threshold: Option<usize>,
    ) -> Result<bool> {
        self.has_signatures_from_keys_threshold(public_keys, threshold)
    }

    /// Checks whether the envelope's subject has a set of signatures.
    pub fn verify_signatures_from(
        &self,
        public_keys: &[&dyn Verifier],
    ) -> Result<Self> {
        self.verify_signatures_from_threshold(public_keys, None)
    }

    /// Checks whether the envelope's subject has some threshold of signatures.
    ///
    /// If `threshold` is `nil`, then *all* signers in `public_keys` must have
    /// signed. If `threshold` is `1`, then at least one signer must have signed.
    ///
    /// Used for chaining a series of operations that include validating signatures.
    ///
    /// - Parameters:
    ///   - public_keys: An array of potential signers' `Verifier`s.
    ///   - threshold: Optional minimum number of signers.
    ///
    /// - Returns: This envelope.
    ///
    /// - Throws: Throws an exception if the threshold of valid signatures is not met.
    pub fn verify_signatures_from_threshold(
        &self,
        public_keys: &[&dyn Verifier],
        threshold: Option<usize>,
    ) -> Result<Self> {
        self.verify_signatures_from_keys_threshold(public_keys, threshold)
    }
}

#[doc(hidden)]
impl Envelope {
    fn is_signature_from_key(
        &self,
        signature: &Signature,
        key: &dyn Verifier
    ) -> bool {
        key.verify(signature, self.subject().digest().as_ref())
    }

    fn verify_signature_from_key(
        &self,
        signature: &Signature,
        key: &dyn Verifier,
    ) -> Result<Self> {
        if !self.is_signature_from_key(signature, key) {
            bail!(EnvelopeError::UnverifiedSignature);
        }
        Ok(self.clone())
    }

    fn has_some_signature_from_key(
        &self,
        key: &dyn Verifier
    ) -> Result<bool> {
        let signatures = self.signatures();
        let signatures = signatures?;
        let result = signatures.iter().any(|signature| {
            self.is_signature_from_key(signature, key)
        });
        Ok(result)
    }

    fn verify_has_some_signature_from_key(
        &self,
        key: &dyn Verifier
    ) -> Result<Self> {
        if !self.has_some_signature_from_key(key)? {
            bail!(EnvelopeError::UnverifiedSignature);
        }
        Ok(self.clone())
    }

    fn has_signatures_from_keys_threshold(
        &self,
        keys: &[&dyn Verifier],
        threshold: Option<usize>
    ) -> Result<bool> {
        let threshold = threshold.unwrap_or(keys.len());
        let mut count = 0;
        for key in keys {
            if self.clone().has_some_signature_from_key(*key)? {
                count += 1;
                if count >= threshold {
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }

    fn verify_signatures_from_keys_threshold(
        &self,
        keys: &[&dyn Verifier],
        threshold: Option<usize>
    ) -> Result<Self> {
        if !self.has_signatures_from_keys_threshold(keys, threshold)? {
            bail!(EnvelopeError::UnverifiedSignature);
        }
        Ok(self.clone())
    }
}
