use bc_components::{Digest, PrivateKeyBase, PublicKeyBase, SSKRShare, Salt, SealedMessage, Signature, ARID, URI, UUID};
#[cfg(feature = "encrypt")]
use bc_components::EncryptedMessage;
#[cfg(feature = "compress")]
use bc_components::Compressed;
#[cfg(any(feature = "encrypt", feature = "compress"))]
use anyhow::{Error, Result};
use dcbor::CBOR;

use crate::{Assertion, Envelope};

pub trait EnvelopeEncodable {
    fn into_envelope(self) -> Envelope;
    fn to_envelope(&self) -> Envelope where Self: Clone {
        self.clone().into_envelope()
    }
}

impl<T> EnvelopeEncodable for T where T: Into<Envelope> + Clone {
    fn into_envelope(self) -> Envelope {
        self.into()
    }
}

impl EnvelopeEncodable for Assertion {
    fn into_envelope(self) -> Envelope {
        Envelope::new_with_assertion(self)
    }
}

#[cfg(feature = "encrypt")]
impl TryFrom<EncryptedMessage> for Envelope {
    type Error = Error;

    fn try_from(value: EncryptedMessage) -> Result<Self> {
        Envelope::new_with_encrypted(value)
    }
}

#[cfg(feature = "compress")]
impl TryFrom<Compressed> for Envelope {
    type Error = Error;

    fn try_from(compressed: Compressed) -> Result<Self> {
        Envelope::new_with_compressed(compressed)
    }
}

impl EnvelopeEncodable for CBOR {
    fn into_envelope(self) -> Envelope {
        Envelope::new_leaf(self)
    }
}

impl EnvelopeEncodable for String {
    fn into_envelope(self) -> Envelope {
        Envelope::new_leaf(self)
    }
}

impl EnvelopeEncodable for &str {
    fn into_envelope(self) -> Envelope {
        Envelope::new_leaf(self)
    }
}

macro_rules! impl_envelope_encodable {
    ($type:ty) => {
        impl EnvelopeEncodable for $type {
            fn into_envelope(self) -> Envelope {
                Envelope::new_leaf(self)
            }
        }
    }
}

impl_envelope_encodable!(u8);
impl_envelope_encodable!(u16);
impl_envelope_encodable!(u32);
impl_envelope_encodable!(u64);
impl_envelope_encodable!(usize);
impl_envelope_encodable!(i8);
impl_envelope_encodable!(i16);
impl_envelope_encodable!(i32);
impl_envelope_encodable!(i64);
impl_envelope_encodable!(bool);
impl_envelope_encodable!(f64);
impl_envelope_encodable!(f32);

impl_envelope_encodable!(dcbor::ByteString);

impl_envelope_encodable!(dcbor::Date);
impl_envelope_encodable!(PublicKeyBase);
impl_envelope_encodable!(PrivateKeyBase);
impl_envelope_encodable!(SealedMessage);
impl_envelope_encodable!(Signature);
impl_envelope_encodable!(SSKRShare);
impl_envelope_encodable!(Digest);
impl_envelope_encodable!(ARID);
impl_envelope_encodable!(Salt);
impl_envelope_encodable!(URI);
impl_envelope_encodable!(UUID);
