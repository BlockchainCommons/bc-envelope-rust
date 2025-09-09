//! Extensible metadata attachment container.
//!
//! This module provides the infrastructure for attaching arbitrary metadata
//! to envelopes. Attachments enable flexible, extensible data storage
//! without modifying the core data model, facilitating interoperability and
//! future compatibility.

use std::collections::HashMap;

use bc_components::{Digest, DigestProvider};

use crate::{Envelope, EnvelopeEncodable, Result};

/// A container for vendor-specific metadata attachments.
///
/// `Attachments` provides a flexible mechanism for attaching arbitrary metadata
/// to envelopes without modifying their core structure.
#[derive(Debug, Clone, PartialEq)]
pub struct Attachments {
    /// Storage mapping from digest to envelope
    envelopes: HashMap<Digest, Envelope>,
}

impl Default for Attachments {
    fn default() -> Self { Self::new() }
}

impl Attachments {
    /// Creates a new empty attachments container.
    pub fn new() -> Self { Self { envelopes: HashMap::new() } }

    /// Adds a new attachment with the specified payload and metadata.
    ///
    /// # Arguments
    /// * `payload` - The data to attach, which must be encodable in an envelope
    /// * `vendor` - A string identifying the entity that defined the attachment
    ///   format
    /// * `conforms_to` - An optional string identifying the structure the
    ///   payload conforms to
    pub fn add(
        &mut self,
        payload: impl EnvelopeEncodable,
        vendor: impl AsRef<str>,
        conforms_to: Option<impl AsRef<str>>,
    ) {
        let attachment = Envelope::new_attachment(
            payload,
            vendor.as_ref(),
            conforms_to.as_ref().map(|s| s.as_ref()),
        );
        self.envelopes
            .insert(attachment.digest().into_owned(), attachment);
    }

    /// Retrieves an attachment by its digest.
    ///
    /// # Arguments
    /// * `digest` - The unique digest of the attachment to retrieve
    ///
    /// # Returns
    /// A reference to the envelope if found, or None if no attachment exists
    /// with the given digest
    pub fn get(&self, digest: &Digest) -> Option<&Envelope> {
        self.envelopes.get(digest)
    }

    /// Removes an attachment by its digest.
    ///
    /// # Arguments
    /// * `digest` - The unique digest of the attachment to remove
    ///
    /// # Returns
    /// The removed envelope if found, or None if no attachment exists with the
    /// given digest
    pub fn remove(&mut self, digest: &Digest) -> Option<Envelope> {
        self.envelopes.remove(digest)
    }

    /// Removes all attachments from the container.
    pub fn clear(&mut self) { self.envelopes.clear(); }

    /// Returns whether the container has any attachments.
    ///
    /// # Returns
    /// `true` if there are no attachments, `false` otherwise
    pub fn is_empty(&self) -> bool { self.envelopes.is_empty() }

    pub fn add_to_envelope(&self, envelope: Envelope) -> Envelope {
        let mut new_envelope = envelope;
        for (_digest, envelope) in self.envelopes.iter() {
            new_envelope =
                new_envelope.add_assertion_envelope(envelope).unwrap();
        }
        new_envelope
    }

    pub fn try_from_envelope(envelope: &Envelope) -> Result<Attachments> {
        let attachment_envelopes = envelope.attachments()?;
        let mut attachments = Attachments::new();
        for attachment in attachment_envelopes {
            let digest = attachment.digest().into_owned();
            attachments.envelopes.insert(digest, attachment);
        }
        Ok(attachments)
    }
}

/// A trait for types that can have metadata attachments.
///
/// `Attachable` provides a consistent interface for working with metadata
/// attachments. Types implementing this trait can store and retrieve
/// vendor-specific data without modifying their core structure.
#[allow(dead_code)]
pub trait Attachable {
    /// Returns a reference to the attachments container.
    fn attachments(&self) -> &Attachments;

    /// Returns a mutable reference to the attachments container.
    fn attachments_mut(&mut self) -> &mut Attachments;

    /// Adds a new attachment with the specified payload and metadata.
    ///
    /// # Arguments
    /// * `payload` - The data to attach, which must be encodable in an envelope
    /// * `vendor` - A string identifying the entity that defined the attachment
    ///   format
    /// * `conforms_to` - An optional string identifying the structure the
    ///   payload conforms to
    fn add_attachment(
        &mut self,
        payload: impl EnvelopeEncodable,
        vendor: &str,
        conforms_to: Option<&str>,
    ) {
        self.attachments_mut().add(payload, vendor, conforms_to);
    }

    /// Retrieves an attachment by its digest.
    ///
    /// # Arguments
    /// * `digest` - The unique digest of the attachment to retrieve
    ///
    /// # Returns
    /// A reference to the envelope if found, or None if no attachment exists
    /// with the given digest
    fn get_attachment(&self, digest: &Digest) -> Option<&Envelope> {
        self.attachments().get(digest)
    }

    /// Removes an attachment by its digest.
    ///
    /// # Arguments
    /// * `digest` - The unique digest of the attachment to remove
    ///
    /// # Returns
    /// The removed envelope if found, or None if no attachment exists with the
    /// given digest
    fn remove_attachment(&mut self, digest: &Digest) -> Option<Envelope> {
        self.attachments_mut().remove(digest)
    }

    /// Removes all attachments from the object.
    fn clear_attachments(&mut self) { self.attachments_mut().clear(); }

    /// Returns whether the object has any attachments.
    ///
    /// # Returns
    /// `true` if there are attachments, `false` otherwise
    fn has_attachments(&self) -> bool { !self.attachments().is_empty() }
}
