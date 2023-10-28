use crate::Envelope;

pub trait EnvelopeDecodable: TryFrom<Envelope> {
    fn from_envelope(envelope: Envelope) -> anyhow::Result<Self>
    where
        Self: Sized;
}
