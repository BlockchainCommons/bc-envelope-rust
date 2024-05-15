use anyhow::{bail, Error, Result};
use bc_components::{Digest, DigestProvider};
#[cfg(feature = "encrypt")]
use bc_components::EncryptedMessage;
#[cfg(feature = "compress")]
use bc_components::Compressed;
use dcbor::prelude::*;
use std::any::{Any, TypeId};

use crate::{Assertion, Envelope, EnvelopeEncodable, EnvelopeError};
#[cfg(feature = "known_value")]
use crate::extension::KnownValue;

use super::envelope::EnvelopeCase;

/// Support for various queries on envelopes.
impl Envelope {
    /// The envelope's subject.
    ///
    /// For an envelope with no assertions, returns the same envelope.
    pub fn subject(&self) -> Self {
        match self.case() {
            EnvelopeCase::Node { subject, .. } => subject.clone(),
            _ => self.clone(),
        }
    }

    /// The envelope's assertions.
    pub fn assertions(&self) -> Vec<Self> {
        match self.case() {
            EnvelopeCase::Node { assertions, .. } => assertions.clone(),
            _ => vec![],
        }
    }

    /// `true` if the envelope has at least one assertion, `false` otherwise.
    pub fn has_assertions(&self) -> bool {
        match self.case() {
            EnvelopeCase::Node { assertions, .. } => !assertions.is_empty(),
            _ => false,
        }
    }

    /// If the envelope's subject is an assertion return it, else return `None`.
    pub fn as_assertion(&self) -> Option<Self> {
        match self.case() {
            EnvelopeCase::Assertion(_) => Some(self.clone()),
            _ => None,
        }
    }

    /// If the envelope's subject is an assertion return it, else return an error.
    pub fn try_assertion(&self) -> Result<Self> {
        self.as_assertion().ok_or(EnvelopeError::NotAssertion.into())
    }

    /// The envelope's predicate, or `None` if the envelope is not an assertion.
    pub fn as_predicate(&self) -> Option<Self> {
        match self.case() {
            EnvelopeCase::Assertion(assertion) => Some(assertion.predicate()),
            _ => None,
        }
    }

    /// The envelope's predicate, or an error if the envelope is not an assertion.
    pub fn try_predicate(&self) -> Result<Self> {
        self.as_predicate().ok_or(EnvelopeError::NotAssertion.into())
    }

    /// The envelope's object, or `None` if the envelope is not an assertion.
    pub fn as_object(&self) -> Option<Self> {
        match self.case() {
            EnvelopeCase::Assertion(assertion) => Some(assertion.object()),
            _ => None,
        }
    }

    /// The envelope's object, or an error if the envelope is not an assertion.
    pub fn try_object(&self) -> Result<Self> {
        self.as_object().ok_or(EnvelopeError::NotAssertion.into())
    }

    /// The envelope's leaf CBOR object, or `None` if the envelope is not a leaf.
    pub fn as_leaf(&self) -> Option<CBOR> {
        match self.case() {
            EnvelopeCase::Leaf { cbor, .. } => Some(cbor.clone()),
            _ => None,
        }
    }

    /// The envelope's leaf CBOR object, or an error if the envelope is not a leaf.
    pub fn try_leaf(&self) -> Result<CBOR> {
        self.as_leaf().ok_or(EnvelopeError::NotLeaf.into())
    }

    /// The envelope's `KnownValue`, or `None` if the envelope is not case `::KnownValue`.
    #[cfg(feature = "known_value")]
    pub fn as_known_value(&self) -> Option<&KnownValue> {
        match self.case() {
            EnvelopeCase::KnownValue { value, .. } => Some(value),
            _ => None,
        }
    }

    /// The envelope's `KnownValue`, or an error if the envelope is not case `::KnownValue`.
    #[cfg(feature = "known_value")]
    pub fn try_known_value(&self) -> Result<&KnownValue> {
        self.as_known_value().ok_or(EnvelopeError::NotKnownValue.into())
    }

