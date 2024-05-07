use bytes::Bytes;

use crate::Envelope;

impl TryFrom<Envelope> for Bytes {
    type Error = anyhow::Error;

    fn try_from(envelope: Envelope) -> anyhow::Result<Self> {
        envelope.try_leaf()?.try_into()
    }
}
