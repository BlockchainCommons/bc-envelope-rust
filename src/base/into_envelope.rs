use std::rc::Rc;

use bc_components::{SealedMessage, Digest, ARID, Salt, URI, UUID};
#[cfg(feature = "encrypt")]
use bc_components::EncryptedMessage;
#[cfg(feature = "compress")]
use bc_components::Compressed;
use dcbor::prelude::*;

use crate::{Envelope, Assertion};

/// A type that can be converted into an envelope.
pub trait EnvelopeEncodable {
    fn into_envelope(self) -> Rc<Envelope>;
}

impl EnvelopeEncodable for Rc<Envelope> {
    fn into_envelope(self) -> Rc<Envelope> {
        self
    }
}

impl EnvelopeEncodable for Envelope {
    fn into_envelope(self) -> Rc<Envelope> {
        Rc::new(self)
    }
}

impl EnvelopeEncodable for Assertion {
    fn into_envelope(self) -> Rc<Envelope> {
        Rc::new(Envelope::new_with_assertion(self))
    }
}

impl EnvelopeEncodable for &Assertion {
    fn into_envelope(self) -> Rc<Envelope> {
        self.clone().into_envelope()
    }
}

#[cfg(feature = "encrypt")]
impl EnvelopeEncodable for EncryptedMessage {
    fn into_envelope(self) -> Rc<Envelope> {
        Rc::new(Envelope::new_with_encrypted(self).unwrap())
    }
}

#[cfg(feature = "compress")]
impl EnvelopeEncodable for Compressed {
    fn into_envelope(self) -> Rc<Envelope> {
        Rc::new(Envelope::new_with_compressed(self).unwrap())
    }
}

impl EnvelopeEncodable for CBOR {
    fn into_envelope(self) -> Rc<Envelope> {
        Rc::new(Envelope::new_leaf(self))
    }
}

impl EnvelopeEncodable for &Box<CBOR> {
    fn into_envelope(self) -> Rc<Envelope> {
        self.cbor().into_envelope()
    }
}

impl EnvelopeEncodable for String {
    fn into_envelope(self) -> Rc<Envelope> {
        self.cbor().into_envelope()
    }
}

impl EnvelopeEncodable for &String {
    fn into_envelope(self) -> Rc<Envelope> {
        self.cbor().into_envelope()
    }
}

impl EnvelopeEncodable for &str {
    fn into_envelope(self) -> Rc<Envelope> {
        self.cbor().into_envelope()
    }
}

impl<const N: usize> EnvelopeEncodable for &[u8; N] {
    fn into_envelope(self) -> Rc<Envelope> {
        self.cbor().into_envelope()
    }
}

impl EnvelopeEncodable for &[u8] {
    fn into_envelope(self) -> Rc<Envelope> {
        self.to_vec().cbor().into_envelope()
    }
}

/// A macro that implements EnvelopeEncodable for a type and its reference.
#[macro_export]
macro_rules! impl_into_envelope {
    ($type:ty) => {
        impl $crate::EnvelopeEncodable for $type {
            fn into_envelope(self) -> std::rc::Rc<$crate::Envelope> {
                <Self as dcbor::CBOREncodable>::cbor(&self).into_envelope()
            }
        }

        impl<'a> $crate::EnvelopeEncodable for &'a $type {
            fn into_envelope(self) -> std::rc::Rc<$crate::Envelope> {
                <Self as dcbor::CBOREncodable>::cbor(&self).into_envelope()
            }
        }
    };
}

impl_into_envelope!(u8);
impl_into_envelope!(u16);
impl_into_envelope!(u32);
impl_into_envelope!(u64);
impl_into_envelope!(usize);
impl_into_envelope!(i8);
impl_into_envelope!(i16);
impl_into_envelope!(i32);
impl_into_envelope!(i64);
impl_into_envelope!(bool);
impl_into_envelope!(f64);
impl_into_envelope!(f32);

impl_into_envelope!(SealedMessage);
impl_into_envelope!(Digest);
impl_into_envelope!(ARID);
impl_into_envelope!(dcbor::Date);
impl_into_envelope!(Salt);
impl_into_envelope!(URI);
impl_into_envelope!(UUID);
