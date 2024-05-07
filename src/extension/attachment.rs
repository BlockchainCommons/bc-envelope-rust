use anyhow::bail;

use crate::{base::envelope::EnvelopeCase, extension::known_values, Assertion, Envelope, EnvelopeEncodable, EnvelopeError};

impl Assertion {
    /// Creates an attachment assertion. See:
    /// [BCR-2023-006](https://github.com/BlockchainCommons/Research/blob/master/papers/bcr-2023-006-envelope-attachment.md)
    pub fn new_attachment(payload: impl EnvelopeEncodable, vendor: &str, conforms_to: Option<&str>) -> Self {
        let conforms_to: Option<String> = conforms_to.map(|c| c.to_string());
        Self::new(
            known_values::ATTACHMENT,
            payload
                .to_envelope()
                .wrap_envelope()
                .add_assertion(known_values::VENDOR, vendor.to_string())
                .add_optional_assertion(known_values::CONFORMS_TO, conforms_to)
        )
    }

    /// Returns the payload of the given attachment assertion.
    pub fn attachment_payload(&self) -> anyhow::Result<Envelope> {
        self.object().unwrap_envelope()
    }

    /// Returns the `vendor` of the given attachment assertion.
    pub fn attachment_vendor(&self) -> anyhow::Result<String> {
        self.object().extract_object_for_predicate(known_values::VENDOR)
    }

    /// Returns the `conformsTo` of the given attachment assertion.
    pub fn attachment_conforms_to(&self) -> anyhow::Result<Option<String>> {
        self.object().extract_optional_object_for_predicate(known_values::CONFORMS_TO)
    }

    /// Validates the given attachment assertion.
    ///
    /// Ensures:
    /// - The attachment assertion's predicate is `known_values::ATTACHMENT`.
    /// - The attachment assertion's object is an envelope.
    /// - The attachment assertion's object has a `'vendor': String` assertion.
    /// - The attachment assertion's object has an optional `'conformsTo': String` assertion.
    pub fn validate_attachment(&self) -> anyhow::Result<()> {
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

impl Envelope {
    /// Returns a new attachment envelope.
    ///
    /// The payload envelope has a `'vendor': String` assertion and an optional
    /// `'conformsTo': String` assertion.
    pub fn new_attachment(payload: impl EnvelopeEncodable, vendor: &str, conforms_to: Option<&str>) -> Self
    {
        Assertion::new_attachment(payload, vendor, conforms_to).to_envelope()
    }

    /// Returns a new envelope with an added `'attachment': Envelope` assertion.
    ///
    /// The payload envelope has a `'vendor': String` assertion and an optional
    /// `'conformsTo': String` assertion.
    pub fn add_attachment(&self, payload: impl EnvelopeEncodable, vendor: &str, conforms_to: Option<&str>) -> Self {
        self.add_assertion_envelope(
            Assertion::new_attachment(payload, vendor, conforms_to)
        ).unwrap()
    }
}

impl Envelope {
    /// Returns the payload of the given attachment envelope.
    pub fn attachment_payload(&self) -> anyhow::Result<Self> {
        if let EnvelopeCase::Assertion(assertion) = self.case() {
            Ok(assertion.attachment_payload()?)
        } else {
            bail!(EnvelopeError::InvalidAttachment)
        }
    }

    /// Returns the `vendor` of the given attachment envelope.
    pub fn attachment_vendor(&self) -> anyhow::Result<String> {
        if let EnvelopeCase::Assertion(assertion) = self.case() {
            Ok(assertion.attachment_vendor()?)
        } else {
            bail!(EnvelopeError::InvalidAttachment);
        }
    }

    /// Returns the `conformsTo` of the given attachment envelope.
    pub fn attachment_conforms_to(&self) -> anyhow::Result<Option<String>> {
        if let EnvelopeCase::Assertion(assertion) = self.case() {
            Ok(assertion.attachment_conforms_to()?)
        } else {
            bail!(EnvelopeError::InvalidAttachment);
        }
    }

    /// Searches the envelope's attachments for any that match the given
    /// `vendor` and `conformsTo`.
    ///
    /// If `vendor` is `None`, matches any vendor. If `conformsTo` is `None`,
    /// matches any `conformsTo`. If both are `None`, matches any attachment. On
    /// success, returns a vector of matching attachments. Returns an error if
    /// any of the attachments are invalid.
    pub fn attachments_with_vendor_and_conforms_to(&self, vendor: Option<&str>, conforms_to: Option<&str>)
    -> anyhow::Result<Vec<Self>>
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
        anyhow::Result::Ok(matching_assertions)
    }

    pub fn attachments(&self) -> anyhow::Result<Vec<Self>> {
        self.attachments_with_vendor_and_conforms_to(None::<&str>, None::<&str>)
    }

    /// Validates the given attachment envelope.
    ///
    /// Ensures:
    /// - The attachment envelope is a valid assertion envelope.
    /// - The attachment envelope's predicate is `known_values::ATTACHMENT`.
    /// - The attachment envelope's object is an envelope.
    /// - The attachment envelope's object has a `'vendor': String` assertion.
    /// - The attachment envelope's object has an optional `'conformsTo': String` assertion.
    pub fn validate_attachment(&self) -> anyhow::Result<()> {
        if let EnvelopeCase::Assertion(assertion) = self.case() {
            assertion.validate_attachment()?;
            Ok(())
        } else {
            bail!(EnvelopeError::InvalidAttachment);
        }
    }

    /// Searches the envelope's attachments for any that match the given
    /// `vendor` and `conformsTo`.
    ///
    /// If `vendor` is `None`, matches any vendor. If `conformsTo` is `None`,
    /// matches any `conformsTo`. If both are `None`, matches any attachment. On
    /// success, returns the first matching attachment. Returns an error if
    /// more than one attachment matches, or if any of the attachments are
    /// invalid.
    pub fn attachment_with_vendor_and_conforms_to(&self, vendor: Option<&str>, conforms_to: Option<&str>)
    -> anyhow::Result<Self>
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
