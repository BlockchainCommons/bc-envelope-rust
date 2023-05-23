use std::{rc::Rc, any::{Any, TypeId}, borrow::Cow};

use bc_components::{Digest, DigestProvider, tags_registry};
use dcbor::{CBORTagged, Tag, CBOR, CBOREncodable, CBORDecodable, CBORError, CBORCodable, CBORTaggedEncodable, CBORTaggedDecodable, CBORTaggedCodable};

use crate::envelope::{Envelope, IntoEnvelope};

/// Represents an assertion.
#[derive(Clone, Debug)]
pub struct Assertion {
    predicate: Rc<Envelope>,
    object: Rc<Envelope>,
    digest: Digest,
}

impl Assertion {
    /// Creates an assertion and calculates its digest.
    pub fn new<P: IntoEnvelope, O: IntoEnvelope>(predicate: P, object: O) -> Self {
        let p = if TypeId::of::<P>() == TypeId::of::<Rc<Envelope>>() {
            (&predicate as &dyn Any).downcast_ref::<Rc<Envelope>>().unwrap().clone()
        } else if TypeId::of::<P>() == TypeId::of::<Envelope>() {
            Rc::new((&predicate as &dyn Any).downcast_ref::<Envelope>().unwrap().clone())
        } else {
            Envelope::new(predicate)
        };

        let o = if TypeId::of::<O>() == TypeId::of::<Rc<Envelope>>() {
            (&object as &dyn Any).downcast_ref::<Rc<Envelope>>().unwrap().clone()
        } else if TypeId::of::<O>() == TypeId::of::<Envelope>() {
            Rc::new((&object as &dyn Any).downcast_ref::<Envelope>().unwrap().clone())
        } else {
            Envelope::new(object)
        };

        let digest = Digest::from_digests(&[p.digest().into_owned(), o.digest().into_owned()]);
        Self {
            predicate: p,
            object: o,
            digest
        }
    }

    // Returns the predicate of the assertion.
    pub fn predicate(&self) -> Rc<Envelope> {
        self.predicate.clone()
    }

    // Returns the object of the assertion.
    pub fn object(&self) -> Rc<Envelope> {
        self.object.clone()
    }

    pub fn digest_ref(&self) -> &Digest {
        &self.digest
    }
}

impl PartialEq for Assertion {
    fn eq(&self, other: &Self) -> bool {
        self.digest_ref() == other.digest_ref()
    }
}

impl Eq for Assertion { }

impl DigestProvider for Assertion {
    fn digest(&self) -> Cow<Digest> {
        Cow::Borrowed(&self.digest)
    }
}

impl CBORTagged for Assertion {
    const CBOR_TAG: Tag = tags_registry::ASSERTION;
}

impl CBOREncodable for Assertion {
    fn cbor(&self) -> CBOR {
        self.tagged_cbor()
    }
}

impl CBORDecodable for Assertion {
    fn from_cbor(cbor: &CBOR) -> Result<Rc<Self>, CBORError> {
        Self::from_tagged_cbor(cbor)
    }
}

impl CBORCodable for Assertion { }

impl CBORTaggedEncodable for Assertion {
    fn untagged_cbor(&self) -> CBOR {
        vec![self.predicate.cbor(), self.object.cbor()].cbor()
    }
}

impl CBORTaggedDecodable for Assertion {
    fn from_untagged_cbor(cbor: &CBOR) -> Result<Rc<Self>, CBORError> {
        let array: Rc<Vec<Envelope>> = Vec::<Envelope>::from_cbor(cbor)?;
        if array.len() != 2 {
            return Err(CBORError::InvalidFormat);
        }
        Ok(Rc::new(Self::new(array[0].clone(), array[1].clone())))
    }
}

impl CBORTaggedCodable for Assertion { }
