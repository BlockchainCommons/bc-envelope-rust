use anyhow::bail;
use bc_components::{tags, ARID};
use dcbor::prelude::*;

use crate::{Envelope, EnvelopeEncodable, EnvelopeError};
use crate::extension::known_values;

use super::{Function, Parameter};

/// Envelope Expressions: Function Construction
impl Envelope {
    /// Creates an envelope with a `«function»` subject.
    pub fn new_function(function: impl Into<Function>) -> Self {
        Self::new(function.into())
    }
}

/// Envelope Expressions: Parameter Construction
impl Envelope {
    /// Creates a new envelope containing a `❰parameter❱: value` assertion.
    ///
    /// - Parameters:
    ///   - param: A ``Parameter``. This will be encoded as either an unsigned integer or a string.
    ///   - value: The argument value.
    ///
    /// - Returns: The new assertion envelope. If `value` is `None`, returns `None`.
    pub fn new_parameter(param: impl Into<Parameter>, value: impl EnvelopeEncodable) -> Self {
        Self::new_assertion(param.into(), value)
    }

    /// Optionally adds a `❰parameter❱: value` assertion to the envelope.
    pub fn new_optional_parameter(param: impl Into<Parameter>, value: Option<impl EnvelopeEncodable>) -> Option<Self> {
        value.map(|value| Self::new_parameter(param, value))
    }

    /// Adds a `❰parameter❱: value` assertion to the envelope.
    ///
    /// - Parameters:
    ///   - param: A ``Parameter``. This will be encoded as either an unsigned integer or a string.
    ///   - value: The argument value.
    ///
    /// - Returns: The new envelope.
    pub fn add_parameter(&self, param: impl Into<Parameter>, value: impl EnvelopeEncodable) -> Self {
        self.add_assertion_envelope(Self::new_parameter(param, value))
            .unwrap()
    }

    /// Optionally adds a `❰parameter❱: value` assertion to the envelope.
    ///
    /// - Parameters:
    ///   - param: A ``Parameter``. This will be encoded as either an unsigned integer or a string.
    ///   - value: The optional argument value.
    ///
    /// - Returns: The new envelope. If `value` is `None`, returns the original envelope.
    pub fn add_optional_parameter(
        &self,
        param: impl Into<Parameter>,
        value: Option<impl EnvelopeEncodable>,
    ) -> Self {
        self.add_optional_assertion_envelope(Self::new_optional_parameter(param, value))
            .unwrap()
    }
}

/// Envelope Expressions: Request Construction
impl Envelope {
    /// Creates an envelope with a `Request(ARID)` subject.
    pub fn new_request(id: impl AsRef<ARID>) -> Envelope {
        let subject = CBOR::to_tagged_value(tags::REQUEST, id.as_ref().clone());
        Envelope::new(subject)
    }

    /// Creates an envelope with a `Request(ARID)` subject and a `body: «function»`
    /// assertion.
    ///
    /// Also adds a `'note'` assertion if `note` is not empty, and a
    /// `'date'` assertion if `date` is not `nil`.
    pub fn into_request_with_metadata(
        self,
        id: impl AsRef<ARID>,
        note: impl Into<String>,
        date: Option<dcbor::Date>,
    ) -> Self {
        let note = note.into();

        Self::new_request(id)
            .add_assertion(known_values::BODY, self)
            .add_assertion_if(!note.is_empty(), known_values::NOTE, note)
            .add_optional_assertion(known_values::DATE, date)
    }

    /// Creates an envelope with a `Request(ARID)` subject and a `body: «function»`
    /// assertion.
    pub fn into_request(
        self,
        id: impl AsRef<ARID>,
    ) -> Self {
        self.into_request_with_metadata(id, "", None)
    }
}

/// Envelope Expression: Request Parsing
impl Envelope {
    /// Parses the request envelope and returns the id, body, note, and date.
    pub fn from_request_with_metadata(&self, expected_function: Option<&Function>) -> anyhow::Result<(ARID, Envelope, Function, String, Option<dcbor::Date>)> {
        let id = self.request_id()?;
        let body = self.request_body()?;
        let function = body.check_function(expected_function)?;
        let note = self.request_note()?;
        let date = self.request_date()?;
        Ok((id, body, function, note, date))
    }

    /// Parses the request envelope and returns the id and body.
    pub fn from_request(&self, expected_function: Option<&Function>) -> anyhow::Result<(ARID, Envelope, Function)> {
        let id = self.request_id()?;
        let body = self.request_body()?;
        let function = body.check_function(expected_function)?;
        Ok((id, body, function))
    }

    /// Parses the request envelope and returns the id.
    pub fn request_id(&self) -> anyhow::Result<ARID> {
        let id = self
            .subject()
            .try_leaf()?
            .try_into_expected_tagged_value(tags::REQUEST)?
            .try_into()?;
        Ok(id)
    }

