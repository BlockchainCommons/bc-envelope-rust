use bc_components::tags;
use dcbor::{CBORTagged, Tag, CBOREncodable, CBORTaggedEncodable, CBOR, CBORDecodable, CBORTaggedDecodable};
pub use crate::parameters::*;
use crate::{ParametersStore, string_utils::StringUtils};

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

    pub fn new_named(name: &str) -> Self {
        Self::Named(ParameterName::Dynamic(name.into()))
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

impl CBORTaggedEncodable for Parameter {
    fn untagged_cbor(&self) -> CBOR {
        match self {
            Parameter::Known(value, _) => value.cbor(),
            Parameter::Named(name) => name.value().cbor(),
        }
    }
}

impl CBORDecodable for Parameter {
    fn from_cbor(cbor: &CBOR) -> Result<Self, dcbor::Error> {
        Self::from_tagged_cbor(cbor)
    }
}

impl CBORTaggedDecodable for Parameter {
    fn from_untagged_cbor(untagged_cbor: &CBOR) -> Result<Self, dcbor::Error> {
        match untagged_cbor {
            CBOR::Unsigned(value) => Ok(Self::new_known(*value, None)),
            CBOR::Text(name) => Ok(Self::new_named(name)),
            _ => Err(dcbor::Error::InvalidFormat),
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
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.description(None))
    }
}
