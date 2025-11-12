use bc_components::{
    DigestProvider, Signature, Signer, SigningOptions, Verifier,
};
#[cfg(feature = "known_value")]
use known_values;

use super::SignatureMetadata;
use crate::{Envelope, EnvelopeEncodable, Error, Result};

/// Support for signing envelopes and verifying signatures.
///
/// This implementation provides methods for digitally signing envelopes and
/// verifying signatures. It supports both basic signatures and signatures with
/// metadata, as well as multi-signature scenarios.
impl Envelope {
    /// Creates a signature for the envelope's subject and returns a new
    /// envelope with a `'signed': Signature` assertion.
    ///
    /// - Parameters:
    ///   - private_key: The signer's `SigningPrivateKey`
    ///
    /// - Returns: The signed envelope.
    pub fn add_signature(&self, private_key: &dyn Signer) -> Self {
        self.add_signature_opt(private_key, None, None)
    }

    #[doc(hidden)]
    /// Creates a signature for the envelope's subject and returns a new
    /// envelope with a `'signed': Signature` assertion.
    ///
    /// - Parameters:
    ///   - private_key: A signer's `PrivateKeyBase` or `SigningPrivateKey`.
    ///   - options: Optional signing options.
    ///   - metadata: Optional metadata for the signature, which itself will be
    ///     signed.
    ///
    /// - Returns: The signed envelope.
    pub fn add_signature_opt(
        &self,
        private_key: &dyn Signer,
        options: Option<SigningOptions>,
        metadata: Option<SignatureMetadata>,
    ) -> Self {
        let digest = *self.subject().digest().data();
        let mut signature = Envelope::new(
            private_key
                .sign_with_options(&digest as &dyn AsRef<[u8]>, options.clone())
                .unwrap(),
        );

        if let Some(metadata) = metadata
            && metadata.has_assertions()
        {
            let mut signature_with_metadata = signature;

            metadata.assertions().iter().for_each(|assertion| {
                signature_with_metadata = signature_with_metadata
                    .add_assertion_envelope(assertion.to_envelope())
                    .unwrap();
            });

            signature_with_metadata = signature_with_metadata.wrap();

            let outer_signature = Envelope::new(
                private_key
                    .sign_with_options(
                        &signature_with_metadata.digest().as_ref(),
                        options,
                    )
                    .unwrap(),
            );
            signature = signature_with_metadata
                .add_assertion(known_values::SIGNED, outer_signature);
        }

        self.add_assertion(known_values::SIGNED, signature)
    }

    #[doc(hidden)]
    /// Creates several signatures for the envelope's subject and returns a new
    /// envelope with additional `'signed': Signature` assertions.
    ///
    /// - Parameters:
    ///  - private_keys: An array of signers' `SigningPrivateKey`s.
    ///
    /// - Returns: The signed envelope.
    pub fn add_signatures(&self, private_keys: &[&dyn Signer]) -> Self {
        private_keys
            .iter()
            .fold(self.clone(), |envelope, private_key| {
                envelope.add_signature(*private_key)
            })
    }

    #[doc(hidden)]
    /// Creates several signatures for the envelope's subject and returns a new
    /// envelope with additional `'signed': Signature` assertions.
    ///
    /// - Parameters:
    ///   - private_keys: An array of signers' `SigningPrivateKey`s and optional
    ///     `SigningOptions`.
    ///
    /// - Returns: The signed envelope.
    pub fn add_signatures_opt(
        &self,
        private_keys: &[(
            &dyn Signer,
            Option<SigningOptions>,
            Option<SignatureMetadata>,
        )],
    ) -> Self {
        private_keys.iter().fold(
            self.clone(),
            |envelope, (private_key, options, metadata)| {
                envelope.add_signature_opt(
                    *private_key,
                    options.clone(),
                    metadata.clone(),
                )
            },
        )
    }

