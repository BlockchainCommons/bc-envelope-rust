use bytes::Bytes;
use dcbor::prelude::*;
use anyhow::{Error, Result};

use crate::Envelope;

impl TryFrom<Envelope> for Bytes {
    type Error = Error;

    fn try_from(envelope: Envelope) -> Result<Self> {
        envelope.try_leaf()?.try_into()
    }
}

impl Envelope {
    pub fn try_from_cbor(cbor: CBOR) -> Result<Self> {
        cbor.try_into()
    }

    pub fn try_from_cbor_data(data: Bytes) -> Result<Self> {
        let cbor = CBOR::try_from_data(data)?;
        Self::try_from_cbor(cbor)
    }
}
