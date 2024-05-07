use anyhow::{bail, Result};
use bc_components::{PrivateKeyBase, Signature, PublicKeyBase, SigningPublicKey, DigestProvider};
use bc_rand::{RandomNumberGenerator, SecureRandomNumberGenerator};

use crate::{Envelope, EnvelopeError};
#[cfg(feature = "known_value")]
use crate::extension::known_values;

/// Support for signing envelopes and verifying signatures.
impl Envelope {
    /// Creates a signature for the envelope's subject and returns a new envelope with a `verifiedBy: Signature` assertion.
    ///
    /// - Parameters:
    ///   - private_key: The signer's `PrivateKeyBase`
    ///
    /// - Returns: The signed envelope.
    pub fn add_signature_with(&self, private_key: &PrivateKeyBase) -> Self {
        self.add_signature_with_opt(private_key, None, [])
    }

    #[doc(hidden)]
    /// Creates a signature for the envelope's subject and returns a new envelope with a `verifiedBy: Signature` assertion.
    ///
    /// - Parameters:
    ///   - private_key: The signer's `PrivateKeyBase`
    ///   - note: Optional text note to add to the `Signature`
    ///   - rng: The random number generator to use.
    ///
    /// - Returns: The signed envelope.
    pub fn add_signature_with_opt_using<D>(
        &self,
        private_key: &PrivateKeyBase,
        note: Option<&str>,
        tag: D,
        rng: &mut impl RandomNumberGenerator
    ) -> Self
    where
        D: AsRef<[u8]>,
    {
        let mut assertions: Vec<Envelope> = vec![];
        if let Some(note) = note {
            assertions.push(Self::new_assertion(known_values::NOTE, note));
        }
        self.add_signatures_with_uncovered_assertions_using(private_key, &assertions, tag, rng)
    }

    /// Creates a signature for the envelope's subject and returns a new envelope with a `verifiedBy: Signature` assertion.
    ///
    /// - Parameters:
    ///   - private_key: The signer's `PrivateKeyBase`
    ///   - note: Optional text note to add to the `Signature`
    ///
    /// - Returns: The signed envelope.
    pub fn add_signature_with_opt<D>(
        &self,
        private_key: &PrivateKeyBase,
        note: Option<&str>,
        tag: D,
    ) -> Self
    where
        D: AsRef<[u8]>,
    {
        let mut rng = SecureRandomNumberGenerator;
        self.add_signature_with_opt_using(private_key, note, tag, &mut rng)
    }

    #[doc(hidden)]
    pub fn add_signature_with_using(
        &self,
        private_key: &PrivateKeyBase,
        rng: &mut impl RandomNumberGenerator
    ) -> Self {
        self.add_signature_with_opt_using(private_key, None, [], rng)
    }

    pub fn add_signatures_with_keys<T>(
        &self,
        private_keys: &[T],
    ) -> Self
    where
        T: AsRef<PrivateKeyBase>,
    {
        self.add_signatures_with_keys_opt(private_keys, [])
    }

    /// Creates several signatures for the envelope's subject and returns a new envelope with additional `verifiedBy: Signature` assertions.
    ///
    /// - Parameters:
    ///   - private_key: An array of signers' `PrivateKeyBase`s.
    ///
    /// - Returns: The signed envelope.
    pub fn add_signatures_with_keys_opt<D, T>(
        &self,
        private_keys: &[T],
        tag: D,
    ) -> Self
    where
        D: AsRef<[u8]> + Clone,
        T: AsRef<PrivateKeyBase>,
    {
        let mut rng = SecureRandomNumberGenerator;
        self.add_signatures_with_keys_opt_using(private_keys, tag, &mut rng)
    }

