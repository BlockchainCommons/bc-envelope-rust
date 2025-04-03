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

/// A value in a namespace of unsigned integers that represents a stand-alone ontological concept.
///
/// Known Values provide a compact, deterministic way to represent commonly used ontological concepts
/// such as relationships between entities, classes of entities, properties, or enumerated values.
/// They are particularly useful as predicates in Envelope assertions, offering a more compact and
/// deterministic alternative to URIs.
///
/// In an Envelope, a Known Value is represented as a 64-bit unsigned integer with an optional
/// human-readable name. This approach ensures:
///
/// - **Compact binary representation** - Each Known Value requires only 1-9 bytes depending on value range
/// - **Deterministic encoding** - Every concept has exactly one valid binary representation
/// - **Enhanced security** - Eliminates URI manipulation vulnerabilities
/// - **Standardized semantics** - Values are registered in a central registry
///
/// While Known Values are most commonly used as predicates in assertions, they can appear in any
/// position in an Envelope (subject, predicate, or object).
///
/// # Examples
///
/// ```
/// use bc_envelope::prelude::*;
///
/// // Create a Known Value with a numeric value
/// let known_value = KnownValue::new(42);
/// assert_eq!(known_value.value(), 42);
///
/// // Create a Known Value with a name
/// let named_value = KnownValue::new_with_name(1u64, "isA".to_string());
/// assert_eq!(named_value.value(), 1);
/// assert_eq!(named_value.name(), "isA");
///
/// // Convert a Known Value to an Envelope
/// let envelope = named_value.into_envelope();
/// assert_eq!(envelope.extract_subject::<KnownValue>().unwrap().value(), 1);
///
/// // Use a pre-defined Known Value from the registry
/// let is_a_value = known_values::IS_A;
/// assert_eq!(is_a_value.value(), 1);
/// assert_eq!(is_a_value.name(), "isA");
/// ```
///
/// # Specification
///
/// Known Values are defined in [BCR-2023-002](https://github.com/BlockchainCommons/Research/blob/master/papers/bcr-2023-002-known-value.md)
/// and implemented as an Envelope extension in [BCR-2023-003](https://github.com/BlockchainCommons/Research/blob/master/papers/bcr-2023-003-envelope-known-value.md).
#[derive(Clone, Debug)]
pub struct KnownValue {
    /// The known value as coded into CBOR.
    value: u64,
    /// A name assigned to the known value used for debugging and formatted output.
    assigned_name: Option<KnownValueName>,
}

impl KnownValue {
    /// Creates a new KnownValue with the given numeric value and no name.
    ///
    /// # Examples
    ///
    /// ```
    /// use bc_envelope::prelude::*;
    /// 
    /// let known_value = KnownValue::new(42);
    /// assert_eq!(known_value.value(), 42);
    /// ```
    pub fn new(value: u64) -> Self {
        Self { value, assigned_name: None }
    }

    /// Creates a KnownValue with the given value and associated name.
    ///
    /// This function accepts any type that can be converted into a `u64` and
    /// a String for the name. The name is stored as a dynamic value.
    ///
    /// # Examples
    ///
    /// ```
    /// use bc_envelope::prelude::*;
    /// 
    /// let known_value = KnownValue::new_with_name(1u64, "isA".to_string());
    /// assert_eq!(known_value.value(), 1);
    /// assert_eq!(known_value.name(), "isA");
    /// ```
    pub fn new_with_name<T: Into<u64>>(value: T, assigned_name: String) -> Self {
        Self { value: value.into(), assigned_name: Some(KnownValueName::Dynamic(assigned_name)) }
    }

    /// Creates a KnownValue at compile time with the given value and static name.
    ///
    /// This function is used primarily with the `known_value_constant!` macro to
    /// define known values as constants in the registry.
    ///
    /// # Examples
    ///
    /// ```
    /// use bc_envelope::prelude::*;
    /// 
    /// // This is similar to how registry constants are defined
    /// const IS_A: KnownValue = KnownValue::new_with_static_name(1, "isA");
    /// 
    /// assert_eq!(IS_A.value(), 1);
    /// assert_eq!(IS_A.name(), "isA");
    /// ```
    pub const fn new_with_static_name(value: u64, name: &'static str) -> Self {
        Self { value, assigned_name: Some(KnownValueName::Static(name)) }
    }

    /// Returns the numeric value of the KnownValue.
    ///
    /// This is the raw 64-bit unsigned integer that identifies the concept.
    ///
    /// # Examples
    ///
    /// ```
    /// use bc_envelope::prelude::*;
    /// 
    /// assert_eq!(known_values::IS_A.value(), 1);
    /// assert_eq!(known_values::NOTE.value(), 4);
    /// ```
    pub fn value(&self) -> u64 {
        self.value
    }

