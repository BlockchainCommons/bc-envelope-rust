use std::{borrow::Cow, rc::Rc};

use bc_components::{tags, Digest, DigestProvider};
use dcbor::{
    CBORCodable, CBORDecodable, CBOREncodable, CBORTagged, CBORTaggedCodable, CBORTaggedDecodable,
    CBORTaggedEncodable, Tag, CBOR,
};

use crate::{envelope::Envelope, IntoEnvelope};

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
        P: IntoEnvelope,
        O: IntoEnvelope,
    {
        let predicate = Envelope::new(predicate);
        let object = Envelope::new(object);
        let digest = Digest::from_digests(&[
            predicate.digest().into_owned(),
            object.digest().into_owned(),
        ]);
        Self {
            predicate,
            object,
            digest,
        }
    }

    /// Returns the predicate of the assertion.
    pub fn predicate(&self) -> Rc<Envelope> {
        self.predicate.clone()
    }

    /// Returns the object of the assertion.
    pub fn object(&self) -> Rc<Envelope> {
        self.object.clone()
    }

    /// Returns the digest of the assertion.
    pub fn digest_ref(&self) -> &Digest {
        &self.digest
    }
}

impl PartialEq for Assertion {
    fn eq(&self, other: &Self) -> bool {
        self.digest_ref() == other.digest_ref()
    }
}

impl Eq for Assertion {}

impl DigestProvider for Assertion {
    fn digest(&self) -> Cow<'_, Digest> {
        Cow::Borrowed(&self.digest)
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
    fn from_cbor(cbor: &CBOR) -> Result<Self, dcbor::Error> {
        Self::from_tagged_cbor(cbor)
    }
}

impl CBORCodable for Assertion {}

impl CBORTaggedEncodable for Assertion {
    fn untagged_cbor(&self) -> CBOR {
        vec![self.predicate.cbor(), self.object.cbor()].cbor()
    }
}

impl CBORTaggedDecodable for Assertion {
    fn from_untagged_cbor(cbor: &CBOR) -> Result<Self, dcbor::Error> {
        let array: Vec<Rc<Envelope>> = Vec::<Envelope>::from_cbor(cbor)?
            .into_iter()
            .map(Rc::new)
            .collect();
        if array.len() != 2 {
            return Err(dcbor::Error::InvalidFormat);
        }
        Ok(Self::new(array[0].clone(), array[1].clone()))
    }
}

impl CBORTaggedCodable for Assertion {}
