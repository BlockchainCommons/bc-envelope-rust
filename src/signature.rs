use std::{rc::Rc};

use bc_components::{PrivateKeyBase, Signature, PublicKeyBase, SigningPublicKey, DigestProvider};
use bc_crypto::{RandomNumberGenerator, SecureRandomNumberGenerator};

use crate::{Envelope, Error, known_value_registry, Enclosable, enclose_cbor};

impl Envelope {
    /// Creates a signature for the envelope's subject and returns a new envelope with a `verifiedBy: Signature` assertion.
    ///
    /// - Parameters:
    ///   - privateKeys: The signer's `PrivateKeyBase`
    ///   - note: Optional text note to add to the `Signature`
    ///   - rng: The random number generator to use.
    ///
    /// - Returns: The signed envelope.
    pub fn sign_with_using<D>(
        self: Rc<Self>,
        _private_keys: &PrivateKeyBase,
        _note: Option<&str>,
        _tag: D,
        _rng: &mut impl RandomNumberGenerator
    ) -> Rc<Self>
    where
        D: AsRef<[u8]>,
    {
        todo!()
    }

    /// Creates a signature for the envelope's subject and returns a new envelope with a `verifiedBy: Signature` assertion.
    ///
    /// - Parameters:
    ///   - privateKeys: The signer's `PrivateKeyBase`
    ///   - note: Optional text note to add to the `Signature`
    ///
    /// - Returns: The signed envelope.
    pub fn sign_with<D>(
        self: Rc<Self>,
        private_keys: &PrivateKeyBase,
        note: Option<&str>,
        tag: D,
    ) -> Rc<Self>
    where
        D: AsRef<[u8]>,
    {
        let mut rng = SecureRandomNumberGenerator;
        self.sign_with_using(private_keys, note, tag, &mut rng)
    }

    /// Creates several signatures for the envelope's subject and returns a new envelope with additional `verifiedBy: Signature` assertions.
    ///
    /// - Parameters:
    ///   - privateKeys: An array of signers' `PrivateKeyBase`s.
    ///   - rng: The random number generator to use.
    ///
    /// - Returns: The signed envelope.
    pub fn sign_with_keys_using<D>(
        self: Rc<Self>,
        _private_keys: &[PrivateKeyBase],
        _tag: D,
        _rng: &mut impl RandomNumberGenerator
    ) -> Rc<Self>
    where
        D: AsRef<[u8]>,
    {
        todo!()
    }

    /// Creates several signatures for the envelope's subject and returns a new envelope with additional `verifiedBy: Signature` assertions.
    ///
    /// - Parameters:
    ///   - privateKeys: An array of signers' `PrivateKeyBase`s.
    ///
    /// - Returns: The signed envelope.
    pub fn sign_with_keys<D>(
        self: Rc<Self>,
        private_keys: &[PrivateKeyBase],
        tag: D,
    ) -> Rc<Self>
    where
        D: AsRef<[u8]>,
    {
        let mut rng = SecureRandomNumberGenerator;
        self.sign_with_keys_using(private_keys, tag, &mut rng)
    }

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
        _private_keys: &PrivateKeyBase,
        _uncovered_assertions: &[Rc<Self>],
        _tag: D,
        _rng: &mut impl RandomNumberGenerator
    ) -> Rc<Self>
    where
        D: AsRef<[u8]>,
    {
        todo!()
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
}

impl Envelope {
    /// Convenience constructor for a `verifiedBy: Signature` assertion envelope.
    ///
    /// - Parameters:
    ///   - signature: The `Signature` for the object.
    ///   - note: An optional note to be added to the `Signature`.
    ///
    /// - Returns: The new assertion envelope.
    /*```swift
    static func verifiedBy(signature: Signature, note: String? = nil) -> Envelope {
        Envelope(
            .verifiedBy,
            Envelope(signature)
                .addAssertion(if: note != nil, .note, note!)
        )
    }
    ```*/
    pub fn verified_by_signature(
        self: Rc<Self>,
        signature: &Signature,
        note: Option<&str>
    ) -> Rc<Self> {
        let verified_by = known_value_registry::VERIFIED_BY.enclose();
        let signature = enclose_cbor(signature);
        let mut envelope = Envelope::new_assertion_with_predobj(verified_by, signature);
        if let Some(note) = note {
            envelope = envelope.add_assertion_with_predobj(known_value_registry::NOTE.enclose(), note.enclose());
        }
        envelope
    }
}

