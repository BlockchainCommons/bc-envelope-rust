use std::collections::{HashMap, HashSet};

#[cfg(feature = "compress")]
use bc_components::Compressed;
#[cfg(feature = "encrypt")]
use bc_components::EncryptedMessage;
use bc_components::{
    ARID, Digest, EncryptedKey, PrivateKeyBase, PrivateKeys, PublicKeys,
    Reference, SSKRShare, Salt, SealedMessage, Signature, URI, UUID, XID,
};
use dcbor::prelude::*;

use crate::{Assertion, Envelope};
#[cfg(any(feature = "encrypt", feature = "compress"))]
use crate::{Error, Result};

/// A trait for types that can be encoded as a Gordian Envelope.
///
/// This trait defines the interface for converting a value into an envelope.
/// Types implementing this trait can be used directly with envelope
/// construction functions without explicit conversion.
///
/// There are numerous built-in implementations for common types including:
/// - Primitive types (numbers, strings, booleans)
/// - CBOR values
/// - Cryptographic types from `bc-components` (digests, keys, etc.)
/// - Assertions
/// - Other envelopes
///
/// # Example
///
/// ```
/// # use bc_envelope::prelude::*;
///
/// // String implements EnvelopeEncodable
/// let e1 = "Hello".into_envelope();
///
/// // Numbers implement EnvelopeEncodable
/// let e2 = 42.into_envelope();
///
/// // Using in envelope construction
/// let envelope = Envelope::new("subject")
///     .add_assertion("name", "Alice")  // Uses EnvelopeEncodable for both predicate and object
///     .add_assertion("age", 30);       // Uses EnvelopeEncodable for the numeric object
/// ```
pub trait EnvelopeEncodable {
    /// Converts this value into a Gordian Envelope.
    ///
    /// This is the core method of the trait, converting the implementing type
    /// into an envelope representation. Most implementations will convert the
    /// value to a leaf envelope containing the value.
    ///
    /// # Returns
    ///
    /// A new envelope containing the value.
    fn into_envelope(self) -> Envelope;

    /// Converts a reference to this value into a Gordian Envelope.
    ///
    /// This is a convenience method that clones the value before converting it.
    /// It is implemented automatically for any type that implements `Clone`.
    ///
    /// # Returns
    ///
    /// A new envelope containing a clone of the value.
    fn to_envelope(&self) -> Envelope
    where
        Self: Clone,
    {
        self.clone().into_envelope()
    }
}

/// Generic implementation for any type that can be converted into an Envelope.
///
/// This implementation allows any type that implements `Into<Envelope>` to
/// automatically implement `EnvelopeEncodable`. This is a powerful way to
/// provide envelope encoding capabilities to a wide range of types.
impl<T> EnvelopeEncodable for T
where
    T: Into<Envelope> + Clone,
{
    /// Converts the value into an envelope by using its `Into<Envelope>`
    /// implementation.
    fn into_envelope(self) -> Envelope { self.into() }
}

/// Implementation of `EnvelopeEncodable` for `Assertion`.
///
/// This implementation converts an assertion into an envelope with the
/// assertion as its subject.
impl EnvelopeEncodable for Assertion {
    /// Creates an envelope with this assertion as its subject.
    fn into_envelope(self) -> Envelope { Envelope::new_with_assertion(self) }
}

/// TryFrom implementation to convert an encrypted message into an envelope.
///
/// This conversion is only available when the `encrypt` feature is enabled.
#[cfg(feature = "encrypt")]
impl TryFrom<EncryptedMessage> for Envelope {
    type Error = Error;

    /// Attempts to create an envelope with an encrypted message as its subject.
    ///
    /// This uses the specialized envelope constructor for encrypted content.
    fn try_from(value: EncryptedMessage) -> Result<Self> {
        Envelope::new_with_encrypted(value)
    }
}

/// TryFrom implementation to convert compressed data into an envelope.
///
/// This conversion is only available when the `compress` feature is enabled.
#[cfg(feature = "compress")]
impl TryFrom<Compressed> for Envelope {
    type Error = Error;

