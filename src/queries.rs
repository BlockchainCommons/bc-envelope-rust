use bc_components::{Compressed, Digest, EncryptedMessage};
use dcbor::{CBORDecodable, CBOR, CBOREncodable};
use std::{any::{Any, TypeId}, rc::Rc};

use crate::{Assertion, Envelope, EnvelopeError, KnownValue};

impl Envelope {
    /// The envelope's subject.
    ///
    /// For an envelope with no assertions, returns the same envelope.
    pub fn subject(self: Rc<Envelope>) -> Rc<Envelope> {
        match &*self {
            Envelope::Node { subject, .. } => subject.clone(),
            _ => self,
        }
    }

    /// The envelope's assertions.
    pub fn assertions(self: Rc<Envelope>) -> Vec<Rc<Envelope>> {
        match &*self {
            Envelope::Node { assertions, .. } => assertions.clone(),
            _ => vec![],
        }
    }

    /// `true` if the envelope has at least one assertion, `false` otherwise.
    pub fn has_assertions(&self) -> bool {
        match self {
            Envelope::Node { assertions, .. } => !assertions.is_empty(),
            _ => false,
        }
    }

    /// If the envelope's subject is an assertion return it, else return `None`.
    pub fn assertion(self: Rc<Envelope>) -> Option<Rc<Envelope>> {
        match &*self {
            Envelope::Assertion(_) => Some(self),
            _ => None,
        }
    }

    /// The envelope's predicate, or `None` if the envelope is not an assertion.
    pub fn predicate(self: Rc<Envelope>) -> Option<Rc<Envelope>> {
        match &*self {
            Envelope::Assertion(assertion) => Some(assertion.predicate()),
            _ => None,
        }
    }

    /// The envelope's object, or `None` if the envelope is not an assertion.
    pub fn object(self: Rc<Envelope>) -> Option<Rc<Envelope>> {
        match &*self {
            Envelope::Assertion(assertion) => Some(assertion.object()),
            _ => None,
        }
    }

    /// The envelope's leaf CBOR object, or `None` if the envelope is not a leaf.
    pub fn leaf(&self) -> Option<&CBOR> {
        match self {
            Envelope::Leaf { cbor, .. } => Some(cbor),
            _ => None,
        }
    }

    /// The envelope's `KnownValue`, or `None` if the envelope is not case `::KnownValue`.
    pub fn known_value(&self) -> Option<&KnownValue> {
        match self {
            Envelope::KnownValue { value, .. } => Some(value),
            _ => None,
        }
    }
}

impl Envelope {
    /// `true` if the envelope is case `::Leaf`, `false` otherwise.
    pub fn is_leaf(&self) -> bool {
        match self {
            Envelope::Leaf { .. } => true,
            _ => false,
        }
    }

    /// `true` if the envelope is case `::Node`, `false` otherwise.
    pub fn is_node(&self) -> bool {
        match self {
            Envelope::Node { .. } => true,
            _ => false,
        }
    }

    /// `true` if the envelope is case `::Wrapped`, `false` otherwise.
    pub fn is_wrapped(&self) -> bool {
        match self {
            Envelope::Wrapped { .. } => true,
            _ => false,
        }
    }

    /// `true` if the envelope is case `::KnownValue`, `false` otherwise.
    pub fn is_known_value(&self) -> bool {
        match self {
            Envelope::KnownValue { .. } => true,
            _ => false,
        }
    }

    /// `true` if the envelope is case `::Assertion`, `false` otherwise.
    pub fn is_assertion(&self) -> bool {
        match self {
            Envelope::Assertion(_) => true,
            _ => false,
        }
    }

    /// `true` if the envelope is case `::Encrypted`, `false` otherwise.
    pub fn is_encrypted(&self) -> bool {
        match self {
            Envelope::Encrypted(_) => true,
            _ => false,
        }
    }

    /// `true` if the envelope is case `::Compressed`, `false` otherwise.
    pub fn is_compressed(&self) -> bool {
        match self {
            Envelope::Compressed(_) => true,
            _ => false,
        }
    }
}

