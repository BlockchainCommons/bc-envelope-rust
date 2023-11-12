use bc_components::{tags, ARID};
use dcbor::prelude::*;

use crate::{EnvelopeEncodable, Envelope, EnvelopeError};
use crate::extension::known_values;

use super::{Function, Parameter};

/// Envelope Expressions: Function Construction
impl Envelope {
    /// Creates an envelope with a `«function»` subject.
    pub fn new_function(function: impl Into<Function>) -> Self {
        Envelope::new(function.into())
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
    /// Creates an envelope with an `ARID` subject and a `body: «function»`
    /// assertion.
    ///
    /// Also adds a `'note'` assertion if `note` is not empty, and a
    /// `'date'` assertion if `date` is not `nil`.
    pub fn new_request_with_metadata(id: impl AsRef<ARID>, body: impl EnvelopeEncodable, note: impl Into<String>, date: Option<dcbor::Date>) -> Self {
        let note = note.into();
        Envelope::new(CBOR::tagged_value(tags::REQUEST, id.as_ref()))
            .add_assertion(known_values::BODY, body)
            .add_assertion_if(!note.is_empty(), known_values::NOTE, note)
            .add_optional_assertion(known_values::DATE, date)
    }

    /// Creates an envelope with an `ARID` subject and a `body: «function»` assertion.
    pub fn new_request(id: impl AsRef<ARID>, body: impl EnvelopeEncodable) -> Self {
        Envelope::new_request_with_metadata(id, body, "", None)
    }
}

/// Envelope Expression: Request Parsing
impl Envelope {
    pub fn request_id(&self) -> anyhow::Result<ARID> {
        let id = self
            .subject()
            .expect_leaf()?
            .expect_tagged_value(tags::REQUEST)?
            .try_into()?;
        Ok(id)
    }

    pub fn request_body(&self) -> Result<Self, EnvelopeError> {
        self.object_for_predicate(known_values::BODY)
    }

    pub fn request_note(&self) -> anyhow::Result<String> {
        self.extract_object_for_predicate(known_values::NOTE)
    }

    pub fn request_date(&self) -> anyhow::Result<Option<dcbor::Date>> {
        self.extract_optional_object_for_predicate(known_values::DATE)
    }
}

/// Envelope Expressions: Response Construction
impl Envelope {
    /// Creates an envelope with an `ARID` subject and a `result: value` assertion.
    pub fn new_response(response_id: impl AsRef<ARID>, result: impl EnvelopeEncodable) -> Self {
        Envelope::new(CBOR::tagged_value(tags::RESPONSE, response_id.as_ref()))
            .add_assertion(known_values::RESULT, result)
    }

    /// Creates an envelope with an `ARID` subject and a `result: value` assertion for each provided result.
    pub fn new_response_with_result(response_id: impl AsRef<ARID>, results: &[impl EnvelopeEncodable + Clone]) -> Self {
        let mut envelope = Envelope::new(CBOR::tagged_value(tags::RESPONSE, response_id.as_ref()));

        for result in results {
            envelope = envelope.add_assertion(
                known_values::RESULT,
                result.clone(),
            );
        }

        envelope
    }

    /// Creates an envelope with an `ARID` subject and a `error: value` assertion.
    pub fn new_error_response_with_id(response_id: impl AsRef<ARID>, error: impl EnvelopeEncodable) -> Self {
        Envelope::new(CBOR::tagged_value(tags::RESPONSE, response_id.as_ref()))
            .add_assertion(known_values::ERROR, error)
    }

    /// Creates an envelope with an `unknown` subject and a `error: value` assertion.
    ///
    /// If `error` is `None`, no assertion will be added.
    ///
    /// Used for an immediate response to a request without a proper ID, for example
    /// when a encrypted request envelope is received and the decryption fails, making
    /// it impossible to extract the request ID.
    pub fn new_error_response(error: Option<impl EnvelopeEncodable>) -> Self {
        if let Some(error) = error {
            Envelope::new(CBOR::tagged_value(tags::RESPONSE, "unknown"))
                .add_assertion(known_values::ERROR, error)
        } else {
            Envelope::new(CBOR::tagged_value(tags::RESPONSE, "unknown"))
        }
    }
}

/// Envelope Expressions: Parameter Decoding
impl Envelope {
    pub fn object_for_parameter(&self, param: impl Into<Parameter>) -> anyhow::Result<Envelope> {
        Ok(self.object_for_predicate(param.into())?)
    }

