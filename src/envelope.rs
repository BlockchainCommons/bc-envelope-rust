use std::{rc::Rc, any::{Any, TypeId}};
use bc_components::{Digest, Compressed, EncryptedMessage, DigestProvider};
use dcbor::{CBOR, CBOREncodable};
use crate::{assertion::Assertion, KnownValue, EnvelopeError};

/// A flexible container for structured data.
///
/// Envelopes are immutable. You create "mutations" by creating new envelopes from old envelopes.
#[derive(Clone, Debug)]
pub enum Envelope {
    /// Represents an envelope with one or more assertions.
    Node { subject: Rc<Envelope>, assertions: Vec<Rc<Envelope>>, digest: Digest },

    /// Represents an envelope with encoded CBOR data.
    Leaf { cbor: CBOR, digest: Digest },

    /// Represents an envelope that wraps another envelope.
    Wrapped { envelope: Rc<Envelope>, digest: Digest },

    /// Represents a value from a namespace of unsigned integers.
    KnownValue { value: KnownValue, digest: Digest },

    /// Represents an assertion.
    ///
    /// An assertion is a predicate-object pair, each of which is itself an ``Envelope``.
    Assertion(Assertion),

    /// Represents an encrypted envelope.
    Encrypted(EncryptedMessage),

    /// Represents a compressed envelope.
    Compressed(Compressed),

    /// Represents an elided envelope.
    Elided(Digest),
}

impl Envelope {
    /// Create an Envelope from a &dyn CBOREncodable
    pub fn from_cbor_encodable(cbor_encodable: &dyn CBOREncodable) -> Rc<Self> {
        let cbor = cbor_encodable.cbor();
        let digest = Digest::from_image(&cbor.cbor_data());
        Rc::new(Envelope::Leaf {
            cbor,
            digest,
        })
    }
}

// Conversion from &CBOREncodable to Envelope
impl<T> From<&T> for Envelope
where
    T: CBOREncodable,
{
    fn from(t: &T) -> Self {
        let cbor = t.cbor();
        let digest = Digest::from_image(&cbor.cbor_data());
        Envelope::Leaf {
            cbor,
            digest,
        }
    }
}

pub trait RcEnvelope { }

impl RcEnvelope for Rc<Envelope> { }

pub fn new_envelope_with_unchecked_assertions(subject: Rc<Envelope>, unchecked_assertions: Vec<Rc<Envelope>>) -> Rc<Envelope> {
    assert!(!unchecked_assertions.is_empty());
    let mut sorted_assertions = unchecked_assertions;
    sorted_assertions.sort_by(|a, b| a.digest_ref().cmp(b.digest_ref()));
    let mut digests = vec![subject.digest()];
    digests.extend(sorted_assertions.iter().map(|a| a.digest()));
    let digest = Digest::from_digests(&digests);
    Rc::new(Envelope::Node { subject: subject, assertions: sorted_assertions, digest })
}

/// Internal constructors
impl Envelope {
    pub(crate) fn new_with_assertions(subject: Rc<Envelope>, assertions: Vec<Rc<Envelope>>) -> Result<Rc<Self>, EnvelopeError> {
        if !assertions.iter().all(|a| a.is_subject_assertion() || a.is_subject_obscured()) {
            return Err(EnvelopeError::InvalidFormat);
        }
        Ok(new_envelope_with_unchecked_assertions(subject, assertions))
    }

    pub(crate) fn new_with_assertion(assertion: Assertion) -> Rc<Self> {
        Rc::new(Self::Assertion(assertion))
    }

    pub(crate) fn new_with_encrypted(encrypted_message: EncryptedMessage) -> Result<Rc<Self>, EnvelopeError> {
        if !encrypted_message.has_digest() {
            return Err(EnvelopeError::MissingDigest);
        }
        Ok(Rc::new(Self::Encrypted(encrypted_message)))
    }

    pub(crate) fn new_with_compressed(compressed: Compressed) -> Result<Rc<Self>, EnvelopeError> {
        if !compressed.has_digest() {
            return Err(EnvelopeError::MissingDigest);
        }
        Ok(Rc::new(Self::Compressed(compressed)))
    }

    pub(crate) fn new_elided(digest: Digest) -> Rc<Self> {
        Rc::new(Self::Elided(digest))
    }

    pub(crate) fn new_leaf<T: CBOREncodable>(cbor: T) -> Rc<Self> {
        let cbor = cbor.cbor();
        let digest = Digest::from_image(&cbor.cbor_data());
        Rc::new(Self::Leaf { cbor, digest })
    }

    pub(crate) fn new_wrapped(envelope: Rc<Envelope>) -> Rc<Self> {
        let digest = Digest::from_digests(&[envelope.digest()]);
        Rc::new(Self::Wrapped { envelope, digest })
    }
}

