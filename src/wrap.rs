use std::rc::Rc;
use crate::{Envelope, Error};

impl Envelope {
    /// Return a new envelope which wraps the current envelope.
    pub fn wrap_envelope(self: Rc<Self>) -> Rc<Self> {
        Envelope::new_wrapped(self)
    }

    /// Unwraps and returns the inner envelope.
    ///
    /// Returns an error if this is not a wrapped envelope.
    pub fn unwrap_envelope(self: Rc<Self>) -> Result<Rc<Self>, Error> {
        match &*self {
            Envelope::Wrapped { envelope, .. } => Ok(envelope.clone()),
            _ => Err(Error::NotWrapped),
        }
    }
}