    #[doc(hidden)]
    /// Creates several signatures for the envelope's subject and returns a new envelope with additional `verifiedBy: Signature` assertions.
    ///
    /// - Parameters:
    ///   - private_key: An array of signers' `PrivateKeyBase`s.
    ///   - rng: The random number generator to use.
    ///
    /// - Returns: The signed envelope.
    pub fn add_signatures_with_keys_opt_using<D, T>(
        &self,
        private_keys: &[T],
        tag: D,
        rng: &mut impl RandomNumberGenerator
    ) -> Self
    where
        D: AsRef<[u8]> + Clone,
        T: AsRef<PrivateKeyBase>,
    {
        private_keys.iter().fold(self.clone(), |envelope, private_key| {
            envelope.add_signature_with_opt_using(private_key.as_ref(), None, tag.clone(), rng)
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
    pub fn add_signatures_with_uncovered_assertions_using<D>(
        &self,
        private_key: &PrivateKeyBase,
        uncovered_assertions: &[Self],
        tag: D,
        rng: &mut impl RandomNumberGenerator
    ) -> Self
    where
        D: AsRef<[u8]>,
    {
        let signing_private_key = private_key.signing_private_key();
        let digest = *self.subject().digest().data();
        let signature = Envelope::new(signing_private_key.schnorr_sign_using(digest, tag, rng))
            .add_assertion_envelopes(uncovered_assertions)
            .unwrap();
        self.add_assertion(known_values::VERIFIED_BY, signature)
    }

    /// Creates a signature for the envelope's subject and returns a new envelope with a `verifiedBy: Signature` assertion.
    ///
    /// - Parameters:
    ///   - private_key: The signer's `PrivateKeyBase`
    ///   - uncoveredAssertions: Assertions to add to the `Signature`.
    ///
    /// - Returns: The signed envelope.
    pub fn add_signatures_with_uncovered_assertions<D>(
        &self,
        private_key: &PrivateKeyBase,
        uncovered_assertions: &[Self],
        tag: D,
    ) -> Self
    where
        D: AsRef<[u8]>,
    {
        let mut rng = SecureRandomNumberGenerator;
        self.add_signatures_with_uncovered_assertions_using(private_key, uncovered_assertions, tag, &mut rng)
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
    ///   - public_key: The potential signer's `PublicKeyBase`.
    ///
    /// - Returns: `true` if the signature is valid for this envelope's subject, `false` otherwise.
    pub fn is_verified_signature(
        &self,
        signature: &Signature,
        public_key: &PublicKeyBase,
    ) -> bool {
        self.is_signature_from_key(signature, public_key.signing_public_key())
    }

    /// Checks whether the given signature is valid for the given public key.
    ///
    /// Used for chaining a series of operations that include validating signatures.
    ///
    /// - Parameters:
    ///   - signature: The `Signature` to be checked.
    ///   - public_key: The potential signer's `PublicKeyBase`.
    ///
    /// - Returns: This envelope.
    ///
    /// - Throws: Throws `EnvelopeError.unverifiedSignature` if the signature is not valid.
    /// valid.
    pub fn verify_signature(
        &self,
        signature: &Signature,
        public_key: &PublicKeyBase,
    ) -> Result<Self> {
        self.verify_signature_from_key(signature, public_key.signing_public_key())
    }

    /// Returns whether the envelope's subject has a valid signature from the given public key.
    ///
    /// - Parameters:
    ///   - public_key: The potential signer's `PublicKeyBase`.
    ///
    /// - Returns: `true` if the signature is valid for this envelope's subject, `false` otherwise.
    ///
    /// - Throws: Throws an exception if any `verifiedBy` assertion doesn't contain a
    /// valid `Signature` as its object.
    pub fn has_signature_from(
        &self,
        public_key: &PublicKeyBase,
    ) -> Result<bool> {
        self.has_some_signature_from_key(public_key.signing_public_key())
    }

    /// Returns whether the envelope's subject has a valid signature from the given public key.
    ///
    /// Used for chaining a series of operations that include validating signatures.
    ///
    /// - Parameters:
    ///   - public_key: The potential signer's `PublicKeyBase`.
    ///
    /// - Returns: This envelope.
    ///
    /// - Throws: Throws `EnvelopeError.unverifiedSignature` if the signature is not valid.
    /// valid.
    pub fn verify_signature_from(
        &self,
        public_key: &PublicKeyBase,
    ) -> Result<Self> {
        self.verify_has_some_signature_from_key(public_key.signing_public_key())
    }

    /// Checks whether the envelope's subject has a set of signatures.
    pub fn has_signatures_from<T>(
        &self,
        public_keys: &[T],
    ) -> Result<bool>
    where
        T: AsRef<PublicKeyBase>,
    {
        self.has_signatures_from_threshold(public_keys, None)
    }

    /// Returns whether the envelope's subject has some threshold of signatures.
    ///
    /// If `threshold` is `nil`, then *all* signers in `public_keys` must have
    /// signed. If `threshold` is `1`, then at least one signer must have signed.
    ///
    /// - Parameters:
    ///   - public_keys: An array of potential signers' `PublicKeyBase`s.
    ///   - threshold: Optional minimum number of signers.
    ///
    /// - Returns: `true` if the threshold of valid signatures is met, `false` otherwise.
    ///
    /// - Throws: Throws an exception if any `verifiedBy` assertion doesn't contain a
    /// valid `Signature` as its object.
    pub fn has_signatures_from_threshold<T>(
        &self,
        public_keys: &[T],
        threshold: Option<usize>,
    ) -> Result<bool>
    where
        T: AsRef<PublicKeyBase>,
    {
        let public_keys = public_keys
            .iter()
            .map(|public_key| public_key.as_ref())
            .collect::<Vec<_>>();
        self.has_signatures_from_keys_threshold(&public_keys, threshold)
    }

    /// Checks whether the envelope's subject has a set of signatures.
    pub fn verify_signatures_from<T>(
        &self,
        public_keys: &[T],
    ) -> Result<Self>
    where
        T: AsRef<PublicKeyBase>,
    {
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
    ///   - public_keys: An array of potential signers' `PublicKeyBase`s.
    ///   - threshold: Optional minimum number of signers.
    ///
    /// - Returns: This envelope.
    ///
    /// - Throws: Throws an exception if the threshold of valid signatures is not met.
    pub fn verify_signatures_from_threshold<T>(
        &self,
        public_keys: &[T],
        threshold: Option<usize>,
    ) -> Result<Self>
    where
        T: AsRef<PublicKeyBase>,
    {
        let public_keys = public_keys
            .iter()
            .map(|public_key| public_key.as_ref())
            .collect::<Vec<_>>();
        self.verify_signatures_from_keys_threshold(&public_keys, threshold)
    }
}

#[doc(hidden)]
impl Envelope {
    fn is_signature_from_key<T>(
        &self,
        signature: &Signature,
        key: T
    ) -> bool
    where
        T: AsRef<SigningPublicKey>,
    {
        key.as_ref().verify(signature, self.subject().digest().as_ref())
    }

    fn verify_signature_from_key(
        &self,
        signature: &Signature,
        key: &SigningPublicKey
    ) -> Result<Self> {
        if !self.is_signature_from_key(signature, key) {
            bail!(EnvelopeError::UnverifiedSignature);
        }
        Ok(self.clone())
    }

    fn has_some_signature_from_key<T>(
        &self,
        key: T
    ) -> Result<bool>
    where
        T: AsRef<SigningPublicKey>,
    {
        let signatures = self.signatures();
        let signatures = signatures?;
        let result = signatures.iter().any(|signature| {
            self.is_signature_from_key(signature, key.as_ref())
        });
        Ok(result)
    }

    fn verify_has_some_signature_from_key(
        &self,
        key: &SigningPublicKey
    ) -> Result<Self> {
        if !self.has_some_signature_from_key(key)? {
            bail!(EnvelopeError::UnverifiedSignature);
        }
        Ok(self.clone())
    }

    fn has_signatures_from_keys_threshold<T>(
        &self,
        keys: &[T],
        threshold: Option<usize>
    ) -> Result<bool>
    where
        T: AsRef<SigningPublicKey>,
    {
        let threshold = threshold.unwrap_or(keys.len());
        let mut count = 0;
        for key in keys {
            if self.clone().has_some_signature_from_key(key)? {
                count += 1;
                if count >= threshold {
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }

    fn verify_signatures_from_keys_threshold<T>(
        &self,
        keys: &[T],
        threshold: Option<usize>
    ) -> Result<Self>
    where
        T: AsRef<SigningPublicKey>,
    {
        if !self.has_signatures_from_keys_threshold(keys, threshold)? {
            bail!(EnvelopeError::UnverifiedSignature);
        }
        Ok(self.clone())
    }
}
