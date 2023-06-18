use std::rc::Rc;

use bc_components::{tags, CID};
use dcbor::{CBOR, CBORDecodable};

use crate::{IntoEnvelope, Envelope, Error, known_values::{self, KnownValue}};

use super::{Function, Parameter};

/// Function Construction
impl Envelope {
    /// Creates an envelope with a `«function»` subject.
    pub fn new_function<F>(function: F) -> Rc<Self>
    where
        F: Into<Function>,
    {
        Envelope::new(function.into())
    }
}

/// Parameter Construction
impl Envelope {
    /// Creates a new envelope containing a `❰parameter❱: value` assertion.
    ///
    /// - Parameters:
    ///   - param: A ``Parameter``. This will be encoded as either an unsigned integer or a string.
    ///   - value: The argument value.
    ///
    /// - Returns: The new assertion envelope. If `value` is `None`, returns `None`.
    pub fn new_parameter<P, V>(param: P, value: V) -> Rc<Self>
    where
        P: Into<Parameter>,
        V: IntoEnvelope,
    {
        Self::new_assertion(param.into(), value)
    }

    pub fn new_optional_parameter<P, V>(param: P, value: Option<V>) -> Option<Rc<Self>>
    where
        P: Into<Parameter>,
        V: IntoEnvelope,
    {
        value.map(|value| Self::new_parameter(param, value))
    }

    /// Adds a `❰parameter❱: value` assertion to the envelope.
    ///
    /// - Parameters:
    ///   - param: A ``Parameter``. This will be encoded as either an unsigned integer or a string.
    ///   - value: The argument value.
    ///
    /// - Returns: The new envelope.
    pub fn add_parameter<P, V>(self: Rc<Self>, param: P, value: V) -> Rc<Self>
    where
        P: Into<Parameter>,
        V: IntoEnvelope,
    {
        self.add_assertion_envelope(Self::new_parameter(param, value))
            .unwrap()
    }

    /// Adds a `❰parameter❱: value` assertion to the envelope.
    ///
    /// - Parameters:
    ///   - param: A ``Parameter``. This will be encoded as either an unsigned integer or a string.
    ///   - value: The optional argument value.
    ///
    /// - Returns: The new envelope. If `value` is `None`, returns the original envelope.
    pub fn add_optional_parameter<V>(
        self: Rc<Self>,
        param: Parameter,
        value: Option<V>,
    ) -> Rc<Self>
    where
        V: IntoEnvelope,
    {
        self.add_optional_assertion_envelope(Self::new_optional_parameter(param, value))
            .unwrap()
    }
}

/// Request Construction
impl Envelope {
    /// Creates an envelope with a `CID` subject and a `body: «function»` assertion.
    pub fn new_request<C, B>(request_id: C, body: B) -> Rc<Self>
    where
        C: AsRef<CID>,
        B: IntoEnvelope,
    {
        Envelope::new(CBOR::tagged_value(tags::REQUEST, request_id.as_ref()))
            .add_assertion(known_values::BODY, body)
    }
}

/// Response Construction
impl Envelope {
    /// Creates an envelope with a `CID` subject and a `result: value` assertion.
    pub fn new_response<C, R>(response_id: C, result: R) -> Rc<Self>
    where
        C: AsRef<CID>,
        R: IntoEnvelope,
    {
        Envelope::new(CBOR::tagged_value(tags::RESPONSE, response_id.as_ref()))
            .add_assertion(known_values::RESULT, result)
    }

    /// Creates an envelope with a `CID` subject and a `result: value` assertion for each provided result.
    pub fn new_response_with_results<C, R>(response_id: C, results: &[R]) -> Rc<Self>
    where
        C: AsRef<CID>,
        R: IntoEnvelope + Clone,
    {
        let mut envelope = Envelope::new(CBOR::tagged_value(tags::RESPONSE, response_id.as_ref()));

        for result in results {
            envelope = envelope.add_assertion(
                known_values::RESULT,
                result.clone(),
            );
        }

        envelope
    }

    /// Creates an envelope with a `CID` subject and a `error: value` assertion.
    pub fn new_error_response_with_id<C, E>(response_id: C, error: E) -> Rc<Self>
    where
        C: AsRef<CID>,
        E: IntoEnvelope,
    {
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
    pub fn new_error_response<E>(error: Option<E>) -> Rc<Self>
    where
        E: IntoEnvelope,
    {
        if let Some(error) = error {
            Envelope::new(CBOR::tagged_value(tags::RESPONSE, "unknown"))
                .add_assertion(known_values::ERROR, error)
        } else {
            Envelope::new(CBOR::tagged_value(tags::RESPONSE, "unknown"))
        }
    }
}

/// Parameter Decoding
impl Envelope {
    /// Returns the argument for the given parameter.
    ///
    /// - Throws: Throws an exception if there is not exactly one matching `parameter`,
    /// or if the parameter value is not the correct type.
    pub fn extract_object_for_parameter<T, P>(self: Rc<Self>, param: P) -> Result<Rc<T>, Error>
    where
        T: CBORDecodable + 'static,
        P: Into<Parameter>,
    {
        self.extract_object_for_predicate(param.into())
    }

    /// Returns an array of arguments for the given parameter.
    ///
    /// - Throws: Throws an exception if any of the parameter values are not the correct type.
    pub fn extract_objects_for_parameter<T, P>(self: Rc<Self>, param: P) -> Result<Vec<Rc<T>>, Error>
    where
        T: CBORDecodable + 'static,
        P: Into<Parameter>,
    {
        self.extract_objects_for_predicate(param.into())
    }
}

/// Result Decoding
impl Envelope {
    /// Returns the object of the `result` predicate.
    ///
    /// - Throws: Throws an exception if there is no `result` predicate.
    pub fn result(self: Rc<Self>) -> Result<Rc<Self>, Error> {
        self.object_for_predicate(known_values::RESULT)
    }

    /// Returns the objects of every `result` predicate.
    pub fn results(self: Rc<Self>) -> Vec<Rc<Self>> {
        self.objects_for_predicate(known_values::RESULT)
    }

    /// Returns the object of the `result` predicate.
    ///
    /// - Throws: Throws an exception if there is no `result` predicate, or if its
    /// object cannot be decoded to the specified `type`.
    pub fn extract_result<T>(self: Rc<Self>) -> Result<Rc<T>, Error>
    where
        T: CBORDecodable + 'static,
    {
        self.extract_object_for_predicate(known_values::RESULT)
    }

    /// Returns the objects of every `result` predicate.
    ///
    /// - Throws: Throws an if not all object cannot be decoded to the specified `type`.
    pub fn extract_results<T>(self: Rc<Self>) -> Result<Vec<Rc<T>>, Error>
    where
        T: CBORDecodable + 'static,
    {
        self.extract_objects_for_predicate(known_values::RESULT)
    }

    /// Checks whether the `result` predicate has the `KnownValue` `.ok`.
    ///
    /// - Throws: Throws an exception if there is no `result` predicate.
    pub fn is_result_ok(self: Rc<Self>) -> Result<bool, Error> {
        self.extract_result::<KnownValue>().map(|v| *v == known_values::OK)
    }

    /// Returns the error value.
    ///
    /// - Throws: Throws an exception if there is no `error` predicate.
    pub fn error<T>(self: Rc<Self>) -> Result<Rc<T>, Error>
    where
        T: CBORDecodable + 'static,
    {
        self.extract_object_for_predicate(known_values::ERROR)
    }
}
