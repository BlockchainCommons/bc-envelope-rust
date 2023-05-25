use std::rc::Rc;

use bc_components::tags_registry;
use dcbor::{CBORTagged, Tag, CBOREncodable, CBORTaggedEncodable, CBOR, CBORDecodable, CBORTaggedDecodable, CBORError};
pub use crate::parameter_registry::*;
use crate::KnownParameters;

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

#[derive(Clone, Debug, Eq)]
pub enum Parameter {
    Known(u64, Option<ParameterName>),
    Named(ParameterName),
}

impl Parameter {
    pub fn new_known(value: u64, name: Option<String>) -> Self {
        Self::Known(value, name.map(ParameterName::Dynamic))
    }

    pub fn new_named(name: String) -> Self {
        Self::Named(ParameterName::Dynamic(name))
    }

    pub const fn new_with_static_name(value: u64, name: &'static str) -> Self {
        Self::Known(value, Some(ParameterName::Static(name)))
    }

    pub fn name(&self) -> String {
        match self {
            Self::Known(value, name) => {
                if let Some(name) = name {
                    name.value().to_string()
                } else {
                    value.to_string()
                }
            },
            Self::Named(name) => name.value().to_string(),
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
        Self::new_named(name.to_string())
    }
}

impl CBORTagged for Parameter {
    const CBOR_TAG: Tag = tags_registry::PARAMETER;
}

impl CBOREncodable for Parameter {
    fn cbor(&self) -> CBOR {
        self.tagged_cbor()
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
    fn from_cbor(cbor: &CBOR) -> Result<Rc<Self>, CBORError> {
        Self::from_tagged_cbor(cbor)
    }
}

impl CBORTaggedDecodable for Parameter {
    fn from_untagged_cbor(untagged_cbor: &CBOR) -> Result<Rc<Self>, CBORError> {
        match untagged_cbor {
            CBOR::Unsigned(value) => Ok(Rc::new(Self::new_known(*value, None))),
            CBOR::Text(name) => Ok(Rc::new(Self::new_named(name.clone()))),
            _ => Err(CBORError::InvalidFormat),
        }
    }
}

impl Parameter {
    fn description(&self, known_parameters: Option<&KnownParameters>) -> String {
        match self {
            Parameter::Known(_, _) => {
                KnownParameters::name_for_parameter(self, known_parameters)
            },
            Parameter::Named(name) => {
                format!("\"{}\"", name.value())
            },
        }
    }
}

impl std::fmt::Display for Parameter {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.description(None))
    }
}
