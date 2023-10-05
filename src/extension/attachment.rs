use std::rc::Rc;

use crate::{Assertion, IntoEnvelope, extension::known_values, Envelope, EnvelopeError, prelude::KnownValue};

impl Assertion {
    /// Creates an attachment assertion. See: [BCR-2023-006](https://github.com/BlockchainCommons/Research/blob/master/papers/bcr-2023-006-envelope-attachment.md)
    pub fn new_attachment<A, V, C>(attachment: A, vendor: V, conforms_to: Option<C>) -> Self
    where
        A: IntoEnvelope,
        V: IntoEnvelope,
        C: IntoEnvelope,
    {
        Self::new(
            known_values::ATTACHMENT,
            attachment.into_envelope()
                .wrap_envelope()
                .add_assertion(known_values::VENDOR, vendor)
                .add_optional_assertion(known_values::CONFORMS_TO, conforms_to)
        )
    }
}

impl Envelope {
    pub fn add_attachment<A, V, C>(self: Rc<Self>, attachment: A, vendor: V, conforms_to: Option<C>) -> Result<Rc<Self>, EnvelopeError>
    where
        A: IntoEnvelope,
        V: IntoEnvelope,
        C: IntoEnvelope,
    {
        self.add_assertion_envelope(
            Assertion::new_attachment(attachment, vendor, conforms_to).into_envelope()
        )
    }
}

impl Envelope {
    pub fn attachment_payload(self: Rc<Self>) -> Result<Rc<Self>, EnvelopeError> {
        self.object_or_error()?.unwrap_envelope()
    }

    pub fn attachment_vendor(self: Rc<Self>) -> anyhow::Result<String> {
        Ok(self.object_or_error()?.extract_object_for_predicate::<String, KnownValue>(known_values::VENDOR)?.as_ref().clone())
    }

    pub fn attachment_conforms_to(self: Rc<Self>) -> anyhow::Result<Option<String>> {
        Ok(self.object_or_error()?.extract_optional_object_for_predicate::<String, KnownValue>(known_values::CONFORMS_TO)?.map(|s| s.as_ref().clone()))
    }

    pub fn attachments_with_vendor_and_conforms_to<V, C>(self: Rc<Self>, vendor: Option<V>, conforms_to: Option<C>)
    -> anyhow::Result<Vec<Rc<Self>>>
    where
        V: AsRef<String>,
        C: AsRef<String>,
    {
        let vendor = vendor.map(|v| v.as_ref().clone());
        let conforms_to = conforms_to.map(|c| c.as_ref().clone());

        let assertions = self.assertions_with_predicate(known_values::ATTACHMENT);
        for assertion in &assertions {
            Self::validate_attachment(assertion.clone())?;
        }
        let matching_assertions: Vec<_> = assertions
            .into_iter()
            .filter(|assertion| {
                if let Some(vendor) = &vendor {
                    if let Ok(v) = assertion.clone().attachment_vendor() {
                        if v != *vendor {
                            return false;
                        }
                    }
                }
                if let Some(conforms_to) = &conforms_to {
                    if let Ok(c) = assertion.clone().attachment_conforms_to() {
                        if let Some(c) = c {
                            if c != *conforms_to {
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

    pub fn validate_attachment(attachment: Rc<Self>) -> anyhow::Result<()> {
        let payload = attachment.clone().attachment_payload()?;
        let vendor = attachment.clone().attachment_vendor()?;
        let conforms_to = attachment.clone().attachment_conforms_to()?;
        let assertion = Assertion::new_attachment(payload, vendor, conforms_to);
        let e = assertion.into_envelope();
        if !e.is_equivalent_to(attachment) {
            anyhow::bail!(EnvelopeError::InvalidAttachment);
        }
        Ok(())
    }

    pub fn attachment_with_vendor_and_conforming_to<V, C>(self: Rc<Self>, vendor: Option<V>, conforms_to: Option<C>)
    -> anyhow::Result<Rc<Self>>
    where
        V: AsRef<String>,
        C: AsRef<String>,
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

// ```swift
// public extension Assertion {
//     /// Creates an attachment assertion. See: [BCR-2023-006](https://github.com/BlockchainCommons/Research/blob/master/papers/bcr-2023-006-envelope-attachment.md)
//     init(attachment: Envelope, vendor: String, conformsTo: String? = nil) {
//         self.init(
//             predicate: KnownValue.attachment,
//             object: attachment
//                 .wrap()
//                 .addAssertion(.vendor, vendor)
//                 .addAssertion(.conformsTo, conformsTo)
//         )
//     }
// }

// public extension Envelope {
//     func addAttachment(_ attachment: Envelope, vendor: String, conformsTo: String? = nil, salted: Bool = false) -> Envelope {
//         addAssertion(
//             Assertion(attachment: attachment, vendor: vendor, conformsTo: conformsTo)
//         )
//     }
// }

// public extension Envelope {
//     var attachmentPayload: Envelope {
//         get throws {
//             try object.unwrap()
//         }
//     }

//     var attachmentVendor: String {
//         get throws {
//             try object.extractObject(String.self, forPredicate: .vendor)
//         }
//     }

//     var attachmentConformsTo: String? {
//         get throws {
//             try object.extractOptionalObject(String.self, forPredicate: .conformsTo)
//         }
//     }

//     func attachments(withVendor vendor: String? = nil, conformingTo conformsTo: String? = nil) throws -> [Envelope] {
//         try assertions(withPredicate: .attachment).filter { envelope in
//             try validateAttachment(envelope)
//             if let vendor {
//                 guard try envelope.attachmentVendor == vendor else {
//                     return false
//                 }
//             }
//             if let conformsTo {
//                 guard try envelope.attachmentConformsTo == conformsTo else {
//                     return false
//                 }
//             }
//             return true
//         }
//     }

//     func validateAttachment(_ envelope: Envelope) throws {
//         let payload = try envelope.attachmentPayload
//         let vendor = try envelope.attachmentVendor
//         let conformsTo = try envelope.attachmentConformsTo
//         let assertion = Assertion(attachment: payload, vendor: vendor, conformsTo: conformsTo)
//         let e = Envelope(assertion)
//         guard e.isEquivalent(to: envelope) else {
//             throw EnvelopeError.invalidAttachment
//         }
//     }

//     func attachment(withVendor vendor: String? = nil, conformingTo conformsTo: String? = nil) throws -> Envelope {
//         let attachments = try attachments(withVendor: vendor, conformingTo: conformsTo)
//         guard !attachments.isEmpty else {
//             throw EnvelopeError.nonexistentAttachment
//         }
//         guard attachments.count == 1 else {
//             throw EnvelopeError.ambiguousAttachment
//         }
//         return attachments.first!
//     }
// }
// ```