impl Envelope {
    /// `true` if the subject of the envelope is an assertion, `false` otherwise.
    pub fn is_subject_assertion(&self) -> bool {
        match self {
            Envelope::Assertion(_) => true,
            Envelope::Node { subject, .. } => subject.is_subject_assertion(),
            _ => false,
        }
    }

    /// `true` if the subject of the envelope has been encrypted, `false` otherwise.
    pub fn is_subject_encrypted(&self) -> bool {
        match self {
            Envelope::Encrypted(_) => true,
            Envelope::Node { subject, .. } => subject.is_subject_encrypted(),
            _ => false,
        }
    }

    /// `true` if the subject of the envelope has been compressed, `false` otherwise.
    pub fn is_subject_compressed(&self) -> bool {
        match self {
            Envelope::Compressed(_) => true,
            Envelope::Node { subject, .. } => subject.is_subject_compressed(),
            _ => false,
        }
    }

    /// `true` if the subject of the envelope has been elided, `false` otherwise.
    pub fn is_subject_elided(&self) -> bool {
        match self {
            Envelope::Elided(_) => true,
            Envelope::Node { subject, .. } => subject.is_subject_elided(),
            _ => false,
        }
    }

    /// `true` if the subject of the envelope has been encrypted, elided, or compressed, `false` otherwise.
    ///
    /// Obscured assertion envelopes may exist in the list of an envelope's assertions.
    pub fn is_subject_obscured(&self) -> bool {
        self.is_subject_encrypted() || self.is_subject_compressed() || self.is_subject_elided()
    }
}

impl Envelope {
    /// `true` if the envelope is *internal*, that is, it has child elements, or `false` if it is a leaf node.
    ///
    /// Internal elements include `.node`, `.wrapped`, and `.assertion`.
    pub fn is_internal(&self) -> bool {
        match self {
            Envelope::Node { .. } => true,
            Envelope::Wrapped { .. } => true,
            Envelope::Assertion(_) => true,
            _ => false,
        }
    }

    /// `true` if the envelope is encrypted, elided, or compressed; `false` otherwise.
    pub fn is_obscured(&self) -> bool {
        match self {
            Envelope::Encrypted(_) => true,
            Envelope::Compressed(_) => true,
            Envelope::Elided(_) => true,
            _ => false,
        }
    }
}

impl Envelope {
    /// Returns the envelope's subject, decoded as the given type.
    ///
    /// If the encoded type doesn't match the given type, returns `EnvelopeError::InvalidFormat`.
    pub fn extract_subject<T>(&self) -> Result<Rc<T>, EnvelopeError>
    where
        T: Any + CBORDecodable,
    {
        fn extract_type<T, U>(value: &U) -> Result<Rc<T>, EnvelopeError>
        where
            T: Any,
            U: Any + Clone,
        {
            if TypeId::of::<T>() == TypeId::of::<U>() {
                Ok((Rc::new(value.clone()) as Rc<dyn Any>)
                    .downcast::<T>()
                    .unwrap())
            } else {
                Err(EnvelopeError::InvalidFormat)
            }
        }

        match self {
            Envelope::Wrapped { envelope, .. } => extract_type::<T, Envelope>(&**envelope),
            Envelope::Node { subject, .. } => subject.extract_subject::<T>(),
            Envelope::Leaf { cbor, .. } => Ok(T::from_cbor(cbor).map_err(EnvelopeError::CBORError)?),
            Envelope::KnownValue { value, .. } => extract_type::<T, KnownValue>(&*value),
            Envelope::Assertion(assertion) => extract_type::<T, Assertion>(&*assertion),
            Envelope::Encrypted(encrypted_message) => extract_type::<T, EncryptedMessage>(&*encrypted_message),
            Envelope::Compressed(compressed) => extract_type::<T, Compressed>(&*compressed),
            Envelope::Elided(digest) => extract_type::<T, Digest>(&*digest),
        }
    }
}