impl Envelope {
    /// An array of signatures from all of the envelope's `verifiedBy` predicates.
    ///
    /// - Throws: Throws an exception if any `verifiedBy` assertion doesn't contain a
    /// valid `Signature` as its object.
    pub fn signatures(self: Rc<Self>) -> Result<Vec<Rc<Signature>>, Error> {
        let verified_by = known_value_registry::VERIFIED_BY.enclose();
        self
            .assertions_with_predicate(verified_by).into_iter()
            .map(|assertion| assertion.object().unwrap().extract_subject::<Signature>())
            .collect()
    }

    /// Checks whether the given signature is valid.
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
        self.is_verified_signature_from_key(signature, public_keys.signing_public_key())
    }

    /// Checks whether the given signature is valid.
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

    /// Checks whether the envelope's subject has a valid signature.
    ///
    /// - Parameters:
    ///   - publicKeys: The potential signer's `PublicKeyBase`.
    ///
    /// - Returns: `true` if the signature is valid for this envelope's subject, `false` otherwise.
    ///
    /// - Throws: Throws an exception if any `verifiedBy` assertion doesn't contain a
    /// valid `Signature` as its object.
    pub fn has_verified_signature_from(
        self: Rc<Self>,
        public_keys: &PublicKeyBase,
    ) -> Result<bool, Error> {
        self.has_verified_signature_from_key(public_keys.signing_public_key())
    }

    /// Checks whether the envelope's subject has a valid signature.
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
        self.verify_has_signature_from_key(public_keys.signing_public_key())
    }

    /// Checks whether the envelope's subject has some threshold of signatures.
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
    pub fn has_verified_signatures_from(
        self: Rc<Self>,
        public_keys_array: &[&PublicKeyBase],
        threshold: Option<usize>,
    ) -> Result<bool, Error> {
        let public_keys = public_keys_array
            .iter()
            .map(|public_key| public_key.signing_public_key())
            .collect::<Vec<_>>();
        self.has_verified_signatures_from_keys(&public_keys, threshold)
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
    pub fn verify_signatures_from(
        self: Rc<Self>,
        public_keys_array: &[PublicKeyBase],
        threshold: Option<usize>,
    ) -> Result<Rc<Self>, Error> {
        let public_keys = public_keys_array
            .iter()
            .map(|public_key| public_key.signing_public_key())
            .collect::<Vec<_>>();
        self.verify_signatures_from_keys(&public_keys, threshold)
    }
}

/// Internal methods
impl Envelope {
    fn is_verified_signature_from_key(
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
        if !self.clone().is_verified_signature_from_key(signature, key) {
            return Err(Error::UnverifiedSignature);
        }
        Ok(self)
    }

    fn has_verified_signature_from_key(
        self: Rc<Self>,
        key: &SigningPublicKey
    ) -> Result<bool, Error> {
        let result = self.clone().signatures()?.iter().any(|signature| {
            self.clone().is_verified_signature_from_key(signature, key)
        });
        Ok(result)
    }

    fn verify_has_signature_from_key(
        self: Rc<Self>,
        key: &SigningPublicKey
    ) -> Result<Rc<Self>, Error> {
        if !self.clone().has_verified_signature_from_key(key)? {
            return Err(Error::UnverifiedSignature);
        }
        Ok(self)
    }

    fn has_verified_signatures_from_keys(
        self: Rc<Self>,
        keys: &[&SigningPublicKey],
        threshold: Option<usize>
    ) -> Result<bool, Error> {
        let threshold = threshold.unwrap_or(keys.len());
        let mut count = 0;
        for key in keys {
            if self.clone().has_verified_signature_from_key(key)? {
                count += 1;
                if count >= threshold {
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }

    fn verify_signatures_from_keys(
        self: Rc<Self>,
        keys: &[&SigningPublicKey],
        threshold: Option<usize>
    ) -> Result<Rc<Self>, Error> {
        if !self.clone().has_verified_signatures_from_keys(keys, threshold)? {
            return Err(Error::UnverifiedSignature);
        }
        Ok(self)
    }
}
