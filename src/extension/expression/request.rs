use bc_components::ARID;

use crate::{Envelope, EnvelopeEncodable, EnvelopeDecodable, known_values::{DATE, NOTE}, EnvelopeCodable};

use super::Function;

#[derive(Debug, Clone)]
pub struct Request {
    id: ARID,
    body: Envelope,
    function: Function,
    note: String,
    date: Option<dcbor::Date>,
}

impl Request {
    pub fn new(id: ARID, body: impl EnvelopeEncodable, note: String, date: Option<dcbor::Date>) -> Self {
        let body = body.envelope();
        let function = body.function().unwrap();
        Self {
            id,
            body,
            function,
            note,
            date,
        }
    }

    pub fn transaction_id(&self) -> &ARID {
        &self.id
    }

    pub fn body(&self) -> &Envelope {
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

impl PartialEq for Request {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Request {}

impl EnvelopeEncodable for Request {
    fn envelope(self) -> Envelope {
        Envelope::new_request(self.id, self.body)
            .add_assertion_if(!self.note.is_empty(), NOTE, self.note)
            .add_optional_assertion(DATE, self.date)
    }
}

impl EnvelopeDecodable for Request {
    fn from_envelope(envelope: Envelope) -> anyhow::Result<Self>
    where
        Self: Sized
    {
        let id = envelope.request_id()?;
        let body = envelope.request_body()?;
        let note = envelope.extract_optional_object_for_predicate::<String>(NOTE)?.unwrap_or_default();
        let date = envelope.extract_optional_object_for_predicate::<dcbor::Date>(DATE)?;
        Ok(Self::new(id, body, note, date))
    }
}

impl EnvelopeCodable for Request {}

impl From<Request> for Envelope {
    fn from(value: Request) -> Self {
        value.envelope()
    }
}

impl TryFrom<Envelope> for Request {
    type Error = anyhow::Error;

    fn try_from(value: Envelope) -> anyhow::Result<Self> {
        Self::from_envelope(value)
    }
}