    pub fn objects_for_parameter(&self, param: impl Into<Parameter>) -> Vec<Envelope> {
        self.objects_for_predicate(param.into())
    }

    /// Returns the argument for the given parameter, decoded as the given type.
    ///
    /// - Throws: Throws an exception if there is not exactly one matching `parameter`,
    /// or if the parameter value is not the correct type.
    pub fn extract_object_for_parameter<T>(&self, param: impl Into<Parameter>) -> anyhow::Result<T>
    where
        T: CBORDecodable + 'static,
    {
        self.extract_object_for_predicate(param.into())
    }

    /// Returns the argument for the given parameter, or `None` if there is no matching parameter.
    pub fn extract_optional_object_for_parameter<T: CBORDecodable + 'static>(&self, param: impl Into<Parameter>) -> anyhow::Result<Option<T>> {
        self.extract_optional_object_for_predicate(param.into())
    }

    /// Returns an array of arguments for the given parameter, decoded as the given type.
    ///
    /// - Throws: Throws an exception if any of the parameter values are not the correct type.
    pub fn extract_objects_for_parameter<T>(&self, param: impl Into<Parameter>) -> anyhow::Result<Vec<T>>
    where
        T: CBORDecodable + 'static,
    {
        self.extract_objects_for_predicate(param.into())
    }
}

/// Envelope Expressions: Result Decoding
impl Envelope {
    pub fn response_id(&self) -> anyhow::Result<ARID> {
        let id = self
            .subject()
            .expect_leaf()?
            .expect_tagged_value(tags::RESPONSE)?
            .try_into()?;
        Ok(id)
    }

    /// Returns the object of the `result` predicate.
    ///
    /// - Throws: Throws an exception if there is no `result` predicate.
    pub fn result(&self) -> Result<Self, EnvelopeError> {
        self.object_for_predicate(known_values::RESULT)
    }

    /// Returns the objects of every `result` predicate.
    pub fn results(&self) -> Vec<Self> {
        self.objects_for_predicate(known_values::RESULT)
    }

    /// Returns the object of the `result` predicate, decoded as the given type.
    ///
    /// - Throws: Throws an exception if there is no `result` predicate, or if its
    /// object cannot be decoded to the specified `type`.
    pub fn extract_result<T>(&self) -> anyhow::Result<T>
    where
        T: CBORDecodable + 'static,
    {
        self.extract_object_for_predicate(known_values::RESULT)
    }

    /// Returns the objects of every `result` predicate, decoded as the given type.
    ///
    /// - Throws: Throws an if not all object cannot be decoded to the specified `type`.
    pub fn extract_results<T>(&self) -> anyhow::Result<Vec<T>>
    where
        T: CBORDecodable + 'static,
    {
        self.extract_objects_for_predicate(known_values::RESULT)
    }

    /// Returns whether the `result` predicate has the `KnownValue` `.ok`.
    ///
    /// - Throws: Throws an exception if there is no `result` predicate.
    pub fn is_result_ok(&self) -> anyhow::Result<bool> {
        println!("{}", self.format());
        if let Some(k) = self.result()?.known_value() {
            return Ok(k == &known_values::OK_VALUE);
        }
        Ok(false)
    }

    /// Returns the error value, decoded as the given type.
    ///
    /// - Throws: Throws an exception if there is no `error` predicate.
    pub fn error<T>(&self) -> anyhow::Result<T>
    where
        T: CBORDecodable + 'static,
    {
        self.extract_object_for_predicate(known_values::ERROR)
    }
}
