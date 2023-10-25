use crate::Envelope;

pub trait EnvelopeDecodable {
    fn from_envelope(envelope: Envelope) -> anyhow::Result<Self>
    where
        Self: Sized;
}
