use core::panic;

use anyhow::{bail, Error, Result};
use bc_components::{tags, ARID};
use dcbor::CBOR;

use crate::{known_values, Envelope, EnvelopeEncodable, KnownValue};

/// A `Response` represents a reply to a `Request` containing either a successful result or an error.
///
/// Responses are part of the expression system that enables distributed function calls.
/// Each response contains:
/// - A reference to the original request's ID (ARID) for correlation
/// - Either a successful result or an error message
///
/// The `Response` type is implemented as a wrapper around `Result<(ARID, Envelope), (Option<ARID>, Envelope)>`,
/// where the `Ok` variant represents a successful response and the `Err` variant represents an error response.
///
/// When serialized to an envelope, responses are tagged with `#6.40011` (TAG_RESPONSE).
///
/// # Examples
///
/// ```
/// use bc_envelope::prelude::*;
/// use bc_components::ARID;
///
/// // Create a request ID (normally this would come from the original request)
/// let request_id = ARID::new();
///
/// // Create a successful response
/// let success_response = Response::new_success(request_id)
///     .with_result("Transaction completed");
///
/// // Create an error response
/// let error_response = Response::new_failure(request_id)
///     .with_error("Insufficient funds");
///
/// // Convert to envelopes
/// let success_envelope = success_response.into_envelope();
/// let error_envelope = error_response.into_envelope();
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct Response (Result<(ARID, Envelope), (Option<ARID>, Envelope)>);

impl std::fmt::Display for Response {
    /// Formats the response for display, showing its ID and result or error.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Response({})", self.summary())
    }
}

impl Response {
    /// Returns a human-readable summary of the response.
    pub fn summary(&self) -> String {
        match &self.0 {
            Ok((id, result)) => format!("id: {}, result: {}", id.short_description(), result.format_flat()),
            Err((id, error)) => {
                if let Some(id) = id {
                    format!("id: {} error: {}", id.short_description(), error.format_flat())
                } else {
                    format!("id: 'Unknown' error: {}", error.format_flat())
                }
            }
        }
    }
}

impl Envelope {
    /// Creates an envelope containing the 'Unknown' known value.
    ///
    /// This is used when representing an unknown error or value.
    pub fn unknown() -> Self {
        known_values::UNKNOWN_VALUE.into_envelope()
    }

    /// Creates an envelope containing the 'OK' known value.
    ///
    /// This is used when a response doesn't need to return any specific value,
    /// just an acknowledgment that the request was successful.
    pub fn ok() -> Self {
        known_values::OK_VALUE.into_envelope()
    }
}

impl Response {
    //
    // Success Composition
    //

    /// Creates a new successful response with the specified request ID.
    ///
    /// By default, the result will be the 'OK' known value. Use `with_result`
    /// to set a specific result value.
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the request this response corresponds to
    ///
    /// # Examples
    ///
    /// ```
    /// use bc_envelope::prelude::*;
    /// use bc_components::ARID;
    ///
    /// let request_id = ARID::new();
    /// let response = Response::new_success(request_id);
    /// ```
    pub fn new_success(id: ARID) -> Self {
        Self(Ok((id, Envelope::ok())))
    }

    //
    // Failure Composition
    //

    /// Creates a new failure response with the specified request ID.
    ///
    /// By default, the error will be the 'Unknown' known value. Use `with_error`
    /// to set a specific error message.
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the request this response corresponds to
    ///
    /// # Examples
    ///
    /// ```
    /// use bc_envelope::prelude::*;
    /// use bc_components::ARID;
    ///
    /// let request_id = ARID::new();
    /// let response = Response::new_failure(request_id)
    ///     .with_error("Operation failed");
    /// ```
    pub fn new_failure(id: ARID) -> Self {
        Self(Err((Some(id), Envelope::unknown())))
    }

    /// Creates a new early failure response without a request ID.
    ///
    /// An early failure occurs when the error happens before the request
    /// has been fully processed, so the request ID is not known.
    ///
    /// # Examples
    ///
    /// ```
    /// use bc_envelope::prelude::*;
    ///
    /// let response = Response::new_early_failure()
    ///     .with_error("Authentication failed");
    /// ```
    pub fn new_early_failure() -> Self {
        Self(Err((None, Envelope::unknown())))
    }
}

/// Trait that defines the behavior of a response.
///
/// This trait provides methods for composing responses with results or errors,
/// and for extracting information from responses.
pub trait ResponseBehavior {
    //
    // Success Composition
    //

    /// Sets the result value for a successful response.
    ///
    /// # Panics
    ///
    /// This method will panic if called on a failure response.
    fn with_result(self, result: impl EnvelopeEncodable) -> Self;

    /// Sets the result value for a successful response if provided,
    /// otherwise sets the result to null.
    ///
    /// # Panics
    ///
    /// This method will panic if called on a failure response.
    fn with_optional_result(self, result: Option<impl EnvelopeEncodable>) -> Self;

