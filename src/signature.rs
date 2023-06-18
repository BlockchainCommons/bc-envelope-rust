use std::{rc::Rc};

use bc_components::{PrivateKeyBase, Signature, PublicKeyBase, SigningPublicKey, DigestProvider};
use bc_crypto::{RandomNumberGenerator, SecureRandomNumberGenerator};

use crate::{Envelope, Error, known_values, impl_into_envelope};

/// Support for signing envelopes and verifying signatures.
impl Envelope {
    /// Creates a signature for the envelope's subject and returns a new envelope with a `verifiedBy: Signature` assertion.
    ///
    /// - Parameters:
    ///   - privateKeys: The signer's `PrivateKeyBase`
    ///
    /// - Returns: The signed envelope.
    pub fn sign_with(self: Rc<Self>, private_keys: &PrivateKeyBase) -> Rc<Self> {
        self.sign_with_opt(private_keys, None, [])
    }

    #[doc(hidden)]
    /// Creates a signature for the envelope's subject and returns a new envelope with a `verifiedBy: Signature` assertion.
    ///
    /// - Parameters:
    ///   - privateKeys: The signer's `PrivateKeyBase`
    ///   - note: Optional text note to add to the `Signature`
    ///   - rng: The random number generator to use.
    ///
    /// - Returns: The signed envelope.
    pub fn sign_with_opt_using<D>(
        self: Rc<Self>,
        private_keys: &PrivateKeyBase,
        note: Option<&str>,
        tag: D,
        rng: &mut impl RandomNumberGenerator
    ) -> Rc<Self>
    where
        D: AsRef<[u8]>,
    {
        let mut assertions: Vec<Rc<Envelope>> = vec![];
        if let Some(note) = note {
            assertions.push(Self::new_assertion(known_values::NOTE, note));
        }
        self.sign_with_uncovered_assertions_using(private_keys, &assertions, tag, rng)
    }

    /// Creates a signature for the envelope's subject and returns a new envelope with a `verifiedBy: Signature` assertion.
    ///
    /// - Parameters:
    ///   - privateKeys: The signer's `PrivateKeyBase`
    ///   - note: Optional text note to add to the `Signature`
    ///
    /// - Returns: The signed envelope.
    pub fn sign_with_opt<D>(
        self: Rc<Self>,
        private_keys: &PrivateKeyBase,
        note: Option<&str>,
        tag: D,
    ) -> Rc<Self>
    where
        D: AsRef<[u8]>,
    {
        let mut rng = SecureRandomNumberGenerator;
        self.sign_with_opt_using(private_keys, note, tag, &mut rng)
    }

    #[doc(hidden)]
    pub fn sign_with_using(
        self: Rc<Self>,
        private_keys: &PrivateKeyBase,
        rng: &mut impl RandomNumberGenerator
    ) -> Rc<Self> {
        self.sign_with_opt_using(private_keys, None, [], rng)
    }

    pub fn sign_with_keys(
        self: Rc<Self>,
        private_keys_array: &[&PrivateKeyBase],
    ) -> Rc<Self> {
        self.sign_with_keys_opt(private_keys_array, [])
    }

    /// Creates several signatures for the envelope's subject and returns a new envelope with additional `verifiedBy: Signature` assertions.
    ///
    /// - Parameters:
    ///   - privateKeys: An array of signers' `PrivateKeyBase`s.
    ///
    /// - Returns: The signed envelope.
    pub fn sign_with_keys_opt<D>(
        self: Rc<Self>,
        private_keys_array: &[&PrivateKeyBase],
        tag: D,
    ) -> Rc<Self>
    where
        D: AsRef<[u8]> + Clone,
    {
        let mut rng = SecureRandomNumberGenerator;
        self.sign_with_keys_opt_using(private_keys_array, tag, &mut rng)
    }

    #[doc(hidden)]
    /// Creates several signatures for the envelope's subject and returns a new envelope with additional `verifiedBy: Signature` assertions.
    ///
    /// - Parameters:
    ///   - privateKeys: An array of signers' `PrivateKeyBase`s.
    ///   - rng: The random number generator to use.
    ///
    /// - Returns: The signed envelope.
    pub fn sign_with_keys_opt_using<D>(
        self: Rc<Self>,
        private_keys_array: &[&PrivateKeyBase],
        tag: D,
        rng: &mut impl RandomNumberGenerator
    ) -> Rc<Self>
    where
        D: AsRef<[u8]> + Clone,
    {
        private_keys_array.iter().fold(self, |envelope, private_key| {
            envelope.sign_with_opt_using(private_key, None, tag.clone(), rng)
        })
    }