    /// `true` if the envelope is case `::Leaf`, `false` otherwise.
    pub fn is_leaf(&self) -> bool {
        matches!(self.case(), EnvelopeCase::Leaf { .. })
    }

    /// `true` if the envelope is case `::Node`, `false` otherwise.
    pub fn is_node(&self) -> bool {
        matches!(self.case(), EnvelopeCase::Node { .. })
    }

    /// `true` if the envelope is case `::Wrapped`, `false` otherwise.
    pub fn is_wrapped(&self) -> bool {
        matches!(self.case(), EnvelopeCase::Wrapped { .. })
    }

    /// `true` if the envelope is case `::KnownValue`, `false` otherwise.
    #[cfg(feature = "known_value")]
    pub fn is_known_value(&self) -> bool {
        matches!(self.case(), EnvelopeCase::KnownValue { .. })
    }

    /// `true` if the envelope is case `::Assertion`, `false` otherwise.
    pub fn is_assertion(&self) -> bool {
        matches!(self.case(), EnvelopeCase::Assertion(_))
    }

    /// `true` if the envelope is case `::Encrypted`, `false` otherwise.
    #[cfg(feature = "encrypt")]
    pub fn is_encrypted(&self) -> bool {
        matches!(self.case(), EnvelopeCase::Encrypted(_))
    }

    /// `true` if the envelope is case `::Compressed`, `false` otherwise.
    #[cfg(feature = "compress")]
    pub fn is_compressed(&self) -> bool {
        matches!(self.case(), EnvelopeCase::Compressed(_))
    }

    /// `true` if the envelope is case `::Elided`, `false` otherwise.
    pub fn is_elided(&self) -> bool {
        matches!(self.case(), EnvelopeCase::Elided(_))
    }

    /// `true` if the subject of the envelope is an assertion, `false` otherwise.
    pub fn is_subject_assertion(&self) -> bool {
        match self.case() {
            EnvelopeCase::Assertion(_) => true,
            EnvelopeCase::Node { subject, .. } => subject.is_subject_assertion(),
            _ => false,
        }
    }

    /// `true` if the subject of the envelope has been encrypted, `false` otherwise.
    #[cfg(feature = "encrypt")]
    pub fn is_subject_encrypted(&self) -> bool {
        match self.case() {
            EnvelopeCase::Encrypted(_) => true,
            EnvelopeCase::Node { subject, .. } => subject.is_subject_encrypted(),
            _ => false,
        }
    }

    /// `true` if the subject of the envelope has been compressed, `false` otherwise.
    #[cfg(feature = "compress")]
    pub fn is_subject_compressed(&self) -> bool {
        match self.case() {
            EnvelopeCase::Compressed(_) => true,
            EnvelopeCase::Node { subject, .. } => subject.is_subject_compressed(),
            _ => false,
        }
    }

    /// `true` if the subject of the envelope has been elided, `false` otherwise.
    pub fn is_subject_elided(&self) -> bool {
        match self.case() {
            EnvelopeCase::Elided(_) => true,
            EnvelopeCase::Node { subject, .. } => subject.is_subject_elided(),
            _ => false,
        }
    }

    /// `true` if the subject of the envelope has been encrypted, elided, or compressed, `false` otherwise.
    ///
    /// Obscured assertion envelopes may exist in the list of an envelope's assertions.
    pub fn is_subject_obscured(&self) -> bool {
        if self.is_subject_elided() {
            return true;
        }
        #[cfg(feature = "encrypt")]
        if self.is_subject_encrypted() {
            return true;
        }
        #[cfg(feature = "compress")]
        if self.is_subject_compressed() {
            return true;
        }
        false
    }

    /// `true` if the envelope is *internal*, that is, it has child elements, or `false` if it is a leaf node.
    ///
    /// Internal elements include `.node`, `.wrapped`, and `.assertion`.
    pub fn is_internal(&self) -> bool {
        matches!(
            self.case(),
            EnvelopeCase::Node { .. } | EnvelopeCase::Wrapped { .. } | EnvelopeCase::Assertion(_)
        )
    }

