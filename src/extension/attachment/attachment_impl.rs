use anyhow::{bail, Result};

use crate::{base::envelope::EnvelopeCase, known_values, Assertion, Envelope, EnvelopeEncodable, EnvelopeError};

/// Support for adding vendor-specific attachments to Gordian Envelopes.
///
/// This module extends Gordian Envelope with the ability to add vendor-specific attachments
/// to an envelope. Attachments provide a standardized way for different applications to
/// include their own data in an envelope without interfering with the main data structure
/// or with other attachments.
///
/// Each attachment has:
/// * A payload (arbitrary data)
/// * A required vendor identifier (typically a reverse domain name)
/// * An optional conformsTo URI that indicates the format of the attachment
///
/// This allows for a common envelope format that can be extended by different vendors
/// while maintaining interoperability.
///
/// # Example
///
/// ```
/// use bc_envelope::prelude::*;
///
/// // Create a base envelope
/// let envelope = Envelope::new("Alice")
///     .add_assertion("knows", "Bob");
///
/// // Add a vendor-specific attachment
/// let with_attachment = envelope.add_attachment(
///     "Custom data for this envelope",
///     "com.example",
///     Some("https://example.com/attachment-format/v1")
/// );
///
/// // The attachment is added as an assertion with the 'attachment' predicate
/// assert!(!with_attachment.assertions_with_predicate(known_values::ATTACHMENT).is_empty());
///
/// // The attachment can be extracted later
/// let attachment = with_attachment.attachments().unwrap()[0].clone();
/// let payload = attachment.attachment_payload().unwrap();
/// let vendor = attachment.attachment_vendor().unwrap();
/// let format = attachment.attachment_conforms_to().unwrap();
///
/// assert_eq!(payload.format_flat(), r#""Custom data for this envelope""#);
/// assert_eq!(vendor, "com.example");
/// assert_eq!(format, Some("https://example.com/attachment-format/v1".to_string()));
/// ```
/// Methods for creating and accessing attachments at the assertion level
impl Assertion {
    /// Creates a new attachment assertion.
    ///
    /// An attachment assertion consists of:
    /// * The predicate `known_values::ATTACHMENT`
    /// * An object that is a wrapped envelope containing:
    ///   * The payload (as the subject)
    ///   * A required `'vendor': String` assertion
    ///   * An optional `'conformsTo': String` assertion
    ///
    /// See [BCR-2023-006](https://github.com/BlockchainCommons/Research/blob/master/papers/bcr-2023-006-envelope-attachment.md)
    /// for the detailed specification.
    ///
    /// # Parameters
    ///
    /// * `payload` - The content of the attachment
    /// * `vendor` - A string that uniquely identifies the vendor (typically a reverse domain name)
    /// * `conforms_to` - An optional URI that identifies the format of the attachment
    ///
    /// # Returns
    ///
    /// A new attachment assertion
    ///
    /// # Examples
    ///
    /// Example:
    ///
    /// Create an attachment assertion that contains vendor-specific data,
    /// then use it to access the payload, vendor ID, and conformsTo value.
    ///
    /// The assertion will have a predicate of "attachment" and an object that's
    /// a wrapped envelope containing the payload with vendor and conformsTo
    /// assertions added to it.
    ///
    pub fn new_attachment(payload: impl EnvelopeEncodable, vendor: &str, conforms_to: Option<&str>) -> Self {
        let conforms_to: Option<String> = conforms_to.map(|c| c.to_string());
        Self::new(
            known_values::ATTACHMENT,
            payload
                .into_envelope()
                .wrap_envelope()
                .add_assertion(known_values::VENDOR, vendor.to_string())
                .add_optional_assertion(known_values::CONFORMS_TO, conforms_to)
        )
    }

    /// Returns the payload of an attachment assertion.
    ///
    /// This extracts the subject of the wrapped envelope that is the object of this attachment assertion.
    ///
    /// # Returns
    ///
    /// The payload envelope
    ///
    /// # Errors
    ///
    /// Returns an error if the assertion is not a valid attachment assertion
    pub fn attachment_payload(&self) -> Result<Envelope> {
        self.object().unwrap_envelope()
    }

