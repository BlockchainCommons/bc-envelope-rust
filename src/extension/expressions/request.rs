use anyhow::{Error, Result};
use bc_components::{tags, ARID};
use dcbor::{Date, prelude::*};

use crate::{known_values, Envelope, EnvelopeEncodable, Expression, Function, Parameter};

#[derive(Debug, Clone, PartialEq)]
pub struct Request {
    body: Expression,
    id: ARID,
    note: String,
    date: Option<Date>,
}

//
// Composition
//
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

    /// Adds a parameter to the request.
    pub fn with_parameter(mut self, parameter: impl Into<Parameter>, value: impl EnvelopeEncodable) -> Self {
        self.body = self.body.with_parameter(parameter, value);
        self
    }

    /// Adds a parameter to the request, if the value is not `None`.
    pub fn with_optional_parameter(mut self, parameter: impl Into<Parameter>, value: Option<impl EnvelopeEncodable>) -> Self {
        self.body = self.body.with_optional_parameter(parameter, value);
        self
    }

    /// Adds a note to the request.
    pub fn with_note(mut self, note: impl Into<String>) -> Self {
        self.note = note.into();
        self
    }

    /// Adds a date to the request.
    pub fn with_date(mut self, date: impl AsRef<Date>) -> Self {
        self.date = Some(date.as_ref().clone());
        self
    }
}

//
// Parsing
//
impl Request {
    /// Returns the function of the request.
    pub fn function(&self) -> &Function {
        self.body.function()
    }

    /// Returns the body of the request.
    pub fn body(&self) -> &Expression {
        &self.body
    }

    /// Returns the argument for the given parameter.
    pub fn object_for_parameter(&self, param: impl Into<Parameter>) -> Result<Envelope> {
        self.body.object_for_parameter(param)
    }

    /// Returns the arguments for the given possibly repeated parameter.
    pub fn objects_for_parameter(&self, param: impl Into<Parameter>) -> Vec<Envelope> {
        self.body.objects_for_parameter(param)
    }

    /// Returns the argument for the given parameter, decoded as the given type.
    ///
    /// - Throws: Throws an exception if there is not exactly one matching `parameter`,
    /// or if the parameter value is not the correct type.
    pub fn extract_object_for_parameter<T>(&self, param: impl Into<Parameter>) -> Result<T>
    where
        T: TryFrom<CBOR, Error = Error> + 'static,
    {
        self.body.extract_object_for_parameter(param)
    }

    /// Returns the argument for the given parameter, or `None` if there is no matching parameter.
    pub fn extract_optional_object_for_parameter<T: TryFrom<CBOR, Error = Error> + 'static>(&self, param: impl Into<Parameter>) -> Result<Option<T>> {
        self.body.extract_optional_object_for_parameter(param)
    }

    /// Returns an array of arguments for the given parameter, decoded as the given type.
    ///
    /// - Throws: Throws an exception if any of the parameter values are not the correct type.
    pub fn extract_objects_for_parameter<T>(&self, param: impl Into<Parameter>) -> Result<Vec<T>>
    where
        T: TryFrom<CBOR, Error = Error> + 'static,
    {
        self.body.extract_objects_for_parameter(param)
    }

    /// Returns the ID of the request.
    pub fn id(&self) -> &ARID {
        &self.id
    }

    /// Returns the note of the request.
    pub fn note(&self) -> &str {
        &self.note
    }

    /// Returns the date of the request.
    pub fn date(&self) -> Option<&Date> {
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
        Envelope::new(CBOR::to_tagged_value(tags::REQUEST, request.id))
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
            body: (body_envelope, expected_function).try_into()?,
            id: envelope.subject().try_leaf()?
                .try_into_expected_tagged_value(tags::REQUEST)?
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

        let parsed_request: Request = envelope.try_into()?;
        assert_eq!(parsed_request.extract_object_for_parameter::<i32>("param1")?, 42);
        assert_eq!(parsed_request.extract_object_for_parameter::<String>("param2")?, "hello");
        assert_eq!(parsed_request.note(), "");
        assert_eq!(parsed_request.date(), None);

        assert_eq!(request, parsed_request);

        Ok(())
    }

    #[test]
    fn test_request_with_metadata() -> Result<()> {
        let request_date = "2024-07-04T11:11:11Z".try_into()?;
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

        let parsed_request: Request = envelope.try_into()?;
        assert_eq!(parsed_request.extract_object_for_parameter::<i32>("param1")?, 42);
        assert_eq!(parsed_request.extract_object_for_parameter::<String>("param2")?, "hello");
        assert_eq!(parsed_request.note(), "This is a test");
        assert_eq!(parsed_request.date(), Some(&request_date));

        assert_eq!(request, parsed_request);

        Ok(())
    }
}
