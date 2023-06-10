use bc_components::{Compressed, Digest, DigestProvider, EncryptedMessage};
use dcbor::{CBORDecodable, CBOR};
use std::{
    any::{Any, TypeId},
    rc::Rc,
};

use crate::{Assertion, Envelope, Error, IntoEnvelope, KnownValue};

impl Envelope {
    /// The envelope's subject.
    ///
    /// For an envelope with no assertions, returns the same envelope.
    pub fn subject(self: Rc<Self>) -> Rc<Self> {
        match &*self {
            Self::Node { subject, .. } => subject.clone(),
            _ => self,
        }
    }

    /// The envelope's assertions.
    pub fn assertions(self: Rc<Self>) -> Vec<Rc<Self>> {
        match &*self {
            Self::Node { assertions, .. } => assertions.clone(),
            _ => vec![],
        }
    }

    /// `true` if the envelope has at least one assertion, `false` otherwise.
    pub fn has_assertions(&self) -> bool {
        match self {
            Self::Node { assertions, .. } => !assertions.is_empty(),
            _ => false,
        }
    }

    /// If the envelope's subject is an assertion return it, else return `None`.
    pub fn assertion(self: Rc<Self>) -> Option<Rc<Self>> {
        match &*self {
            Self::Assertion(_) => Some(self),
            _ => None,
        }
    }

    /// The envelope's predicate, or `None` if the envelope is not an assertion.
    pub fn predicate(self: Rc<Self>) -> Option<Rc<Self>> {
        match &*self {
            Self::Assertion(assertion) => Some(assertion.predicate()),
            _ => None,
        }
    }

    /// The envelope's object, or `None` if the envelope is not an assertion.
    pub fn object(self: Rc<Self>) -> Option<Rc<Self>> {
        match &*self {
            Self::Assertion(assertion) => Some(assertion.object()),
            _ => None,
        }
    }

    /// The envelope's leaf CBOR object, or `None` if the envelope is not a leaf.
    pub fn leaf(&self) -> Option<&CBOR> {
        match self {
            Self::Leaf { cbor, .. } => Some(cbor),
            _ => None,
        }
    }

    /// The envelope's `KnownValue`, or `None` if the envelope is not case `::KnownValue`.
    pub fn known_value(&self) -> Option<&KnownValue> {
        match self {
            Self::KnownValue { value, .. } => Some(value),
            _ => None,
        }
    }
}

impl Envelope {
    /// `true` if the envelope is case `::Leaf`, `false` otherwise.
    pub fn is_leaf(&self) -> bool {
        matches!(self, Self::Leaf { .. })
    }

    /// `true` if the envelope is case `::Node`, `false` otherwise.
    pub fn is_node(&self) -> bool {
        matches!(self, Self::Node { .. })
    }

    /// `true` if the envelope is case `::Wrapped`, `false` otherwise.
    pub fn is_wrapped(&self) -> bool {
        matches!(self, Self::Wrapped { .. })
    }

    /// `true` if the envelope is case `::KnownValue`, `false` otherwise.
    pub fn is_known_value(&self) -> bool {
        matches!(self, Self::KnownValue { .. })
    }

    /// `true` if the envelope is case `::Assertion`, `false` otherwise.
    pub fn is_assertion(&self) -> bool {
        matches!(self, Self::Assertion(_))
    }

    /// `true` if the envelope is case `::Encrypted`, `false` otherwise.
    pub fn is_encrypted(&self) -> bool {
        matches!(self, Self::Encrypted(_))
    }

    /// `true` if the envelope is case `::Compressed`, `false` otherwise.
    pub fn is_compressed(&self) -> bool {
        matches!(self, Self::Compressed(_))
    }

    /// `true` if the envelope is case `::Elided`, `false` otherwise.
    pub fn is_elided(&self) -> bool {
        matches!(self, Self::Elided(_))
    }
}

impl Envelope {
    /// `true` if the subject of the envelope is an assertion, `false` otherwise.
    pub fn is_subject_assertion(&self) -> bool {
        match self {
            Self::Assertion(_) => true,
            Self::Node { subject, .. } => subject.is_subject_assertion(),
            _ => false,
        }
    }

    /// `true` if the subject of the envelope has been encrypted, `false` otherwise.
    pub fn is_subject_encrypted(&self) -> bool {
        match self {
            Self::Encrypted(_) => true,
            Self::Node { subject, .. } => subject.is_subject_encrypted(),
            _ => false,
        }
    }

    /// `true` if the subject of the envelope has been compressed, `false` otherwise.
    pub fn is_subject_compressed(&self) -> bool {
        match self {
            Self::Compressed(_) => true,
            Self::Node { subject, .. } => subject.is_subject_compressed(),
            _ => false,
        }
    }