    /// Returns the vendor identifier of an attachment assertion.
    ///
    /// # Returns
    ///
    /// The vendor string
    ///
    /// # Errors
    ///
    /// Returns an error if the assertion is not a valid attachment assertion
    pub fn attachment_vendor(&self) -> dcbor::Result<String> {
        self.object().extract_object_for_predicate(known_values::VENDOR)
    }

    /// Returns the optional conformsTo URI of an attachment assertion.
    ///
    /// # Returns
    ///
    /// The conformsTo string if present, or None
    ///
    /// # Errors
    ///
    /// Returns an error if the assertion is not a valid attachment assertion
    pub fn attachment_conforms_to(&self) -> dcbor::Result<Option<String>> {
        self.object().extract_optional_object_for_predicate(known_values::CONFORMS_TO)
    }

    /// Validates that an assertion is a proper attachment assertion.
    ///
    /// This ensures:
    /// - The attachment assertion's predicate is `known_values::ATTACHMENT`
    /// - The attachment assertion's object is an envelope
    /// - The attachment assertion's object has a `'vendor': String` assertion
    /// - The attachment assertion's object has an optional `'conformsTo': String` assertion
    ///
    /// # Returns
    ///
    /// Ok(()) if the assertion is a valid attachment assertion
    ///
    /// # Errors
    ///
    /// Returns `EnvelopeError::InvalidAttachment` if the assertion is not a valid attachment assertion
    pub fn validate_attachment(&self) -> Result<()> {
        let payload = self.attachment_payload()?;
        let vendor = self.attachment_vendor()?;
        let conforms_to: Option<String> = self.attachment_conforms_to()?;
        let assertion = Assertion::new_attachment(payload, vendor.as_str(), conforms_to.as_deref());
        let e: Envelope = assertion.to_envelope();
        if !e.is_equivalent_to(&self.clone().to_envelope()) {
            bail!(EnvelopeError::InvalidAttachment);
        }
        Ok(())
    }
}

/// Methods for creating attachment envelopes
impl Envelope {
    /// Creates a new envelope with an attachment as its subject.
    ///
    /// This creates an envelope whose subject is an attachment assertion, using the
    /// provided payload, vendor, and optional conformsTo URI.
    ///
    /// # Parameters
    ///
    /// * `payload` - The content of the attachment
    /// * `vendor` - A string that uniquely identifies the vendor (typically a reverse domain name)
    /// * `conforms_to` - An optional URI that identifies the format of the attachment
    ///
    /// # Returns
    ///
    /// A new envelope with the attachment as its subject
    ///
    /// # Examples
    ///
    /// ```
    /// use bc_envelope::prelude::*;
    /// use bc_envelope::base::envelope::EnvelopeCase;
    ///
    /// // Create an attachment envelope
    /// let envelope = Envelope::new_attachment(
    ///     "Attachment data",
    ///     "com.example",
    ///     Some("https://example.com/format/v1")
    /// );
    ///
    /// // The envelope is an assertion
    /// assert!(matches!(envelope.case(), EnvelopeCase::Assertion(_)));
    /// ```
    pub fn new_attachment(payload: impl EnvelopeEncodable, vendor: &str, conforms_to: Option<&str>) -> Self
    {
        Assertion::new_attachment(payload, vendor, conforms_to).to_envelope()
    }

    /// Returns a new envelope with an added `'attachment': Envelope` assertion.
    ///
    /// This adds an attachment assertion to an existing envelope.
    ///
    /// # Parameters
    ///
    /// * `payload` - The content of the attachment
    /// * `vendor` - A string that uniquely identifies the vendor (typically a reverse domain name)
    /// * `conforms_to` - An optional URI that identifies the format of the attachment
    ///
    /// # Returns
    ///
    /// A new envelope with the attachment assertion added
    ///
    /// # Examples
    ///
    /// ```
    /// use bc_envelope::prelude::*;
    ///
    /// // Create a base envelope
    /// let envelope = Envelope::new("User data")
    ///     .add_assertion("name", "Alice");
    ///
    /// // Add an attachment
    /// let with_attachment = envelope.add_attachment(
    ///     "Vendor-specific metadata",
    ///     "com.example",
    ///     Some("https://example.com/metadata/v1")
    /// );
    ///
    /// // The original envelope is unchanged
    /// assert_eq!(envelope.assertions().len(), 1);
    ///
    /// // The new envelope has an additional attachment assertion
    /// assert_eq!(with_attachment.assertions().len(), 2);
    /// assert!(with_attachment.assertions_with_predicate(known_values::ATTACHMENT).len() > 0);
    /// ```
    pub fn add_attachment(&self, payload: impl EnvelopeEncodable, vendor: &str, conforms_to: Option<&str>) -> Self {
        self.add_assertion_envelope(
            Assertion::new_attachment(payload, vendor, conforms_to)
        ).unwrap()
    }
}

