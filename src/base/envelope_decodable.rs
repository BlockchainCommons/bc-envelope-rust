use bytes::Bytes;

use crate::Envelope;

pub trait EnvelopeDecodable: TryFrom<Envelope> {
    fn from_envelope(envelope: Envelope) -> anyhow::Result<Self>;
}

impl EnvelopeDecodable for Bytes {
    fn from_envelope(envelope: Envelope) -> anyhow::Result<Self> {
        envelope.expect_leaf()?.clone().try_into()
    }
}

impl TryFrom<Envelope> for Bytes {
    type Error = anyhow::Error;

    fn try_from(value: Envelope) -> anyhow::Result<Self> {
        Self::from_envelope(value)
    }
}
