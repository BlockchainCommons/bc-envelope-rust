use bytes::Bytes;

use crate::Envelope;

// pub trait EnvelopeDecodable: TryFrom<Envelope, Error = anyhow::Error> {
//     fn from_envelope(envelope: Envelope) -> anyhow::Result<Self> {
//         envelope.try_into()
//     }
// }

// impl<T> EnvelopeDecodable for T
// where
//     T: TryFrom<Envelope, Error = anyhow::Error>
// { }

impl TryFrom<Envelope> for Bytes {
    type Error = anyhow::Error;

    fn try_from(envelope: Envelope) -> anyhow::Result<Self> {
        envelope.expect_leaf()?.try_into()
    }
}
