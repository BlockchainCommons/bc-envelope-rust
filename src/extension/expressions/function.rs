use bc_components::tags;
use dcbor::prelude::*;

use crate::{string_utils::StringUtils, Envelope, EnvelopeEncodable};

use super::FunctionsStore;

/// Internal representation of a function name, which can be either static or dynamic.
///
/// Static names are &'static str references, typically used for compile-time constants.
/// Dynamic names are owned String instances, used for runtime-created functions.
#[derive(Clone, Debug, Eq)]
pub enum FunctionName {
    /// A function name represented as a static string reference.
    Static(&'static str),
    /// A function name represented as a dynamic (owned) string.
    Dynamic(String),
}

impl FunctionName {
    /// Returns the string value of this function name.
    fn value(&self) -> &str {
        match self {
            Self::Static(name) => name,
            Self::Dynamic(name) => name,
        }
    }
}

/// Implementation of equality for FunctionName based on the string value.
impl PartialEq for FunctionName {
    fn eq(&self, other: &Self) -> bool {
        self.value() == other.value()
    }
}

/// Implementation of hashing for FunctionName based on the string value.
impl std::hash::Hash for FunctionName {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.value().hash(state)
    }
}

/// A function identifier used in Gordian Envelope expressions.
///
/// In Gordian Envelope, a function appears as the subject of an expression envelope,
/// with its parameters as assertions on that envelope.
///
/// Functions can be identified in two ways:
/// 1. By a numeric ID (for well-known functions)
/// 2. By a string name (for application-specific or less common functions)
///
/// When encoded in CBOR, functions are tagged with #6.40006.
///
/// # Examples
///
/// A numeric function:
///
/// ```
/// use bc_envelope::prelude::*;
///
/// // Define a function with a numeric ID
/// let add_function = Function::new_known(1, Some("add".to_string()));
/// ```
///
/// A named function:
///
/// ```
/// use bc_envelope::prelude::*;
///
/// // Define a function with a string name
/// let verify_function = Function::new_named("verifySignature");
/// ```
///
/// A function with static name:
///
/// ```
/// use bc_envelope::prelude::*;
///
/// // Define a function with a numeric ID and static name
/// const ADD: Function = Function::new_with_static_name(1, "add");
/// ```
#[derive(Debug, Clone, Eq)]
pub enum Function {
    /// A well-known function identified by a numeric ID, with an optional name.
    Known(u64, Option<FunctionName>),
    /// A function identified by a name.
    Named(FunctionName),
}

impl Function {
    /// Creates a new function with a numeric ID and an optional string name.
    ///
    /// This creates a "known" function, which is identified primarily by its numeric ID.
    /// The optional name is used for display purposes.
    ///
    /// # Parameters
    ///
    /// * `value` - The numeric ID of the function
    /// * `name` - An optional name for the function
    ///
    /// # Returns
    ///
    /// A new `Function` instance
    ///
    /// # Examples
    ///
    /// ```
    /// use bc_envelope::prelude::*;
    ///
    /// // Create a function with ID 1 and name "add"
    /// let add_function = Function::new_known(1, Some("add".to_string()));
    /// ```
    pub fn new_known(value: u64, name: Option<String>) -> Self {
        Self::Known(value, name.map(FunctionName::Dynamic))
    }

    /// Creates a new function identified by a string name.
    ///
    /// This creates a "named" function, which is identified by its string name.
    /// This method cannot be used to declare a function at compile-time, as it
    /// creates a dynamic string.
    ///
    /// # Parameters
    ///
    /// * `name` - The string name of the function
    ///
    /// # Returns
    ///
    /// A new `Function` instance
    ///
    /// # Examples
    ///
    /// ```
    /// use bc_envelope::prelude::*;
    ///
    /// // Create a named function
    /// let verify_function = Function::new_named("verifySignature");
    /// ```
    pub fn new_named(name: &str) -> Self {
        Self::Named(FunctionName::Dynamic(name.into()))
    }

    /// Creates a new function with a numeric ID and a static string name.
    ///
    /// This creates a "known" function, which is identified primarily by its numeric ID.
    /// The static name is used for display purposes. This method can be used to declare
    /// a function at compile-time, as it uses a static string reference.
    ///
    /// # Parameters
    ///
    /// * `value` - The numeric ID of the function
    /// * `name` - A static string name for the function
    ///
    /// # Returns
    ///
    /// A new `Function` instance
    ///
    /// # Examples
    ///
    /// ```
    /// use bc_envelope::prelude::*;
    ///
    /// // Define a function constant
    /// const ADD: Function = Function::new_with_static_name(1, "add");
    /// ```
    pub const fn new_with_static_name(value: u64, name: &'static str) -> Self {
        Self::Known(value, Some(FunctionName::Static(name)))
    }

    /// Creates a new function identified by a static string name.
    ///
    /// This creates a "named" function, which is identified by its string name.
    /// This method can be used to declare a function at compile-time, as it uses
    /// a static string reference.
    ///
    /// # Parameters
    ///
    /// * `name` - A static string name for the function
    ///
    /// # Returns
    ///
    /// A new `Function` instance
    ///
    /// # Examples
    ///
    /// ```
    /// use bc_envelope::prelude::*;
    ///
    /// // Define a named function constant
    /// const VERIFY: Function = Function::new_static_named("verifySignature");
    /// ```
    pub const fn new_static_named(name: &'static str) -> Self {
        Self::Named(FunctionName::Static(name))
    }

