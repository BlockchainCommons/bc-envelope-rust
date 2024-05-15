use core::panic;

use anyhow::{bail, Error, Result};
use bc_components::{tags, ARID};
use dcbor::CBOR;

use crate::{known_values, Envelope, EnvelopeEncodable, KnownValue};

#[derive(Debug, Clone, PartialEq)]
pub struct Response (Result<(ARID, Envelope), (Option<ARID>, Envelope)>);

impl Envelope {
    pub fn unknown() -> Self {
        known_values::UNKNOWN_VALUE.into_envelope()
    }

    pub fn ok() -> Self {
        known_values::OK_VALUE.into_envelope()
    }
}

//
// Success Composition
//
impl Response {
    pub fn new_success(id: impl AsRef<ARID>) -> Self {
        Self(Ok((id.as_ref().clone(), Envelope::ok())))
    }

    pub fn with_result(mut self, result: impl EnvelopeEncodable) -> Self {
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

    /// If the result is `None`, the value of the response will be the null envelope.
    pub fn with_optional_result(self, result: Option<impl EnvelopeEncodable>) -> Self {
        if let Some(result) = result {
            return self.with_result(result);
        }
        self.with_result(Envelope::null())
    }
}

//
// Failure Composition
//
impl Response {
    pub fn new_failure(id: impl AsRef<ARID>) -> Self {
        Self(Err((Some(id.as_ref().clone()), Envelope::unknown())))
    }

    /// An early failure takes place before the message has been decrypted,
    /// and therefore the ID is not known.
    pub fn new_early_failure() -> Self {
        Self(Err((None, Envelope::unknown())))
    }

    /// If no error is provided, the value of the response will be the unknown value.
    pub fn with_error(mut self, error: impl EnvelopeEncodable) -> Self {
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

    /// If the error is `None`, the value of the response will be the unknown value.
    pub fn with_optional_error(self, error: Option<impl EnvelopeEncodable>) -> Self {
        if let Some(error) = error {
            return self.with_error(error);
        }
        self
    }
}

//
// Parsing
//
impl Response {
    pub fn is_ok(&self) -> bool {
        self.0.is_ok()
    }

    pub fn is_err(&self) -> bool {
        self.0.is_err()
    }

    pub fn ok(&self) -> Option<&(ARID, Envelope)> {
        self.0.as_ref().ok()
    }

    pub fn err(&self) -> Option<&(Option<ARID>, Envelope)> {
        self.0.as_ref().err()
    }

    pub fn id(&self) -> Option<&ARID> {
        match &self.0 {
            Ok((id, _)) => Some(id),
            Err((id, _)) => id.as_ref(),
        }
    }

    pub fn expect_id(&self) -> &ARID {
        self.id().expect("Expected an ID")
    }

    pub fn result(&self) -> Result<&Envelope> {
        self.ok().map(|(_, result)| result).ok_or_else(|| Error::msg("Cannot get result from failed response"))
    }

    pub fn extract_result<T>(&self) -> Result<T>
    where
        T: TryFrom<CBOR, Error = Error> + 'static,
    {
        self.result()?.extract_subject()
    }

    pub fn error(&self) -> Result<&Envelope> {
        self.err().map(|(_, error)| error).ok_or_else(|| Error::msg("Cannot get error from successful response"))
    }

    pub fn extract_error<T>(&self) -> Result<T>
    where
        T: TryFrom<CBOR, Error = Error> + 'static,
    {
        self.error()?.extract_subject()
    }
}

impl From<Response> for Envelope {
    fn from(value: Response) -> Self {
        match value.0 {
            Ok((id, result)) => {
                Envelope::new(CBOR::to_tagged_value(tags::RESPONSE, id)).add_assertion(known_values::RESULT, result)
            }
            Err((id, error)) => {
                let subject: Envelope;
                if let Some(id) = id {
                    subject = Envelope::new(CBOR::to_tagged_value(tags::RESPONSE, id.clone()))
                } else {
                    subject = Envelope::new(CBOR::to_tagged_value(tags::RESPONSE, known_values::UNKNOWN_VALUE))
                }
                subject.add_assertion(known_values::ERROR, error)
            }
        }
    }
}

impl TryFrom<Envelope> for Response {
    type Error = Error;

    fn try_from(envelope: Envelope) -> Result<Self> {
        let result = envelope.assertion_with_predicate(known_values::RESULT);
        let error = envelope.assertion_with_predicate(known_values::ERROR);

        if result.is_ok() == error.is_ok() {
            bail!("Invalid response")
        }

        if result.is_ok() {
            let id = envelope
                .subject().try_leaf()?
                .try_into_expected_tagged_value(tags::RESPONSE)?
                .try_into()?;
            let result = envelope.object_for_predicate(known_values::RESULT)?;
            return Ok(Response(Ok((id, result))));
        }

        if error.is_ok() {
            let id_value = envelope
                .subject().try_leaf()?
                .try_into_expected_tagged_value(tags::RESPONSE)?;
            let known_value: Result<KnownValue> = id_value.clone().try_into();
            let id: Option<ARID>;
            if let Ok(known_value) = known_value {
                if known_value == known_values::UNKNOWN_VALUE {
                    id = None;
                } else {
                    bail!("Invalid response")
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
        assert_eq!(parsed_response.expect_id(), &request_id());
        assert_eq!(parsed_response.extract_result::<KnownValue>()?, known_values::OK_VALUE);
        assert_eq!(response, parsed_response);

        Ok(())
    }

    #[test]
    fn test_success_result() -> Result<()> {
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
        assert_eq!(parsed_response.expect_id(), &request_id());
        assert_eq!(parsed_response.extract_result::<String>()?, "It works!");
        assert_eq!(response, parsed_response);

        Ok(())
    }

    #[test]
    fn test_early_failure() -> Result<()> {
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
        assert_eq!(parsed_response.id(), Some(&request_id()));
        assert_eq!(parsed_response.extract_error::<String>()?, "It doesn't work!");
        assert_eq!(response, parsed_response);

        Ok(())
    }
}
