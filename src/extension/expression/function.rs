use anyhow::bail;
use bc_components::tags;
use dcbor::prelude::*;

use crate::{string_utils::StringUtils, Envelope};

use super::FunctionsStore;

#[derive(Clone, Debug, Eq)]
pub enum FunctionName {
    Static(&'static str),
    Dynamic(String),
}

impl FunctionName {
    fn value(&self) -> &str {
        match self {
            Self::Static(name) => name,
            Self::Dynamic(name) => name,
        }
    }
}

impl PartialEq for FunctionName {
    fn eq(&self, other: &Self) -> bool {
        self.value() == other.value()
    }
}

impl std::hash::Hash for FunctionName {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.value().hash(state)
    }
}

/// A declared function.
 #[derive(Debug, Clone, Eq)]
pub enum Function {
    Known(u64, Option<FunctionName>),
    Named(FunctionName),
}

impl Function {
    /// Creates a new function with a value and an optional name.
    pub fn new_known(value: u64, name: Option<String>) -> Self {
        Self::Known(value, name.map(FunctionName::Dynamic))
    }

    /// Creates a new function with a name. This call cannot be used
    /// to declare a function at compile-time.
    pub fn new_named(name: &str) -> Self {
        Self::Named(FunctionName::Dynamic(name.into()))
    }

    /// Creates a new function with a value and a static name.
    /// This call can be used to declare a function at compile-time.
    pub const fn new_with_static_name(value: u64, name: &'static str) -> Self {
        Self::Known(value, Some(FunctionName::Static(name)))
    }

    /// Returns the name of the function.
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
}

impl PartialEq for Function {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Function::Known(l, _), Function::Known(r, _)) => l == r,
            (Function::Named(l), Function::Named(r)) => l == r,
            _ => false,
        }
    }
}

impl std::hash::Hash for Function {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            Function::Known(value, _) => value.hash(state),
            Function::Named(name) => name.hash(state),
        }
    }
}

impl From<u64> for Function {
    fn from(value: u64) -> Self {
        Self::new_known(value, None)
    }
}

impl From<&str> for Function {
    fn from(name: &str) -> Self {
        Self::new_named(name)
    }
}

impl From<&Function> for Function {
    fn from(function: &Function) -> Self {
        function.clone()
    }
}

impl CBORTagged for Function {
    const CBOR_TAG: Tag = tags::FUNCTION;
}

impl CBOREncodable for Function {
    fn cbor(&self) -> CBOR {
        self.tagged_cbor()
    }
}

impl CBORTaggedEncodable for Function {
    fn untagged_cbor(&self) -> CBOR {
        match self {
            Function::Known(value, _) => value.cbor(),
            Function::Named(name) => name.value().cbor(),
        }
    }
}

impl CBORDecodable for Function {
    fn from_cbor(cbor: &CBOR) -> anyhow::Result<Self> {
        Self::from_tagged_cbor(cbor)
    }
}

impl CBORTaggedDecodable for Function {
    fn from_untagged_cbor(untagged_cbor: &CBOR) -> anyhow::Result<Self> {
        match untagged_cbor {
            CBOR::Unsigned(value) => Ok(Self::new_known(*value, None)),
            CBOR::Text(name) => Ok(Self::new_named(name)),
            _ => bail!("invalid function"),
        }
    }
}

impl Function {
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

impl std::fmt::Display for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.description(None))
    }
}

impl Envelope {
    pub fn function(&self) -> anyhow::Result<Function> {
        self.extract_subject()
    }

    pub fn check_function(&self, function: &Function) -> anyhow::Result<()> {
        let envelope_function = self.function()?;
        if envelope_function != *function {
            bail!("unknown function");
        }
        Ok(())
    }
}
