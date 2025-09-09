//! Provides methods for working with envelope leaf nodes,
//! which are dCBOR values of any kind.

use dcbor::prelude::*;

#[cfg(feature = "known_value")]
use super::envelope::EnvelopeCase;
use crate::{Envelope, Result};
#[cfg(feature = "known_value")]
use crate::{Error, extension::KnownValue};

impl Envelope {
    pub fn r#false() -> Self { Self::new_leaf(false) }

    pub fn r#true() -> Self { Self::new_leaf(true) }

    pub fn is_false(&self) -> bool {
        self.extract_subject().ok() == Some(false)
    }

    pub fn is_true(&self) -> bool { self.extract_subject().ok() == Some(true) }

    pub fn is_bool(&self) -> bool {
        matches!(
            self.extract_subject(),
            Ok(dcbor::Simple::True | dcbor::Simple::False)
        )
    }
}

impl Envelope {
    /// `true` if the envelope is a leaf node that contains a number, `false`
    pub fn is_number(&self) -> bool {
        self.as_leaf().map(|c| c.is_number()).unwrap_or(false)
    }

    /// `true` if the subject of the envelope is a number, `false`
    /// otherwise.
    pub fn is_subject_number(&self) -> bool { self.subject().is_number() }

    /// `true` if the envelope is a leaf node that contains the NaN value,
    /// `false` otherwise.
    pub fn is_nan(&self) -> bool {
        self.as_leaf().map(|c| c.is_nan()).unwrap_or(false)
    }

    /// `true` if the subject of the envelope is the NaN value, `false`
    /// otherwise.
    pub fn is_subject_nan(&self) -> bool { self.subject().is_nan() }
}

impl Envelope {
    pub fn null() -> Self { Self::new_leaf(dcbor::Simple::Null) }

    pub fn is_null(&self) -> bool {
        self.extract_subject().ok() == Some(dcbor::Simple::Null)
    }
}

impl Envelope {
    /// The envelope's leaf CBOR object as a CBOR byte string, or an error if
    /// the envelope is not a leaf, or the leaf is not a byte string.
    pub fn try_byte_string(&self) -> Result<Vec<u8>> {
        Ok(self.try_leaf()?.try_into_byte_string()?)
    }

    pub fn as_byte_string(&self) -> Option<Vec<u8>> {
        self.as_leaf().and_then(|c| c.into_byte_string())
    }
}

impl Envelope {
    pub fn as_array(&self) -> Option<Vec<CBOR>> {
        self.as_leaf().and_then(|c| c.into_array())
    }

    pub fn as_map(&self) -> Option<dcbor::Map> {
        self.as_leaf().and_then(|c| c.into_map())
    }

    pub fn as_text(&self) -> Option<String> {
        self.as_leaf().and_then(|c| c.into_text())
    }
}

#[cfg(feature = "known_value")]
impl Envelope {
    /// The envelope's `KnownValue`, or `None` if the envelope is not case
    /// `::KnownValue`.
    pub fn as_known_value(&self) -> Option<&KnownValue> {
        match self.case() {
            EnvelopeCase::KnownValue { value, .. } => Some(value),
            _ => None,
        }
    }

    /// The envelope's `KnownValue`, or an error if the envelope is not case
    /// `::KnownValue`.
    pub fn try_known_value(&self) -> Result<&KnownValue> {
        self.as_known_value().ok_or(Error::NotKnownValue)
    }

    /// `true` if the envelope is case `::KnownValue`, `false` otherwise.
    pub fn is_known_value(&self) -> bool {
        matches!(self.case(), EnvelopeCase::KnownValue { .. })
    }

    pub fn unit() -> Self { Self::new_leaf(known_values::UNIT) }

    pub fn is_subject_unit(&self) -> bool {
        self.extract_subject().ok() == Some(known_values::UNIT)
    }
}
