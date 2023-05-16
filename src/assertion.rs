use std::rc::Rc;

use bc_components::{Digest, DigestProvider, tags};
use dcbor::{CBORTagged, Tag, CBOR, CBOREncodable, CBORDecodable, CBORError, CBORCodable, CBORTaggedEncodable, CBORTaggedDecodable, CBORTaggedCodable};

use crate::envelope::Envelope;

/*
```swift
/// Represents an assertion in an envelope.
///
/// This structure is public but opaque, and the APIs on ``Envelope`` itself should be used to manipulate it.
public struct Assertion {
    let predicate: Envelope
    let object: Envelope
    let digest: Digest

    /// Creates an ``Assertion`` and calculates its digest.
    init(predicate: Any, object: Any) {
        let p: Envelope
        if let predicate = predicate as? Envelope {
            p = predicate
        } else {
            p = Envelope(predicate)
        }
        let o: Envelope
        if let object = object as? Envelope {
            o = object
        } else {
            o = Envelope(object)
        }
        self.predicate = p
        self.object = o
        self.digest = Digest(p.digest + o.digest)
    }
}
```
 */

/// Represents an assertion.
#[derive(Clone, Debug)]
pub struct Assertion {
    predicate: Rc<Envelope>,
    object: Rc<Envelope>,
    digest: Digest,
}

impl Assertion {
    /// Creates an assertion and calculates its digest.
    pub fn new<P, O>(predicate: P, object: O) -> Self
    where
        P: Into<Envelope>,
        O: Into<Envelope>,
    {
        let predicate = predicate.into();
        let object = object.into();
        let digest = Digest::from_image_parts(&[&predicate.cbor_data(), &object.cbor_data()]);
        Self {
            predicate: Rc::new(predicate),
            object: Rc::new(object),
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

impl DigestProvider for Assertion {
    fn digest(&self) -> Digest {
        self.digest.clone()
    }
}

impl CBORTagged for Assertion {
    const CBOR_TAG: Tag = tags::ASSERTION;
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
