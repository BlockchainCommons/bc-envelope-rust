use std::rc::Rc;
use bc_components::DigestProvider;

use crate::{Envelope, EnvelopeError, IntoEnvelope};
use crate::extension::{known_values, KnownValue};

impl Envelope {
    /// Returns the result of adding the given `'IsA'` type assertion to the envelope.
    pub fn add_type<O>(self: Rc<Self>, object: O) -> Rc<Self>
    where
        O: IntoEnvelope,
    {
        self.add_assertion(known_values::IS_A, object)
    }

    /// Returns all of the envelope's `'IsA'` type assertions.
    pub fn types(self: Rc<Self>) -> Vec<Rc<Self>> {
        self.objects_for_predicate(known_values::IS_A)
    }

    /// Gets a single `'IsA'` type assertion from the envelope.
    ///
    /// If there is more than one `'IsA'` type assertion, returns an error.
    pub fn get_type(self: Rc<Self>) -> Result<Rc<Self>, EnvelopeError> {
        let t = self.types();
        if t.len() == 1 {
            Ok(t[0].clone())
        } else {
            Err(EnvelopeError::AmbiguousType)
        }
    }

    /// Returns `true` if the envelope has an `'IsA'` type assertion with the given envelope `t`'s digest.
    pub fn has_type_envelope(self: Rc<Self>, t: Rc<Self>) -> bool {
        self.types().iter().any(|x| x.digest() == t.digest())
    }

    /// Returns `true` if the envelope has an `'IsA'` type assertion with the given known value `t`.
    pub fn has_type(self: Rc<Self>, t: &KnownValue) -> bool {
        let type_envelope = t.clone().into_envelope();
        self.types().iter().any(|x| x.digest() == type_envelope.digest())
    }

    /// Succeeds if the envelope has an `'IsA'` type assertion with the given known value `t`.
    ///
    /// Fails with `EnvelopeError::InvalidType` otherwise.
    pub fn check_type(self: Rc<Self>, t: &KnownValue) -> Result<(), EnvelopeError> {
        if self.has_type(t) {
            Ok(())
        } else {
            Err(EnvelopeError::InvalidType)
        }
    }

    /// Succeeds if the envelope has an `'IsA'` type assertion with the given envelope `t`'s digest.
    ///
    /// Fails with `EnvelopeError::InvalidType` otherwise.
    pub fn check_type_envelope(self: Rc<Self>, t: Rc<Self>) -> Result<(), EnvelopeError> {
        if self.has_type_envelope(t) {
            Ok(())
        } else {
            Err(EnvelopeError::InvalidType)
        }
    }
}
