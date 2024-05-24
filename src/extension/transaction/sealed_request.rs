use anyhow::{bail, Error, Result};
use bc_components::{PrivateKeyBase, PublicKeyBase, ARID};
use dcbor::{prelude::*, Date};

use crate::{known_values, Envelope, EnvelopeEncodable, Expression, ExpressionBehavior, Function, Parameter, Request, RequestBehavior};

use super::Continuation;

#[derive(Debug, Clone, PartialEq)]
pub struct SealedRequest {
    request: Request,
    sender: PublicKeyBase,
    // This is the continuation we're going to self-encrypt and send to the peer.
    state: Option<Envelope>,
    // This is a continuation we previously received from the peer and want to send back to them.
    peer_continuation: Option<Envelope>,
}

impl std::fmt::Display for SealedRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SealedRequest({}, state: {}, peer_continuation: {})",
            self.request.summary(),
            self.state.clone().map_or_else(|| "None".to_string(), |state| state.format_flat()),
            self.peer_continuation.clone().map_or_else(|| "None".to_string(), |_| "Some".to_string())
        )
    }
}

//
// Composition
//
impl SealedRequest {
    pub fn new(function: impl Into<Function>, id: impl AsRef<ARID>, sender: impl AsRef<PublicKeyBase>) -> Self {
        Self {
            request: Request::new(function, id),
            sender: sender.as_ref().clone(),
            state: None,
            peer_continuation: None,
        }
    }

    pub fn new_with_body(body: Expression, id: impl AsRef<ARID>, sender: impl AsRef<PublicKeyBase>) -> Self {
        Self {
            request: Request::new_with_body(body, id),
            sender: sender.as_ref().clone(),
            state: None,
            peer_continuation: None,
        }
    }
}

impl ExpressionBehavior for SealedRequest {
    fn with_parameter(mut self, parameter: impl Into<Parameter>, value: impl EnvelopeEncodable) -> Self {
        self.request = self.request.with_parameter(parameter, value);
        self
    }

    fn with_optional_parameter(mut self, parameter: impl Into<Parameter>, value: Option<impl EnvelopeEncodable>) -> Self {
        self.request = self.request.with_optional_parameter(parameter, value);
        self
    }

    fn function(&self) -> &Function {
        self.request.function()
    }

    fn expression_envelope(&self) -> &Envelope {
        self.request.expression_envelope()
    }

    fn object_for_parameter(&self, param: impl Into<Parameter>) -> Result<Envelope> {
        self.request.body().object_for_parameter(param)
    }

    fn objects_for_parameter(&self, param: impl Into<Parameter>) -> Vec<Envelope> {
        self.request.body().objects_for_parameter(param)
    }

    fn extract_object_for_parameter<T>(&self, param: impl Into<Parameter>) -> Result<T>
    where
        T: TryFrom<CBOR, Error = Error> + 'static,
    {
        self.request.body().extract_object_for_parameter(param)
    }

    fn extract_optional_object_for_parameter<T: TryFrom<CBOR, Error = Error> + 'static>(&self, param: impl Into<Parameter>) -> Result<Option<T>> {
        self.request.body().extract_optional_object_for_parameter(param)
    }

    fn extract_objects_for_parameter<T>(&self, param: impl Into<Parameter>) -> Result<Vec<T>>
    where
        T: TryFrom<CBOR, Error = Error> + 'static,
    {
        self.request.body().extract_objects_for_parameter(param)
    }
}

impl RequestBehavior for SealedRequest {
    fn with_note(mut self, note: impl Into<String>) -> Self {
        self.request = self.request.with_note(note);
        self
    }

    fn with_date(mut self, date: impl AsRef<Date>) -> Self {
        self.request = self.request.with_date(date);
        self
    }

    fn body(&self) -> &Expression {
        self.request.body()
    }

    fn id(&self) -> &ARID {
        self.request.id()
    }

    fn note(&self) -> &str {
        self.request.note()
    }

    fn date(&self) -> Option<&Date> {
        self.request.date()
    }
}

pub trait SealedRequestBehavior: RequestBehavior {
    //
    // Composition
    //

