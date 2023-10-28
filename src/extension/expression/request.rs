use bc_components::ARID;

use crate::{Envelope, EnvelopeEncodable, EnvelopeDecodable, EnvelopeCodable};

use super::Function;

pub trait RequestBody: EnvelopeCodable {
    fn function() -> Function;
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
            function: Body::function(),
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

    pub fn encrypt(self, sender: &PrivateKeyBase, recipient: &PublicKeyBase) -> anyhow::Result<Envelope> {
        Ok(self.envelope()
            .wrap_envelope()
            .sign_with(sender)
            .encrypt_subject_to_recipient(recipient)?)
    }
}

#[cfg(feature = "encrypt")]
use bc_components::{PrivateKeyBase, PublicKeyBase};

#[cfg(feature = "encrypt")]
impl<Body: RequestBody> Request<Body> {
    pub fn new_encrypted_request(
        id: Option<ARID>,
        body: Body,
        note: String,
        date: Option<dcbor::Date>,
        sender: PrivateKeyBase,
        recipient: PublicKeyBase
    ) -> Envelope {
        let request = Request::new(id, body, note, date);
        request.envelope()
            .wrap_envelope()
            .sign_with(&sender)
            .encrypt_subject_to_recipient(&recipient).unwrap()
    }

    pub fn parse_encrypted_request(request: Envelope, recipient: PrivateKeyBase) -> anyhow::Result<Request<Body>> {
        let decrypted_request = request
            .decrypt_to_recipient(&recipient)?
            .unwrap_envelope()?;

        let request = Request::from_envelope(decrypted_request)?;

        Ok(request)
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
        body_envelope.check_function(&Body::function())?;
        let body = Body::from_envelope(body_envelope)?;
        let note = envelope.request_note()?;
        let date = envelope.request_date()?;
        Ok(Self::new(Some(id), body, note, date))
    }
}

impl<Body: RequestBody> EnvelopeCodable for Request<Body> {}

impl<Body: RequestBody> TryFrom<Envelope> for Request<Body> {
    type Error = anyhow::Error;

    fn try_from(value: Envelope) -> anyhow::Result<Self> {
        Self::from_envelope(value)
    }
}
