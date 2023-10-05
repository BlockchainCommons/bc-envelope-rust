use std::rc::Rc;

use crate::{Assertion, IntoEnvelope, extension::known_values, Envelope, EnvelopeError, prelude::KnownValue};

impl Assertion {
    /// Creates an attachment assertion. See:
    /// [BCR-2023-006](https://github.com/BlockchainCommons/Research/blob/master/papers/bcr-2023-006-envelope-attachment.md)
    pub fn new_attachment<A>(attachment: A, vendor: &str, conforms_to: Option<&str>) -> Self
    where
        A: IntoEnvelope,
    {
        let conforms_to: Option<String> = conforms_to.map(|c| c.to_string());
        Self::new(
            known_values::ATTACHMENT,
            attachment.into_envelope()
                .wrap_envelope()
                .add_assertion(known_values::VENDOR, vendor.to_string())
                .add_optional_assertion(known_values::CONFORMS_TO, conforms_to)
        )
    }
}

impl Envelope {
    /// Returns a new envelope with an added `'attachment': Envelope` assertion.
    ///
    /// The payload envelope has a `'vendor': String` assertion and an optional
    /// `'conformsTo': String` assertion.
    pub fn add_attachment<A>(self: Rc<Self>, attachment: A, vendor: &str, conforms_to: Option<&str>) -> Rc<Self>
    where
        A: IntoEnvelope,
    {
        self.add_assertion_envelope(
            Assertion::new_attachment(attachment, vendor, conforms_to).into_envelope()
        ).unwrap()
    }
}

impl Envelope {
    /// Returns the payload of the given attachment envelope.
    pub fn attachment_payload(self: Rc<Self>) -> Result<Rc<Self>, EnvelopeError> {
        self.object_or_error()?.unwrap_envelope()
    }

    /// Returns the `vendor` of the given attachment envelope.
    pub fn attachment_vendor(self: Rc<Self>) -> anyhow::Result<String> {
        Ok(self.object_or_error()?.extract_object_for_predicate::<String, KnownValue>(known_values::VENDOR)?.as_ref().clone())
    }

    /// Returns the `conformsTo` of the given attachment envelope.
    pub fn attachment_conforms_to(self: Rc<Self>) -> anyhow::Result<Option<String>> {
        Ok(self.object_or_error()?.extract_optional_object_for_predicate::<String, KnownValue>(known_values::CONFORMS_TO)?.map(|s| s.as_ref().clone()))
    }

    /// Searches the envelope's attachments for any that match the given
    /// `vendor` and `conformsTo`.
    ///
    /// If `vendor` is `None`, matches any vendor. If `conformsTo` is `None`,
    /// matches any `conformsTo`. If both are `None`, matches any attachment. On
    /// success, returns a vector of matching attachments. Returns an error if
    /// any of the attachments are invalid.
    pub fn attachments_with_vendor_and_conforms_to(self: Rc<Self>, vendor: Option<&str>, conforms_to: Option<&str>)
    -> anyhow::Result<Vec<Rc<Self>>>
    {
        let assertions = self.assertions_with_predicate(known_values::ATTACHMENT);
        for assertion in &assertions {
            Self::validate_attachment(assertion.clone())?;
        }
        let matching_assertions: Vec<_> = assertions
            .into_iter()
            .filter(|assertion| {
                if let Some(vendor) = vendor {
                    if let Ok(v) = assertion.clone().attachment_vendor() {
                        if v != vendor {
                            return false;
                        }
                    }
                }
                if let Some(conforms_to) = conforms_to {
                    if let Ok(c) = assertion.clone().attachment_conforms_to() {
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

    pub fn attachments(self: Rc<Self>) -> anyhow::Result<Vec<Rc<Self>>> {
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
    pub fn validate_attachment(attachment: Rc<Self>) -> anyhow::Result<()> {
        let payload = attachment.clone().attachment_payload()?;
        let vendor = attachment.clone().attachment_vendor()?;
        let conforms_to: Option<String> = attachment.clone().attachment_conforms_to()?;
        let assertion = Assertion::new_attachment(payload, vendor.as_str(), conforms_to.as_deref());
        let e = assertion.into_envelope();
        if !e.is_equivalent_to(attachment) {
            anyhow::bail!(EnvelopeError::InvalidAttachment);
        }
        Ok(())
    }

    /// Searches the envelope's attachments for any that match the given
    /// `vendor` and `conformsTo`.
    ///
    /// If `vendor` is `None`, matches any vendor. If `conformsTo` is `None`,
    /// matches any `conformsTo`. If both are `None`, matches any attachment. On
    /// success, returns the first matching attachment. Returns an error if
    /// more than one attachment matches, or if any of the attachments are
    /// invalid.
    pub fn attachment_with_vendor_and_conforms_to(self: Rc<Self>, vendor: Option<&str>, conforms_to: Option<&str>)
    -> anyhow::Result<Rc<Self>>
    {
        let attachments = self.attachments_with_vendor_and_conforms_to(vendor, conforms_to)?;
        if attachments.is_empty() {
            anyhow::bail!(EnvelopeError::NonexistentAttachment);
        }
        if attachments.len() > 1 {
            anyhow::bail!(EnvelopeError::AmbiguousAttachment);
        }
        Ok(attachments.first().unwrap().clone())
    }
}