    /// Adds state to the request that the receiver must return in the response.
    fn with_state(self, state: impl EnvelopeEncodable) -> Self;

    /// Adds state to the request that the receiver must return in the response.
    fn with_optional_state(self, state: Option<impl EnvelopeEncodable>) -> Self;

    /// Adds a continuation we previously received from the recipient and want to send back to them.
    fn with_peer_continuation(self, peer_continuation: Envelope) -> Self;

    /// Adds a continuation we previously received from the recipient and want to send back to them.
    fn with_optional_peer_continuation(self, peer_continuation: Option<Envelope>) -> Self;

    //
    // Parsing
    //

    /// Returns the request.
    fn request(&self) -> &Request;

    /// Returns the sender of the request.
    fn sender(&self) -> &PublicKeyBase;

    /// Returns the continuation we're going to self-encrypt and send to the recipient.
    fn state(&self) -> Option<&Envelope>;

    /// Returns the continuation we previously received from the recipient and want to send back to them.
    fn peer_continuation(&self) -> Option<&Envelope>;
}

impl SealedRequestBehavior for SealedRequest {
    fn with_state(mut self, state: impl EnvelopeEncodable) -> Self {
        self.state = Some(state.into_envelope());
        self
    }

    fn with_optional_state(mut self, state: Option<impl EnvelopeEncodable>) -> Self {
        self.state = state.map(|state| state.into_envelope());
        self
    }

    fn with_peer_continuation(mut self, peer_continuation: Envelope) -> Self {
        self.peer_continuation = Some(peer_continuation);
        self
    }

    fn with_optional_peer_continuation(mut self, peer_continuation: Option<Envelope>) -> Self {
        self.peer_continuation = peer_continuation;
        self
    }

    fn request(&self) -> &Request {
        &self.request
    }

    fn sender(&self) -> &PublicKeyBase {
        &self.sender
    }

    fn state(&self) -> Option<&Envelope> {
        self.state.as_ref()
    }

    fn peer_continuation(&self) -> Option<&Envelope> {
        self.peer_continuation.as_ref()
    }
}

impl From<SealedRequest> for Request {
    fn from(sealed_request: SealedRequest) -> Self {
        sealed_request.request
    }
}

impl From<SealedRequest> for Expression {
    fn from(sealed_request: SealedRequest) -> Self {
        sealed_request.request.into()
    }
}

/// SealedRequest + optional valid_until date -> Envelope
impl From<(SealedRequest, Option<&Date>)> for Envelope {
    fn from((sealed_request, valid_until): (SealedRequest, Option<&Date>)) -> Self {
        // Even if no state is provided, requests always include a continuation
        // that at least specifies the required valid response ID.
        let state = sealed_request.state.clone().unwrap_or(Envelope::null());
        let continuation = Continuation::new(state)
                    .with_valid_id(sealed_request.id())
                    .with_optional_valid_until(valid_until);
        let sender_continuation: Envelope = (continuation, &sealed_request.sender).into();

        sealed_request.request.into_envelope()
            .add_assertion(known_values::SENDER_PUBLIC_KEY, sealed_request.sender.to_envelope())
            .add_assertion(known_values::SENDER_CONTINUATION, sender_continuation)
            .add_optional_assertion(known_values::RECIPIENT_CONTINUATION, sealed_request.peer_continuation)
    }
}

/// SealedRequst -> Envelope
impl From<SealedRequest> for Envelope {
    fn from(sealed_request: SealedRequest) -> Self {
        (sealed_request, None).into()
    }
}

/// SealedRequst + optional valid until date + sender private key -> signed Envelope
impl From<(SealedRequest, Option<&Date>, &PrivateKeyBase)> for Envelope {
    fn from((sealed_request, valid_until, sender_private_key): (SealedRequest, Option<&Date>, &PrivateKeyBase)) -> Self {
        (sealed_request, valid_until).into_envelope()
            .sign(sender_private_key)
    }
}

/// SealedRequst + sender private key -> signed Envelope
impl From<(SealedRequest, &PrivateKeyBase)> for Envelope {
    fn from((sealed_request, sender_private_key): (SealedRequest, &PrivateKeyBase)) -> Self {
        (sealed_request, None, sender_private_key).into()
    }
}

