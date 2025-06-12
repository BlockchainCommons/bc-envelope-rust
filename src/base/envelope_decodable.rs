use bc_components::{
    ARID, Digest, PrivateKeyBase, PrivateKeys, PublicKeys, Reference,
    SSKRShare, Salt, SealedMessage, Signature, URI, UUID, XID,
};
use dcbor::prelude::*;

use crate::Envelope;

#[macro_export]
macro_rules! impl_envelope_decodable {
    ($type:ty) => {
        impl TryFrom<Envelope> for $type {
            type Error = anyhow::Error;

            fn try_from(envelope: Envelope) -> anyhow::Result<Self> {
                let cbor = envelope.try_leaf()?;
                Ok(cbor.try_into()?)
            }
        }
    };
}

impl_envelope_decodable!(String);

// Numeric types
impl_envelope_decodable!(u8);
impl_envelope_decodable!(u16);
impl_envelope_decodable!(u32);
impl_envelope_decodable!(u64);
impl_envelope_decodable!(usize);
impl_envelope_decodable!(i8);
impl_envelope_decodable!(i16);
impl_envelope_decodable!(i32);
impl_envelope_decodable!(i64);

// Boolean type
impl_envelope_decodable!(bool);

// CBOR types
impl_envelope_decodable!(dcbor::ByteString);
impl_envelope_decodable!(dcbor::Date);

// Floating point types
impl_envelope_decodable!(f64);
impl_envelope_decodable!(f32);

// Cryptographic types
impl_envelope_decodable!(PublicKeys);
impl_envelope_decodable!(PrivateKeys);
impl_envelope_decodable!(PrivateKeyBase);
impl_envelope_decodable!(SealedMessage);
impl_envelope_decodable!(Signature);
impl_envelope_decodable!(SSKRShare);
impl_envelope_decodable!(Digest);
impl_envelope_decodable!(Salt);

// Identifier types
impl_envelope_decodable!(ARID);
impl_envelope_decodable!(URI);
impl_envelope_decodable!(UUID);
impl_envelope_decodable!(XID);
impl_envelope_decodable!(Reference);

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
    /// Returns an error if the CBOR does not represent a valid envelope
    /// structure.
    pub fn try_from_cbor(cbor: CBOR) -> dcbor::Result<Self> { cbor.try_into() }

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
    pub fn try_from_cbor_data(data: Vec<u8>) -> dcbor::Result<Self> {
        let cbor = CBOR::try_from_data(data)?;
        Self::try_from_cbor(cbor)
    }
}
