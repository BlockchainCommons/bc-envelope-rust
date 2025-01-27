use anyhow::{ bail, Result };
use bc_components::{ DigestProvider, Signature, Signer, SigningOptions, Verifier };

use crate::{ Envelope, EnvelopeEncodable, EnvelopeError };
#[cfg(feature = "known_value")]
use crate::extension::known_values;

use super::SignatureMetadata;

/// Support for signing envelopes and verifying signatures.
impl Envelope {
    /// Creates a signature for the envelope's subject and returns a new envelope with a `'signed': Signature` assertion.
    ///
    /// - Parameters:
    ///   - private_key: The signer's `SigningPrivateKey`
    ///
    /// - Returns: The signed envelope.
    pub fn add_signature(&self, private_key: &dyn Signer) -> Self {
        self.add_signature_opt(private_key, None, None)
    }

    #[doc(hidden)]
    /// Creates a signature for the envelope's subject and returns a new envelope with a `'signed': Signature` assertion.
    ///
    /// - Parameters:
    ///   - private_key: A signer's `PrivateKeyBase` or `SigningPrivateKey`.
    ///   - options: Optional signing options.
    ///   - metadata: Optional metadata for the signature, which itself will be signed.
    ///
    /// - Returns: The signed envelope.
    pub fn add_signature_opt(
        &self,
        private_key: &dyn Signer,
        options: Option<SigningOptions>,
        metadata: Option<SignatureMetadata>
    ) -> Self {
        let digest = *self.subject().digest().data();
        let mut signature = Envelope::new(
            private_key.sign_with_options(&digest as &dyn AsRef<[u8]>, options.clone()).unwrap()
        );

        if let Some(metadata) = metadata {
            if metadata.has_assertions() {
                let mut signature_with_metadata = signature;

                metadata
                    .assertions()
                    .iter()
                    .for_each(|assertion| {
                        signature_with_metadata = signature_with_metadata
                            .add_assertion_envelope(assertion.to_envelope())
                            .unwrap();
                    });

                signature_with_metadata = signature_with_metadata.wrap_envelope();

                let outer_signature = Envelope::new(
                    private_key
                        .sign_with_options(&signature_with_metadata.digest().as_ref(), options)
                        .unwrap()
                );
                signature = signature_with_metadata.add_assertion(
                    known_values::SIGNED,
                    outer_signature
                );
            }
        }

        self.add_assertion(known_values::SIGNED, signature)
    }

    #[doc(hidden)]
    /// Creates several signatures for the envelope's subject and returns a new envelope with additional `'signed': Signature` assertions.
    ///
    /// - Parameters:
    ///  - private_keys: An array of signers' `SigningPrivateKey`s.
    ///
    /// - Returns: The signed envelope.
    pub fn add_signatures(&self, private_keys: &[&dyn Signer]) -> Self {
        private_keys
            .iter()
            .fold(self.clone(), |envelope, private_key| { envelope.add_signature(*private_key) })
    }

    #[doc(hidden)]
    /// Creates several signatures for the envelope's subject and returns a new envelope with additional `'signed': Signature` assertions.
    ///
    /// - Parameters:
    ///   - private_keys: An array of signers' `SigningPrivateKey`s and optional `SigningOptions`.
    ///
    /// - Returns: The signed envelope.
    pub fn add_signatures_opt(
        &self,
        private_keys: &[(&dyn Signer, Option<SigningOptions>, Option<SignatureMetadata>)]
    ) -> Self {
        private_keys
            .iter()
            .fold(self.clone(), |envelope, (private_key, options, metadata)| {
                envelope.add_signature_opt(*private_key, options.clone(), metadata.clone())
            })
    }

    /// Convenience constructor for a `'signed': Signature` assertion envelope.
    ///
    /// - Parameters:
    ///   - signature: The `Signature` for the object.
    ///   - note: An optional note to be added to the `Signature`.
    ///
    /// - Returns: The new assertion envelope.
    pub fn make_signed_assertion(&self, signature: &Signature, note: Option<&str>) -> Self {
        let mut envelope = Envelope::new_assertion(known_values::SIGNED, signature.clone());
        if let Some(note) = note {
            envelope = envelope.add_assertion(known_values::NOTE, note);
        }
        envelope
    }

