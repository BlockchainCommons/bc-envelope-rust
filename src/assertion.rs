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
    predicate: Box<Envelope>,
    object: Box<Envelope>,
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
        let digest = Digest::new_from_data_providers(&[&predicate, &object]);
        Self {
            predicate: Box::new(predicate),
            object: Box::new(object),
            digest
        }
    }

    // Returns the predicate of the assertion.
    pub fn predicate(&self) -> &Envelope {
        &self.predicate
    }

    // Returns the object of the assertion.
    pub fn object(&self) -> &Envelope {
        &self.object
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
    fn from_cbor(cbor: &CBOR) -> Result<Box<Self>, CBORError> {
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
    fn from_untagged_cbor(cbor: &CBOR) -> Result<Box<Self>, CBORError> {
        let array: Box<Vec<Envelope>> = Vec::<Envelope>::from_cbor(cbor)?;
        if array.len() != 2 {
            return Err(CBORError::InvalidFormat);
        }
        Ok(Box::new(Self::new(array[0].clone(), array[1].clone())))
    }
}

impl CBORTaggedCodable for Assertion { }