    //
    // Failure Composition
    //

    /// Sets the error value for a failure response.
    ///
    /// # Panics
    ///
    /// This method will panic if called on a successful response.
    fn with_error(self, error: impl EnvelopeEncodable) -> Self;

    /// Sets the error value for a failure response if provided,
    /// otherwise leaves the error as the default 'Unknown' value.
    ///
    /// # Panics
    ///
    /// This method will panic if called on a successful response.
    fn with_optional_error(self, error: Option<impl EnvelopeEncodable>) -> Self;

    //
    // Parsing
    //

    /// Returns true if this is a successful response.
    fn is_ok(&self) -> bool;

    /// Returns true if this is a failure response.
    fn is_err(&self) -> bool;

    /// Returns a reference to the ID and result if this is a successful response.
    fn ok(&self) -> Option<&(ARID, Envelope)>;

    /// Returns a reference to the ID (if known) and error if this is a failure response.
    fn err(&self) -> Option<&(Option<ARID>, Envelope)>;

    /// Returns the ID of the request this response corresponds to, if known.
    fn id(&self) -> Option<ARID>;

    /// Returns the ID of the request this response corresponds to.
    ///
    /// # Panics
    ///
    /// This method will panic if the ID is not known.
    fn expect_id(&self) -> ARID {
        self.id().expect("Expected an ID")
    }

    /// Returns a reference to the result value if this is a successful response.
    ///
    /// # Errors
    ///
    /// Returns an error if this is a failure response.
    fn result(&self) -> Result<&Envelope> {
        self.ok().map(|(_, result)| result).ok_or_else(|| Error::msg("Cannot get result from failed response"))
    }

    /// Extracts a typed result value from a successful response.
    ///
    /// # Errors
    ///
    /// Returns an error if this is a failure response or if the result
    /// cannot be converted to the requested type.
    fn extract_result<T>(&self) -> dcbor::Result<T>
    where
        T: TryFrom<CBOR, Error = dcbor::Error> + 'static,
    {
        self.result()?.extract_subject()
    }

    /// Returns a reference to the error value if this is a failure response.
    ///
    /// # Errors
    ///
    /// Returns an error if this is a successful response.
    fn error(&self) -> Result<&Envelope> {
        self.err().map(|(_, error)| error).ok_or_else(|| Error::msg("Cannot get error from successful response"))
    }

    /// Extracts a typed error value from a failure response.
    ///
    /// # Errors
    ///
    /// Returns an error if this is a successful response or if the error
    /// cannot be converted to the requested type.
    fn extract_error<T>(&self) -> dcbor::Result<T>
    where
        T: TryFrom<CBOR, Error = dcbor::Error> + 'static,
    {
        self.error()?.extract_subject()
    }
}

impl ResponseBehavior for Response {
    fn with_result(mut self, result: impl EnvelopeEncodable) -> Self {
        match self.0 {
            Ok(_) => {
                self.0 = Ok((self.0.unwrap().0, result.into_envelope()));
                self
            }
            Err(_) => {
                panic!("Cannot set result on a failed response");
            }
        }
    }

    fn with_optional_result(self, result: Option<impl EnvelopeEncodable>) -> Self {
        if let Some(result) = result {
            return self.with_result(result);
        }
        self.with_result(Envelope::null())
    }

    fn with_error(mut self, error: impl EnvelopeEncodable) -> Self {
        match self.0 {
            Ok(_) => {
                panic!("Cannot set error on a successful response");
            }
            Err(_) => {
                self.0 = Err((self.0.err().unwrap().0, error.into_envelope()));
                self
            }
        }
    }

    fn with_optional_error(self, error: Option<impl EnvelopeEncodable>) -> Self {
        if let Some(error) = error {
            return self.with_error(error);
        }
        self
    }

    fn is_ok(&self) -> bool {
        self.0.is_ok()
    }

    fn is_err(&self) -> bool {
        self.0.is_err()
    }

    fn ok(&self) -> Option<&(ARID, Envelope)> {
        self.0.as_ref().ok()
    }

    fn err(&self) -> Option<&(Option<ARID>, Envelope)> {
        self.0.as_ref().err()
    }

    fn id(&self) -> Option<ARID> {
        match self.0 {
            Ok((id, _)) => Some(id),
            Err((id, _)) => id,
        }
    }
}

/// Converts a `Response` to an `Envelope`.
///
/// Successful responses have the request ID as the subject and a 'result' assertion.
/// Failure responses have the request ID (or 'Unknown' if not known) as the subject
/// and an 'error' assertion.
impl From<Response> for Envelope {
    fn from(value: Response) -> Self {
        match value.0 {
            Ok((id, result)) => {
                Envelope::new(CBOR::to_tagged_value(tags::TAG_RESPONSE, id)).add_assertion(known_values::RESULT, result)
            }
            Err((id, error)) => {
                let subject: Envelope;
                if let Some(id) = id {
                    subject = Envelope::new(CBOR::to_tagged_value(tags::TAG_RESPONSE, id))
                } else {
                    subject = Envelope::new(CBOR::to_tagged_value(tags::TAG_RESPONSE, known_values::UNKNOWN_VALUE))
                }
                subject.add_assertion(known_values::ERROR, error)
            }
        }
    }
}

