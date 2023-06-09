use std::rc::Rc;

use bc_components::{EncryptedMessage, Compressed, Signature, SealedMessage, SSKRShare, Digest, CID};
use dcbor::{CBOREncodable, CBOR, Date};

use crate::{Envelope, KnownValue, Assertion};

impl Envelope {
    pub fn enclose_cbor(cbor_encodable: &dyn CBOREncodable) -> Rc<Envelope> {
        cbor_encodable.cbor().enclose()
    }
}

pub trait Enclosable {
    fn enclose(self) -> Rc<Envelope>;
}

impl Enclosable for Rc<Envelope> {
    fn enclose(self) -> Rc<Envelope> {
        Rc::new(Envelope::new_wrapped(self))
    }
}

impl Enclosable for Envelope {
    fn enclose(self) -> Rc<Envelope> {
        Rc::new(Envelope::new_wrapped(Rc::new(self)))
    }
}

impl Enclosable for KnownValue {
    fn enclose(self) -> Rc<Envelope> {
        Rc::new(Envelope::new_with_known_value(self))
    }
}

impl Enclosable for Assertion {
    fn enclose(self) -> Rc<Envelope> {
        Rc::new(Envelope::new_with_assertion(self))
    }
}

impl Enclosable for EncryptedMessage {
    fn enclose(self) -> Rc<Envelope> {
        Rc::new(Envelope::new_with_encrypted(self).unwrap())
    }
}

impl Enclosable for Compressed {
    fn enclose(self) -> Rc<Envelope> {
        Rc::new(Envelope::new_with_compressed(self).unwrap())
    }
}

impl Enclosable for CBOR {
    fn enclose(self) -> Rc<Envelope> {
        Rc::new(Envelope::new_leaf(self))
    }
}

impl Enclosable for &str {
    fn enclose(self) -> Rc<Envelope> {
        Rc::new(Envelope::new_leaf(CBOR::Text(self.to_string())))
    }
}

impl Enclosable for u8 {
    fn enclose(self) -> Rc<Envelope> {
        Rc::new(Envelope::new_leaf(self.cbor()))
    }
}

impl Enclosable for u16 {
    fn enclose(self) -> Rc<Envelope> {
        Rc::new(Envelope::new_leaf(self.cbor()))
    }
}

impl Enclosable for u32 {
    fn enclose(self) -> Rc<Envelope> {
        Rc::new(Envelope::new_leaf(self.cbor()))
    }
}

impl Enclosable for u64 {
    fn enclose(self) -> Rc<Envelope> {
        Rc::new(Envelope::new_leaf(self.cbor()))
    }
}

impl Enclosable for usize {
    fn enclose(self) -> Rc<Envelope> {
        Rc::new(Envelope::new_leaf(self.cbor()))
    }
}

impl Enclosable for i8 {
    fn enclose(self) -> Rc<Envelope> {
        Rc::new(Envelope::new_leaf(self.cbor()))
    }
}

impl Enclosable for i16 {
    fn enclose(self) -> Rc<Envelope> {
        Rc::new(Envelope::new_leaf(self.cbor()))
    }
}

impl Enclosable for i32 {
    fn enclose(self) -> Rc<Envelope> {
        Rc::new(Envelope::new_leaf(self.cbor()))
    }
}

impl Enclosable for i64 {
    fn enclose(self) -> Rc<Envelope> {
        Rc::new(Envelope::new_leaf(self.cbor()))
    }
}

impl Enclosable for &Signature {
    fn enclose(self) -> Rc<Envelope> {
        Envelope::enclose_cbor(self)
    }
}

impl Enclosable for &SealedMessage {
    fn enclose(self) -> Rc<Envelope> {
        Envelope::enclose_cbor(self)
    }
}

impl Enclosable for &SSKRShare {
    fn enclose(self) -> Rc<Envelope> {
        Envelope::enclose_cbor(self)
    }
}

impl Enclosable for &Digest {
    fn enclose(self) -> Rc<Envelope> {
        Envelope::enclose_cbor(self)
    }
}

impl Enclosable for CID {
    fn enclose(self) -> Rc<Envelope> {
        Envelope::enclose_cbor(&self)
    }
}

impl Enclosable for Date {
    fn enclose(self) -> Rc<Envelope> {
        Envelope::enclose_cbor(&self)
    }
}
