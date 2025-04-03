use anyhow::{bail, Result};

use crate::{Envelope, EnvelopeError};

use super::envelope::EnvelopeCase;

/// Support for wrapping and unwrapping envelopes.
impl Envelope {
    /// Returns a new envelope which wraps the current envelope.
    ///
    /// Wrapping an envelope allows you to treat an envelope (including its assertions)
    /// as a single unit, making it possible to add assertions about the envelope as a whole.
    ///
    /// # Examples
    ///
    /// ```
    /// # use bc_envelope::prelude::*;
    /// # use indoc::indoc;
    /// // Create an envelope with an assertion
    /// let envelope = Envelope::new("Hello.")
    ///     .add_assertion("language", "English");
    ///     
    /// // Wrap it to add an assertion about the envelope as a whole
    /// let wrapped = envelope.wrap_envelope()
    ///     .add_assertion("authenticated", true);
    ///     
    /// assert_eq!(wrapped.format(), indoc! {r#"
    /// {
    ///     "Hello." [
    ///         "language": "English"
    ///     ]
    /// } [
    ///     "authenticated": true
    /// ]
    /// "#}.trim());
    /// ```
    pub fn wrap_envelope(&self) -> Self {
        Self::new_wrapped(self.clone())
    }

    /// Unwraps and returns the inner envelope.
    ///
    /// This extracts the envelope contained within a wrapped envelope.
    ///
    /// # Errors
    ///
    /// Returns `EnvelopeError::NotWrapped` if this is not a wrapped envelope.
    ///
    /// # Examples
    ///
    /// ```
    /// # use bc_envelope::prelude::*;
    /// // Create an envelope and wrap it
    /// let envelope = Envelope::new("Hello.");
    /// let wrapped = envelope.wrap_envelope();
    /// 
    /// // Unwrap to get the original envelope
    /// let unwrapped = wrapped.unwrap_envelope().unwrap();
    /// assert_eq!(unwrapped.format_flat(), r#""Hello.""#);
    /// ```
    pub fn unwrap_envelope(&self) -> Result<Self> {
        match self.subject().case() {
            EnvelopeCase::Wrapped { envelope, .. } => Ok(envelope.clone()),
            _ => bail!(EnvelopeError::NotWrapped),
        }
    }
}
