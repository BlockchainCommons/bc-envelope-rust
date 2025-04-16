//! Provides methods for querying envelope structure and extracting data.
//!
//! The `queries` module contains methods for:
//!
//! 1. **Structural queries**: Methods for examining the envelope's structure
//!    (`subject()`, `assertions()`)
//! 2. **Type queries**: Methods for determining the envelope's type
//!    (`is_leaf()`, `is_node()`, etc.)
//! 3. **Content extraction**: Methods for extracting typed content from
//!    envelopes (`extract_subject()`, `extract_object_for_predicate()`)
//! 4. **Assertion queries**: Methods for finding assertions with specific
//!    predicates (`assertion_with_predicate()`)
//!
//! These methods enable traversal and inspection of envelope hierarchies,
//! allowing for flexible manipulation and access to envelope data structures.
//!
//! # Examples
//!
//! ```
//! use bc_envelope::prelude::*;
//!
//! // Create an envelope with assertions
//! let envelope = Envelope::new("Alice")
//!     .add_assertion("name", "Alice Adams")
//!     .add_assertion("age", 30)
//!     .add_assertion("email", "alice@example.com");
//!
//! // Query the envelope structure
//! let subject = envelope.subject(); // Returns "Alice"
//! let assertions = envelope.assertions(); // Returns all assertions
//!
//! // Find assertions with a specific predicate
//! if let Ok(email_assertion) = envelope.assertion_with_predicate("email") {
//!     let email = email_assertion.as_object().unwrap();
//!     assert_eq!(email.extract_subject::<String>().unwrap(), "alice@example.com");
//! }
//!
//! // Extract typed data directly from the envelope
//! let name = envelope.extract_object_for_predicate::<String>("name").unwrap();
//! let age = envelope.extract_object_for_predicate::<i32>("age").unwrap();
//!
//! assert_eq!(name, "Alice Adams");
//! assert_eq!(age, 30);
//! ```

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

    /// The envelope's leaf CBOR object as a CBOR byte string, or an error if
    /// the envelope is not a leaf, or the leaf is not a byte string.
    pub fn try_byte_string(&self) -> Result<Vec<u8>> {
        self.try_leaf()?.try_into_byte_string()
    }

    /// The envelope's `KnownValue`, or `None` if the envelope is not case
    /// `::KnownValue`.
    #[cfg(feature = "known_value")]
    pub fn as_known_value(&self) -> Option<&KnownValue> {
        match self.case() {
            EnvelopeCase::KnownValue { value, .. } => Some(value),
            _ => None,
        }
    }

    /// The envelope's `KnownValue`, or an error if the envelope is not case
    /// `::KnownValue`.
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

    /// `true` if the subject of the envelope is an assertion, `false`
    /// otherwise.
    pub fn is_subject_assertion(&self) -> bool {
        match self.case() {
            EnvelopeCase::Assertion(_) => true,
            EnvelopeCase::Node { subject, .. } => subject.is_subject_assertion(),
            _ => false,
        }
    }

    /// `true` if the subject of the envelope has been encrypted, `false`
    /// otherwise.
    #[cfg(feature = "encrypt")]
    pub fn is_subject_encrypted(&self) -> bool {
        match self.case() {
            EnvelopeCase::Encrypted(_) => true,
            EnvelopeCase::Node { subject, .. } => subject.is_subject_encrypted(),
            _ => false,
        }
    }

    /// `true` if the subject of the envelope has been compressed, `false`
    /// otherwise.
    #[cfg(feature = "compress")]
    pub fn is_subject_compressed(&self) -> bool {
        match self.case() {
            EnvelopeCase::Compressed(_) => true,
            EnvelopeCase::Node { subject, .. } => subject.is_subject_compressed(),
            _ => false,
        }
    }

    /// `true` if the subject of the envelope has been elided, `false`
    /// otherwise.
    pub fn is_subject_elided(&self) -> bool {
        match self.case() {
            EnvelopeCase::Elided(_) => true,
            EnvelopeCase::Node { subject, .. } => subject.is_subject_elided(),
            _ => false,
        }
    }

    /// `true` if the subject of the envelope has been encrypted, elided, or
    /// compressed, `false` otherwise.
    ///
    /// Obscured assertion envelopes may exist in the list of an envelope's
    /// assertions.
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

    /// `true` if the envelope is *internal*, that is, it has child elements, or
    /// `false` if it is a leaf node.
    ///
    /// Internal elements include `.node`, `.wrapped`, and `.assertion`.
    pub fn is_internal(&self) -> bool {
        matches!(
            self.case(),
            EnvelopeCase::Node { .. } | EnvelopeCase::Wrapped { .. } | EnvelopeCase::Assertion(_)
        )
    }

    /// `true` if the envelope is encrypted, elided, or compressed; `false`
    /// otherwise.
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

    /// Returns the envelope's subject, decoded as the given CBOR type.
    ///
    /// This method attempts to convert the envelope's subject into the
    /// requested type `T`. The conversion will succeed if the underlying CBOR
    /// data can be properly decoded as the specified type.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The target type to convert the subject into. Must implement
    ///   `TryFrom<CBOR>`.
    ///
    /// # Returns
    ///
    /// * `Result<T>` - The decoded subject value or an error if conversion
    ///   fails
    ///
    /// # Errors
    ///
    /// * Returns `Error::InvalidFormat` if the encoded type doesn't match the
    ///   requested type.
    ///
    /// # Examples
    ///
    /// ```
    /// use bc_envelope::prelude::*;
    ///
    /// // Extract a string
    /// let envelope = Envelope::new("Hello");
    /// let text: String = envelope.extract_subject().unwrap();
    /// assert_eq!(text, "Hello");
    ///
    /// // Extract a number
    /// let envelope = Envelope::new(42);
    /// let number: i32 = envelope.extract_subject().unwrap();
    /// assert_eq!(number, 42);
    ///
    /// // Extract fails with wrong type
    /// let envelope = Envelope::new("Not a number");
    /// let result = envelope.extract_subject::<i32>();
    /// assert!(result.is_err());
    /// ```
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
                let from_cbor = T::try_from(cbor.clone())?;
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
                    .subject()
                    .as_predicate()
                    .map(|p| p.digest() == predicate.digest())
                    .unwrap_or(false)
            })
            .collect()
    }

    /// Returns the assertion with the given predicate.
    ///
    /// Searches the envelope's assertions for one with a predicate matching the
    /// provided value. The match is determined by comparing the digests of the
    /// predicates.
    ///
    /// # Arguments
    ///
    /// * `predicate` - The predicate to search for, can be any type that
    ///   implements `EnvelopeEncodable`
    ///
    /// # Returns
    ///
    /// * `Result<Self>` - The matching assertion envelope if found
    ///
    /// # Errors
    ///
    /// * Returns `EnvelopeError::NonexistentPredicate` if no assertion has the
    ///   specified predicate
    /// * Returns `EnvelopeError::AmbiguousPredicate` if multiple assertions
    ///   have the specified predicate
    ///
    /// # Examples
    ///
    /// ```
    /// use bc_envelope::prelude::*;
    ///
    /// let envelope = Envelope::new("Person")
    ///     .add_assertion("name", "Alice")
    ///     .add_assertion("age", 30);
    ///
    /// // Find assertion with predicate "name"
    /// let name_assertion = envelope.assertion_with_predicate("name").unwrap();
    /// let name = name_assertion.as_object().unwrap().extract_subject::<String>().unwrap();
    /// assert_eq!(name, "Alice");
    ///
    /// // Trying to find a non-existent predicate produces an error
    /// assert!(envelope.assertion_with_predicate("address").is_err());
    /// ```
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

    /// Returns the assertion with the given predicate, or `None` if there is no
    /// matching predicate.
    ///
    /// Returns an error if there are multiple matching predicates.
    pub fn optional_assertion_with_predicate(&self, predicate: impl EnvelopeEncodable) -> Result<Option<Self>> {
        let a = self.assertions_with_predicate(predicate);
        if a.is_empty() {
            Ok(None)
        } else if a.len() == 1 {
            Ok(Some(a[0].clone()))
        } else {
            bail!(EnvelopeError::AmbiguousPredicate);
        }
    }

    /// Returns the object of the assertion with the given predicate.
    ///
    /// This is a convenience method that finds an assertion with the specified
    /// predicate and returns its object. It's a common operation when working
    /// with envelopes that have assertions containing data or metadata.
    ///
    /// # Arguments
    ///
    /// * `predicate` - The predicate to search for, can be any type that
    ///   implements `EnvelopeEncodable`
    ///
    /// # Returns
    ///
    /// * `Result<Self>` - The object part of the matching assertion
    ///
    /// # Errors
    ///
    /// * Returns `EnvelopeError::NonexistentPredicate` if no assertion has the
    ///   specified predicate
    /// * Returns `EnvelopeError::AmbiguousPredicate` if multiple assertions
    ///   have the specified predicate
    ///
    /// # Examples
    ///
    /// ```
    /// use bc_envelope::prelude::*;
    ///
    /// let envelope = Envelope::new("Person")
    ///     .add_assertion("name", "Alice")
    ///     .add_assertion("age", 30);
    ///
    /// // Get the object directly
    /// let name = envelope.object_for_predicate("name").unwrap();
    /// assert_eq!(name.extract_subject::<String>().unwrap(), "Alice");
    ///
    /// let age = envelope.object_for_predicate("age").unwrap();
    /// assert_eq!(age.extract_subject::<i32>().unwrap(), 30);
    /// ```
    pub fn object_for_predicate(&self, predicate: impl EnvelopeEncodable) -> Result<Self> {
        Ok(self.assertion_with_predicate(predicate)?.as_object().unwrap())
    }

    /// Returns the envelope decoded as the given type.
    ///
    /// This method attempts to convert the envelope into the requested type
    /// `T`. The conversion will succeed if the envelope can be properly decoded
    /// as the specified type.
    pub fn try_as<T>(&self) -> Result<T>
    where
        T: TryFrom<Envelope, Error = Error>,
    {
        self.clone().try_into()
    }

    /// Returns the object of the assertion with the given predicate, decoded as
    /// the given type.
    ///
    /// This is a convenience method that finds an assertion with the specified
    /// predicate and returns its object, decoded as the specified type.
    pub fn try_object_for_predicate<T>(&self, predicate: impl EnvelopeEncodable) -> Result<T>
    where
        T: TryFrom<Envelope, Error = Error>,
    {
        self.object_for_predicate(predicate)?
            .try_into()
    }

    /// Returns the object of the assertion with the given predicate, or `None`
    /// if there is no matching predicate.
    ///
    /// Returns an error if there are multiple matching predicates.
    pub fn optional_object_for_predicate(&self, predicate: impl EnvelopeEncodable) -> Result<Option<Self>> {
        let a = self.assertions_with_predicate(predicate);
        if a.is_empty() {
            Ok(None)
        } else if a.len() == 1 {
            Ok(Some(a[0].subject().as_object().unwrap()))
        } else {
            bail!(EnvelopeError::AmbiguousPredicate);
        }
    }

    /// Returns the object of the assertion with the given predicate, or `None`
    /// if there is no matching predicate.
    pub fn try_optional_object_for_predicate<T>(&self, predicate: impl EnvelopeEncodable) -> Result<Option<T>>
    where
        T: TryFrom<Envelope, Error = Error>,
    {
        self.optional_object_for_predicate(predicate)?
            .map(TryInto::try_into).transpose()
    }

    /// Returns the object of the assertion, decoded as the given CBOR type.
    ///
    /// This method works with assertion envelopes (created with
    /// `Envelope::new_assertion()`) and extracts the object part of the
    /// assertion as the specified type.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The target type to convert the object into. Must implement
    ///   `TryFrom<CBOR>`.
    ///
    /// # Returns
    ///
    /// * `Result<T>` - The decoded object value
    ///
    /// # Errors
    ///
    /// * Returns `EnvelopeError::NotAssertion` if the envelope is not an
    ///   assertion
    /// * Returns `Error::InvalidFormat` if the encoded type doesn't match the
    ///   requested type
    ///
    /// # Examples
    ///
    /// ```
    /// use bc_envelope::prelude::*;
    ///
    /// // Create an assertion envelope
    /// let assertion = Envelope::new_assertion("age", 30);
    ///
    /// // Extract the object value
    /// let age: i32 = assertion.extract_object().unwrap();
    /// assert_eq!(age, 30);
    ///
    /// // Not an assertion, gives an error
    /// let envelope = Envelope::new("Alice");
    /// assert!(envelope.extract_object::<String>().is_err());
    /// ```
    pub fn extract_object<T: TryFrom<CBOR, Error = Error> + 'static>(&self) -> Result<T> {
        self.try_object()?
            .extract_subject()
    }

    /// Returns the predicate of the assertion, decoded as the given CBOR type.
    ///
    /// Returns an error if the envelope is not an assertion. Returns an error
    /// if the encoded type doesn't match the given type.
    pub fn extract_predicate<T: TryFrom<CBOR, Error = Error> + 'static>(&self) -> Result<T> {
        self.try_predicate()?
            .extract_subject()
    }

    /// Returns the object of the assertion with the given predicate, decoded as
    /// the given CBOR type.
    ///
    /// This is a high-level convenience method that combines finding an
    /// assertion by predicate and extracting its object as a specific type.
    /// This is particularly useful when working with envelopes containing typed
    /// data (strings, numbers, etc.) as assertion objects.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The target type to convert the object into. Must implement
    ///   `TryFrom<CBOR>`.
    ///
    /// # Arguments
    ///
    /// * `predicate` - The predicate to search for
    ///
    /// # Returns
    ///
    /// * `Result<T>` - The decoded object value
    ///
    /// # Errors
    ///
    /// * Returns `EnvelopeError::NonexistentPredicate` if no assertion has the
    ///   specified predicate
    /// * Returns `EnvelopeError::AmbiguousPredicate` if multiple assertions
    ///   have the specified predicate
    /// * Returns `Error::InvalidFormat` if the encoded type doesn't match the
    ///   requested type
    ///
    /// # Examples
    ///
    /// ```
    /// use bc_envelope::prelude::*;
    ///
    /// let envelope = Envelope::new("Person")
    ///     .add_assertion("name", "Alice")
    ///     .add_assertion("age", 30);
    ///
    /// // Extract typed values directly
    /// let name: String = envelope.extract_object_for_predicate("name").unwrap();
    /// let age: i32 = envelope.extract_object_for_predicate("age").unwrap();
    ///
    /// assert_eq!(name, "Alice");
    /// assert_eq!(age, 30);
    ///
    /// // Type mismatch causes an error
    /// let result = envelope.extract_object_for_predicate::<i32>("name");
    /// assert!(result.is_err());
    /// ```
    pub fn extract_object_for_predicate<T: TryFrom<CBOR, Error = Error> + 'static>(&self, predicate: impl EnvelopeEncodable) -> Result<T> {
        self.assertion_with_predicate(predicate)?
            .extract_object()
    }

    /// Returns the object of the assertion with the given predicate decoded as
    /// the given CBOR type, or `None` if there is no matching predicate.
    ///
    /// Returns an error if there are multiple matching predicates.
    pub fn extract_optional_object_for_predicate<T: TryFrom<CBOR, Error = Error> + 'static>(&self, predicate: impl EnvelopeEncodable) -> Result<Option<T>> {
        self.optional_object_for_predicate(predicate)?
            .map_or(Ok(None), |o| Ok(Some(o.extract_subject()?)))
    }

    /// Returns the object of the assertion with the given predicate decoded as
    /// the given CBOR type, or a default value if there is no matching
    /// predicate.
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
    /// decoded as the given CBOR type.
    ///
    /// Returns an error if the encoded type doesn't match the given type.
    pub fn extract_objects_for_predicate<T: TryFrom<CBOR, Error = Error> + 'static>(&self, predicate: impl EnvelopeEncodable) -> Result<Vec<T>> {
        self.objects_for_predicate(predicate)
            .into_iter()
            .map(|a| a.extract_subject::<T>())
            .collect::<Result<Vec<T>>>()
    }

    /// Returns the objects of all assertions with the matching predicate,
    /// decoded as the given type.
    ///
    /// Returns an error if the encoded type doesn't match the given type.
    pub fn try_objects_for_predicate<T: TryFrom<Envelope, Error = Error> + 'static>(&self, predicate: impl EnvelopeEncodable) -> Result<Vec<T>> {
        self.objects_for_predicate(predicate)
            .into_iter()
            .map(|a| a.try_as::<T>())
            .collect::<Result<Vec<T>>>()
    }

    /// Returns the number of structural elements in the envelope, including
    /// itself.
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
