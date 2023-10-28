use bc_components::ARID;

use crate::{Envelope, EnvelopeEncodable, EnvelopeDecodable, EnvelopeCodable};

#[derive(Debug, Clone)]
pub struct Response {
    id: ARID,
    result: Envelope,
}

impl Response {
    pub fn new(id: ARID, result: impl EnvelopeEncodable) -> Self {
        Self {
            id,
            result: result.envelope(),
        }
    }

    pub fn id(&self) -> &ARID {
        &self.id
    }

    pub fn result(&self) -> &Envelope {
        &self.result
    }
}

impl PartialEq for Response {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Response {}

impl EnvelopeEncodable for Response {
    fn envelope(self) -> Envelope {
        Envelope::new_response(self.id, self.result)
    }
}

impl From<Response> for Envelope {
    fn from(response: Response) -> Self {
        response.envelope()
    }
}

impl EnvelopeDecodable for Response {
    fn from_envelope(envelope: Envelope) -> anyhow::Result<Self>
    where
        Self: Sized
    {
        let id = envelope.request_id()?;
        let result = envelope.result()?;
        Ok(Self::new(id, result))
    }
}

impl EnvelopeCodable for Response {}

impl TryFrom<Envelope> for Response {
    type Error = anyhow::Error;

    fn try_from(value: Envelope) -> anyhow::Result<Self> {
        Self::from_envelope(value)
    }
}
