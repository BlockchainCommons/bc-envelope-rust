use bc_components::tags;
use dcbor::prelude::*;

use super::ParametersStore;
use crate::{Envelope, EnvelopeEncodable, string_utils::StringUtils};

/// Internal representation of a parameter name, which can be either static or
/// dynamic.
///
/// Static names are &'static str references, typically used for compile-time
/// constants. Dynamic names are owned String instances, used for
/// runtime-created parameters.
#[derive(Clone, Debug, Eq)]
pub enum ParameterName {
    /// A parameter name represented as a static string reference.
    Static(&'static str),
    /// A parameter name represented as a dynamic (owned) string.
    Dynamic(String),
}

impl ParameterName {
    /// Returns the string value of this parameter name.
    fn value(&self) -> &str {
        match self {
            Self::Static(name) => name,
            Self::Dynamic(name) => name,
        }
    }
}

/// Implementation of equality for ParameterName based on the string value.
impl PartialEq for ParameterName {
    fn eq(&self, other: &Self) -> bool { self.value() == other.value() }
}

/// Implementation of hashing for ParameterName based on the string value.
impl std::hash::Hash for ParameterName {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.value().hash(state)
    }
}

/// A parameter identifier used in Gordian Envelope expressions.
///
/// In Gordian Envelope, a parameter appears as a predicate in an assertion on
/// an expression envelope. The parameter identifies the name of the argument,
/// and the object of the assertion is the argument value.
///
/// Parameters can be identified in two ways:
/// 1. By a numeric ID (for well-known parameters)
/// 2. By a string name (for application-specific or less common parameters)
///
/// When encoded in CBOR, parameters are tagged with #6.40007.
///
/// # Examples
///
/// A numeric parameter:
///
/// ```
/// use bc_envelope::prelude::*;
///
/// // Define a parameter with a numeric ID
/// let lhs_param = Parameter::new_known(2, Some("lhs".to_string()));
/// ```
///
/// A named parameter:
///
/// ```
/// use bc_envelope::prelude::*;
///
/// // Define a parameter with a string name
/// let key_param = Parameter::new_named("key");
/// ```
///
/// A parameter with static name:
///
/// ```
/// use bc_envelope::prelude::*;
///
/// // Define a parameter with a numeric ID and static name
/// const LHS: Parameter = Parameter::new_with_static_name(2, "lhs");
/// ```
#[derive(Clone, Debug, Eq)]
pub enum Parameter {
    /// A well-known parameter identified by a numeric ID, with an optional
    /// name.
    Known(u64, Option<ParameterName>),
    /// A parameter identified by a name.
    Named(ParameterName),
}

impl Parameter {
    /// Creates a new parameter with a numeric ID and an optional string name.
    ///
    /// This creates a "known" parameter, which is identified primarily by its
    /// numeric ID. The optional name is used for display purposes.
    ///
    /// # Parameters
    ///
    /// * `value` - The numeric ID of the parameter
    /// * `name` - An optional name for the parameter
    ///
    /// # Returns
    ///
    /// A new `Parameter` instance
    ///
    /// # Examples
    ///
    /// ```
    /// use bc_envelope::prelude::*;
    ///
    /// // Create a parameter with ID 2 and name "lhs"
    /// let lhs_param = Parameter::new_known(2, Some("lhs".to_string()));
    /// ```
    pub fn new_known(value: u64, name: Option<String>) -> Self {
        Self::Known(value, name.map(ParameterName::Dynamic))
    }

    /// Creates a new parameter identified by a string name.
    ///
    /// This creates a "named" parameter, which is identified by its string
    /// name. This method cannot be used to declare a parameter at
    /// compile-time, as it creates a dynamic string.
    ///
    /// # Parameters
    ///
    /// * `name` - The string name of the parameter
    ///
    /// # Returns
    ///
    /// A new `Parameter` instance
    ///
    /// # Examples
    ///
    /// ```
    /// use bc_envelope::prelude::*;
    ///
    /// // Create a named parameter
    /// let key_param = Parameter::new_named("key");
    /// ```
    pub fn new_named(name: &str) -> Self {
        Self::Named(ParameterName::Dynamic(name.into()))
    }

    /// Creates a new parameter with a numeric ID and a static string name.
    ///
    /// This creates a "known" parameter, which is identified primarily by its
    /// numeric ID. The static name is used for display purposes. This
    /// method can be used to declare a parameter at compile-time, as it
    /// uses a static string reference.
    ///
    /// # Parameters
    ///
    /// * `value` - The numeric ID of the parameter
    /// * `name` - A static string name for the parameter
    ///
    /// # Returns
    ///
    /// A new `Parameter` instance
    ///
    /// # Examples
    ///
    /// ```
    /// use bc_envelope::prelude::*;
    ///
    /// // Define a parameter constant
    /// const LHS: Parameter = Parameter::new_with_static_name(2, "lhs");
    /// ```
    pub const fn new_with_static_name(value: u64, name: &'static str) -> Self {
        Self::Known(value, Some(ParameterName::Static(name)))
    }

    /// Creates a new parameter identified by a static string name.
    ///
    /// This creates a "named" parameter, which is identified by its string
    /// name. This method can be used to declare a parameter at
    /// compile-time, as it uses a static string reference.
    ///
    /// # Parameters
    ///
    /// * `name` - A static string name for the parameter
    ///
    /// # Returns
    ///
    /// A new `Parameter` instance
    ///
    /// # Examples
    ///
    /// ```
    /// use bc_envelope::prelude::*;
    ///
    /// // Define a named parameter constant
    /// const KEY: Parameter = Parameter::new_static_named("key");
    /// ```
    pub const fn new_static_named(name: &'static str) -> Self {
        Self::Named(ParameterName::Static(name))
    }

