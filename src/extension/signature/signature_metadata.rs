use crate::{Assertion, EnvelopeEncodable};

/// Metadata associated with a signature in a Gordian Envelope.
///
/// `SignatureMetadata` provides a way to attach additional information to signatures,
/// such as the signer's identity, the signing date, or the purpose of the signature.
/// When used with the signature extension, this metadata is included in a structured
/// way that is also signed, ensuring the metadata cannot be tampered with without
/// invalidating the signature.
///
/// The metadata is represented as a collection of assertions that are attached to
/// the signature envelope and then themselves signed with the same key that signed
/// the payload.
///
/// # Examples
///
/// ```ignore
/// use bc_envelope::prelude::*;
/// use known_values::NOTE;
///
/// # fn main() -> anyhow::Result<()> {
/// # // In a real application, you would use proper key generation
/// # let alice_private_key = /* your private key */;
/// 
/// // Create metadata with a note
/// let metadata = SignatureMetadata::new()
///     .with_assertion(NOTE, "Alice signed this.")
///     .with_assertion("date", "2024-04-02")
///     .with_assertion("purpose", "Proof of Identity");
///
/// // Create and sign an envelope with the metadata
/// let envelope = Envelope::new("Important document")
///     .wrap_envelope()
///     .add_signature_opt(&alice_private_key, None, Some(metadata));
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct SignatureMetadata {
    assertions: Vec<Assertion>,
}

impl SignatureMetadata {
    /// Creates a new, empty `SignatureMetadata` instance.
    ///
    /// # Returns
    ///
    /// A new `SignatureMetadata` with no assertions.
    pub fn new() -> Self {
        Self {
            assertions: Vec::new(),
        }
    }

    /// Creates a new `SignatureMetadata` with the specified assertions.
    ///
    /// # Parameters
    ///
    /// * `assertions` - A vector of `Assertion`s to include in the metadata.
    ///
    /// # Returns
    ///
    /// A new `SignatureMetadata` containing the provided assertions.
    pub fn new_with_assertions(assertions: Vec<Assertion>) -> Self {
        Self {
            assertions,
        }
    }

    /// Returns a reference to the assertions contained in this metadata.
    ///
    /// # Returns
    ///
    /// A slice containing all assertions in this metadata.
    pub fn assertions(&self) -> &[Assertion] {
        &self.assertions
    }

    /// Adds an assertion to this metadata.
    ///
    /// # Parameters
    ///
    /// * `assertion` - The `Assertion` to add.
    ///
    /// # Returns
    ///
    /// A new `SignatureMetadata` with the assertion added.
    pub fn add_assertion(mut self, assertion: Assertion) -> Self {
        self.assertions.push(assertion);
        self
    }

    /// Adds a new assertion to this metadata using the provided predicate and object.
    ///
    /// This is a convenience method that creates an `Assertion` from the predicate
    /// and object and adds it to the metadata.
    ///
    /// # Parameters
    ///
    /// * `predicate` - The predicate for the assertion.
    /// * `object` - The object for the assertion.
    ///
    /// # Returns
    ///
    /// A new `SignatureMetadata` with the assertion added.
    pub fn with_assertion(self, predicate: impl EnvelopeEncodable, object: impl EnvelopeEncodable) -> Self {
        self.add_assertion(Assertion::new(predicate, object))
    }

    /// Returns whether this metadata contains any assertions.
    ///
    /// # Returns
    ///
    /// `true` if this metadata contains at least one assertion, `false` otherwise.
    pub fn has_assertions(&self) -> bool {
        !self.assertions.is_empty()
    }
}

/// Default implementation for `SignatureMetadata`.
///
/// Creates an empty `SignatureMetadata` with no assertions.
impl Default for SignatureMetadata {
    fn default() -> Self {
        Self::new()
    }
}