    /// Returns whether the given signature is valid.
    ///
    /// - Parameters:
    ///   - signature: The `Signature` to be checked.
    ///   - public_key: The potential signer's `Verifier`.
    ///
    /// - Returns: `true` if the signature is valid for this envelope's subject, `false` otherwise.
    pub fn is_verified_signature(&self, signature: &Signature, public_key: &dyn Verifier) -> bool {
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
    ///     valid.
    pub fn verify_signature(
        &self,
        signature: &Signature,
        public_key: &dyn Verifier
    ) -> Result<Self> {
        if !self.is_signature_from_key(signature, public_key) {
            bail!(EnvelopeError::UnverifiedSignature);
        }
        Ok(self.clone())
    }

    /// Returns whether the envelope's subject has a valid signature from the given public key.
    ///
    /// - Parameters:
    ///   - public_key: The potential signer's `Verifier`.
    ///
    /// - Returns: `true` if any signature is valid for this envelope's subject, `false` otherwise.
    ///
    /// - Throws: Throws an exception if any `'signed'` assertion doesn't contain a
    ///     valid `Signature` as its object.
    pub fn has_signature_from(&self, public_key: &dyn Verifier) -> Result<bool> {
        self.has_some_signature_from_key(public_key)
    }

    /// Returns whether the envelope's subject has a valid signature from the
    /// given public key by returning the signature metadata.
    ///
    /// - Parameters:
    ///  - public_key: The potential signer's `Verifier`.
    ///
    /// - Returns: The metadata envelope if the signature is valid, `None` otherwise.
    pub fn has_signature_from_returning_metadata(&self, public_key: &dyn Verifier) -> Result<Option<Envelope>> {
        self.has_some_signature_from_key_returning_metadata(public_key)
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
    ///     valid.
    pub fn verify_signature_from(&self, public_key: &dyn Verifier) -> Result<Self> {
        if !self.has_some_signature_from_key(public_key)? {
            bail!(EnvelopeError::UnverifiedSignature);
        }
        Ok(self.clone())
    }

    pub fn verify_signature_from_returning_metadata(&self, public_key: &dyn Verifier) -> Result<Envelope> {
        let metadata = self.has_some_signature_from_key_returning_metadata(public_key)?;
        if metadata.is_none() {
            bail!(EnvelopeError::UnverifiedSignature);
        }
        Ok(metadata.unwrap())
    }

    /// Checks whether the envelope's subject has a set of signatures.
    pub fn has_signatures_from(&self, public_keys: &[&dyn Verifier]) -> Result<bool> {
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
    /// - Throws: Throws an exception if any `'signed'` assertion doesn't contain a
    ///     valid `Signature` as its object.
    pub fn has_signatures_from_threshold(
        &self,
        public_keys: &[&dyn Verifier],
        threshold: Option<usize>
    ) -> Result<bool> {
            let threshold = threshold.unwrap_or(public_keys.len());
            let mut count = 0;
            for key in public_keys {
                if self.clone().has_some_signature_from_key(*key)? {
                    count += 1;
                    if count >= threshold {
                        return Ok(true);
                    }
                }
            }
            Ok(false)
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
        threshold: Option<usize>
    ) -> Result<Self> {
        if !self.has_signatures_from_threshold(public_keys, threshold)? {
            bail!(EnvelopeError::UnverifiedSignature);
        }
        Ok(self.clone())
    }

    /// Checks whether the envelope's subject has a set of signatures.
    pub fn verify_signatures_from(&self, public_keys: &[&dyn Verifier]) -> Result<Self> {
        self.verify_signatures_from_threshold(public_keys, None)
    }
}

#[doc(hidden)]
impl Envelope {
    fn is_signature_from_key(&self, signature: &Signature, key: &dyn Verifier) -> bool {
        key.verify(signature, self.subject().digest().as_ref())
    }

    fn has_some_signature_from_key(&self, key: &dyn Verifier) -> Result<bool> {
        self.has_some_signature_from_key_returning_metadata(key).map(|x| x.is_some())
    }

    fn has_some_signature_from_key_returning_metadata(&self, key: &dyn Verifier) -> Result<Option<Envelope>> {
        // Valid signature objects are either:
        //
        // - `Signature` objects, or
        // - `Signature` objects with additional metadata assertions, wrapped
        // and then signed by the same key.
        let signature_objects = self.objects_for_predicate(known_values::SIGNED);
        let result: Option<Result<Option<Envelope>>> = signature_objects.iter().find_map(|signature_object| {
            let signature_object_subject = signature_object.subject();
            if signature_object_subject.is_wrapped() {
                if
                    let Ok(outer_signature_object) = signature_object.object_for_predicate(
                        known_values::SIGNED
                    )
                {
                    if let Ok(outer_signature) = outer_signature_object.extract_subject::<Signature>() {
                        if !signature_object_subject.is_signature_from_key(&outer_signature, key) {
                            return None;
                        }
                    } else {
                        return Some(Err(anyhow::anyhow!("Unexpected outer signature object type.")));
                    }
                }

                let signature_metadata_envelope = signature_object_subject.unwrap_envelope().unwrap();
                if let Ok(signature) = signature_metadata_envelope.extract_subject::<Signature>() {
                    let signing_target = self.subject();
                    if !signing_target.is_signature_from_key(&signature, key) {
                        return Some(Err(anyhow::anyhow!("Inner signature not made with same key as outer signature.")));
                    }
                    Some(Ok(Some(signature_metadata_envelope)))
                } else {
                    Some(Err(anyhow::anyhow!("Unexpected inner signature object type.")))
                }
            } else if let Ok(signature) = signature_object.extract_subject::<Signature>() {
                if !self.is_signature_from_key(&signature, key) {
                    return None;
                }
                Some(Ok(Some(signature_object.clone())))
            } else {
                Some(Err(anyhow::anyhow!("Unexpected signature object type.")))
            }
        });

        match result {
            Some(Ok(Some(envelope))) => Ok(Some(envelope)),
            Some(Err(err)) => Err(err),
            _ => Ok(None),
        }
    }
}

impl Envelope {
    pub fn sign(&self, signer: &dyn Signer) -> Envelope {
        self.wrap_envelope().add_signature(signer)
    }

    pub fn verify(&self, verifier: &dyn Verifier) -> Result<Envelope> {
        self.verify_signature_from(verifier)?.unwrap_envelope()
    }

    pub fn verify_returning_metadata(&self, verifier: &dyn Verifier) -> Result<(Envelope, Envelope)> {
        let metadata = self.verify_signature_from_returning_metadata(verifier)?;
        Ok((self.unwrap_envelope()?, metadata))
    }
}
