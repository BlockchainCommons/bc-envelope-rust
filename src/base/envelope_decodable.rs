use dcbor::prelude::*;
use anyhow::{Error, Result};

use crate::Envelope;

/// Implementation for extracting a ByteString from an envelope.
///
/// This allows converting an envelope to a binary data type when the envelope
/// contains appropriate data.
impl TryFrom<Envelope> for ByteString {
    type Error = Error;

    /// Attempts to extract a ByteString from an envelope.
    ///
    /// This will succeed if the envelope is a leaf node containing a ByteString.
    ///
    /// # Returns
    ///
    /// The ByteString contained in the envelope.
    ///
    /// # Errors
    ///
    /// Returns an error if the envelope is not a leaf or does not contain a ByteString.
    fn try_from(envelope: Envelope) -> Result<Self> {
        let cbor = envelope.try_leaf()?;
        cbor.try_into()
    }
}

/// Static methods for creating envelopes from CBOR data.
impl Envelope {
    /// Creates an envelope from a CBOR value.
    ///
    /// # Parameters
    ///
    /// * `cbor` - The CBOR value to convert into an envelope
    ///
    /// # Returns
    ///
    /// A new envelope created from the CBOR data.
    ///
    /// # Errors
    ///
    /// Returns an error if the CBOR does not represent a valid envelope structure.
    pub fn try_from_cbor(cbor: CBOR) -> Result<Self> {
        cbor.try_into()
    }

    /// Creates an envelope from raw CBOR binary data.
    ///
    /// # Parameters
    ///
    /// * `data` - The raw CBOR binary data to convert into an envelope
    ///
    /// # Returns
    ///
    /// A new envelope created from the CBOR data.
    ///
    /// # Errors
    ///
    /// Returns an error if the data is not valid CBOR or does not represent
    /// a valid envelope structure.
    pub fn try_from_cbor_data(data: Vec<u8>) -> Result<Self> {
        let cbor = CBOR::try_from_data(data)?;
        Self::try_from_cbor(cbor)
    }
}

/// Implementation for extracting a String from an envelope.
///
/// This allows converting an envelope to a string when the envelope
/// contains text data.
impl TryFrom<Envelope> for String {
    type Error = Error;

    /// Attempts to extract a String from an envelope.
    ///
    /// This will succeed if the envelope is a leaf node containing a string.
    ///
    /// # Returns
    ///
    /// The String contained in the envelope.
    ///
    /// # Errors
    ///
    /// Returns an error if the envelope is not a leaf or does not contain a string.
    fn try_from(envelope: Envelope) -> Result<Self> {
        let cbor = envelope.try_leaf()?;
        cbor.try_into()
    }
}