use bc_components::{Digest, PrivateKeyBase, PublicKeyBase, SSKRShare, Salt, SealedMessage, Signature, ARID, URI, UUID};
#[cfg(feature = "encrypt")]
use bc_components::EncryptedMessage;
#[cfg(feature = "compress")]
use bc_components::Compressed;
use bytes::Bytes;
use dcbor::CBOR;

use crate::{Assertion, Envelope};

pub trait EnvelopeEncodable {
    fn to_envelope(&self) -> Envelope;
}

impl EnvelopeEncodable for Envelope {
    fn to_envelope(&self) -> Envelope {
        self.clone()
    }
}

impl EnvelopeEncodable for &Envelope {
    fn to_envelope(&self) -> Envelope {
        (*self).clone()
    }
}

impl EnvelopeEncodable for Assertion {
    fn to_envelope(&self) -> Envelope {
        Envelope::new_with_assertion(self.clone())
    }
}

#[cfg(feature = "encrypt")]
impl TryFrom<EncryptedMessage> for Envelope {
    type Error = anyhow::Error;

    fn try_from(value: EncryptedMessage) -> Result<Self, Self::Error> {
        Envelope::new_with_encrypted(value).map_err(|e| e.into())
    }
}

#[cfg(feature = "compress")]
impl TryFrom<Compressed> for Envelope {
    type Error = anyhow::Error;

    fn try_from(compressed: Compressed) -> anyhow::Result<Self> {
        Envelope::new_with_compressed(compressed).map_err(|e| e.into())
    }
}

impl EnvelopeEncodable for CBOR {
    fn to_envelope(&self) -> Envelope {
        Envelope::new_leaf(self.clone())
    }
}

// impl From<CBOR> for Envelope {
//     fn from(cbor: CBOR) -> Self {
//         Envelope::new_leaf(cbor)
//     }
// }

// impl From<Box<CBOR>> for Envelope {
//     fn from(cbor: Box<CBOR>) -> Self {
//         Envelope::new_leaf(*cbor)
//     }
// }

// impl From<&Box<CBOR>> for Envelope {
//     fn from(cbor: &Box<CBOR>) -> Self {
//         Envelope::new_leaf(*cbor.clone())
//     }
// }

impl EnvelopeEncodable for String {
    fn to_envelope(&self) -> Envelope {
        Envelope::new_leaf(self.clone())
    }
}

impl EnvelopeEncodable for &str {
    fn to_envelope(&self) -> Envelope {
        Envelope::new_leaf(*self)
    }
}

macro_rules! impl_envelope_encodable {
    ($type:ty) => {
        impl EnvelopeEncodable for $type {
            fn to_envelope(&self) -> Envelope {
                Envelope::new_leaf(self.clone())
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

impl_envelope_encodable!(Bytes);

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
