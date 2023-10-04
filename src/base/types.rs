use std::rc::Rc;
use bc_components::DigestProvider;

use crate::{Envelope, EnvelopeError, known_values, KnownValue, IntoEnvelope};

impl Envelope {
    pub fn add_type<O>(self: Rc<Self>, object: O) -> Rc<Self>
    where
        O: IntoEnvelope,
    {
        self.add_assertion(known_values::IS_A, object)
    }

    pub fn types(self: Rc<Self>) -> Vec<Rc<Self>> {
        self.objects_for_predicate(known_values::IS_A)
    }

    pub fn get_type(self: Rc<Self>) -> Result<Rc<Self>, EnvelopeError> {
        let t = self.types();
        if t.len() == 1 {
            Ok(t[0].clone())
        } else {
            Err(EnvelopeError::AmbiguousPredicate)
        }
    }

    pub fn has_type_envelope(self: Rc<Self>, t: Rc<Self>) -> bool {
        self.types().iter().any(|x| x.digest() == t.digest())
    }

    pub fn has_type(self: Rc<Self>, t: &KnownValue) -> bool {
        self.types().iter().cloned().any(|x| x.predicate().unwrap().known_value() == Some(t))
    }

    pub fn check_type(self: Rc<Self>, t: &KnownValue) -> Result<(), EnvelopeError> {
        if self.has_type(t) {
            Ok(())
        } else {
            Err(EnvelopeError::InvalidFormat)
        }
    }

    pub fn check_type_envelope(self: Rc<Self>, t: Rc<Self>) -> Result<(), EnvelopeError> {
        if self.has_type_envelope(t) {
            Ok(())
        } else {
            Err(EnvelopeError::InvalidFormat)
        }
    }
}
