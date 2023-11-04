use bc_components::ARID;

use crate::{Envelope, EnvelopeEncodable, EnvelopeDecodable, EnvelopeCodable};

use super::Function;

#[cfg(feature = "encrypt")]
use bc_components::{PrivateKeyBase, PublicKeyBase};

pub trait RequestBody: EnvelopeCodable {
    const FUNCTION_NAME: &'static str;
    const FUNCTION: &'static Function = &Function::new_static_named(Self::FUNCTION_NAME);
}

#[derive(Debug, Clone)]
pub struct Request<Body: RequestBody> {
    id: ARID,
    body: Body,
    function: Function,
    note: String,
    date: Option<dcbor::Date>,
}

impl<Body: RequestBody> Request<Body> {
    pub fn new(id: Option<ARID>, body: Body, note: impl Into<String>, date: Option<dcbor::Date>) -> Self {
        Self {
            id: id.unwrap_or_default(),
            body,
            function: Body::FUNCTION.clone(),
            note: note.into(),
            date,
        }
    }

    pub fn id(&self) -> &ARID {
        &self.id
    }

    pub fn body(&self) -> &Body {
        &self.body
    }

    pub fn function(&self) -> &Function {
        &self.function
    }

    pub fn note(&self) -> &str {
        self.note.as_ref()
    }

    pub fn date(&self) -> Option<&dcbor::Date> {
        self.date.as_ref()
    }
}

#[cfg(all(feature = "signature", feature = "encrypt"))]
impl Envelope {
    pub fn sign_and_encrypt(&self, sender: &PrivateKeyBase, recipient: &PublicKeyBase) -> anyhow::Result<Envelope> {
        Ok(self
            .wrap_envelope()
            .sign_with(sender)
            .encrypt_subject_to_recipient(recipient)?)
    }

    pub fn verify_and_decrypt(&self, sender: &PublicKeyBase, recipient: &PrivateKeyBase) -> anyhow::Result<Envelope> {
        Ok(self
            .verify_signature_from(sender)?
            .decrypt_to_recipient(recipient)?
            .unwrap_envelope()?)
    }
}

impl<Body: RequestBody> PartialEq for Request<Body> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<Body: RequestBody> Eq for Request<Body> {}

impl<Body: RequestBody> EnvelopeEncodable for Request<Body> {
    fn envelope(self) -> Envelope {
        Envelope::new_request_with_metadata(self.id, self.body, self.note, self.date)
    }
}

impl<Body: RequestBody> From<Request<Body>> for Envelope {
    fn from(request: Request<Body>) -> Self {
        request.envelope()
    }
}

impl<Body: RequestBody> EnvelopeDecodable for Request<Body> {
    fn from_envelope(envelope: Envelope) -> anyhow::Result<Self>
    where
        Self: Sized
    {
        let id = envelope.request_id()?;
        let body_envelope = envelope.request_body()?;
        body_envelope.check_function(Body::FUNCTION)?;
        let body = Body::from_envelope(body_envelope)?;
        let note = envelope.request_note()?;
        let date = envelope.request_date()?;
        Ok(Self::new(Some(id), body, note, date))
    }
}

impl<Body: RequestBody> TryFrom<Envelope> for Request<Body> {
    type Error = anyhow::Error;

    fn try_from(value: Envelope) -> anyhow::Result<Self> {
        Self::from_envelope(value)
    }
}

impl<Body: RequestBody> EnvelopeCodable for Request<Body> {}
