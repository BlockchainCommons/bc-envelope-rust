use bc_components::{SealedMessage, Digest, ARID, Salt, URI, UUID};
#[cfg(feature = "encrypt")]
use bc_components::EncryptedMessage;
#[cfg(feature = "compress")]
use bc_components::Compressed;
use dcbor::prelude::*;

use crate::{Envelope, Assertion};

/// A type that can be converted into an envelope.
pub trait EnvelopeEncodable {
    fn envelope(self) -> Envelope;
}

impl EnvelopeEncodable for Envelope {
    fn envelope(self) -> Envelope {
        self
    }
}

impl EnvelopeEncodable for Assertion {
    fn envelope(self) -> Envelope {
        Envelope::new_with_assertion(self)
    }
}

impl EnvelopeEncodable for &Assertion {
    fn envelope(self) -> Envelope {
        self.clone().envelope()
    }
}

#[cfg(feature = "encrypt")]
impl EnvelopeEncodable for EncryptedMessage {
    fn envelope(self) -> Envelope {
        Envelope::new_with_encrypted(self).unwrap()
    }
}

#[cfg(feature = "compress")]
impl EnvelopeEncodable for Compressed {
    fn envelope(self) -> Envelope {
        Envelope::new_with_compressed(self).unwrap()
    }
}

impl EnvelopeEncodable for CBOR {
    fn envelope(self) -> Envelope {
        Envelope::new_leaf(self)
    }
}

impl EnvelopeEncodable for &Box<CBOR> {
    fn envelope(self) -> Envelope {
        self.cbor().envelope()
    }
}

impl EnvelopeEncodable for String {
    fn envelope(self) -> Envelope {
        self.cbor().envelope()
    }
}

impl EnvelopeEncodable for &String {
    fn envelope(self) -> Envelope {
        self.cbor().envelope()
    }
}

impl EnvelopeEncodable for &str {
    fn envelope(self) -> Envelope {
        self.cbor().envelope()
    }
}

impl<const N: usize> EnvelopeEncodable for &[u8; N] {
    fn envelope(self) -> Envelope {
        self.cbor().envelope()
    }
}

impl EnvelopeEncodable for &[u8] {
    fn envelope(self) -> Envelope {
        self.to_vec().cbor().envelope()
    }
}

/// A macro that implements EnvelopeEncodable for a type and its reference.
#[macro_export]
macro_rules! impl_envelope_encodable {
    ($type:ty) => {
        impl $crate::EnvelopeEncodable for $type {
            fn envelope(self) -> $crate::Envelope {
                <Self as dcbor::CBOREncodable>::cbor(&self).envelope()
            }
        }

        impl<'a> $crate::EnvelopeEncodable for &'a $type {
            fn envelope(self) -> $crate::Envelope {
                <Self as dcbor::CBOREncodable>::cbor(&self).envelope()
            }
        }
    };
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

impl_envelope_encodable!(SealedMessage);
impl_envelope_encodable!(Digest);
impl_envelope_encodable!(ARID);
impl_envelope_encodable!(dcbor::Date);
impl_envelope_encodable!(Salt);
impl_envelope_encodable!(URI);
impl_envelope_encodable!(UUID);
