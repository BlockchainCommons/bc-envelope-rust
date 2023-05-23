use std::rc::Rc;
use crate::{Envelope, EnvelopeError};

impl Envelope {
    /// Return a new envelope which wraps the current envelope.
    pub fn wrap_envelope(self: Rc<Envelope>) -> Rc<Envelope> {
        Envelope::new_wrapped(self)
    }

    /// Unwraps and returns the inner envelope.
    ///
    /// Returns an error if this is not a wrapped envelope.
    pub fn unwrap_envelope(self: Rc<Envelope>) -> Result<Rc<Envelope>, EnvelopeError> {
        match &*self {
            Envelope::Wrapped { envelope, .. } => Ok(envelope.clone()),
            _ => Err(EnvelopeError::NotWrapped),
        }
    }
}