/// Methods for accessing attachments in envelopes
impl Envelope {
    /// Returns the payload of an attachment envelope.
    ///
    /// # Returns
    ///
    /// The payload envelope
    ///
    /// # Errors
    ///
    /// Returns an error if the envelope is not a valid attachment envelope
    pub fn attachment_payload(&self) -> Result<Self> {
        if let EnvelopeCase::Assertion(assertion) = self.case() {
            Ok(assertion.attachment_payload()?)
        } else {
            bail!(EnvelopeError::InvalidAttachment)
        }
    }

    /// Returns the vendor identifier of an attachment envelope.
    ///
    /// # Returns
    ///
    /// The vendor string
    ///
    /// # Errors
    ///
    /// Returns an error if the envelope is not a valid attachment envelope
    pub fn attachment_vendor(&self) -> Result<String> {
        if let EnvelopeCase::Assertion(assertion) = self.case() {
            Ok(assertion.attachment_vendor()?)
        } else {
            bail!(EnvelopeError::InvalidAttachment);
        }
    }

    /// Returns the optional conformsTo URI of an attachment envelope.
    ///
    /// # Returns
    ///
    /// The conformsTo string if present, or None
    ///
    /// # Errors
    ///
    /// Returns an error if the envelope is not a valid attachment envelope
    pub fn attachment_conforms_to(&self) -> Result<Option<String>> {
        if let EnvelopeCase::Assertion(assertion) = self.case() {
            Ok(assertion.attachment_conforms_to()?)
        } else {
            bail!(EnvelopeError::InvalidAttachment);
        }
    }

    /// Searches the envelope's assertions for attachments that match the given vendor and conformsTo.
    ///
    /// This method finds all attachment assertions in the envelope that match the specified
    /// criteria:
    ///
    /// * If `vendor` is `None`, matches any vendor
    /// * If `conformsTo` is `None`, matches any conformsTo value
    /// * If both are `None`, matches all attachments
    ///
    /// # Parameters
    ///
    /// * `vendor` - Optional vendor identifier to match
    /// * `conforms_to` - Optional conformsTo URI to match
    ///
    /// # Returns
    ///
    /// A vector of matching attachment envelopes
    ///
    /// # Errors
    ///
    /// Returns an error if any of the envelope's attachments are invalid
    ///
    /// # Examples
    ///
    /// ```
    /// use bc_envelope::prelude::*;
    ///
    /// // Create an envelope with two attachments from the same vendor
    /// let envelope = Envelope::new("Data")
    ///     .add_attachment("Attachment 1", "com.example", Some("https://example.com/format/v1"))
    ///     .add_attachment("Attachment 2", "com.example", Some("https://example.com/format/v2"));
    ///
    /// // Find all attachments
    /// let all_attachments = envelope.attachments().unwrap();
    /// assert_eq!(all_attachments.len(), 2);
    ///
    /// // Find attachments by vendor
    /// let vendor_attachments = envelope
    ///     .attachments_with_vendor_and_conforms_to(Some("com.example"), None)
    ///     .unwrap();
    /// assert_eq!(vendor_attachments.len(), 2);
    ///
    /// // Find attachments by specific format
    /// let v1_attachments = envelope
    ///     .attachments_with_vendor_and_conforms_to(None, Some("https://example.com/format/v1"))
    ///     .unwrap();
    /// assert_eq!(v1_attachments.len(), 1);
    /// ```
    pub fn attachments_with_vendor_and_conforms_to(&self, vendor: Option<&str>, conforms_to: Option<&str>)
    -> Result<Vec<Self>>
    {
        let assertions = self.assertions_with_predicate(known_values::ATTACHMENT);
        for assertion in &assertions {
            Self::validate_attachment(assertion)?;
        }
        let matching_assertions: Vec<_> = assertions
            .into_iter()
            .filter(|assertion| {
                if let Some(vendor) = vendor {
                    if let Ok(v) = assertion.attachment_vendor() {
                        if v != vendor {
                            return false;
                        }
                    }
                }
                if let Some(conforms_to) = conforms_to {
                    if let Ok(c) = assertion.attachment_conforms_to() {
                        if let Some(c) = c {
                            if c != conforms_to {
                                return false;
                            }
                        } else {
                            return false;
                        }
                    }
                }
                true
            })
            .collect();
        Result::Ok(matching_assertions)
    }