    /// Returns the assigned name of the KnownValue, if one exists.
    ///
    /// # Examples
    ///
    /// ```
    /// use bc_envelope::prelude::*;
    /// 
    /// let named_value = KnownValue::new_with_name(1u64, "isA".to_string());
    /// assert_eq!(named_value.assigned_name(), Some("isA"));
    ///
    /// let unnamed_value = KnownValue::new(42);
    /// assert_eq!(unnamed_value.assigned_name(), None);
    /// ```
    pub fn assigned_name(&self) -> Option<&str> {
        match &self.assigned_name {
            Some(KnownValueName::Static(name)) => Some(name),
            Some(KnownValueName::Dynamic(name)) => Some(name),
            None => None,
        }
    }

    /// Returns a human-readable name for the KnownValue.
    ///
    /// If the KnownValue has an assigned name, that name is returned.
    /// Otherwise, the string representation of the numeric value is returned.
    ///
    /// # Examples
    ///
    /// ```
    /// use bc_envelope::prelude::*;
    /// 
    /// let named_value = KnownValue::new_with_name(1u64, "isA".to_string());
    /// assert_eq!(named_value.name(), "isA");
    ///
    /// let unnamed_value = KnownValue::new(42);
    /// assert_eq!(unnamed_value.name(), "42");
    /// ```
    pub fn name(&self) -> String {
        match &self.assigned_name {
            Some(KnownValueName::Static(name)) => name.to_string(),
            Some(KnownValueName::Dynamic(name)) => name.clone(),
            None => self.value.to_string(),
        }
    }
}

/// Equality for KnownValue is based solely on the numeric value, ignoring the name.
impl PartialEq for KnownValue {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

/// KnownValue implements Eq since equality is based on the numeric value, which can be compared for equality.
impl Eq for KnownValue { }

/// Hash implementation for KnownValue that considers only the numeric value.
impl std::hash::Hash for KnownValue {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.value.hash(state);
    }
}

/// Formats the KnownValue for display.
///
/// If a name is assigned, the name is displayed. Otherwise, the numeric value is displayed.
impl Display for KnownValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self.assigned_name {
            Some(KnownValueName::Static(name)) => write!(f, "{}", name),
            Some(KnownValueName::Dynamic(name)) => write!(f, "{}", name),
            None => write!(f, "{}", self.value),
        }
    }
}

/// Converts a KnownValue to an Envelope.
impl EnvelopeEncodable for KnownValue {
    fn into_envelope(self) -> Envelope {
        Envelope::new_with_known_value(self)
    }
}

/// Provides a cryptographic digest for a KnownValue.
impl DigestProvider for KnownValue {
    fn digest(&self) -> Cow<'_, Digest> {
        Cow::Owned(Digest::from_image(self.tagged_cbor().to_cbor_data()))
    }
}

/// Specifies the CBOR tag used for KnownValue.
impl CBORTagged for KnownValue {
    fn cbor_tags() -> Vec<Tag> {
        tags_for_values(&[tags::TAG_KNOWN_VALUE])
    }
}

/// Converts a KnownValue to CBOR.
impl From<KnownValue> for CBOR {
    fn from(value: KnownValue) -> Self {
        value.tagged_cbor()
    }
}

/// Attempts to convert CBOR to a KnownValue.
impl TryFrom<CBOR> for KnownValue {
    type Error = Error;

    fn try_from(cbor: CBOR) -> Result<Self> {
        Self::from_tagged_cbor(cbor)
    }
}

/// Provides the untagged CBOR representation of a KnownValue.
impl CBORTaggedEncodable for KnownValue {
    fn untagged_cbor(&self) -> CBOR {
        self.value.into()
    }
}

/// Creates a KnownValue from untagged CBOR.
impl CBORTaggedDecodable for KnownValue {
    fn from_untagged_cbor(cbor: CBOR) -> Result<Self> {
        let value = u64::try_from(cbor)?;
        Ok(Self::new(value))
    }
}

/// Creates a KnownValue from a u64.
impl From<u64> for KnownValue {
    fn from(value: u64) -> Self {
        KnownValue::new(value)
    }
}

/// Creates a KnownValue from an i32.
impl From<i32> for KnownValue {
    fn from(value: i32) -> Self {
        KnownValue::new(value as u64)
    }
}

/// Creates a KnownValue from a usize.
impl From<usize> for KnownValue {
    fn from(value: usize) -> Self {
        KnownValue::new(value as u64)
    }
}