    #[doc(hidden)]
    /// Creates a signature for the envelope's subject and returns a new envelope with a `verifiedBy: Signature` assertion.
    ///
    /// - Parameters:
    ///   - privateKeys: The signer's `PrivateKeyBase`
    ///   - uncoveredAssertions: Assertions to add to the `Signature`.
    ///   - rng: The random number generator to use.
    ///
    /// - Returns: The signed envelope.
    pub fn sign_with_uncovered_assertions_using<D>(
        self: Rc<Self>,
        private_keys: &PrivateKeyBase,
        uncovered_assertions: &[Rc<Self>],
        tag: D,
        rng: &mut impl RandomNumberGenerator
    ) -> Rc<Self>
    where
        D: AsRef<[u8]>,
    {
        let signing_private_key = private_keys.signing_private_key();
        let digest = *self.clone().subject().digest().data();
        let signature = Envelope::new(signing_private_key.schnorr_sign_using(digest, tag, rng))
            .add_assertion_envelopes(uncovered_assertions)
            .unwrap();
        self.add_assertion(known_values::VERIFIED_BY, signature)
    }

    /// Creates a signature for the envelope's subject and returns a new envelope with a `verifiedBy: Signature` assertion.
    ///
    /// - Parameters:
    ///   - privateKeys: The signer's `PrivateKeyBase`
    ///   - uncoveredAssertions: Assertions to add to the `Signature`.
    ///
    /// - Returns: The signed envelope.
    pub fn sign_with_uncovered_assertions<D>(
        self: Rc<Self>,
        private_keys: &PrivateKeyBase,
        uncovered_assertions: &[Rc<Self>],
        tag: D,
    ) -> Rc<Self>
    where
        D: AsRef<[u8]>,
    {
        let mut rng = SecureRandomNumberGenerator;
        self.sign_with_uncovered_assertions_using(private_keys, uncovered_assertions, tag, &mut rng)
    }

    /// Convenience constructor for a `verifiedBy: Signature` assertion envelope.
    ///
    /// - Parameters:
    ///   - signature: The `Signature` for the object.
    ///   - note: An optional note to be added to the `Signature`.
    ///
    /// - Returns: The new assertion envelope.
    pub fn make_verified_by_signature(
        self: Rc<Self>,
        signature: &Signature,
        note: Option<&str>
    ) -> Rc<Self> {
        let verified_by = known_values::VERIFIED_BY;
        let mut envelope = Envelope::new_assertion(verified_by, Envelope::new(signature));
        if let Some(note) = note {
            envelope = envelope.add_assertion(known_values::NOTE, note);
        }
        envelope
    }

    /// Returns an array of `Signature`s from all of the envelope's `verifiedBy` predicates.
    ///
    /// - Throws: Throws an exception if any `verifiedBy` assertion doesn't contain a
    /// valid `Signature` as its object.
    pub fn signatures(self: Rc<Self>) -> Result<Vec<Rc<Signature>>, Error> {
        let verified_by = known_values::VERIFIED_BY;
        self
            .assertions_with_predicate(verified_by).into_iter()
            .map(|assertion| assertion.object().unwrap().extract_subject::<Signature>())
            .collect()
    }

    /// Returns whether the given signature is valid.
    ///
    /// - Parameters:
    ///   - signature: The `Signature` to be checked.
    ///   - publicKeys: The potential signer's `PublicKeyBase`.
    ///
    /// - Returns: `true` if the signature is valid for this envelope's subject, `false` otherwise.
    pub fn is_verified_signature(
        self: Rc<Self>,
        signature: &Signature,
        public_keys: &PublicKeyBase,
    ) -> bool {
        self.is_signature_from_key(signature, public_keys.signing_public_key())
    }

    /// Checks whether the given signature is valid for the given public key.
    ///
    /// Used for chaining a series of operations that include validating signatures.
    ///
    /// - Parameters:
    ///   - signature: The `Signature` to be checked.
    ///   - publicKeys: The potential signer's `PublicKeyBase`.
    ///
    /// - Returns: This envelope.
    ///
    /// - Throws: Throws `EnvelopeError.unverifiedSignature` if the signature is not valid.
    /// valid.
    pub fn verify_signature(
        self: Rc<Self>,
        signature: &Signature,
        public_keys: &PublicKeyBase,
    ) -> Result<Rc<Self>, Error> {
        self.verify_signature_from_key(signature, public_keys.signing_public_key())
    }

    /// Returns whether the envelope's subject has a valid signature from the given public key.
    ///
    /// - Parameters:
    ///   - publicKeys: The potential signer's `PublicKeyBase`.
    ///
    /// - Returns: `true` if the signature is valid for this envelope's subject, `false` otherwise.
    ///
    /// - Throws: Throws an exception if any `verifiedBy` assertion doesn't contain a
    /// valid `Signature` as its object.
    pub fn has_signature_from(
        self: Rc<Self>,
        public_keys: &PublicKeyBase,
    ) -> Result<bool, Error> {
        self.has_some_signature_from_key(public_keys.signing_public_key())
    }