    /// `true` if the envelope is encrypted, elided, or compressed; `false` otherwise.
    pub fn is_obscured(&self) -> bool {
        if self.is_elided() {
            return true;
        }
        #[cfg(feature = "encrypt")]
        if self.is_encrypted() {
            return true;
        }
        #[cfg(feature = "compress")]
        if self.is_compressed() {
            return true;
        }
        false
    }

    /// Returns the envelope's subject, decoded as the given type.
    ///
    /// If the encoded type doesn't match the given type, returns `Error::InvalidFormat`.
    pub fn extract_subject<T>(&self) -> Result<T>
    where
        T: Any + TryFrom<CBOR, Error = Error>,
    {
        fn extract_type<T, U>(value: &U) -> Result<T>
        where
            T: Any,
            U: Any + Clone,
        {
            if TypeId::of::<T>() == TypeId::of::<U>() {
                let cloned: Box<dyn Any> = Box::new(value.clone());
                let downcast = cloned
                    .downcast::<T>()
                    .unwrap();
                Ok(*downcast)
            } else {
                bail!(EnvelopeError::InvalidFormat)
            }
        }

        match self.case() {
            EnvelopeCase::Wrapped { envelope, .. } => extract_type::<T, Self>(envelope),
            EnvelopeCase::Node { subject, .. } => subject.extract_subject::<T>(),
            EnvelopeCase::Leaf { cbor, .. } => {
                let from_cbor: T = cbor.clone().try_into()?;
                Ok(from_cbor)
            },
            EnvelopeCase::Assertion(assertion) => extract_type::<T, Assertion>(assertion),
            EnvelopeCase::Elided(digest) => extract_type::<T, Digest>(digest),
            #[cfg(feature = "known_value")]
            EnvelopeCase::KnownValue { value, .. } => extract_type::<T, KnownValue>(value),
            #[cfg(feature = "encrypt")]
            EnvelopeCase::Encrypted(encrypted_message) => extract_type::<T, EncryptedMessage>(encrypted_message),
            #[cfg(feature = "compress")]
            EnvelopeCase::Compressed(compressed) => extract_type::<T, Compressed>(compressed),
        }
    }

    /// Returns all assertions with the given predicate. Match by comparing digests.
    pub fn assertions_with_predicate(&self, predicate: impl EnvelopeEncodable) -> Vec<Self> {
        let predicate = Envelope::new(predicate);
        self.assertions()
            .into_iter()
            .filter(|assertion| {
                assertion
                    .as_predicate()
                    .map(|p| p.digest() == predicate.digest())
                    .unwrap_or(false)
            })
            .collect()
    }

    /// Returns the assertion with the given predicate.
    ///
    /// Returns an error if there is no matching predicate or multiple matching predicates.
    pub fn assertion_with_predicate(&self, predicate: impl EnvelopeEncodable) -> Result<Self> {
        let a = self.assertions_with_predicate(predicate);
        if a.is_empty() {
            bail!(EnvelopeError::NonexistentPredicate);
        } else if a.len() == 1 {
            Ok(a[0].clone())
        } else {
            bail!(EnvelopeError::AmbiguousPredicate);
        }
    }

    /// Returns the object of the assertion with the given predicate.
    ///
    /// Returns an error if there is no matching predicate or multiple matching predicates.
    pub fn object_for_predicate(&self, predicate: impl EnvelopeEncodable) -> Result<Self> {
        Ok(self.assertion_with_predicate(predicate)?.as_object().unwrap())
    }

    /// Returns the object of the assertion with the given predicate, or `None` if there is no matching predicate.
    ///
    /// Returns an error if there are multiple matching predicates.
    pub fn optional_object_for_predicate(&self, predicate: impl EnvelopeEncodable) -> Result<Option<Self>> {
        let a = self.assertions_with_predicate(predicate);
        if a.is_empty() {
            Ok(None)
        } else if a.len() == 1 {
            Ok(Some(a[0].as_object().unwrap()))
        } else {
            bail!(EnvelopeError::AmbiguousPredicate);
        }
    }