    /// Returns the display name of the parameter.
    ///
    /// For known parameters with a name, returns the name.
    /// For known parameters without a name, returns the numeric ID as a string.
    /// For named parameters, returns the name enclosed in quotes.
    ///
    /// # Returns
    ///
    /// A string representation of the parameter name
    ///
    /// # Examples
    ///
    /// ```
    /// use bc_envelope::prelude::*;
    ///
    /// let lhs = Parameter::new_known(2, Some("lhs".to_string()));
    /// assert_eq!(lhs.name(), "lhs");
    ///
    /// let unknown = Parameter::new_known(42, None);
    /// assert_eq!(unknown.name(), "42");
    ///
    /// let key = Parameter::new_named("key");
    /// assert_eq!(key.name(), "\"key\"");
    /// ```
    pub fn name(&self) -> String {
        match self {
            Self::Known(value, name) => {
                if let Some(name) = name {
                    name.value().to_string()
                } else {
                    value.to_string()
                }
            }
            Self::Named(name) => {
                name.value().to_string().flanked_by("\"", "\"")
            }
        }
    }
}

/// Implementation of equality for Parameter.
///
/// Known parameters are equal if they have the same numeric ID (names are
/// ignored). Named parameters are equal if they have the same name.
/// Known and named parameters are never equal to each other.
impl PartialEq for Parameter {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Known(l, _), Self::Known(r, _)) => l == r,
            (Self::Named(l), Self::Named(r)) => l == r,
            _ => false,
        }
    }
}

/// Implementation of hash for Parameter.
///
/// Known parameters are hashed by their numeric ID.
/// Named parameters are hashed by their name.
impl std::hash::Hash for Parameter {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            Self::Known(value, _) => value.hash(state),
            Self::Named(name) => name.hash(state),
        }
    }
}

/// Allows creating a Parameter from a u64.
///
/// This creates a known parameter with the given numeric ID and no name.
impl From<u64> for Parameter {
    fn from(value: u64) -> Self { Self::new_known(value, None) }
}

/// Allows creating a Parameter from a string reference.
///
/// This creates a named parameter with the given name.
impl From<&str> for Parameter {
    fn from(name: &str) -> Self { Self::new_named(name) }
}

/// Allows creating a Parameter from a reference to a Parameter.
///
/// This clones the parameter.
impl From<&Parameter> for Parameter {
    fn from(parameter: &Parameter) -> Self { parameter.clone() }
}

/// Implementation of the CBORTagged trait for Parameter.
///
/// Parameters are tagged with #6.40007 (TAG_PARAMETER).
impl CBORTagged for Parameter {
    fn cbor_tags() -> Vec<Tag> { tags_for_values(&[tags::TAG_PARAMETER]) }
}

/// Allows creating a CBOR value from a Parameter.
impl From<Parameter> for CBOR {
    fn from(value: Parameter) -> Self { value.tagged_cbor() }
}

/// Implementation of the CBORTaggedEncodable trait for Parameter.
///
/// Known parameters are encoded as unsigned integers.
/// Named parameters are encoded as text strings.
impl CBORTaggedEncodable for Parameter {
    fn untagged_cbor(&self) -> CBOR {
        match self {
            Parameter::Known(value, _) => (*value).into(),
            Parameter::Named(name) => name.value().into(),
        }
    }
}

/// Allows creating a Parameter from a CBOR value.
impl TryFrom<CBOR> for Parameter {
    type Error = dcbor::Error;

    fn try_from(cbor: CBOR) -> dcbor::Result<Self> {
        Self::from_tagged_cbor(cbor)
    }
}

/// Implementation of the CBORTaggedDecodable trait for Parameter.
///
/// Unsigned integers are decoded as known parameters.
/// Text strings are decoded as named parameters.
impl CBORTaggedDecodable for Parameter {
    fn from_untagged_cbor(untagged_cbor: CBOR) -> dcbor::Result<Self> {
        match untagged_cbor.as_case() {
            CBORCase::Unsigned(value) => Ok(Self::new_known(*value, None)),
            CBORCase::Text(name) => Ok(Self::new_named(name)),
            _ => Err("invalid parameter".into()),
        }
    }
}

impl Parameter {
    /// Returns a description of this parameter for display.
    ///
    /// For known parameters, attempts to look up the name in the provided
    /// parameters store, if available.
    /// For named parameters, returns the name in quotes.
    fn description(&self, parameters: Option<&ParametersStore>) -> String {
        match self {
            Parameter::Known(_, _) => {
                ParametersStore::name_for_parameter(self, parameters)
            }
            Parameter::Named(name) => {
                format!("\"{}\"", name.value())
            }
        }
    }
}

/// Implements display for Parameter.
///
/// Uses the description method with no parameters store.
impl std::fmt::Display for Parameter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.description(None))
    }
}

/// Implements the EnvelopeEncodable trait for Parameter.
///
/// This allows a Parameter to be directly converted to an Envelope.
impl EnvelopeEncodable for Parameter {
    fn into_envelope(self) -> Envelope { Envelope::new_leaf(self) }
}