impl Envelope {
    /// Returns all assertions with the given predicate. Match by comparing digests.
    pub fn assertions_with_predicate(self: Rc<Envelope>, predicate: Rc<Envelope>) -> Vec<Rc<Envelope>> {
        self.assertions()
            .into_iter()
            .filter(|assertion|
                assertion
                    .clone()
                    .predicate()
                    .map(|p| p.digest_ref() == predicate.digest_ref())
                    .unwrap_or(false)
            )
            .collect()
    }

    /// Returns all assertions with the given predicate.
    pub fn assertions_with_predicate_cbor(self: Rc<Envelope>, predicate: &dyn CBOREncodable) -> Vec<Rc<Envelope>> {
        self.assertions_with_predicate(Envelope::from_cbor_encodable(predicate))
    }

    /// Returns all assertions with the given predicate.
    pub fn assertions_with_predicate_known_value(self: Rc<Envelope>, predicate: &KnownValue) -> Vec<Rc<Envelope>> {
        self.assertions_with_predicate(Rc::new(Envelope::from(predicate)))
    }
}

impl Envelope {
    /// Returns the assertion with the given predicate.
    ///
    /// Returns an error if there is no matching predicate or multiple matching predicates.
    pub fn assertion_with_predicate(self: Rc<Envelope>, predicate: Rc<Envelope>) -> Result<Rc<Envelope>, EnvelopeError> {
        let a = self.assertions_with_predicate(predicate);
        if a.is_empty() {
            Err(EnvelopeError::NonexistentPredicate)
        } else if a.len() == 1 {
            Ok(a[0].clone())
        } else {
            Err(EnvelopeError::AmbiguousPredicate)
        }
    }

    /// Returns the assertion with the given predicate.
    ///
    /// Returns an error if there is no matching predicate or multiple matching predicates.
    pub fn assertion_with_predicate_cbor(self: Rc<Envelope>, predicate: &dyn CBOREncodable) -> Result<Rc<Envelope>, EnvelopeError> {
        self.assertion_with_predicate(Envelope::from_cbor_encodable(predicate))
    }

    /// Returns the assertion with the given predicate.
    ///
    /// Returns an error if there is no matching predicate or multiple matching predicates.
    pub fn assertion_with_predicate_known_value(self: Rc<Envelope>, predicate: &KnownValue) -> Result<Rc<Envelope>, EnvelopeError> {
        self.assertion_with_predicate(Rc::new(Envelope::from(predicate)))
    }
}

impl Envelope {
    /// Returns the object of the assertion with the given predicate.
    ///
    /// Returns an error if there is no matching predicate or multiple matching predicates.
    pub fn extract_object_for_predicate(self: Rc<Envelope>, predicate: Rc<Envelope>) -> Result<Rc<Envelope>, EnvelopeError> {
        Ok(self.assertion_with_predicate(predicate)?.object().unwrap())
    }

    /// Returns the object of the assertion with the given predicate.
    ///
    /// Returns an error if there is no matching predicate or multiple matching predicates.
    pub fn extract_object_for_predicate_cbor(self: Rc<Envelope>, predicate: &dyn CBOREncodable) -> Result<Rc<Envelope>, EnvelopeError> {
        self.extract_object_for_predicate(Envelope::from_cbor_encodable(predicate))
    }

    /// Returns the object of the assertion with the given predicate.
    ///
    /// Returns an error if there is no matching predicate or multiple matching predicates.
    pub fn extract_object_for_predicate_known_value(self: Rc<Envelope>, predicate: &KnownValue) -> Result<Rc<Envelope>, EnvelopeError> {
        self.extract_object_for_predicate(Rc::new(Envelope::from(predicate)))
    }

    /// Returns the object of the assertion with the given predicate.
    ///
    /// Returns an error if there is no matching predicate or multiple matching predicates.
    /// Returns an error if the encoded type doesn't match the given type.
    pub fn extract_object<T>(self: Rc<Envelope>, predicate: Rc<Envelope>) -> Result<Rc<T>, EnvelopeError>
        where T: CBORDecodable + 'static
    {
        self.assertion_with_predicate(predicate)?.object().unwrap().extract_subject()
    }