    /// Attempts to create an envelope with compressed data as its subject.
    ///
    /// This uses the specialized envelope constructor for compressed content.
    fn try_from(compressed: Compressed) -> Result<Self> {
        Envelope::new_with_compressed(compressed)
    }
}

/// Implementation of `EnvelopeEncodable` for `CBOR`.
///
/// This allows CBOR values to be directly encoded as envelope leaf nodes.
impl EnvelopeEncodable for CBOR {
    /// Creates a leaf envelope containing this CBOR value.
    fn into_envelope(self) -> Envelope { Envelope::new_leaf(self) }
}

/// Implementation of `EnvelopeEncodable` for `String`.
///
/// This allows Rust strings to be directly encoded as envelope leaf nodes.
impl EnvelopeEncodable for String {
    /// Creates a leaf envelope containing this string.
    fn into_envelope(self) -> Envelope { Envelope::new_leaf(self) }
}

/// Implementation of `EnvelopeEncodable` for `&str`.
///
/// This allows string slices to be directly encoded as envelope leaf nodes.
impl EnvelopeEncodable for &str {
    /// Creates a leaf envelope containing this string slice.
    fn into_envelope(self) -> Envelope { Envelope::new_leaf(self) }
}

impl<T> EnvelopeEncodable for Vec<T>
where
    T: CBOREncodable,
{
    fn into_envelope(self) -> Envelope { Envelope::new(CBOR::from(self)) }
}

impl<K, V> EnvelopeEncodable for HashMap<K, V>
where
    K: CBOREncodable,
    V: CBOREncodable,
{
    fn into_envelope(self) -> Envelope { Envelope::new(CBOR::from(self)) }
}

impl<T> EnvelopeEncodable for HashSet<T>
where
    T: CBOREncodable,
{
    fn into_envelope(self) -> Envelope { Envelope::new(CBOR::from(self)) }
}

impl EnvelopeEncodable for Map {
    fn into_envelope(self) -> Envelope { Envelope::new(CBOR::from(self)) }
}

impl EnvelopeEncodable for Set {
    fn into_envelope(self) -> Envelope { Envelope::new(CBOR::from(self)) }
}

/// Macro for implementing `EnvelopeEncodable` for a series of types.
///
/// This macro generates implementations that convert values to leaf envelopes.
/// It's used to reduce repetition when implementing for primitive types and
/// common data structures.
macro_rules! impl_envelope_encodable {
    ($type:ty) => {
        impl From<$type> for Envelope {
            /// Converts this value into an envelope.
            fn from(value: $type) -> Self { Envelope::new_leaf(value) }
        }
    };
}

// Numeric types
impl_envelope_encodable!(u8);
impl_envelope_encodable!(u16);
impl_envelope_encodable!(u32);
impl_envelope_encodable!(u64);
impl_envelope_encodable!(usize);
impl_envelope_encodable!(i8);
impl_envelope_encodable!(i16);
impl_envelope_encodable!(i32);
impl_envelope_encodable!(i64);

// Boolean type
impl_envelope_encodable!(bool);

// Floating point types
impl_envelope_encodable!(f64);
impl_envelope_encodable!(f32);

// CBOR types
impl_envelope_encodable!(ByteString);
impl_envelope_encodable!(Date);

// Cryptographic types
impl_envelope_encodable!(PublicKeys);
impl_envelope_encodable!(PrivateKeys);
impl_envelope_encodable!(PrivateKeyBase);
impl_envelope_encodable!(SealedMessage);
impl_envelope_encodable!(EncryptedKey);
impl_envelope_encodable!(Signature);
impl_envelope_encodable!(SSKRShare);
impl_envelope_encodable!(Digest);
impl_envelope_encodable!(Salt);

// Identifier types
impl_envelope_encodable!(ARID);
impl_envelope_encodable!(URI);
impl_envelope_encodable!(UUID);
impl_envelope_encodable!(XID);
impl_envelope_encodable!(Reference);
