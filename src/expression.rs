use std::rc::Rc;

use bc_components::{tags_registry, CID};
use dcbor::{CBOR, CBORDecodable};

use crate::{known_value_registry, Enclosable, Envelope, Function, Parameter, Error, KnownValue};

/// Function Construction
impl Envelope {
    /// Creates an envelope with a `«function»` subject.
    pub fn new_function<F>(function: F) -> Rc<Self>
    where
        F: Into<Function>,
    {
        function.into().enclose()
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
    pub fn new_parameter<P>(param: P, value: Rc<Self>) -> Rc<Self>
    where
        P: Into<Parameter>,
    {
        Self::new_assertion(param.into().enclose(), value)
    }

    pub fn new_optional_parameter<P>(param: P, value: Option<Rc<Self>>) -> Option<Rc<Self>>
    where
        P: Into<Parameter>,
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
    pub fn add_parameter<P>(self: Rc<Self>, param: P, value: Rc<Self>) -> Rc<Self>
    where
        P: Into<Parameter>,
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
    pub fn add_optional_parameter(
        self: Rc<Self>,
        param: Parameter,
        value: Option<Rc<Self>>,
    ) -> Rc<Self> {
        self.add_optional_assertion_envelope(Self::new_optional_parameter(param, value))
            .unwrap()
    }
}

/// Request Construction
impl Envelope {
    /// Creates an envelope with a `CID` subject and a `body: «function»` assertion.
    pub fn new_request<C>(request_id: C, body: Rc<Self>) -> Rc<Self>
    where
        C: AsRef<CID>,
    {
        CBOR::tagged_value(tags_registry::REQUEST, request_id.as_ref())
            .enclose()
            .add_assertion(known_value_registry::BODY.enclose(), body)
    }
}

/// Response Construction
impl Envelope {
    /// Creates an envelope with a `CID` subject and a `result: value` assertion.
    pub fn new_response<C>(response_id: C, result: Rc<Self>) -> Rc<Self>
    where
        C: AsRef<CID>,
    {
        CBOR::tagged_value(tags_registry::RESPONSE, response_id.as_ref()).enclose()
            .add_assertion(known_value_registry::RESULT.enclose(), result)
    }

    /// Creates an envelope with a `CID` subject and a `result: value` assertion for each provided result.
    pub fn new_response_with_results<C>(response_id: C, results: &[Rc<Self>]) -> Rc<Self>
    where
        C: AsRef<CID>,
    {
        let mut envelope = CBOR::tagged_value(tags_registry::RESPONSE, response_id.as_ref()).enclose();

        for result in results {
            envelope = envelope.add_assertion(
                known_value_registry::RESULT.enclose(),
                result.clone(),
            );
        }

        envelope
    }

    /// Creates an envelope with a `CID` subject and a `error: value` assertion.
    pub fn new_error<C>(response_id: C, error: Rc<Self>) -> Rc<Self>
    where
        C: AsRef<CID>,
    {
        CBOR::tagged_value(tags_registry::RESPONSE, response_id.as_ref()).enclose()
            .add_assertion(known_value_registry::ERROR.enclose(), error)
    }

    /// Creates an envelope with an `unknown` subject and a `error: value` assertion.
    ///
    /// If `error` is `None`, no assertion will be added.
    ///
    /// Used for an immediate response to a request without a proper ID, for example
    /// when a encrypted request envelope is received and the decryption fails, making
    /// it impossible to extract the request ID.
    pub fn new_error_without_id(error: Option<Rc<Self>>) -> Rc<Self> {
        if let Some(error) = error {
            CBOR::tagged_value(tags_registry::RESPONSE, "unknown").enclose()
                .add_assertion(known_value_registry::ERROR.enclose(), error)
        } else {
            CBOR::tagged_value(tags_registry::RESPONSE, "unknown").enclose()
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
        self.extract_object_for_predicate(param.into().enclose())
    }

    /// Returns an array of arguments for the given parameter.
    ///
    /// - Throws: Throws an exception if any of the parameter values are not the correct type.
    pub fn extract_objects_for_parameter<T, P>(self: Rc<Self>, param: P) -> Result<Vec<Rc<T>>, Error>
    where
        T: CBORDecodable + 'static,
        P: Into<Parameter>,
    {
        self.extract_objects_for_predicate(param.into().enclose())
    }
}

/// Result Decoding
impl Envelope {
    /// Returns the object of the `result` predicate.
    ///
    /// - Throws: Throws an exception if there is no `result` predicate.
    pub fn result(self: Rc<Self>) -> Result<Rc<Self>, Error> {
        self.object_for_predicate(known_value_registry::RESULT.enclose())
    }

    /// Returns the objects of every `result` predicate.
    pub fn results(self: Rc<Self>) -> Vec<Rc<Self>> {
        self.objects_for_predicate(known_value_registry::RESULT.enclose())
    }

    /// Returns the object of the `result` predicate.
    ///
    /// - Throws: Throws an exception if there is no `result` predicate, or if its
    /// object cannot be decoded to the specified `type`.
    pub fn extract_result<T>(self: Rc<Self>) -> Result<Rc<T>, Error>
    where
        T: CBORDecodable + 'static,
    {
        self.extract_object_for_predicate(known_value_registry::RESULT.enclose())
    }

    /// Returns the objects of every `result` predicate.
    ///
    /// - Throws: Throws an if not all object cannot be decoded to the specified `type`.
    pub fn extract_results<T>(self: Rc<Self>) -> Result<Vec<Rc<T>>, Error>
    where
        T: CBORDecodable + 'static,
    {
        self.extract_objects_for_predicate(known_value_registry::RESULT.enclose())
    }

    /// Checks whether the `result` predicate has the `KnownValue` `.ok`.
    ///
    /// - Throws: Throws an exception if there is no `result` predicate.
    pub fn is_result_ok(self: Rc<Self>) -> Result<bool, Error> {
        self.extract_result::<KnownValue>().map(|v| *v == known_value_registry::OK)
    }

    /// Returns the error value.
    ///
    /// - Throws: Throws an exception if there is no `error` predicate.
    pub fn error<T>(self: Rc<Self>) -> Result<Rc<T>, Error>
    where
        T: CBORDecodable + 'static,
    {
        self.extract_object_for_predicate(known_value_registry::ERROR.enclose())
    }
}
