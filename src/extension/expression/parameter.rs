use anyhow::bail;
use bc_components::tags;
use dcbor::prelude::*;
use crate::{string_utils::StringUtils, impl_envelope_encodable, EnvelopeEncodable};

use super::ParametersStore;

#[derive(Clone, Debug, Eq)]
pub enum ParameterName {
    Static(&'static str),
    Dynamic(String),
}

impl ParameterName {
    fn value(&self) -> &str {
        match self {
            Self::Static(name) => name,
            Self::Dynamic(name) => name,
        }
    }
}

impl PartialEq for ParameterName {
    fn eq(&self, other: &Self) -> bool {
        self.value() == other.value()
    }
}

impl std::hash::Hash for ParameterName {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.value().hash(state)
    }
}

/// A declared parameter.
#[derive(Clone, Debug, Eq)]
pub enum Parameter {
    Known(u64, Option<ParameterName>),
    Named(ParameterName),
}

impl Parameter {
    /// Creates a new parameter with a value and an optional name.
    pub fn new_known(value: u64, name: Option<String>) -> Self {
        Self::Known(value, name.map(ParameterName::Dynamic))
    }

    /// Creates a new parameter with a name. This call cannot be used
    /// to declare a parameter at compile-time.
    pub fn new_named(name: &str) -> Self {
        Self::Named(ParameterName::Dynamic(name.into()))
    }

    /// Creates a new parameter with a value and a static name.
    /// This call can be used to declare a parameter at compile-time.
    pub const fn new_with_static_name(value: u64, name: &'static str) -> Self {
        Self::Known(value, Some(ParameterName::Static(name)))
    }

    /// Creates a new parameter with a static name.
    /// This call can be used to declare a parameter at compile-time.
    pub const fn new_static_named(name: &'static str) -> Self {
        Self::Named(ParameterName::Static(name))
    }

    /// Returns the name of the parameter.
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

impl PartialEq for Parameter {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Known(l, _), Self::Known(r, _)) => l == r,
            (Self::Named(l), Self::Named(r)) => l == r,
            _ => false,
        }
    }
}

impl std::hash::Hash for Parameter {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            Self::Known(value, _) => value.hash(state),
            Self::Named(name) => name.hash(state),
        }
    }
}

impl From<u64> for Parameter {
    fn from(value: u64) -> Self {
        Self::new_known(value, None)
    }
}

impl From<&str> for Parameter {
    fn from(name: &str) -> Self {
        Self::new_named(name)
    }
}

impl From<&Parameter> for Parameter {
    fn from(parameter: &Parameter) -> Self {
        parameter.clone()
    }
}

impl CBORTagged for Parameter {
    const CBOR_TAG: Tag = tags::PARAMETER;
}

impl CBOREncodable for Parameter {
    fn cbor(&self) -> CBOR {
        self.tagged_cbor()
    }
}

impl From<Parameter> for CBOR {
    fn from(value: Parameter) -> Self {
        value.cbor()
    }
}

impl CBORTaggedEncodable for Parameter {
    fn untagged_cbor(&self) -> CBOR {
        match self {
            Parameter::Known(value, _) => value.cbor(),
            Parameter::Named(name) => name.value().cbor(),
        }
    }
}

impl CBORDecodable for Parameter {
    fn from_cbor(cbor: &CBOR) -> anyhow::Result<Self> {
        Self::from_tagged_cbor(cbor)
    }
}

impl TryFrom<CBOR> for Parameter {
    type Error = anyhow::Error;

    fn try_from(cbor: CBOR) -> Result<Self, Self::Error> {
        Self::from_cbor(&cbor)
    }
}

impl CBORTaggedDecodable for Parameter {
    fn from_untagged_cbor(untagged_cbor: &CBOR) -> anyhow::Result<Self> {
        match untagged_cbor {
            CBOR::Unsigned(value) => Ok(Self::new_known(*value, None)),
            CBOR::Text(name) => Ok(Self::new_named(name)),
            _ => bail!("invalid parameter"),
        }
    }
}

impl Parameter {
    fn description(&self, parameters: Option<&ParametersStore>) -> String {
        match self {
            Parameter::Known(_, _) => {
                ParametersStore::name_for_parameter(self, parameters)
            },
            Parameter::Named(name) => {
                format!("\"{}\"", name.value())
            },
        }
    }
}

impl std::fmt::Display for Parameter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.description(None))
    }
}

impl_envelope_encodable!(Parameter);