/*
```swift
public extension Envelope {
    /// Create an envelope with the given subject.
    ///
    /// If the subject is another ``Envelope``, a wrapped envelope is created.
    /// If the subject is a ``KnownValue``, a known value envelope is created.
    /// If the subject is an ``Assertion``, an assertion envelope is created.
    /// If the subject is an `EncryptedMessage`, with a properly declared `Digest`, then an encrypted Envelope is created.
    /// If the subject is any type conforming to `CBOREncodable`, then a leaf envelope is created.
    /// Any other type passed as `subject` is a programmer error and results in a trap.
    ///
    /// ```swift
    /// let e = Envelope("Hello")
    /// print(e.format)
    /// ```
    ///
    /// ```
    /// "Hello"
    /// ```
    /// - Parameter subject: The envelope's subject.
    init(_ subject: Any) {
        if let envelope = subject as? Envelope {
            self.init(wrapped: envelope)
        } else if let knownValue = subject as? KnownValue {
            self.init(knownValue: knownValue)
        } else if let assertion = subject as? Assertion {
            self.init(assertion: assertion)
        } else if
            let encryptedMessage = subject as? EncryptedMessage,
            encryptedMessage.digest != nil
        {
            try! self.init(encryptedMessage: encryptedMessage)
        } else if let cborItem = subject as? CBOREncodable {
            self.init(leaf: cborItem.cbor)
        } else {
            preconditionFailure()
        }
    }

```
 */

impl Envelope {
    pub fn new<S>(subject: S) -> Rc<Self>
        where
            S: Any + Clone + CBOREncodable
    {
        if TypeId::of::<S>() == TypeId::of::<Rc<Envelope>>() {
            return (&subject as &dyn Any).downcast_ref::<Rc<Envelope>>().unwrap().clone()
        }

        if TypeId::of::<S>() == TypeId::of::<Envelope>() {
            return Rc::new((&subject as &dyn Any).downcast_ref::<Envelope>().unwrap().clone())
        }

        if TypeId::of::<S>() == TypeId::of::<KnownValue>() {
            let known_value = (&subject as &dyn Any).downcast_ref::<KnownValue>().unwrap().clone();
            return Self::new_with_known_value(known_value)
        }

        if TypeId::of::<S>() == TypeId::of::<Assertion>() {
            let assertion = (&subject as &dyn Any).downcast_ref::<Assertion>().unwrap().clone();
            return Self::new_with_assertion(assertion)
        }

        if TypeId::of::<S>() == TypeId::of::<EncryptedMessage>() {
            let encrypted_message = (&subject as &dyn Any).downcast_ref::<EncryptedMessage>().unwrap().clone();
            return Self::new_with_encrypted(encrypted_message).unwrap()
        }

        if TypeId::of::<S>() == TypeId::of::<Compressed>() {
            let compressed = (&subject as &dyn Any).downcast_ref::<Compressed>().unwrap().clone();
            return Self::new_with_compressed(compressed).unwrap()
        }

        let a = &subject;
        let cbor_encodable = a as &dyn CBOREncodable;
        let cbor = cbor_encodable.cbor();
        return Self::new_leaf(cbor);
    }

    pub fn new_assertion<P, O>(predicate: P, object: O) -> Rc<Self>
        where
            P: Any + Clone + CBOREncodable,
            O: Any + Clone + CBOREncodable
    {
        Self::new_with_assertion(Assertion::new(predicate, object))
    }

    pub fn new_with_known_value(value: KnownValue) -> Rc<Self> {
        let digest = value.digest();
        Rc::new(Self::KnownValue { value, digest })
    }

    pub fn new_with_known_value_predicate<O>(predicate: KnownValue, object: O) -> Rc<Self>
        where
            O: Any + Clone + CBOREncodable
    {
        Self::new_assertion(predicate, object)
    }
}

#[cfg(test)]
mod tests {
    use bc_components::{DigestProvider, Compressed};
    use crate::{Envelope, KnownValue, Assertion};

    #[test]
    fn test_any_envelope() {
        let e1 = Envelope::new_leaf("Hello");
        let e2 = Envelope::new(e1.clone());
        assert_eq!(e1.format(), e2.format());
        assert_eq!(e1.digest(), e2.digest());
    }

    #[test]
    fn test_any_known_value() {
        let known_value = KnownValue::new(100);
        let e1 = Envelope::new_with_known_value(known_value.clone());
        let e2 = Envelope::new(known_value);
        assert_eq!(e1.format(), e2.format());
        assert_eq!(e1.digest(), e2.digest());
    }

    #[test]
    fn test_any_assertion() {
        let assertion = Assertion::new(&"knows", &"Bob");
        let e1 = Envelope::new_with_assertion(assertion.clone());
        let e2 = Envelope::new(assertion);
        assert_eq!(e1.format(), e2.format());
        assert_eq!(e1.digest(), e2.digest());
    }

    #[test]
    fn test_any_encrypted() {
        //todo!()
    }

    #[test]
    fn test_any_compressed() {
        let data = "Hello".as_bytes();
        let digest = data.digest();
        let compressed = Compressed::from_uncompressed_data(data, Some(digest));
        let e1 = Envelope::new_with_compressed(compressed.clone()).unwrap();
        let e2 = Envelope::new(compressed);
        assert_eq!(e1.format(), e2.format());
        assert_eq!(e1.digest(), e2.digest());
    }

    #[test]
    fn test_any_cbor_encodable() {
        let e1 = Envelope::new_leaf(1);
        let e2 = Envelope::new(1);
        assert_eq!(e1.format(), e2.format());
        assert_eq!(e1.digest(), e2.digest());
    }
}