    /// Convenience constructor for a `'signed': Signature` assertion envelope.
    ///
    /// - Parameters:
    ///   - signature: The `Signature` for the object.
    ///   - note: An optional note to be added to the `Signature`.
    ///
    /// - Returns: The new assertion envelope.
    pub fn make_signed_assertion(
        &self,
        signature: &Signature,
        note: Option<&str>,
    ) -> Self {
        let mut envelope =
            Envelope::new_assertion(known_values::SIGNED, signature.clone());
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
    /// - Returns: `true` if the signature is valid for this envelope's subject,
    ///   `false` otherwise.
    pub fn is_verified_signature(
        &self,
        signature: &Signature,
        public_key: &dyn Verifier,
    ) -> bool {
        self.is_signature_from_key(signature, public_key)
    }

    /// Checks whether the given signature is valid for the given public key.
    ///
    /// Used for chaining a series of operations that include validating
    /// signatures.
    ///
    /// - Parameters:
    ///   - signature: The `Signature` to be checked.
    ///   - public_key: The potential signer's `Verifier`.
    ///
    /// - Returns: This envelope.
    ///
    /// - Throws: Throws `EnvelopeError.unverifiedSignature` if the signature is
    ///   not valid. valid.
    pub fn verify_signature(
        &self,
        signature: &Signature,
        public_key: &dyn Verifier,
    ) -> Result<Self> {
        if !self.is_signature_from_key(signature, public_key) {
            return Err(Error::UnverifiedSignature);
        }
        Ok(self.clone())
    }

    /// Returns whether the envelope's subject has a valid signature from the
    /// given public key.
    ///
    /// - Parameters:
    ///   - public_key: The potential signer's `Verifier`.
    ///
    /// - Returns: `true` if any signature is valid for this envelope's subject,
    ///   `false` otherwise.
    ///
    /// - Throws: Throws an exception if any `'signed'` assertion doesn't
    ///   contain a valid `Signature` as its object.
    pub fn has_signature_from(
        &self,
        public_key: &dyn Verifier,
    ) -> Result<bool> {
        self.has_some_signature_from_key(public_key)
    }

    /// Returns whether the envelope's subject has a valid signature from the
    /// given public key by returning the signature metadata.
    ///
    /// - Parameters:
    ///  - public_key: The potential signer's `Verifier`.
    ///
    /// - Returns: The metadata envelope if the signature is valid, `None`
    ///   otherwise.
    pub fn has_signature_from_returning_metadata(
        &self,
        public_key: &dyn Verifier,
    ) -> Result<Option<Envelope>> {
        self.has_some_signature_from_key_returning_metadata(public_key)
    }

    /// Returns whether the envelope's subject has a valid signature from the
    /// given public key.
    ///
    /// Used for chaining a series of operations that include validating
    /// signatures.
    ///
    /// - Parameters:
    ///   - public_key: The potential signer's `Verifier`.
    ///
    /// - Returns: This envelope.
    ///
    /// - Throws: Throws `EnvelopeError.unverifiedSignature` if the signature is
    ///   not valid. valid.
    pub fn verify_signature_from(
        &self,
        public_key: &dyn Verifier,
    ) -> Result<Self> {
        if !self.has_some_signature_from_key(public_key)? {
            return Err(Error::UnverifiedSignature);
        }
        Ok(self.clone())
    }

    pub fn verify_signature_from_returning_metadata(
        &self,
        public_key: &dyn Verifier,
    ) -> Result<Envelope> {
        let metadata =
            self.has_some_signature_from_key_returning_metadata(public_key)?;
        if metadata.is_none() {
            return Err(Error::UnverifiedSignature);
        }
        Ok(metadata.unwrap())
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
    /// signed. If `threshold` is `1`, then at least one signer must have
    /// signed.
    ///
    /// - Parameters:
    ///   - public_keys: An array of potential signers' `Verifier`s.
    ///   - threshold: Optional minimum number of signers.
    ///
    /// - Returns: `true` if the threshold of valid signatures is met, `false`
    ///   otherwise.
    ///
    /// - Throws: Throws an exception if any `'signed'` assertion doesn't
    ///   contain a valid `Signature` as its object.
    pub fn has_signatures_from_threshold(
        &self,
        public_keys: &[&dyn Verifier],
        threshold: Option<usize>,
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
    /// signed. If `threshold` is `1`, then at least one signer must have
    /// signed.
    ///
    /// Used for chaining a series of operations that include validating
    /// signatures.
    ///
    /// - Parameters:
    ///   - public_keys: An array of potential signers' `Verifier`s.
    ///   - threshold: Optional minimum number of signers.
    ///
    /// - Returns: This envelope.
    ///
    /// - Throws: Throws an exception if the threshold of valid signatures is
    ///   not met.
    pub fn verify_signatures_from_threshold(
        &self,
        public_keys: &[&dyn Verifier],
        threshold: Option<usize>,
    ) -> Result<Self> {
        if !self.has_signatures_from_threshold(public_keys, threshold)? {
            return Err(Error::UnverifiedSignature);
        }
        Ok(self.clone())
    }

    /// Checks whether the envelope's subject has a set of signatures.
    pub fn verify_signatures_from(
        &self,
        public_keys: &[&dyn Verifier],
    ) -> Result<Self> {
        self.verify_signatures_from_threshold(public_keys, None)
    }
}

/// Internal implementation details for signature operations.
#[doc(hidden)]
impl Envelope {
    fn is_signature_from_key(
        &self,
        signature: &Signature,
        key: &dyn Verifier,
    ) -> bool {
        key.verify(signature, self.subject().digest().as_ref())
    }

    fn has_some_signature_from_key(&self, key: &dyn Verifier) -> Result<bool> {
        self.has_some_signature_from_key_returning_metadata(key)
            .map(|x| x.is_some())
    }

    fn has_some_signature_from_key_returning_metadata(
        &self,
        key: &dyn Verifier,
    ) -> Result<Option<Envelope>> {
        // Valid signature objects are either:
        //
        // - `Signature` objects, or
        // - `Signature` objects with additional metadata assertions, wrapped
        // and then signed by the same key.
        let signature_objects =
            self.objects_for_predicate(known_values::SIGNED);
        let result: Option<Result<Option<Envelope>>> =
            signature_objects.iter().find_map(|signature_object| {
                let signature_object_subject = signature_object.subject();
                if signature_object_subject.is_wrapped() {
                    if let Ok(outer_signature_object) = signature_object
                        .object_for_predicate(known_values::SIGNED)
                    {
                        if let Ok(outer_signature) = outer_signature_object
                            .extract_subject::<Signature>(
                        ) {
                            if !signature_object_subject
                                .is_signature_from_key(&outer_signature, key)
                            {
                                return None;
                            }
                        } else {
                            return Some(Err(Error::InvalidOuterSignatureType));
                        }
                    }

                    let signature_metadata_envelope =
                        signature_object_subject.try_unwrap().unwrap();
                    if let Ok(signature) = signature_metadata_envelope
                        .extract_subject::<Signature>()
                    {
                        let signing_target = self.subject();
                        if !signing_target
                            .is_signature_from_key(&signature, key)
                        {
                            return Some(Err(Error::UnverifiedInnerSignature));
                        }
                        Some(Ok(Some(signature_metadata_envelope)))
                    } else {
                        Some(Err(Error::InvalidInnerSignatureType))
                    }
                } else if let Ok(signature) =
                    signature_object.extract_subject::<Signature>()
                {
                    if !self.is_signature_from_key(&signature, key) {
                        return None;
                    }
                    Some(Ok(Some(signature_object.clone())))
                } else {
                    Some(Err(Error::InvalidSignatureType))
                }
            });

        match result {
            Some(Ok(Some(envelope))) => Ok(Some(envelope)),
            Some(Err(err)) => Err(err),
            _ => Ok(None),
        }
    }
}

/// Convenience methods for signing and verifying envelopes.
///
/// These methods provide a simpler API for common signature operations,
/// particularly for signing entire envelopes by automatically wrapping them.
impl Envelope {
    /// Signs the entire envelope (subject and assertions) by wrapping it first.
    ///
    /// This is a convenience method that wraps the envelope before signing,
    /// ensuring that all assertions are included in the signature, not just
    /// the subject.
    ///
    /// # Parameters
    ///
    /// * `signer` - The signer that will produce the signature.
    ///
    /// # Returns
    ///
    /// A new envelope with the wrapped envelope as subject and a signature
    /// assertion.
    pub fn sign(&self, signer: &dyn Signer) -> Envelope {
        self.sign_opt(signer, None)
    }

    /// Signs the entire envelope with options but no metadata.
    ///
    /// This is a convenience method that wraps the envelope before signing with
    /// the specified options.
    ///
    /// # Parameters
    ///
    /// * `signer` - The signer that will produce the signature.
    /// * `options` - Optional signing options to customize the signature
    ///   generation.
    ///
    /// # Returns
    ///
    /// A new envelope with the wrapped envelope as subject and a signature
    /// assertion.
    pub fn sign_opt(
        &self,
        signer: &dyn Signer,
        options: Option<SigningOptions>,
    ) -> Envelope {
        self.wrap().add_signature_opt(signer, options, None)
    }

    /// Verifies that the envelope has a valid signature from the specified
    /// verifier.
    ///
    /// This method assumes the envelope is wrapped (i.e., was signed using
    /// `sign()` rather than `add_signature()`), and unwraps it after
    /// verification.
    ///
    /// # Parameters
    ///
    /// * `verifier` - The verifier to check the signature against.
    ///
    /// # Returns
    ///
    /// The unwrapped envelope if verification succeeds, otherwise an error.
    ///
    /// # Errors
    ///
    /// Returns an error if the signature verification fails or if the envelope
    /// cannot be unwrapped.
    pub fn verify(&self, verifier: &dyn Verifier) -> Result<Envelope> {
        self.verify_signature_from(verifier)?.try_unwrap()
    }

    /// Verifies the envelope's signature and returns both the unwrapped
    /// envelope and signature metadata.
    ///
    /// This method verifies that the envelope has a valid signature from the
    /// specified verifier, then unwraps it and returns both the envelope
    /// and any metadata associated with the signature.
    ///
    /// # Parameters
    ///
    /// * `verifier` - The verifier to check the signature against.
    ///
    /// # Returns
    ///
    /// A tuple containing the unwrapped envelope and the signature metadata
    /// envelope if verification succeeds, otherwise an error.
    ///
    /// # Errors
    ///
    /// Returns an error if the signature verification fails or if the envelope
    /// cannot be unwrapped.
    pub fn verify_returning_metadata(
        &self,
        verifier: &dyn Verifier,
    ) -> Result<(Envelope, Envelope)> {
        let metadata =
            self.verify_signature_from_returning_metadata(verifier)?;
        Ok((self.try_unwrap()?, metadata))
    }
}