/// Converts an `Envelope` to a `Response`.
///
/// The envelope must have a TAG_RESPONSE-tagged subject and either a 'result'
/// or 'error' assertion (but not both).
impl TryFrom<Envelope> for Response {
    type Error = Error;

    fn try_from(envelope: Envelope) -> Result<Self> {
        let result = envelope.assertion_with_predicate(known_values::RESULT);
        let error = envelope.assertion_with_predicate(known_values::ERROR);

        if result.is_ok() == error.is_ok() {
            bail!("Invalid response - must have either a result or an error, but not both")
        }

        if result.is_ok() {
            let id = envelope
                .subject().try_leaf()?
                .try_into_expected_tagged_value(tags::TAG_RESPONSE)?
                .try_into()?;
            let result = envelope.object_for_predicate(known_values::RESULT)?;
            return Ok(Response(Ok((id, result))));
        }

        if error.is_ok() {
            let id_value = envelope
                .subject().try_leaf()?
                .try_into_expected_tagged_value(tags::TAG_RESPONSE)?;
            let known_value = KnownValue::try_from(id_value.clone());
            let id: Option<ARID>;
            if let Ok(known_value) = known_value {
                if known_value == known_values::UNKNOWN_VALUE {
                    id = None;
                } else {
                    bail!("Invalid response - unknown known value in subject")
                }
            } else {
                id = Some(id_value.try_into()?);
            }
            let error = envelope.object_for_predicate(known_values::ERROR)?;
            return Ok(Response(Err((id, error))));
        }

        bail!("Invalid response")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hex_literal::hex;
    use indoc::indoc;

    fn request_id() -> ARID {
        ARID::from_data(hex!("c66be27dbad7cd095ca77647406d07976dc0f35f0d4d654bb0e96dd227a1e9fc"))
    }

    #[test]
    fn test_success_ok() -> Result<()> {
        crate::register_tags();

        let response = Response::new_success(request_id());
        let envelope: Envelope = response.clone().into();

        //println!("{}", envelope.format());
        assert_eq!(envelope.format(),
        indoc!{r#"
        response(ARID(c66be27d)) [
            'result': 'OK'
        ]
        "#}.trim());

        let parsed_response = Response::try_from(envelope)?;
        assert!(parsed_response.is_ok());
        assert_eq!(parsed_response.expect_id(), request_id());
        assert_eq!(parsed_response.extract_result::<KnownValue>()?, known_values::OK_VALUE);
        assert_eq!(response, parsed_response);

        Ok(())
    }

    #[test]
    fn test_success_result() -> Result<()> {
        crate::register_tags();

        let response = Response::new_success(request_id())
            .with_result("It works!");
        let envelope: Envelope = response.clone().into();

        //println!("{}", envelope.format());
        assert_eq!(envelope.format(),
        indoc!{r#"
        response(ARID(c66be27d)) [
            'result': "It works!"
        ]
        "#}.trim());

        let parsed_response = Response::try_from(envelope)?;
        assert!(parsed_response.is_ok());
        assert_eq!(parsed_response.expect_id(), request_id());
        assert_eq!(parsed_response.extract_result::<String>()?, "It works!");
        assert_eq!(response, parsed_response);

        Ok(())
    }

    #[test]
    fn test_early_failure() -> Result<()> {
        crate::register_tags();

        let response = Response::new_early_failure();
        let envelope: Envelope = response.clone().into();

        // println!("{}", envelope.format());
        assert_eq!(envelope.format(),
        indoc!{r#"
        response('Unknown') [
            'error': 'Unknown'
        ]
        "#}.trim());

        let parsed_response = Response::try_from(envelope)?;
        assert!(parsed_response.is_err());
        assert_eq!(parsed_response.id(), None);
        assert_eq!(parsed_response.extract_error::<KnownValue>()?, known_values::UNKNOWN_VALUE);
        assert_eq!(response, parsed_response);

        Ok(())
    }

    #[test]
    fn test_failure() -> Result<()> {
        crate::register_tags();

        let response = Response::new_failure(request_id())
            .with_error("It doesn't work!");
        let envelope: Envelope = response.clone().into();

        // println!("{}", envelope.format());
        assert_eq!(envelope.format(),
        indoc!{r#"
        response(ARID(c66be27d)) [
            'error': "It doesn't work!"
        ]
        "#}.trim());

        let parsed_response = Response::try_from(envelope)?;
        assert!(parsed_response.is_err());
        assert_eq!(parsed_response.id(), Some(request_id()));
        assert_eq!(parsed_response.extract_error::<String>()?, "It doesn't work!");
        assert_eq!(response, parsed_response);

        Ok(())
    }
}