    /// Returns whether the envelope's subject has a valid signature from the given public key.
    ///
    /// Used for chaining a series of operations that include validating signatures.
    ///
    /// - Parameters:
    ///   - publicKeys: The potential signer's `PublicKeyBase`.
    ///
    /// - Returns: This envelope.
    ///
    /// - Throws: Throws `EnvelopeError.unverifiedSignature` if the signature is not valid.
    /// valid.
    pub fn verify_signature_from(
        self: Rc<Self>,
        public_keys: &PublicKeyBase,
    ) -> Result<Rc<Self>, Error> {
        self.verify_has_some_signature_from_key(public_keys.signing_public_key())
    }

    /// Checks whether the envelope's subject has a set of signatures.
    pub fn has_signatures_from(
        self: Rc<Self>,
        public_keys_array: &[&PublicKeyBase],
    ) -> Result<bool, Error> {
        self.has_signatures_from_threshold(public_keys_array, None)
    }

    /// Returns whether the envelope's subject has some threshold of signatures.
    ///
    /// If `threshold` is `nil`, then *all* signers in `publicKeysArray` must have
    /// signed. If `threshold` is `1`, then at least one signer must have signed.
    ///
    /// - Parameters:
    ///   - publicKeysArray: An array of potential signers' `PublicKeyBase`s.
    ///   - threshold: Optional minimum number of signers.
    ///
    /// - Returns: `true` if the threshold of valid signatures is met, `false` otherwise.
    ///
    /// - Throws: Throws an exception if any `verifiedBy` assertion doesn't contain a
    /// valid `Signature` as its object.
    pub fn has_signatures_from_threshold(
        self: Rc<Self>,
        public_keys_array: &[&PublicKeyBase],
        threshold: Option<usize>,
    ) -> Result<bool, Error> {
        let public_keys = public_keys_array
            .iter()
            .map(|public_key| public_key.signing_public_key())
            .collect::<Vec<_>>();
        self.has_signatures_from_keys_threshold(&public_keys, threshold)
    }

    /// Checks whether the envelope's subject has a set of signatures.
    pub fn verify_signatures_from(
        self: Rc<Self>,
        public_keys_array: &[&PublicKeyBase],
    ) -> Result<Rc<Self>, Error> {
        self.verify_signatures_from_threshold(public_keys_array, None)
    }

    /// Checks whether the envelope's subject has some threshold of signatures.
    ///
    /// If `threshold` is `nil`, then *all* signers in `publicKeysArray` must have
    /// signed. If `threshold` is `1`, then at least one signer must have signed.
    ///
    /// Used for chaining a series of operations that include validating signatures.
    ///
    /// - Parameters:
    ///   - publicKeysArray: An array of potential signers' `PublicKeyBase`s.
    ///   - threshold: Optional minimum number of signers.
    ///
    /// - Returns: This envelope.
    ///
    /// - Throws: Throws an exception if the threshold of valid signatures is not met.
    pub fn verify_signatures_from_threshold(
        self: Rc<Self>,
        public_keys_array: &[&PublicKeyBase],
        threshold: Option<usize>,
    ) -> Result<Rc<Self>, Error> {
        let public_keys = public_keys_array
            .iter()
            .map(|public_key| public_key.signing_public_key())
            .collect::<Vec<_>>();
        self.verify_signatures_from_keys_threshold(&public_keys, threshold)
    }
}

#[doc(hidden)]
impl Envelope {
    fn is_signature_from_key(
        self: Rc<Self>,
        signature: &Signature,
        key: &SigningPublicKey
    ) -> bool {
        key.verify(signature, self.subject().digest().as_ref())
    }

    fn verify_signature_from_key(
        self: Rc<Self>,
        signature: &Signature,
        key: &SigningPublicKey
    ) -> Result<Rc<Self>, Error> {
        if !self.clone().is_signature_from_key(signature, key) {
            return Err(Error::UnverifiedSignature);
        }
        Ok(self)
    }

    fn has_some_signature_from_key(
        self: Rc<Self>,
        key: &SigningPublicKey
    ) -> Result<bool, Error> {
        let signatures = self.clone().signatures();
        let signatures = signatures?;
        let result = signatures.iter().any(|signature| {
            self.clone().is_signature_from_key(signature, key)
        });
        Ok(result)
    }

    fn verify_has_some_signature_from_key(
        self: Rc<Self>,
        key: &SigningPublicKey
    ) -> Result<Rc<Self>, Error> {
        if !self.clone().has_some_signature_from_key(key)? {
            return Err(Error::UnverifiedSignature);
        }
        Ok(self)
    }

    fn has_signatures_from_keys_threshold(
        self: Rc<Self>,
        keys: &[&SigningPublicKey],
        threshold: Option<usize>
    ) -> Result<bool, Error> {
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

    fn verify_signatures_from_keys_threshold(
        self: Rc<Self>,
        keys: &[&SigningPublicKey],
        threshold: Option<usize>
    ) -> Result<Rc<Self>, Error> {
        if !self.clone().has_signatures_from_keys_threshold(keys, threshold)? {
            return Err(Error::UnverifiedSignature);
        }
        Ok(self)
    }
}

impl_into_envelope!(Signature);