/// SealedRequst + optional valid until date + sender private key + recipient public key -> signed and encrypted Envelope
impl From<(SealedRequest, Option<&Date>, &PrivateKeyBase, &PublicKeyBase)> for Envelope {
    fn from((sealed_request, valid_until, sender_private_key, recipient_public_key): (SealedRequest, Option<&Date>, &PrivateKeyBase, &PublicKeyBase)) -> Self {
        (sealed_request, valid_until, sender_private_key).into_envelope()
            .encrypt_to_recipient(recipient_public_key)
    }
}

/// SealedRequst + sender private key + recipient public key -> signed and encrypted Envelope
impl From<(SealedRequest, &PrivateKeyBase, &PublicKeyBase)> for Envelope {
    fn from((sealed_request, sender_private_key, recipient_public_key): (SealedRequest, &PrivateKeyBase, &PublicKeyBase)) -> Self {
        (sealed_request, None, sender_private_key, recipient_public_key).into()
    }
}

/// Envelope -> SealedRequest
impl TryFrom<Envelope> for SealedRequest {
    type Error = Error;

    fn try_from(signed_envelope: Envelope) -> Result<Self> {
        let sender_public_key: PublicKeyBase = signed_envelope.unwrap_envelope()?.extract_object_for_predicate(known_values::SENDER_PUBLIC_KEY)?;
        let request_envelope = signed_envelope.verify(&sender_public_key)?;
        let peer_continuation = request_envelope.optional_object_for_predicate(known_values::SENDER_CONTINUATION)?;
        if peer_continuation.is_none() {
            bail!("Requests must contain a peer continuation");
        }
        let request = Request::try_from(request_envelope)?;
        Ok(Self {
            request,
            sender: sender_public_key,
            state: None,
            peer_continuation,
        })
    }
}

/// Envelope + optional expected ID + optional valid until date + sender private key -> SealedRequest
///
/// Sender private key is needed to self-encrypt the state continuation.
impl TryFrom<(Envelope, Option<&ARID>, Option<&Date>, &PrivateKeyBase)> for SealedRequest {
    type Error = Error;

    fn try_from((encrypted_envelope, id, now, recipient_private_key): (Envelope, Option<&ARID>, Option<&Date>, &PrivateKeyBase)) -> Result<Self> {
        let signed_envelope = encrypted_envelope.decrypt_to_recipient(recipient_private_key)?;
        let sender_public_key: PublicKeyBase = signed_envelope.unwrap_envelope()?.extract_object_for_predicate(known_values::SENDER_PUBLIC_KEY)?;
        let request_envelope = signed_envelope.verify(&sender_public_key)?;
        let peer_continuation = request_envelope.optional_object_for_predicate(known_values::SENDER_CONTINUATION)?;
        if let Some(some_peer_continuation) = peer_continuation.clone() {
            if !some_peer_continuation.subject().is_encrypted() {
                bail!("Peer continuation must be encrypted");
            }
        }
        let encrypted_continuation = request_envelope.optional_object_for_predicate(known_values::RECIPIENT_CONTINUATION)?;
        let state: Option<Envelope>;
        if let Some(encrypted_continuation) = encrypted_continuation {
            let continuation = Continuation::try_from((encrypted_continuation, id, now, recipient_private_key))?;
            state = Some(continuation.state().clone());
        } else {
            state = None;
        }
        let request = Request::try_from(request_envelope)?;
        Ok(Self {
            request,
            sender: sender_public_key,
            state,
            peer_continuation,
        })
    }
}

/// Envelope + expected ID + valid until date + sender private key -> SealedRequest
impl TryFrom<(Envelope, &ARID, &Date, &PrivateKeyBase)> for SealedRequest {
    type Error = Error;

    fn try_from((encrypted_envelope, id, now, recipient_private_key): (Envelope, &ARID, &Date, &PrivateKeyBase)) -> Result<Self> {
        Self::try_from((encrypted_envelope, Some(id), Some(now), recipient_private_key))
    }
}

/// Envelope + expected ID + sender private key -> SealedRequest
impl TryFrom<(Envelope, &ARID, &PrivateKeyBase)> for SealedRequest {
    type Error = Error;

