use std::{fmt::{Formatter, Display}, borrow::Cow};

use anyhow::{Result, Error};
use bc_components::{tags, DigestProvider, Digest};
use dcbor::prelude::*;

use crate::{Envelope, EnvelopeEncodable};

#[derive(Debug, Clone)]
enum KnownValueName {
    Static(&'static str),
    Dynamic(String),
}

/// A value in a namespace of unsigned integers, frequently used as predicates.
///
/// Known values are a specific case of envelope that defines a namespace consisting
/// of single unsigned integers. The expectation is that the most common and widely
/// useful predicates will be assigned in this namespace, but known values may be
/// used in any position in an envelope.
#[derive(Clone, Debug)]
pub struct KnownValue {
    /// The known value as coded into CBOR.
    value: u64,
    /// A name assigned to the known value used for debugging and formatted output.
    assigned_name: Option<KnownValueName>,
}

impl KnownValue {
    /// Create a known value with the given value and no name.
    pub fn new(value: u64) -> Self {
        Self { value, assigned_name: None }
    }

    /// Create a known value with the given value and associated name.
    pub fn new_with_name<T: Into<u64>>(value: T, assigned_name: String) -> Self {
        Self { value: value.into(), assigned_name: Some(KnownValueName::Dynamic(assigned_name)) }
    }

    /// Creates a known value at compile time with the given value and associated name.
    pub const fn new_with_static_name(value: u64, name: &'static str) -> Self {
        Self { value, assigned_name: Some(KnownValueName::Static(name)) }
    }

    /// The known value as coded into CBOR.
    pub fn value(&self) -> u64 {
        self.value
    }

    /// The name assigned to the known value.
    pub fn assigned_name(&self) -> Option<&str> {
        match &self.assigned_name {
            Some(KnownValueName::Static(name)) => Some(name),
            Some(KnownValueName::Dynamic(name)) => Some(name),
            None => None,
        }
    }

    /// The human readable name.
    ///
    /// Defaults to the numerical value if no name has been assigned.
    pub fn name(&self) -> String {
        match &self.assigned_name {
            Some(KnownValueName::Static(name)) => name.to_string(),
            Some(KnownValueName::Dynamic(name)) => name.clone(),
            None => self.value.to_string(),
        }
    }
}

impl PartialEq for KnownValue {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl Eq for KnownValue { }

impl std::hash::Hash for KnownValue {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.value.hash(state);
    }
}

impl Display for KnownValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self.assigned_name {
            Some(KnownValueName::Static(name)) => write!(f, "{}", name),
            Some(KnownValueName::Dynamic(name)) => write!(f, "{}", name),
            None => write!(f, "{}", self.value),
        }
    }
}

impl EnvelopeEncodable for KnownValue {
    fn into_envelope(self) -> Envelope {
        Envelope::new_with_known_value(self)
    }
}

impl DigestProvider for KnownValue {
    fn digest(&self) -> Cow<'_, Digest> {
        Cow::Owned(Digest::from_image(self.tagged_cbor().to_cbor_data()))
    }
}

impl CBORTagged for KnownValue {
    fn cbor_tags() -> Vec<Tag> {
        tags_for_values(&[tags::TAG_KNOWN_VALUE])
    }
}

impl From<KnownValue> for CBOR {
    fn from(value: KnownValue) -> Self {
        value.tagged_cbor()
    }
}


impl TryFrom<CBOR> for KnownValue {
    type Error = Error;

    fn try_from(cbor: CBOR) -> Result<Self> {
        Self::from_tagged_cbor(cbor)
    }
}

impl CBORTaggedEncodable for KnownValue {
    fn untagged_cbor(&self) -> CBOR {
        self.value.into()
    }
}

impl CBORTaggedDecodable for KnownValue {
    fn from_untagged_cbor(cbor: CBOR) -> Result<Self> {
        let value = u64::try_from(cbor)?;
        Ok(Self::new(value))
    }
}

impl From<u64> for KnownValue {
    fn from(value: u64) -> Self {
        KnownValue::new(value)
    }
}

impl From<i32> for KnownValue {
    fn from(value: i32) -> Self {
        KnownValue::new(value as u64)
    }
}

impl From<usize> for KnownValue {
    fn from(value: usize) -> Self {
        KnownValue::new(value as u64)
    }
}
