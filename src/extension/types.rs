use anyhow::{bail, Result};
use bc_components::DigestProvider;

use crate::{Envelope, EnvelopeEncodable, EnvelopeError};
use crate::extension::{known_values, KnownValue};

impl Envelope {
    /// Returns the result of adding the given `'IsA'` type assertion to the envelope.
    pub fn add_type(&self, object: impl EnvelopeEncodable) -> Self {
        self.add_assertion(known_values::IS_A, object)
    }

    /// Returns all of the envelope's `'IsA'` type assertions.
    pub fn types(&self) -> Vec<Self> {
        self.objects_for_predicate(known_values::IS_A)
    }

    /// Gets a single `'IsA'` type assertion from the envelope.
    ///
    /// If there is more than one `'IsA'` type assertion, returns an error.
    pub fn get_type(&self) -> Result<Self> {
        let t = self.types();
        if t.len() == 1 {
            Ok(t[0].clone())
        } else {
            bail!(EnvelopeError::AmbiguousType)
        }
    }

    /// Returns `true` if the envelope has an `'IsA'` type assertion with the given envelope `t`'s digest.
    pub fn has_type_envelope(&self, t: impl EnvelopeEncodable) -> bool {
        let e = t.into_envelope();
        self.types().iter().any(|x| x.digest() == e.digest())
    }

    /// Returns `true` if the envelope has an `'IsA'` type assertion with the given known value `t`.
    pub fn has_type(&self, t: &KnownValue) -> bool {
        let type_envelope: Envelope = t.clone().to_envelope();
        self.types().iter().any(|x| x.digest() == type_envelope.digest())
    }

    /// Succeeds if the envelope has an `'IsA'` type assertion with the given known value `t`.
    ///
    /// Fails with `EnvelopeError::InvalidType` otherwise.
    pub fn check_type(&self, t: &KnownValue) -> Result<()> {
        if self.has_type(t) {
            Ok(())
        } else {
            bail!(EnvelopeError::InvalidType)
        }
    }

    /// Succeeds if the envelope has an `'IsA'` type assertion with the given envelope `t`'s digest.
    ///
    /// Fails with `EnvelopeError::InvalidType` otherwise.
    pub fn check_type_envelope(&self, t: impl EnvelopeEncodable) -> Result<()> {
        if self.has_type_envelope(t) {
            Ok(())
        } else {
            bail!(EnvelopeError::InvalidType)
        }
    }
}