    /// Returns the object of the assertion with the given predicate.
    ///
    /// Returns an error if there is no matching predicate or multiple matching predicates.
    /// Returns an error if the encoded type doesn't match the given type.
    pub fn extract_object_cbor<T>(self: Rc<Envelope>, predicate: &dyn CBOREncodable) -> Result<Rc<T>, EnvelopeError>
        where T: CBORDecodable + 'static
    {
        self.extract_object(Envelope::from_cbor_encodable(predicate))
    }

    /// Returns the object of the assertion with the given predicate.
    ///
    /// Returns an error if there is no matching predicate or multiple matching predicates.
    /// Returns an error if the encoded type doesn't match the given type.
    pub fn extract_object_known_value<T>(self: Rc<Envelope>, predicate: &KnownValue) -> Result<Rc<T>, EnvelopeError>
        where T: CBORDecodable + 'static
    {
        self.extract_object(Rc::new(Envelope::from(predicate)))
    }
}

impl Envelope {
    /// Returns the objects of all assertions with the matching predicate.
    pub fn extract_objects_for_predicate(self: Rc<Envelope>, predicate: Rc<Envelope>) -> Vec<Rc<Envelope>> {
        self.assertions_with_predicate(predicate).into_iter().map(|a| a.object().unwrap()).collect()
    }

    /// Returns the objects of all assertions with the matching predicate.
    pub fn extract_objects_for_predicate_cbor(self: Rc<Envelope>, predicate: &dyn CBOREncodable) -> Vec<Rc<Envelope>> {
        self.extract_objects_for_predicate(Envelope::from_cbor_encodable(predicate))
    }

    /// Returns the objects of all assertions with the matching predicate.
    pub fn extract_objects_for_predicate_known_value(self: Rc<Envelope>, predicate: &KnownValue) -> Vec<Rc<Envelope>> {
        self.extract_objects_for_predicate(Rc::new(Envelope::from(predicate)))
    }

    /// Returns the objects of all assertions with the matching predicate.
    ///
    /// Returns an error if the encoded type doesn't match the given type.
    pub fn extract_objects<T>(self: Rc<Envelope>, predicate: Rc<Envelope>) -> Result<Vec<Rc<T>>, EnvelopeError>
        where T: CBORDecodable
    {
        self.assertions_with_predicate(predicate).into_iter().map(|a| a.object().unwrap().extract_subject()).collect()
    }

    /// Returns the objects of all assertions with the matching predicate.
    ///
    /// Returns an error if the encoded type doesn't match the given type.
    pub fn extract_objects_cbor<T>(self: Rc<Envelope>, predicate: &dyn CBOREncodable) -> Result<Vec<Rc<T>>, EnvelopeError>
        where T: CBORDecodable
    {
        self.extract_objects(Envelope::from_cbor_encodable(predicate))
    }

    /// Returns the objects of all assertions with the matching predicate.
    ///
    /// Returns an error if the encoded type doesn't match the given type.
    pub fn extract_objects_known_value<T>(self: Rc<Envelope>, predicate: &KnownValue) -> Result<Vec<Rc<T>>, EnvelopeError>
        where T: CBORDecodable
    {
        self.extract_objects(Rc::new(Envelope::from(predicate)))
    }
}

// The above Swift translated into Rust:
impl Envelope {
    /// Returns the number of structural elements in the envelope, including itself.
    pub fn elements_count(&self) -> usize {
        let mut result = 0;

        fn _count(envelope: &Envelope, result: &mut usize) {
            *result += 1;
            match envelope {
                Envelope::Node { subject, assertions, .. } => {
                    *result += subject.elements_count();
                    for assertion in assertions {
                        *result += assertion.elements_count();
                    }
                },
                Envelope::Assertion(assertion) => {
                    *result += assertion.predicate().elements_count();
                    *result += assertion.object().elements_count();
                },
                Envelope::Wrapped { envelope, .. } => {
                    *result += envelope.elements_count();
                },
                _ => {}
            }
        }

        _count(self, &mut result);

        result
    }
}
