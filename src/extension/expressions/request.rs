use anyhow::{Error, Result};
use bc_components::{tags, ARID};
use dcbor::{Date, prelude::*};

use crate::{known_values, Envelope, EnvelopeEncodable, Expression, ExpressionBehavior, Function, Parameter};

#[derive(Debug, Clone, PartialEq)]
pub struct Request {
    body: Expression,
    id: ARID,
    note: String,
    date: Option<Date>,
}

impl std::fmt::Display for Request {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Request({})", self.summary())
    }
}

impl Request {
    pub fn summary(&self) -> String {
        format!("id: {}, body: {}", self.id.short_description(), self.body.expression_envelope().format_flat())
    }
}

pub trait RequestBehavior: ExpressionBehavior {
    //
    // Composition
    //

    /// Adds a note to the request.
    fn with_note(self, note: impl Into<String>) -> Self;

    /// Adds a date to the request.
    fn with_date(self, date: impl AsRef<Date>) -> Self;

    //
    // Parsing
    //

    /// Returns the body of the request.
    fn body(&self) -> &Expression;

    /// Returns the ID of the request.
    fn id(&self) -> &ARID;

    /// Returns the note of the request.
    fn note(&self) -> &str;

    /// Returns the date of the request.
    fn date(&self) -> Option<&Date>;
}

impl Request {
    pub fn new_with_body(body: Expression, id: impl AsRef<ARID>) -> Self {
        Self {
            body,
            id: id.as_ref().clone(),
            note: String::new(),
            date: None,
        }
    }

    pub fn new(function: impl Into<Function>, id: impl AsRef<ARID>) -> Self {
        Self::new_with_body(Expression::new(function), id)
    }
}

impl ExpressionBehavior for Request {
    fn with_parameter(mut self, parameter: impl Into<Parameter>, value: impl EnvelopeEncodable) -> Self {
        self.body = self.body.with_parameter(parameter, value);
        self
    }

    fn with_optional_parameter(mut self, parameter: impl Into<Parameter>, value: Option<impl EnvelopeEncodable>) -> Self {
        self.body = self.body.with_optional_parameter(parameter, value);
        self
    }

    fn function(&self) -> &Function {
        self.body.function()
    }

    fn expression_envelope(&self) -> &Envelope {
        self.body.expression_envelope()
    }

    fn object_for_parameter(&self, param: impl Into<Parameter>) -> Result<Envelope> {
        self.body.object_for_parameter(param)
    }

    fn objects_for_parameter(&self, param: impl Into<Parameter>) -> Vec<Envelope> {
        self.body.objects_for_parameter(param)
    }

    fn extract_object_for_parameter<T>(&self, param: impl Into<Parameter>) -> Result<T>
    where
        T: TryFrom<CBOR, Error = Error> + 'static,
    {
        self.body.extract_object_for_parameter(param)
    }

    fn extract_optional_object_for_parameter<T: TryFrom<CBOR, Error = Error> + 'static>(&self, param: impl Into<Parameter>) -> Result<Option<T>> {
        self.body.extract_optional_object_for_parameter(param)
    }

    fn extract_objects_for_parameter<T>(&self, param: impl Into<Parameter>) -> Result<Vec<T>>
    where
        T: TryFrom<CBOR, Error = Error> + 'static,
    {
        self.body.extract_objects_for_parameter(param)
    }
}

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
    fn id(&self) -> &ARID {
        &self.id
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

impl From<Request> for Expression {
    fn from(request: Request) -> Self {
        request.body
    }
}

impl From<Request> for Envelope {
    fn from(request: Request) -> Self {
        Envelope::new(CBOR::to_tagged_value(tags::TAG_REQUEST, request.id))
            .add_assertion(known_values::BODY, request.body.into_envelope())
            .add_assertion_if(!request.note.is_empty(), known_values::NOTE, request.note)
            .add_optional_assertion(known_values::DATE, request.date)
    }
}

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
        let request = Request::new("test", request_id())
            .with_parameter("param1", 42)
            .with_parameter("param2", "hello");

        let envelope: Envelope = request.clone().into();
        // println!("{}", envelope.format());
        assert_eq!(envelope.format(),
        indoc!{r#"
        request(ARID(c66be27d)) [
            'body': «"test"» [
                ❰"param1"❱: 42
                ❰"param2"❱: "hello"
            ]
        ]
        "#}.trim());

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
