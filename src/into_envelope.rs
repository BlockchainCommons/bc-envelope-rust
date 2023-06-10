use std::rc::Rc;

use bc_components::{EncryptedMessage, Compressed, Signature, SealedMessage, SSKRShare, Digest, CID, Salt};
use dcbor::{CBOREncodable, CBOR, Date};

use crate::{Envelope, KnownValue, Assertion, Function, Parameter};

impl Envelope {
    pub fn cbor_into_envelope(cbor_encodable: &dyn CBOREncodable) -> Rc<Envelope> {
        cbor_encodable.cbor().into_envelope()
    }
}

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

impl IntoEnvelope for &dyn CBOREncodable {
    fn into_envelope(self) -> Rc<Envelope> {
        self.cbor().into_envelope()
    }
}

impl IntoEnvelope for &Box<CBOR> {
    fn into_envelope(self) -> Rc<Envelope> {
        Rc::new(Envelope::new_leaf(self.as_ref().clone()))
    }
}

impl IntoEnvelope for &str {
    fn into_envelope(self) -> Rc<Envelope> {
        Rc::new(Envelope::new_leaf(CBOR::Text(self.to_string())))
    }
}

impl IntoEnvelope for u8 {
    fn into_envelope(self) -> Rc<Envelope> {
        Rc::new(Envelope::new_leaf(self.cbor()))
    }
}

impl IntoEnvelope for u16 {
    fn into_envelope(self) -> Rc<Envelope> {
        Rc::new(Envelope::new_leaf(self.cbor()))
    }
}

impl IntoEnvelope for u32 {
    fn into_envelope(self) -> Rc<Envelope> {
        Rc::new(Envelope::new_leaf(self.cbor()))
    }
}

impl IntoEnvelope for u64 {
    fn into_envelope(self) -> Rc<Envelope> {
        Rc::new(Envelope::new_leaf(self.cbor()))
    }
}

impl IntoEnvelope for usize {
    fn into_envelope(self) -> Rc<Envelope> {
        Rc::new(Envelope::new_leaf(self.cbor()))
    }
}

impl IntoEnvelope for i8 {
    fn into_envelope(self) -> Rc<Envelope> {
        Rc::new(Envelope::new_leaf(self.cbor()))
    }
}

impl IntoEnvelope for i16 {
    fn into_envelope(self) -> Rc<Envelope> {
        Rc::new(Envelope::new_leaf(self.cbor()))
    }
}

impl IntoEnvelope for i32 {
    fn into_envelope(self) -> Rc<Envelope> {
        Rc::new(Envelope::new_leaf(self.cbor()))
    }
}

impl IntoEnvelope for i64 {
    fn into_envelope(self) -> Rc<Envelope> {
        Rc::new(Envelope::new_leaf(self.cbor()))
    }
}

impl IntoEnvelope for Function {
    fn into_envelope(self) -> Rc<Envelope> {
        Rc::new(Envelope::new_leaf(self.cbor()))
    }
}

impl IntoEnvelope for Parameter {
    fn into_envelope(self) -> Rc<Envelope> {
        Rc::new(Envelope::new_leaf(self.cbor()))
    }
}

impl IntoEnvelope for &Signature {
    fn into_envelope(self) -> Rc<Envelope> {
        Envelope::cbor_into_envelope(self)
    }
}

impl IntoEnvelope for &SealedMessage {
    fn into_envelope(self) -> Rc<Envelope> {
        Envelope::cbor_into_envelope(self)
    }
}

impl IntoEnvelope for &SSKRShare {
    fn into_envelope(self) -> Rc<Envelope> {
        Envelope::cbor_into_envelope(self)
    }
}

impl IntoEnvelope for &Digest {
    fn into_envelope(self) -> Rc<Envelope> {
        Envelope::cbor_into_envelope(self)
    }
}

impl IntoEnvelope for CID {
    fn into_envelope(self) -> Rc<Envelope> {
        Envelope::cbor_into_envelope(&self)
    }
}

impl IntoEnvelope for Date {
    fn into_envelope(self) -> Rc<Envelope> {
        Envelope::cbor_into_envelope(&self)
    }
}

impl IntoEnvelope for Salt {
    fn into_envelope(self) -> Rc<Envelope> {
        Envelope::cbor_into_envelope(&self)
    }
}

impl IntoEnvelope for SealedMessage {
    fn into_envelope(self) -> Rc<Envelope> {
        Envelope::cbor_into_envelope(&self)
    }
}