    fn try_from((encrypted_envelope, id, recipient_private_key): (Envelope, &ARID, &PrivateKeyBase)) -> Result<Self> {
        Self::try_from((encrypted_envelope, Some(id), None, recipient_private_key))
    }
}

/// Envelope + valid until date + sender private key -> SealedRequest
impl TryFrom<(Envelope, &Date, &PrivateKeyBase)> for SealedRequest {
    type Error = Error;

    fn try_from((encrypted_envelope, now, recipient_private_key): (Envelope, &Date, &PrivateKeyBase)) -> Result<Self> {
        Self::try_from((encrypted_envelope, None, Some(now), recipient_private_key))
    }
}

/// Envelope with no ID or Date + sender private key -> SealedRequest
impl TryFrom<(Envelope, &PrivateKeyBase)> for SealedRequest {
    type Error = Error;

    fn try_from((encrypted_envelope, recipient_private_key): (Envelope, &PrivateKeyBase)) -> Result<Self> {
        Self::try_from((encrypted_envelope, None, None, recipient_private_key))
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;
    use anyhow::Result;
    use crate::{extension::transaction::Continuation, prelude::*};
    use bc_components::{PrivateKeyBase, ARID};
    use hex_literal::hex;
    use indoc::indoc;
    use dcbor::Date;

    fn request_id() -> ARID {
        ARID::from_data(hex!("c66be27dbad7cd095ca77647406d07976dc0f35f0d4d654bb0e96dd227a1e9fc"))
    }

    #[test]
    fn test_1() -> Result<()> {
        //
        // Generate keypairs for the server and client.
        //

        let server_private_key = PrivateKeyBase::new();
        let server_public_key = server_private_key.public_key();

        let client_private_key = PrivateKeyBase::new();
        let client_public_key = client_private_key.public_key();

        let now = Date::try_from("2024-07-04T11:11:11Z")?;

        //
        // The server has previously sent the client this continuation. To the
        // client, it is just an encrypted envelope and cannot be read or
        // modified; it can only be sent back to the server.
        //

        // The server sent this response 30 seconds ago.
        let server_response_date = now.clone() - Duration::from_secs(30);
        // And its continuation is valid for 60 seconds.
        let server_continuation_valid_until = server_response_date + Duration::from_secs(60);
        let server_state = Expression::new("nextPage")
            .with_parameter("fromRecord", 100)
            .with_parameter("toRecord", 199);
        // Normally you'll never need to compose a `Continuation` struct directly.
        // It is indirectly constructed using the `state` attribute of a `SealedRequest`
        // or `SealedResponse` struct.
        let server_continuation = Continuation::new(server_state)
            .with_valid_until(server_continuation_valid_until);
        let server_continuation: Envelope = (server_continuation, &server_public_key).into();

        //
        // The client composes a request to the server, returning to it the
        // continuation the server previously sent. The client is also going to
        // include its own continuation ("state"), which the server will return
        // in its response.
        //

        // The client's continuation is valid for 60 seconds from now.
        let client_continuation_valid_until = now.clone() + Duration::from_secs(60);
        let client_request = SealedRequest::new("test", request_id(), &client_public_key)
            .with_parameter("param1", 42)
            .with_parameter("param2", "hello")
            .with_note("This is a test")
            .with_date(&now)
            .with_state("The state of things.")
            .with_peer_continuation(server_continuation);

        //
        // We examine the form of the request envelope after it is signed by the
        // client, but before it is encrypted to the server. In production you
        // would skip this and go straight to the next step.
        //

        let signed_client_request_envelope: Envelope = (client_request.clone(), Some(&client_continuation_valid_until), &client_private_key).clone().into();
        // println!("{}", envelope.format());
        assert_eq!(signed_client_request_envelope.format(),
        indoc!{r#"
        {
            request(ARID(c66be27d)) [
                'body': «"test"» [
                    ❰"param1"❱: 42
                    ❰"param2"❱: "hello"
                ]
                'date': 2024-07-04T11:11:11Z
                'note': "This is a test"
                'recipientContinuation': ENCRYPTED [
                    'hasRecipient': SealedMessage
                ]
                'senderContinuation': ENCRYPTED [
                    'hasRecipient': SealedMessage
                ]
                'senderPublicKey': PublicKeyBase
            ]
        } [
            'verifiedBy': Signature
        ]
        "#}.trim());

        //
        // Create the ready-to-send request envelope, signed by the client and
        // encrypted to the server.
        //

        let sealed_client_request_envelope: Envelope = (client_request, Some(&client_continuation_valid_until), &client_private_key, &server_public_key).into();

        //
        // The server receives and parses the envelope. No expected ID is
        // provided because the server didn't know what the client's request ID
        // would be. The current date is provided so that the server can check that
        // any returned continuation has not expired.
        //

        let parsed_client_request = SealedRequest::try_from((sealed_client_request_envelope, None, Some(&now), &server_private_key))?;
        assert_eq!(*parsed_client_request.function(), Into::<Function>::into("test"));
        assert_eq!(parsed_client_request.extract_object_for_parameter::<i32>("param1")?, 42);
        assert_eq!(parsed_client_request.extract_object_for_parameter::<String>("param2")?, "hello");
        assert_eq!(parsed_client_request.note(), "This is a test");
        assert_eq!(parsed_client_request.date(), Some(&now));

        //
        // The server can now use the continuation state amd execute the request.
        //

        let state = parsed_client_request.state().unwrap();
        // println!("{}", state.format());
        assert_eq!(state.format(),
        indoc!{r#"
        «"nextPage"» [
            ❰"fromRecord"❱: 100
            ❰"toRecord"❱: 199
        ]
        "#}.trim());

        //
        // Now the server constructs its successful response to the client.
        //

        // The state we're sending to ourselves is the continuation of this retrival.
        let state = Expression::new("nextPage")
            .with_parameter("fromRecord", 200)
            .with_parameter("toRecord", 299);
        // The state we're sending back to the client is whatever they sent us.
        let peer_continuation = parsed_client_request.peer_continuation();

        let server_response = SealedResponse::new_success(parsed_client_request.id(), server_public_key)
            .with_result("Records retrieved: 100-199")
            .with_state(state)
            .with_peer_continuation(peer_continuation);

        //
        // We examine the form of the response envelope after it is signed by the
        // server, but before it is encrypted to the client. In production you
        // would skip this and go straight to the next step.
        //

        let server_continuation_valid_until = now.clone() + Duration::from_secs(60);
        let signed_server_response_envelope: Envelope = (server_response.clone(), Some(&server_continuation_valid_until), &server_private_key).into();
        // println!("{}", signed_server_response_envelope.format());
        assert_eq!(signed_server_response_envelope.format(),
        indoc!{r#"
        {
            response(ARID(c66be27d)) [
                'recipientContinuation': ENCRYPTED [
                    'hasRecipient': SealedMessage
                ]
                'result': "Records retrieved: 100-199"
                'senderContinuation': ENCRYPTED [
                    'hasRecipient': SealedMessage
                ]
                'senderPublicKey': PublicKeyBase
            ]
        } [
            'verifiedBy': Signature
        ]
        "#}.trim());

        //
        // Create the ready-to-send response envelope, signed by the server and encrypted
        // to the client.
        //

        let sealed_server_response_envelope: Envelope = (server_response, Some(&server_continuation_valid_until), &server_private_key, &client_public_key).into();

        //
        // The server receives and parses the envelope. The ID of the original
        // request is provided so the client can match the response to the
        // request. The current date is provided so that the client can check
        // that any returned continuation has not expired.
        //

        let parsed_server_response = SealedResponse::try_from((sealed_server_response_envelope, Some(parsed_client_request.id()), Some(&now), &client_private_key))?;

        // println!("{}", parsed_server_response.result()?.format());
        assert_eq!(parsed_server_response.result()?.format(),
        indoc!{r#"
        "Records retrieved: 100-199"
        "#}.trim());

        //
        // The client can now use the continuation state and take the next action based on the result.
        //

        // println!("{}", parsed_server_response.state().unwrap().format());
        assert_eq!(parsed_server_response.state().unwrap().format(),
        indoc!{r#"
        "The state of things."
        "#}.trim());

        Ok(())
    }
}
