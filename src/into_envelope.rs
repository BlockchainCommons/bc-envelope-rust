use std::rc::Rc;

use bc_components::{EncryptedMessage, Compressed, SealedMessage, Digest, CID, Salt};
use dcbor::{CBOREncodable, CBOR, Date};

use crate::{Envelope, Assertion, known_values::KnownValue};

pub trait IntoEnvelope {
    fn into_envelope(self) -> Rc<Envelope>;
}

impl IntoEnvelope for Rc<Envelope> {
    fn into_envelope(self) -> Rc<Envelope> {
        self
    }
}

impl IntoEnvelope for Envelope {
    fn into_envelope(self) -> Rc<Envelope> {
        Rc::new(Envelope::new_wrapped(Rc::new(self)))
    }
}

impl IntoEnvelope for KnownValue {
    fn into_envelope(self) -> Rc<Envelope> {
        Rc::new(Envelope::new_with_known_value(self))
    }
}

impl IntoEnvelope for Assertion {
    fn into_envelope(self) -> Rc<Envelope> {
        Rc::new(Envelope::new_with_assertion(self))
    }
}

impl IntoEnvelope for EncryptedMessage {
    fn into_envelope(self) -> Rc<Envelope> {
        Rc::new(Envelope::new_with_encrypted(self).unwrap())
    }
}

impl IntoEnvelope for Compressed {
    fn into_envelope(self) -> Rc<Envelope> {
        Rc::new(Envelope::new_with_compressed(self).unwrap())
    }
}

impl IntoEnvelope for CBOR {
    fn into_envelope(self) -> Rc<Envelope> {
        Rc::new(Envelope::new_leaf(self))
    }
}

impl IntoEnvelope for &Box<CBOR> {
    fn into_envelope(self) -> Rc<Envelope> {
        self.cbor().into_envelope()
    }
}

impl IntoEnvelope for &str {
    fn into_envelope(self) -> Rc<Envelope> {
        self.cbor().into_envelope()
    }
}

/// A macro that implements IntoEnvelope for a type and its reference.
#[macro_export]
macro_rules! impl_into_envelope {
    ($type:ty) => {
        impl $crate::IntoEnvelope for $type {
            fn into_envelope(self) -> std::rc::Rc<$crate::Envelope> {
                <Self as dcbor::CBOREncodable>::cbor(&self).into_envelope()
            }
        }

        impl<'a> $crate::IntoEnvelope for &'a $type {
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

impl_into_envelope!(SealedMessage);
impl_into_envelope!(Digest);
impl_into_envelope!(CID);
impl_into_envelope!(Date);
impl_into_envelope!(Salt);
