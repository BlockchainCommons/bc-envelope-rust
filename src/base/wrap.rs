use std::rc::Rc;
use crate::{Envelope, EnvelopeError};

/// Support for wrapping and unwrapping envelopes.
impl Envelope {
    /// Return a new envelope which wraps the current envelope.
    pub fn wrap_envelope(self: Rc<Self>) -> Rc<Self> {
        Rc::new(Self::new_wrapped(self))
    }

    /// Unwraps and returns the inner envelope.
    ///
    /// Returns an error if this is not a wrapped envelope.
    pub fn unwrap_envelope(self: Rc<Self>) -> Result<Rc<Self>, EnvelopeError> {
        match &*self.subject() {
            Self::Wrapped { envelope, .. } => Ok(envelope.clone()),
            _ => Err(EnvelopeError::NotWrapped),
        }
    }
}