    /// Returns the display name of the function.
    ///
    /// For known functions with a name, returns the name.
    /// For known functions without a name, returns the numeric ID as a string.
    /// For named functions, returns the name enclosed in quotes.
    ///
    /// # Returns
    ///
    /// A string representation of the function name
    ///
    /// # Examples
    ///
    /// ```
    /// use bc_envelope::prelude::*;
    ///
    /// let add = Function::new_known(1, Some("add".to_string()));
    /// assert_eq!(add.name(), "add");
    ///
    /// let unknown = Function::new_known(42, None);
    /// assert_eq!(unknown.name(), "42");
    ///
    /// let verify = Function::new_named("verifySignature");
    /// assert_eq!(verify.name(), "\"verifySignature\"");
    /// ```
    pub fn name(&self) -> String {
        match self {
            Self::Known(value, name) => {
                if let Some(name) = name {
                    name.value().to_string()
                } else {
                    value.to_string()
                }
            },
            Self::Named(name) => name.value().to_string().flanked_by("\"", "\""),
        }
    }

    /// Returns the name of a named function, if available.
    ///
    /// This method returns the raw string name for named functions, without quotes.
    /// For known functions (numeric IDs), it returns None.
    ///
    /// # Returns
    ///
    /// An Option containing the function name string if this is a named function,
    /// or None if it's a known (numeric) function.
    ///
    /// # Examples
    ///
    /// ```
    /// use bc_envelope::prelude::*;
    ///
    /// let add = Function::new_known(1, Some("add".to_string()));
    /// assert_eq!(add.named_name(), None);
    ///
    /// let verify = Function::new_named("verifySignature");
    /// assert_eq!(verify.named_name(), Some("verifySignature".to_string()));
    /// ```
    pub fn named_name(&self) -> Option<String> {
        match self {
            Self::Known(_, _) => None,
            Self::Named(name) => Some(name.value().to_string()),
        }
    }
}

/// Implementation of equality for Function.
///
/// Known functions are equal if they have the same numeric ID (names are ignored).
/// Named functions are equal if they have the same name.
/// Known and named functions are never equal to each other.
impl PartialEq for Function {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Function::Known(l, _), Function::Known(r, _)) => l == r,
            (Function::Named(l), Function::Named(r)) => l == r,
            _ => false,
        }
    }
}

/// Implementation of hash for Function.
///
/// Known functions are hashed by their numeric ID.
/// Named functions are hashed by their name.
impl std::hash::Hash for Function {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            Function::Known(value, _) => value.hash(state),
            Function::Named(name) => name.hash(state),
        }
    }
}

/// Allows creating a Function from a u64.
///
/// This creates a known function with the given numeric ID and no name.
impl From<u64> for Function {
    fn from(value: u64) -> Self {
        Self::new_known(value, None)
    }
}

/// Allows creating a Function from a string reference.
///
/// This creates a named function with the given name.
impl From<&str> for Function {
    fn from(name: &str) -> Self {
        Self::new_named(name)
    }
}

/// Allows creating a Function from a reference to a Function.
///
/// This clones the function.
impl From<&Function> for Function {
    fn from(function: &Function) -> Self {
        function.clone()
    }
}

/// Implementation of the CBORTagged trait for Function.
///
/// Functions are tagged with #6.40006 (TAG_FUNCTION).
impl CBORTagged for Function {
    fn cbor_tags() -> Vec<Tag> {
        tags_for_values(&[tags::TAG_FUNCTION])
    }
}

/// Allows creating a CBOR value from a Function.
impl From<Function> for CBOR {
    fn from(value: Function) -> Self {
        value.tagged_cbor()
    }
}

/// Implementation of the CBORTaggedEncodable trait for Function.
///
/// Known functions are encoded as unsigned integers.
/// Named functions are encoded as text strings.
impl CBORTaggedEncodable for Function {
    fn untagged_cbor(&self) -> CBOR {
        match self {
            Function::Known(value, _) => (*value).into(),
            Function::Named(name) => name.value().into(),
        }
    }
}

/// Allows creating a Function from a CBOR value.
impl TryFrom<CBOR> for Function {
    type Error = dcbor::Error;

    fn try_from(cbor: CBOR) -> dcbor::Result<Self> {
        Self::from_tagged_cbor(cbor)
    }
}

/// Implementation of the CBORTaggedDecodable trait for Function.
///
/// Unsigned integers are decoded as known functions.
/// Text strings are decoded as named functions.
impl CBORTaggedDecodable for Function {
    fn from_untagged_cbor(untagged_cbor: CBOR) -> dcbor::Result<Self> {
        match untagged_cbor.as_case() {
            CBORCase::Unsigned(value) => Ok(Self::new_known(*value, None)),
            CBORCase::Text(name) => Ok(Self::new_named(name)),
            _ => return Err("invalid function".into()),
        }
    }
}

impl Function {
    /// Returns a description of this function for display.
    ///
    /// For known functions, attempts to look up the name in the provided
    /// functions store, if available.
    /// For named functions, returns the name in quotes.
    fn description(&self, functions: Option<&FunctionsStore>) -> String {
        match self {
            Function::Known(_, _) => {
                FunctionsStore::name_for_function(self, functions)
            },
            Function::Named(name) => {
                format!("\"{}\"", name.value())
            },
        }
    }
}

/// Implements display for Function.
///
/// Uses the description method with no functions store.
impl std::fmt::Display for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.description(None))
    }
}

/// Implements the EnvelopeEncodable trait for Function.
///
/// This allows a Function to be directly converted to an Envelope.
impl EnvelopeEncodable for Function {
    fn into_envelope(self) -> Envelope {
        Envelope::new_leaf(self)
    }
}

/// Implements conversion from Envelope to Function.
///
/// This attempts to extract a Function from an Envelope leaf.
impl TryFrom<Envelope> for Function {
    type Error = dcbor::Error;

    fn try_from(envelope: Envelope) -> dcbor::Result<Self> {
        Function::try_from(envelope.try_leaf()?)
    }
}