    /// Returns the object of the assertion, decoded as the given type.
    ///
    /// Returns an error if the envelope is not an assertion.
    /// Returns an error if the encoded type doesn't match the given type.
    pub fn extract_object<T: TryFrom<CBOR, Error = Error> + 'static>(&self) -> Result<T> {
        self.try_object()?
            .extract_subject()
    }

    /// Returns the predicate of the assertion, decoded as the given type.
    ///
    /// Returns an error if the envelope is not an assertion.
    /// Returns an error if the encoded type doesn't match the given type.
    pub fn extract_predicate<T: TryFrom<CBOR, Error = Error> + 'static>(&self) -> Result<T> {
        self.try_predicate()?
            .extract_subject()
    }

    /// Returns the object of the assertion with the given predicate, decoded as the given type.
    ///
    /// Returns an error if there is no matching predicate or multiple matching predicates.
    /// Returns an error if the encoded type doesn't match the given type.
    pub fn extract_object_for_predicate<T: TryFrom<CBOR, Error = Error> + 'static>(&self, predicate: impl EnvelopeEncodable) -> Result<T> {
        self.assertion_with_predicate(predicate)?
            .extract_object()
    }

    /// Returns the object of the assertion with the given predicate, or `None` if there is no matching predicate.
    ///
    /// Returns an error if there are multiple matching predicates.
    pub fn extract_optional_object_for_predicate<T: TryFrom<CBOR, Error = Error> + 'static>(&self, predicate: impl EnvelopeEncodable) -> Result<Option<T>> {
        self.optional_object_for_predicate(predicate)?
            .map_or(Ok(None), |o| Ok(Some(o.extract_subject()?)))
    }

    /// Returns the object of the assertion with the given predicate, or a default value if there is no matching predicate.
    pub fn extract_object_for_predicate_with_default<T: TryFrom<CBOR, Error = Error> + 'static>(&self, predicate: impl EnvelopeEncodable, default: T) -> Result<T> {
        self.extract_optional_object_for_predicate(predicate)?
            .map_or(Ok(default), Ok)
    }

    /// Returns the objects of all assertions with the matching predicate.
    pub fn objects_for_predicate(&self, predicate: impl EnvelopeEncodable) -> Vec<Self> {
        self.assertions_with_predicate(predicate)
            .into_iter()
            .map(|a| a.as_object().unwrap())
            .collect()
    }

    /// Returns the objects of all assertions with the matching predicate,
    /// decoded as the given type.
    ///
    /// Returns an error if the encoded type doesn't match the given type.
    pub fn extract_objects_for_predicate<T: TryFrom<CBOR, Error = Error> + 'static>(&self, predicate: impl EnvelopeEncodable) -> Result<Vec<T>> {
        self.objects_for_predicate(predicate)
            .into_iter()
            .map(|a| a.extract_subject::<T>())
            .collect::<Result<Vec<T>>>()
    }

    /// Returns the number of structural elements in the envelope, including itself.
    pub fn elements_count(&self) -> usize {
        let mut result = 0;

        fn _count(envelope: &Envelope, result: &mut usize) {
            *result += 1;
            match envelope.case() {
                EnvelopeCase::Node {
                    subject,
                    assertions,
                    ..
                } => {
                    *result += subject.elements_count();
                    for assertion in assertions {
                        *result += assertion.elements_count();
                    }
                }
                EnvelopeCase::Assertion(assertion) => {
                    *result += assertion.predicate().elements_count();
                    *result += assertion.object().elements_count();
                }
                EnvelopeCase::Wrapped { envelope, .. } => {
                    *result += envelope.elements_count();
                }
                _ => {}
            }
        }

        _count(self, &mut result);

        result
    }
}