    /// Parses the request envelope and returns the body.
    pub fn request_body(&self) -> anyhow::Result<Self> {
        self.object_for_predicate(known_values::BODY)
    }

    /// Parses the request envelope and returns the note.
    pub fn request_note(&self) -> anyhow::Result<String> {
        self.extract_object_for_predicate_with_default(known_values::NOTE, "".to_string())
    }

    /// Parses the request envelope and returns the date.
    pub fn request_date(&self) -> anyhow::Result<Option<dcbor::Date>> {
        self.extract_optional_object_for_predicate(known_values::DATE)
    }
}

/// Envelope Expressions: Parameter Decoding
impl Envelope {
    /// Returns the argument for the given parameter.
    pub fn object_for_parameter(&self, param: impl Into<Parameter>) -> anyhow::Result<Envelope> {
        self.object_for_predicate(param.into())
    }

    /// Returns the arguments for the given possibly repeated parameter.
    pub fn objects_for_parameter(&self, param: impl Into<Parameter>) -> Vec<Envelope> {
        self.objects_for_predicate(param.into())
    }

    /// Returns the argument for the given parameter, decoded as the given type.
    ///
    /// - Throws: Throws an exception if there is not exactly one matching `parameter`,
    /// or if the parameter value is not the correct type.
    pub fn extract_object_for_parameter<T>(&self, param: impl Into<Parameter>) -> anyhow::Result<T>
    where
        T: TryFrom<CBOR, Error = anyhow::Error> + 'static,
    {
        self.extract_object_for_predicate(param.into())
    }

    /// Returns the argument for the given parameter, or `None` if there is no matching parameter.
    pub fn extract_optional_object_for_parameter<T: TryFrom<CBOR, Error = anyhow::Error> + 'static>(&self, param: impl Into<Parameter>) -> anyhow::Result<Option<T>> {
        self.extract_optional_object_for_predicate(param.into())
    }

    /// Returns an array of arguments for the given parameter, decoded as the given type.
    ///
    /// - Throws: Throws an exception if any of the parameter values are not the correct type.
    pub fn extract_objects_for_parameter<T>(&self, param: impl Into<Parameter>) -> anyhow::Result<Vec<T>>
    where
        T: TryFrom<CBOR, Error = anyhow::Error> + 'static,
    {
        self.extract_objects_for_predicate(param.into())
    }
}

/// Envelope Expressions: Response Construction
impl Envelope {
    /// Creates an envelope with a `Response(ID)` subject.
    ///
    /// The `id` parameter is the ID of the response. Typically, this will be an
    /// ARID that matches the corresponding request. In certain error cases,
    /// there may not be a known ID, in which case `None` can be passed for
    /// `id`.
    pub fn new_response(id: Option<&ARID>) -> Envelope {
        if let Some(id) = id {
            Self::new(CBOR::to_tagged_value(tags::RESPONSE, id.clone()))
        } else {
            Self::new(CBOR::to_tagged_value(tags::RESPONSE, known_values::UNKNOWN_VALUE))
        }
    }

    /// Creates an envelope with a `Response(ID)` subject and a `'result': value`
    /// assertion, where `value` is `self`.
    ///
    /// The `id` parameter is the ID of the response. Typically, this will be an
    /// ARID that matches the corresponding request. In certain error cases,
    /// there may not be a known ID, in which case `None` can be passed for
    /// `id`.
    pub fn into_response(self, id: Option<&ARID>, is_success: bool) -> Self {
        Self::new_response(id)
            .add_assertion(
                if is_success { known_values::RESULT } else { known_values::ERROR },
                self
            )
    }

    /// Creates an envelope with a `Response(ID)` subject and a `'result': value`
    /// assertion, where `value` is `self`.
    ///
    /// The `id` parameter is the ID of the response. Typically, this will be an
    /// ARID that matches the corresponding request.
    pub fn into_success_response(self, id: impl AsRef<ARID>) -> Self {
        self.into_response(Some(id.as_ref()), true)
    }

    /// Creates an envelope with a `Response(ID)` subject and a `'error': value`
    /// assertion.
    ///
    /// The `id` parameter is the ID of the response. Typically, this will be an
    /// ARID that matches the corresponding request.
    ///
    /// If there is no explicit result, the `result` predicate will be set to
    /// the known value `OK`.
    pub fn success_response(id: impl AsRef<ARID>, result: Option<Envelope>) -> Envelope {
        result.unwrap_or_else(|| known_values::OK_VALUE.to_envelope()).into_success_response(id)
    }

    /// Creates an envelope with a `Response(ID)` subject and a `'error': value`
    /// assertion, where `value` is `self`.
    ///
    /// The `id` parameter is the ID of the response. Typically, this will be an
    /// ARID that matches the corresponding request. In certain error cases,
    /// there may not be a known ID, in which case `None` can be passed for
    /// `id`.
    pub fn into_failure_response(self, id: Option<&ARID>) -> Self {
        self.into_response(id, false)
    }

