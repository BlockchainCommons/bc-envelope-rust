use anyhow::{bail, Result};

use crate::{Envelope, EnvelopeError};

use super::envelope::EnvelopeCase;

/// Support for wrapping and unwrapping envelopes.
impl Envelope {
    /// Return a new envelope which wraps the current envelope.
    pub fn wrap_envelope(&self) -> Self {
        Self::new_wrapped(self.clone())
    }

    /// Unwraps and returns the inner envelope.
    ///
    /// Returns an error if this is not a wrapped envelope.
    pub fn unwrap_envelope(&self) -> Result<Self> {
        match self.subject().case() {
            EnvelopeCase::Wrapped { envelope, .. } => Ok(envelope.clone()),
            _ => bail!(EnvelopeError::NotWrapped),
        }
    }
}
