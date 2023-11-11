use std::borrow::Cow;

use anyhow::bail;
use bc_components::{Digest, DigestProvider};
use dcbor::prelude::*;

use crate::{Envelope, EnvelopeEncodable};

/// Represents an assertion.
///
/// Generally you do not create an instance of this type directly, but
/// instead use [`Envelope::new_assertion`], or the various functions
/// on ``Envelope`` that create assertions.
#[derive(Clone, Debug)]
pub struct Assertion {
    predicate: Envelope,
    object: Envelope,
    digest: Digest,
}

impl Assertion {
    /// Creates an assertion and calculates its digest.
    pub fn new(predicate: impl EnvelopeEncodable, object: impl EnvelopeEncodable) -> Self {
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
    pub fn predicate(&self) -> Envelope {
        self.predicate.clone()
    }

    /// Returns the object of the assertion.
    pub fn object(&self) -> Envelope {
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

impl CBOREncodable for Assertion {
    fn cbor(&self) -> CBOR {
        let mut map = Map::new();
        map.insert(self.predicate.cbor(), self.object.cbor());
        map.cbor()
    }
}

impl From<Assertion> for CBOR {
    fn from(value: Assertion) -> Self {
        value.cbor()
    }
}

impl CBORDecodable for Assertion {
    fn from_cbor(cbor: &CBOR) -> anyhow::Result<Self> {
        if let CBORCase::Map(map) = cbor.case() {
            if map.len() != 1 {
                bail!("assertion map must have exactly one element")
            }
            let elem = map.iter().next().unwrap();
            let predicate = Envelope::from_cbor(elem.0)?;
            let object = Envelope::from_cbor(elem.1)?;
            return Ok(Self::new(predicate, object));
        }
        bail!("assertion must be a map")
    }
}

impl TryFrom<CBOR> for Assertion {
    type Error = anyhow::Error;

    fn try_from(value: CBOR) -> Result<Self, Self::Error> {
        Self::from_cbor(&value)
    }
}

impl CBORCodable for Assertion {}
