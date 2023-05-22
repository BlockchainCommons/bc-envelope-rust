use std::{rc::Rc, any::{Any, TypeId}};

use bc_components::{Digest, DigestProvider, tags_registry};
use dcbor::{CBORTagged, Tag, CBOR, CBOREncodable, CBORDecodable, CBORError, CBORCodable, CBORTaggedEncodable, CBORTaggedDecodable, CBORTaggedCodable};

use crate::envelope::{Envelope, IntoEnvelope};

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

/*
```rust

    /// Returns the envelope's subject, decoded as the given type.
    ///
    /// If the encoded type doesn't match the given type, returns `EnvelopeError::InvalidFormat`.
    pub fn extract_subject<T>(&self) -> Result<Rc<T>, EnvelopeError>
    where
        T: Any + CBORDecodable,
    {
        fn extract_type<T, U>(value: &U) -> Result<Rc<T>, EnvelopeError>
        where
            T: Any,
            U: Any + Clone,
        {
            if TypeId::of::<T>() == TypeId::of::<U>() {
                Ok((Rc::new(value.clone()) as Rc<dyn Any>)
                    .downcast::<T>()
                    .unwrap())
            } else {
                Err(EnvelopeError::InvalidFormat)
            }
        }

        match self {
            Envelope::Wrapped { envelope, .. } => extract_type::<T, Envelope>(&**envelope),
            Envelope::Node { subject, .. } => subject.extract_subject::<T>(),
            Envelope::Leaf { cbor, .. } => Ok(T::from_cbor(cbor).map_err(EnvelopeError::CBORError)?),
            Envelope::KnownValue { value, .. } => extract_type::<T, KnownValue>(&*value),
            Envelope::Assertion(assertion) => extract_type::<T, Assertion>(&*assertion),
            Envelope::Encrypted(encrypted_message) => extract_type::<T, EncryptedMessage>(&*encrypted_message),
            Envelope::Compressed(compressed) => extract_type::<T, Compressed>(&*compressed),
            Envelope::Elided(digest) => extract_type::<T, Digest>(&*digest),
        }
    }
}
```
 */

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

        let digest = Digest::from_digests(&[p.digest(), o.digest()]);
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

impl DigestProvider for Assertion {
    fn digest(&self) -> Digest {
        self.digest.clone()
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