    /// Returns all attachments in the envelope.
    ///
    /// This is equivalent to calling `attachments_with_vendor_and_conforms_to(None, None)`.
    ///
    /// # Returns
    ///
    /// A vector of all attachment envelopes
    ///
    /// # Errors
    ///
    /// Returns an error if any of the envelope's attachments are invalid
    pub fn attachments(&self) -> Result<Vec<Self>> {
        self.attachments_with_vendor_and_conforms_to(None::<&str>, None::<&str>)
    }

    /// Validates that an envelope is a proper attachment envelope.
    ///
    /// This ensures the envelope is an assertion envelope with the predicate `attachment`
    /// and the required structure for an attachment.
    ///
    /// # Returns
    ///
    /// Ok(()) if the envelope is a valid attachment envelope
    ///
    /// # Errors
    ///
    /// Returns `EnvelopeError::InvalidAttachment` if the envelope is not a valid attachment envelope
    pub fn validate_attachment(&self) -> Result<()> {
        if let EnvelopeCase::Assertion(assertion) = self.case() {
            assertion.validate_attachment()?;
            Ok(())
        } else {
            bail!(EnvelopeError::InvalidAttachment);
        }
    }

    /// Finds a single attachment matching the given vendor and conformsTo.
    ///
    /// This works like `attachments_with_vendor_and_conforms_to` but returns a single
    /// attachment envelope rather than a vector. It requires that exactly one attachment
    /// matches the criteria.
    ///
    /// # Parameters
    ///
    /// * `vendor` - Optional vendor identifier to match
    /// * `conforms_to` - Optional conformsTo URI to match
    ///
    /// # Returns
    ///
    /// The matching attachment envelope
    ///
    /// # Errors
    ///
    /// * Returns `EnvelopeError::NonexistentAttachment` if no attachments match
    /// * Returns `EnvelopeError::AmbiguousAttachment` if more than one attachment matches
    /// * Returns an error if any of the envelope's attachments are invalid
    ///
    /// # Examples
    ///
    /// ```
    /// use bc_envelope::prelude::*;
    ///
    /// // Create an envelope with an attachment
    /// let envelope = Envelope::new("Data")
    ///     .add_attachment(
    ///         "Metadata",
    ///         "com.example",
    ///         Some("https://example.com/format/v1")
    ///     );
    ///
    /// // Find a specific attachment by vendor and format
    /// let attachment = envelope
    ///     .attachment_with_vendor_and_conforms_to(
    ///         Some("com.example"),
    ///         Some("https://example.com/format/v1")
    ///     )
    ///     .unwrap();
    ///
    /// // Access the attachment payload
    /// let payload = attachment.attachment_payload().unwrap();
    /// assert_eq!(payload.extract_subject::<String>().unwrap(), "Metadata");
    /// ```
    pub fn attachment_with_vendor_and_conforms_to(&self, vendor: Option<&str>, conforms_to: Option<&str>)
    -> Result<Self>
    {
        let attachments = self.attachments_with_vendor_and_conforms_to(vendor, conforms_to)?;
        if attachments.is_empty() {
            bail!(EnvelopeError::NonexistentAttachment);
        }
        if attachments.len() > 1 {
            bail!(EnvelopeError::AmbiguousAttachment);
        }
        Ok(attachments.first().unwrap().clone())
    }
}
