use anyhow::{bail, Error, Result};
use bc_components::{PrivateKeyBase, PublicKeyBase, ARID};
use dcbor::{prelude::*, Date};

use crate::{known_values, Envelope, EnvelopeEncodable, Response, ResponseBehavior};

use super::Continuation;

#[derive(Debug, Clone, PartialEq)]
pub struct SealedResponse {
    response: Response,
    sender: PublicKeyBase,
    // This is the continuation we're going to self-encrypt and send to the peer.
    state: Option<Envelope>,
    // This is a continuation we previously received from the peer and want to send back to them.
    peer_continuation: Option<Envelope>,
}

impl std::fmt::Display for SealedResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SealedResponse({}, state: {}, peer_continuation: {})",
            self.response.summary(),
            self.state.as_ref().map_or("None".to_string(), |state| state.format_flat()),
            self.peer_continuation.clone().map_or_else(|| "None".to_string(), |_| "Some".to_string())
        )
    }
}

impl SealedResponse {
    //
    // Success Composition
    //

    pub fn new_success(id: impl AsRef<ARID>, sender: impl AsRef<PublicKeyBase>) -> Self {
        Self {
            response: Response::new_success(id),
            sender: sender.as_ref().clone(),
            state: None,
            peer_continuation: None,
        }
    }

    //
    // Failure Composition
    //

    pub fn new_failure(id: impl AsRef<ARID>, sender: impl AsRef<PublicKeyBase>) -> Self {
        Self {
            response: Response::new_failure(id),
            sender: sender.as_ref().clone(),
            state: None,
            peer_continuation: None,
        }
    }

    /// An early failure takes place before the message has been decrypted,
    /// and therefore the ID and sender public key are not known.
    pub fn new_early_failure(sender: impl AsRef<PublicKeyBase>) -> Self {
        Self {
            response: Response::new_early_failure(),
            sender: sender.as_ref().clone(),
            state: None,
            peer_continuation: None,
        }
    }
}

pub trait SealedResponseBehavior: ResponseBehavior {
    //
    // Composition
    //

    /// Adds state to the request that the peer may return at some future time.
    fn with_state(self, state: impl EnvelopeEncodable) -> Self;

    fn with_optional_state(self, state: Option<impl EnvelopeEncodable>) -> Self;

    /// Adds a continuation we previously received from the recipient and want to send back to them.
    fn with_peer_continuation(self, peer_continuation: Option<&Envelope>) -> Self;

    //
    // Parsing
    //

    fn sender(&self) -> &PublicKeyBase;

    fn state(&self) -> Option<&Envelope>;

    fn peer_continuation(&self) -> Option<&Envelope>;
}

impl SealedResponseBehavior for SealedResponse {
    //
    // Composition
    //

    /// Adds state to the request that the peer may return at some future time.
    fn with_state(mut self, state: impl EnvelopeEncodable) -> Self {
        if self.response.is_ok() {
            self.state = Some(state.into_envelope());
        } else {
            panic!("Cannot set state on a failed response");
        }
        self
    }

    fn with_optional_state(mut self, state: Option<impl EnvelopeEncodable>) -> Self {
        if let Some(state) = state {
            self.with_state(state)
        } else {
            self.state = None;
            self
        }
    }

    /// Adds a continuation we previously received from the recipient and want to send back to them.
    fn with_peer_continuation(mut self, peer_continuation: Option<&Envelope>) -> Self {
        self.peer_continuation = peer_continuation.cloned();
        self
    }

    //
    // Parsing
    //

    fn sender(&self) -> &PublicKeyBase {
        self.sender.as_ref()
    }

    fn state(&self) -> Option<&Envelope> {
        self.state.as_ref()
    }

    fn peer_continuation(&self) -> Option<&Envelope> {
        self.peer_continuation.as_ref()
    }
}

impl ResponseBehavior for SealedResponse {
    fn with_result(mut self, result: impl EnvelopeEncodable) -> Self {
        self.response = self.response.with_result(result);
        self
    }

    /// If the result is `None`, the value of the response will be the null envelope.
    fn with_optional_result(mut self, result: Option<impl EnvelopeEncodable>) -> Self {
        self.response = self.response.with_optional_result(result);
        self
    }

    /// If no error is provided, the value of the response will be the unknown value.
    fn with_error(mut self, error: impl EnvelopeEncodable) -> Self {
        self.response = self.response.with_error(error);
        self
    }

    /// If the error is `None`, the value of the response will be the unknown value.
    fn with_optional_error(mut self, error: Option<impl EnvelopeEncodable>) -> Self {
        self.response = self.response.with_optional_error(error);
        self
    }

    fn is_ok(&self) -> bool {
        self.response.is_ok()
    }

    fn is_err(&self) -> bool {
        self.response.is_err()
    }

    fn ok(&self) -> Option<&(ARID, Envelope)> {
        self.response.ok()
    }

    fn err(&self) -> Option<&(Option<ARID>, Envelope)> {
        self.response.err()
    }

    fn id(&self) -> Option<&ARID> {
        self.response.id()
    }

    fn expect_id(&self) -> &ARID {
        self.response.expect_id()
    }

    fn result(&self) -> Result<&Envelope> {
        self.response.result()
    }

