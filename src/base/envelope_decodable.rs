use dcbor::prelude::*;
use anyhow::{Error, Result};

use crate::Envelope;

impl TryFrom<Envelope> for ByteString {
    type Error = Error;

    fn try_from(envelope: Envelope) -> Result<Self> {
        let cbor = envelope.try_leaf()?;
        cbor.try_into()
    }
}

impl Envelope {
    pub fn try_from_cbor(cbor: CBOR) -> Result<Self> {
        cbor.try_into()
    }

    pub fn try_from_cbor_data(data: Vec<u8>) -> Result<Self> {
        let cbor = CBOR::try_from_data(data)?;
        Self::try_from_cbor(cbor)
    }
}

impl TryFrom<Envelope> for String {
    type Error = Error;

    fn try_from(envelope: Envelope) -> Result<Self> {
        let cbor = envelope.try_leaf()?;
        cbor.try_into()
    }
}
