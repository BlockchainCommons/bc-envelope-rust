use anyhow::{Error, Result};
use bc_components::{tags, ARID};
use dcbor::{Date, prelude::*};

use crate::{known_values, Envelope, EnvelopeEncodable, Expression, ExpressionBehavior, Function, Parameter};

/// A `Request` represents a message requesting execution of a function with parameters.
///
/// Requests are part of the expression system that enables distributed function calls
/// and communication between systems. Each request:
/// - Contains a body (an `Expression`) that represents the function to be executed
/// - Has a unique identifier (ARID) for tracking and correlation
/// - May include optional metadata like a note and timestamp
///
/// Requests are designed to be paired with `Response` objects that contain the results
/// of executing the requested function.
///
/// When serialized to an envelope, requests are tagged with `#6.40010` (TAG_REQUEST).
///
/// # Examples
///
/// ```
/// use bc_envelope::prelude::*;
/// use bc_components::ARID;
///
/// // Create a random request ID
/// let request_id = ARID::new();
///
/// // Create a request to execute a function with parameters
/// let request = Request::new("getBalance", request_id)
///     .with_parameter("account", "alice")
///     .with_parameter("currency", "USD")
///     .with_note("Monthly balance check");
///
/// // Convert to an envelope
/// let envelope = request.into_envelope();
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct Request {
    body: Expression,
    id: ARID,
    note: String,
    date: Option<Date>,
}

impl std::fmt::Display for Request {
    /// Formats the request for display, showing its ID and body.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Request({})", self.summary())
    }
}

impl Request {
    /// Returns a human-readable summary of the request.
    pub fn summary(&self) -> String {
        format!("id: {}, body: {}", self.id.short_description(), self.body.expression_envelope().format_flat())
    }
}

/// Trait that defines the behavior of a request.
///
/// This trait extends `ExpressionBehavior` to add methods specific to requests,
/// including metadata management and access to request properties. Types implementing
/// this trait can be used in contexts that expect request functionality.
pub trait RequestBehavior: ExpressionBehavior {
    //
    // Composition
    //

    /// Adds a note to the request.
    ///
    /// This provides human-readable context about the request's purpose.
    fn with_note(self, note: impl Into<String>) -> Self;

    /// Adds a date to the request.
    ///
    /// This timestamp typically represents when the request was created.
    fn with_date(self, date: impl AsRef<Date>) -> Self;

    //
    // Parsing
    //

    /// Returns the body of the request, which is the expression to be evaluated.
    fn body(&self) -> &Expression;

    /// Returns the unique identifier (ARID) of the request.
    fn id(&self) -> ARID;

    /// Returns the note attached to the request, or an empty string if none exists.
    fn note(&self) -> &str;

    /// Returns the date attached to the request, if any.
    fn date(&self) -> Option<&Date>;
}

impl Request {
    /// Creates a new request with the specified expression body and ID.
    ///
    /// # Arguments
    ///
    /// * `body` - The expression to be executed
    /// * `id` - Unique identifier for the request
    pub fn new_with_body(body: Expression, id: ARID) -> Self {
        Self {
            body,
            id,
            note: String::new(),
            date: None,
        }
    }

    /// Creates a new request with a function and ID.
    ///
    /// This is a convenience method that creates an expression from the function
    /// and then creates a request with that expression.
    ///
    /// # Arguments
    ///
    /// * `function` - The function to be executed
    /// * `id` - Unique identifier for the request
    ///
    /// # Examples
    ///
    /// ```
    /// use bc_envelope::prelude::*;
    /// use bc_components::ARID;
    ///
    /// let request_id = ARID::new();
    /// let request = Request::new("transferFunds", request_id)
    ///     .with_parameter("from", "alice")
    ///     .with_parameter("to", "bob")
    ///     .with_parameter("amount", 100);
    /// ```
    pub fn new(function: impl Into<Function>, id: ARID) -> Self {
        Self::new_with_body(Expression::new(function), id)
    }
}

/// Implementation of `ExpressionBehavior` for `Request`.
///
/// This delegates most operations to the request's body expression.
impl ExpressionBehavior for Request {
    /// Adds a parameter to the request.
    fn with_parameter(mut self, parameter: impl Into<Parameter>, value: impl EnvelopeEncodable) -> Self {
        self.body = self.body.with_parameter(parameter, value);
        self
    }

    /// Adds an optional parameter to the request.
    fn with_optional_parameter(mut self, parameter: impl Into<Parameter>, value: Option<impl EnvelopeEncodable>) -> Self {
        self.body = self.body.with_optional_parameter(parameter, value);
        self
    }

    /// Returns the function of the request.
    fn function(&self) -> &Function {
        self.body.function()
    }

    /// Returns the expression envelope of the request.
    fn expression_envelope(&self) -> &Envelope {
        self.body.expression_envelope()
    }

    /// Returns the object for a parameter in the request.
    fn object_for_parameter(&self, param: impl Into<Parameter>) -> Result<Envelope> {
        self.body.object_for_parameter(param)
    }

    /// Returns all objects for a parameter in the request.
    fn objects_for_parameter(&self, param: impl Into<Parameter>) -> Vec<Envelope> {
        self.body.objects_for_parameter(param)
    }

    /// Extracts a typed object for a parameter in the request.
    fn extract_object_for_parameter<T>(&self, param: impl Into<Parameter>) -> Result<T>
    where
        T: TryFrom<CBOR, Error = dcbor::Error> + 'static,
    {
        self.body.extract_object_for_parameter(param)
    }