    fn extract_result<T>(&self) -> Result<T>
    where
        T: TryFrom<CBOR, Error = Error> + 'static,
    {
        self.response.extract_result()
    }

    fn error(&self) -> Result<&Envelope> {
        self.response.error()
    }

    fn extract_error<T>(&self) -> Result<T>
    where
        T: TryFrom<CBOR, Error = Error> + 'static,
    {
        self.response.extract_error()
    }
}

/// SealedResponse + optional expiration date -> Envelope
impl From<(SealedResponse, Option<&Date>)> for Envelope {
    fn from((sealed_response, valid_until): (SealedResponse, Option<&Date>)) -> Self {
        let sender_continuation: Option<Envelope>;
        if let Some(state) = &sealed_response.state {
            let continuation = Continuation::new_response(state)
                        .with_optional_valid_until(valid_until);
            sender_continuation =
                Some((
                    continuation,
                    &sealed_response.sender
                ).into());
        } else {
            sender_continuation = None;
        }

        sealed_response.response.into_envelope()
            .add_assertion(known_values::SENDER_PUBLIC_KEY, sealed_response.sender.to_envelope())
            .add_optional_assertion(known_values::SENDER_CONTINUATION, sender_continuation)
            .add_optional_assertion(known_values::RECIPIENT_CONTINUATION, sealed_response.peer_continuation)
    }
}

/// SealedResponse -> Envelope
impl From<SealedResponse> for Envelope {
    fn from(sealed_response: SealedResponse) -> Self {
        (sealed_response, None).into()
    }
}

/// SealedResponse + optional expiration date + sender private key -> signed Envelope
impl From<(SealedResponse, Option<&Date>, &PrivateKeyBase)> for Envelope {
    fn from((sealed_response, valid_until, sender_private_key): (SealedResponse, Option<&Date>, &PrivateKeyBase)) -> Self {
        (sealed_response, valid_until).into_envelope()
            .sign(sender_private_key)
    }
}

/// SealedResponse + sender private key -> signed Envelope
impl From<(SealedResponse, &PrivateKeyBase)> for Envelope {
    fn from((sealed_response, sender_private_key): (SealedResponse, &PrivateKeyBase)) -> Self {
        (sealed_response, None, sender_private_key).into()
    }
}

/// SealedResponse + optional expiration date + sender private key + recipient public key -> signed and encrypted Envelope
impl From<(SealedResponse, Option<&Date>, &PrivateKeyBase, &PublicKeyBase)> for Envelope {
    fn from((sealed_response, valid_until, sender_private_key, recipient_public_key): (SealedResponse, Option<&Date>, &PrivateKeyBase, &PublicKeyBase)) -> Self {
        (sealed_response, valid_until, sender_private_key).into_envelope()
            .encrypt_to_recipient(recipient_public_key)
    }
}

/// SealedResponse + sender private key + recipient public key -> signed and encrypted Envelope
impl From<(SealedResponse, &PrivateKeyBase, &PublicKeyBase)> for Envelope {
    fn from((sealed_response, sender_private_key, recipient_public_key): (SealedResponse, &PrivateKeyBase, &PublicKeyBase)) -> Self {
        (sealed_response, None, sender_private_key, recipient_public_key).into()
    }
}

/// Encrypted Envelope + optional request ID + optional current date + recipient private key -> SealedResponse
impl TryFrom<(Envelope, Option<&ARID>, Option<&Date>, &PrivateKeyBase)> for SealedResponse {
    type Error = Error;

    fn try_from((encrypted_envelope, id, now, recipient_private_key): (Envelope, Option<&ARID>, Option<&Date>, &PrivateKeyBase)) -> Result<Self> {
        let signed_envelope = encrypted_envelope.decrypt_to_recipient(recipient_private_key)?;
        let sender_public_key: PublicKeyBase = signed_envelope.unwrap_envelope()?.extract_object_for_predicate(known_values::SENDER_PUBLIC_KEY)?;
        let response_envelope = signed_envelope.verify(&sender_public_key)?;
        let peer_continuation = response_envelope.optional_object_for_predicate(known_values::SENDER_CONTINUATION)?;
        if let Some(some_peer_continuation) = peer_continuation.clone() {
            if !some_peer_continuation.subject().is_encrypted() {
                bail!("Peer continuation must be encrypted");
            }
        }
        let encrypted_continuation = response_envelope.optional_object_for_predicate(known_values::RECIPIENT_CONTINUATION)?;
        let state: Option<Envelope>;
        if let Some(encrypted_continuation) = encrypted_continuation {
            let continuation = Continuation::try_from((encrypted_continuation, id, now, recipient_private_key))?;
            state = Some(continuation.state().clone());
        } else {
            state = None;
        }
        let response = Response::try_from(response_envelope)?;
        Ok(Self {
            response,
            sender: sender_public_key,
            state,
            peer_continuation,
        })
    }
}

/// Encrypted Envelope + recipient private key -> SealedResponse
impl TryFrom<(Envelope, &PrivateKeyBase)> for SealedResponse {
    type Error = Error;

    fn try_from((encrypted_envelope, recipient_private_key): (Envelope, &PrivateKeyBase)) -> Result<Self> {
        Self::try_from((encrypted_envelope, None, None, recipient_private_key))
    }
}