    /// Creates an envelope with a `Response(ID)` subject and a `'error': value`
    /// assertion.
    ///
    /// The `id` parameter is the ID of the response. Typically, this will be an
    /// ARID that matches the corresponding request. In certain error cases,
    /// there may not be a known ID, in which case `None` can be passed for
    /// `id`.
    ///
    /// If there is no known error, the `error` predicate will be set to the
    /// known value `Unknown`.
    pub fn failure_response(id: Option<&ARID>, error: Option<Envelope>) -> Envelope {
        error.unwrap_or_else(|| known_values::UNKNOWN_VALUE.to_envelope()).into_failure_response(id)
    }
}

/// Envelope Expressions: Response Decoding
impl Envelope {
    /// Returns whether the envelope is a success response.
    pub fn is_success(&self) -> bool {
        self.assertion_with_predicate(known_values::RESULT).is_ok()
    }

    /// Returns whether the envelope is a failure response.
    pub fn is_failure(&self) -> bool {
        self.assertion_with_predicate(known_values::ERROR).is_ok()
    }

    /// Returns the ID of the response.
    ///
    /// - Throws: Throws an exception if the subject is not a tagged value with
    ///   the tag `RESPONSE`.
    pub fn response_id(&self) -> anyhow::Result<ARID> {
        let id = self
            .subject()
            .try_leaf()?
            .try_into_expected_tagged_value(tags::RESPONSE)?
            .try_into()?;
        Ok(id)
    }

    /// Returns the response's result.
    ///
    /// - Throws: Throws an exception if there is no `result` predicate.
    pub fn result(&self) -> anyhow::Result<Self> {
        self.object_for_predicate(known_values::RESULT)
    }

    /// Returns the response's result, decoded as the given type.
    ///
    /// - Throws: Throws an exception if there is no `result` predicate.
    pub fn extract_result<T>(&self) -> anyhow::Result<T>
    where
        T: TryFrom<CBOR, Error = anyhow::Error> + 'static,
    {
        self.extract_object_for_predicate(known_values::RESULT)
    }

    /// Returns whether the response's result is the known value `OK`.
    ///
    /// - Throws: Throws an exception if there is no `result` predicate.
    pub fn is_result_ok(&self) -> anyhow::Result<bool> {
        if let Some(k) = self.result()?.as_known_value() {
            return Ok(k == &known_values::OK_VALUE);
        }
        Ok(false)
    }

    /// Returns the error value, decoded as the given type.
    ///
    /// - Throws: Throws an exception if there is no `error` predicate.
    pub fn extract_error<T>(&self) -> anyhow::Result<T>
    where
        T: TryFrom<CBOR, Error = anyhow::Error> + 'static,
    {
        self.extract_object_for_predicate(known_values::ERROR)
    }

    /// Parses the response envelope and returns the result or error value, the
    /// id, and a boolean that is `true` if the response represents success.
    ///
    /// If `expected_id` is provided, the response ID must match it.
    pub fn from_response(&self, expected_id: Option<&ARID>) -> anyhow::Result<(Envelope, ARID, bool)> {
        let id = self.response_id()?;
        if let Some(expected_id) = expected_id {
            if id != *expected_id {
                bail!(EnvelopeError::UnexpectedResponseID);
            }
        }
        let result_assertions = self.assertions_with_predicate(known_values::RESULT);
        let error_assertions = self.assertions_with_predicate(known_values::ERROR);
        if result_assertions.len() == 1 && error_assertions.is_empty() {
            let result = result_assertions[0].as_object().unwrap();
            Ok((result, id, true))
        } else if error_assertions.len() == 1 && result_assertions.is_empty() {
            let error = error_assertions[0].as_object().unwrap();
            Ok((error, id, false))
        } else {
            bail!(EnvelopeError::InvalidFormat)
        }
    }

    /// Assuming success, parses the response envelope and returns the result
    /// value and id.
    ///
    /// If `expected_id` is provided, the response ID must match it.
    pub fn from_success_response(&self, expected_id: Option<&ARID>) -> anyhow::Result<(Envelope, ARID)> {
        let (value, id, is_success) = self.from_response(expected_id)?;
        if is_success {
            Ok((value, id))
        } else {
            bail!(EnvelopeError::InvalidFormat)
        }
    }

    /// Assuming failure, parses the response envelope and returns the error
    /// value and id.
    ///
    /// If `expected_id` is provided, the response ID must match it.
    pub fn from_failure_response(&self, expected_id: Option<&ARID>) -> anyhow::Result<(Envelope, ARID)> {
        let (value, id, is_success) = self.from_response(expected_id)?;
        if !is_success {
            Ok((value, id))
        } else {
            bail!(EnvelopeError::InvalidFormat)
        }
    }
}
