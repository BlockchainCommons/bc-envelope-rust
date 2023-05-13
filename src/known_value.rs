use std::fmt::{Formatter, Display};

use bc_components::tags;
use dcbor::{CBOR, CBORTagged, Tag, CBOREncodable, CBORDecodable, CBORError, CBORCodable, CBORTaggedEncodable, CBORTaggedDecodable, CBORTaggedCodable};

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

    /// The human readable name.
    ///
    /// Defaults to the numerical value if no name has been assigned.
    pub fn name(&self) -> Option<&str> {
        match &self.assigned_name {
            Some(KnownValueName::Static(name)) => Some(name),
            Some(KnownValueName::Dynamic(name)) => Some(name),
            None => None,
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

impl CBORTagged for KnownValue {
    const CBOR_TAG: Tag = tags::KNOWN_VALUE;
}

impl CBOREncodable for KnownValue {
    fn cbor(&self) -> CBOR {
        self.tagged_cbor()
    }
}

impl CBORDecodable for KnownValue {
    fn from_cbor(cbor: &CBOR) -> Result<Box<Self>, CBORError> {
        Self::from_tagged_cbor(cbor)
    }
}

impl CBORCodable for KnownValue { }

impl CBORTaggedEncodable for KnownValue {
    fn untagged_cbor(&self) -> CBOR {
        self.value.cbor()
    }
}

impl CBORTaggedDecodable for KnownValue {
    fn from_untagged_cbor(cbor: &CBOR) -> Result<Box<Self>, CBORError> {
        let value = *u64::from_cbor(cbor)?;
        Ok(Box::new(Self::new(value)))
    }
}

impl CBORTaggedCodable for KnownValue { }

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