    /// Extracts an optional typed object for a parameter in the request.
    fn extract_optional_object_for_parameter<T: TryFrom<CBOR, Error = dcbor::Error> + 'static>(&self, param: impl Into<Parameter>) -> Result<Option<T>> {
        self.body.extract_optional_object_for_parameter(param)
    }

    /// Extracts multiple typed objects for a parameter in the request.
    fn extract_objects_for_parameter<T>(&self, param: impl Into<Parameter>) -> Result<Vec<T>>
    where
        T: TryFrom<CBOR, Error = dcbor::Error> + 'static,
    {
        self.body.extract_objects_for_parameter(param)
    }
}

/// Implementation of `RequestBehavior` for `Request`.
impl RequestBehavior for Request {
    /// Adds a note to the request.
    fn with_note(mut self, note: impl Into<String>) -> Self {
        self.note = note.into();
        self
    }

    /// Adds a date to the request.
    fn with_date(mut self, date: impl AsRef<Date>) -> Self {
        self.date = Some(date.as_ref().clone());
        self
    }

    /// Returns the body of the request.
    fn body(&self) -> &Expression {
        &self.body
    }

    /// Returns the ID of the request.
    fn id(&self) -> ARID {
        self.id
    }

    /// Returns the note of the request.
    fn note(&self) -> &str {
        &self.note
    }

    /// Returns the date of the request.
    fn date(&self) -> Option<&Date> {
        self.date.as_ref()
    }
}

/// Converts a `Request` to an `Expression`.
///
/// This extracts the request's body expression.
impl From<Request> for Expression {
    fn from(request: Request) -> Self {
        request.body
    }
}

/// Converts a `Request` to an `Envelope`.
///
/// The envelope's subject is the request's ID tagged with TAG_REQUEST,
/// and assertions include the request's body, note (if not empty), and date (if present).
impl From<Request> for Envelope {
    fn from(request: Request) -> Self {
        Envelope::new(CBOR::to_tagged_value(tags::TAG_REQUEST, request.id))
            .add_assertion(known_values::BODY, request.body.into_envelope())
            .add_assertion_if(!request.note.is_empty(), known_values::NOTE, request.note)
            .add_optional_assertion(known_values::DATE, request.date)
    }
}

/// Converts an envelope and optional expected function to a `Request`.
///
/// This constructor is used when parsing an envelope that is expected to contain a request.
/// The optional function parameter enables validation of the request's function.
impl TryFrom<(Envelope, Option<&Function>)> for Request {
    type Error = Error;

    fn try_from((envelope, expected_function): (Envelope, Option<&Function>)) -> Result<Self> {
        let body_envelope = envelope.object_for_predicate(known_values::BODY)?;
        Ok(Self {
            body: Expression::try_from((body_envelope, expected_function))?,
            id: envelope.subject().try_leaf()?
                .try_into_expected_tagged_value(tags::TAG_REQUEST)?
                .try_into()?,
            note: envelope.extract_object_for_predicate_with_default(known_values::NOTE, "".to_string())?,
            date: envelope.extract_optional_object_for_predicate(known_values::DATE)?,
        })
    }
}

/// Converts an envelope to a `Request`.
///
/// This simplified constructor doesn't validate the request's function.
impl TryFrom<Envelope> for Request {
    type Error = Error;

    fn try_from(envelope: Envelope) -> Result<Self> {
        Self::try_from((envelope, None))
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
    fn test_basic_request() -> Result<()> {
        crate::register_tags();

        let request = Request::new("test", request_id())
            .with_parameter("param1", 42)
            .with_parameter("param2", "hello");

        let envelope: Envelope = request.clone().into();
        let expected = indoc!{r#"
        request(ARID(c66be27d)) [
            'body': «"test"» [
                ❰"param1"❱: 42
                ❰"param2"❱: "hello"
            ]
        ]
        "#}.trim();
        assert_eq!(envelope.format(), expected);

        let parsed_request = Request::try_from(envelope)?;
        assert_eq!(parsed_request.extract_object_for_parameter::<i32>("param1")?, 42);
        assert_eq!(parsed_request.extract_object_for_parameter::<String>("param2")?, "hello");
        assert_eq!(parsed_request.note(), "");
        assert_eq!(parsed_request.date(), None);

        assert_eq!(request, parsed_request);

        Ok(())
    }

    #[test]
    fn test_request_with_metadata() -> Result<()> {
        crate::register_tags();

        let request_date = Date::try_from("2024-07-04T11:11:11Z")?;
        let request = Request::new("test", request_id())
            .with_parameter("param1", 42)
            .with_parameter("param2", "hello")
            .with_note("This is a test")
            .with_date(&request_date);

        let envelope: Envelope = request.clone().into();
        // println!("{}", envelope.format());
        assert_eq!(envelope.format(),
        indoc!{r#"
        request(ARID(c66be27d)) [
            'body': «"test"» [
                ❰"param1"❱: 42
                ❰"param2"❱: "hello"
            ]
            'date': 2024-07-04T11:11:11Z
            'note': "This is a test"
        ]
        "#}.trim());

        let parsed_request = Request::try_from(envelope)?;
        assert_eq!(parsed_request.extract_object_for_parameter::<i32>("param1")?, 42);
        assert_eq!(parsed_request.extract_object_for_parameter::<String>("param2")?, "hello");
        assert_eq!(parsed_request.note(), "This is a test");
        assert_eq!(parsed_request.date(), Some(&request_date));

        assert_eq!(request, parsed_request);

        Ok(())
    }
}