    /// `true` if the subject of the envelope has been elided, `false` otherwise.
    pub fn is_subject_elided(&self) -> bool {
        match self {
            Self::Elided(_) => true,
            Self::Node { subject, .. } => subject.is_subject_elided(),
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
        matches!(
            self,
            Self::Node { .. } | Self::Wrapped { .. } | Self::Assertion(_)
        )
    }

    /// `true` if the envelope is encrypted, elided, or compressed; `false` otherwise.
    pub fn is_obscured(&self) -> bool {
        matches!(
            self,
            Self::Encrypted(_) | Self::Compressed(_) | Self::Elided(_)
        )
    }
}

impl Envelope {
    /// Returns the envelope's subject, decoded as the given type.
    ///
    /// If the encoded type doesn't match the given type, returns `Error::InvalidFormat`.
    pub fn extract_subject<T>(&self) -> Result<Rc<T>, Error>
    where
        T: Any + CBORDecodable,
    {
        fn extract_type<T, U>(value: &U) -> Result<Rc<T>, Error>
        where
            T: Any,
            U: Any + Clone,
        {
            if TypeId::of::<T>() == TypeId::of::<U>() {
                Ok((Rc::new(value.clone()) as Rc<dyn Any>)
                    .downcast::<T>()
                    .unwrap())
            } else {
                Err(Error::InvalidFormat)
            }
        }

        match self {
            Self::Wrapped { envelope, .. } => extract_type::<T, Self>(&**envelope),
            Self::Node { subject, .. } => subject.extract_subject::<T>(),
            Self::Leaf { cbor, .. } => Ok(Rc::new(T::from_cbor(cbor)?)),
            Self::KnownValue { value, .. } => extract_type::<T, KnownValue>(value),
            Self::Assertion(assertion) => extract_type::<T, Assertion>(assertion),
            Self::Encrypted(encrypted_message) => {
                extract_type::<T, EncryptedMessage>(encrypted_message)
            }
            Self::Compressed(compressed) => extract_type::<T, Compressed>(compressed),
            Self::Elided(digest) => extract_type::<T, Digest>(digest),
        }
    }
}

impl Envelope {
    /// Returns all assertions with the given predicate. Match by comparing digests.
    pub fn assertions_with_predicate<P>(self: Rc<Self>, predicate: P) -> Vec<Rc<Self>>
    where
        P: IntoEnvelope,
    {
        let predicate = Envelope::new(predicate);
        self.assertions()
            .into_iter()
            .filter(|assertion| {
                assertion
                    .clone()
                    .predicate()
                    .map(|p| p.digest() == predicate.digest())
                    .unwrap_or(false)
            })
            .collect()
    }
}

impl Envelope {
    /// Returns the assertion with the given predicate.
    ///
    /// Returns an error if there is no matching predicate or multiple matching predicates.
    pub fn assertion_with_predicate<P>(
        self: Rc<Self>,
        predicate: P,
    ) -> Result<Rc<Self>, Error>
    where
        P: IntoEnvelope,
    {
        let a = self.assertions_with_predicate(predicate);
        if a.is_empty() {
            Err(Error::NonexistentPredicate)
        } else if a.len() == 1 {
            Ok(a[0].clone())
        } else {
            Err(Error::AmbiguousPredicate)
        }
    }
}

impl Envelope {
    /// Returns the object of the assertion with the given predicate.
    ///
    /// Returns an error if there is no matching predicate or multiple matching predicates.
    pub fn object_for_predicate<P>(self: Rc<Self>, predicate: P) -> Result<Rc<Self>, Error>
    where
        P: IntoEnvelope,
    {
        Ok(self.assertion_with_predicate(predicate)?.object().unwrap())
    }

    /// Returns the object of the assertion with the given predicate.
    ///
    /// Returns an error if there is no matching predicate or multiple matching predicates.
    /// Returns an error if the encoded type doesn't match the given type.
    pub fn extract_object_for_predicate<T, P>(
        self: Rc<Self>,
        predicate: P,
    ) -> Result<Rc<T>, Error>
    where
        T: CBORDecodable + 'static,
        P: IntoEnvelope,
    {
        self.assertion_with_predicate(predicate)?
            .object()
            .unwrap()
            .extract_subject()
    }
}

impl Envelope {
    /// Returns the objects of all assertions with the matching predicate.
    pub fn objects_for_predicate<P>(self: Rc<Self>, predicate: P) -> Vec<Rc<Self>>
    where
        P: IntoEnvelope,
    {
        self.assertions_with_predicate(predicate)
            .into_iter()
            .map(|a| a.object().unwrap())
            .collect()
    }

    /// Returns the objects of all assertions with the matching predicate.
    ///
    /// Returns an error if the encoded type doesn't match the given type.
    pub fn extract_objects_for_predicate<T, P>(
        self: Rc<Self>,
        predicate: P,
    ) -> Result<Vec<Rc<T>>, Error>
    where
        T: CBORDecodable,
        P: IntoEnvelope,
    {
        self.assertions_with_predicate(predicate)
            .into_iter()
            .map(|a| a.object().unwrap().extract_subject())
            .collect()
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
                Envelope::Node {
                    subject,
                    assertions,
                    ..
                } => {
                    *result += subject.elements_count();
                    for assertion in assertions {
                        *result += assertion.elements_count();
                    }
                }
                Envelope::Assertion(assertion) => {
                    *result += assertion.predicate().elements_count();
                    *result += assertion.object().elements_count();
                }
                Envelope::Wrapped { envelope, .. } => {
                    *result += envelope.elements_count();
                }
                _ => {}
            }
        }

        _count(self, &mut result);

        result
    }
}
