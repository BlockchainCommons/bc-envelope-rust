use std::error::Error;

use bc_components::{tags, ARID, URI, UUID, Digest};
use dcbor::prelude::*;

use crate::{FormatContext, string_utils::StringUtils};
#[cfg(feature = "known_value")]
use crate::extension::known_values::KnownValuesStore;

#[cfg(feature = "expression")]
use crate::{Envelope, extension::expression::{Function, FunctionsStore, Parameter, ParametersStore}};

pub trait EnvelopeSummary {
    fn envelope_summary(&self, max_length: usize, context: &FormatContext) -> Result<String, Box<dyn Error>>;
}

impl EnvelopeSummary for CBOR {
    fn envelope_summary(&self, max_length: usize, context: &FormatContext) -> Result<String, Box<dyn Error>> {
        match self.case() {
            CBORCase::Unsigned(n) => Ok(n.to_string()),
            CBORCase::Negative(n) => Ok((-1 - (*n as i128)).to_string()),
            CBORCase::ByteString(data) => Ok(format!("Bytes({})", data.len())),
            CBORCase::Text(string) => {
                let string = if string.len() > max_length {
                    format!("{}…", string.chars().take(max_length).collect::<String>())
                } else {
                    string.clone()
                };
                Ok(string.replace('\n', "\\n").flanked_by("\"", "\""))
            }
            CBORCase::Array(elements) => {
                Ok(elements
                    .iter()
                    .map(|element| element.envelope_summary(max_length, context))
                    .collect::<Result<Vec<String>, Box<dyn Error>>>()?
                    .join(", ")
                    .flanked_by("[", "]")
                )
            },
            CBORCase::Map(_) => Ok("Map".to_string()),
            CBORCase::Tagged(tag, untagged_cbor) => {
                match tag.value() {
                    tags::ENVELOPE_VALUE => Ok("Envelope".to_string()),
                    #[cfg(feature = "known_value")]
                    tags::KNOWN_VALUE_VALUE => {
                        if let CBORCase::Unsigned(raw_value) = untagged_cbor.case() {
                            Ok(KnownValuesStore::known_value_for_raw_value(*raw_value, Some(context.known_values()))
                                .to_string().flanked_by("'", "'",))
                        } else {
                            Ok("<not a known value>".to_string())
                        }
                    },
                    tags::SIGNATURE_VALUE => Ok("Signature".to_string()),
                    tags::NONCE_VALUE => Ok("Nonce".to_string()),
                    tags::SALT_VALUE => Ok("Salt".to_string()),
                    tags::SEALED_MESSAGE_VALUE => Ok("SealedMessage".to_string()),
                    tags::SSKR_SHARE_VALUE => Ok("SSKRShare".to_string()),
                    tags::PUBLIC_KEY_BASE_VALUE => Ok("PublicKeyBase".to_string()),
                    tags::DATE_VALUE => {
                        let date = dcbor::Date::from_untagged_cbor(untagged_cbor)?;
                        let s = date.to_string();
                        if s.len() == 20 && s.ends_with("T00:00:00Z") {
                            Ok(s.chars().take(10).collect::<String>())
                        } else {
                            Ok(s)
                        }
                    },
                    tags::ARID_VALUE => {
                        Ok(ARID::from_untagged_cbor(untagged_cbor)?.short_description().flanked_by("ARID(", ")"))
                    },
                    tags::URI_VALUE => {
                        Ok(URI::from_untagged_cbor(untagged_cbor)?.to_string().flanked_by("URI(", ")"))
                    },
                    tags::UUID_VALUE => {
                        Ok(UUID::from_untagged_cbor(untagged_cbor)?.to_string().flanked_by("UUID(", ")"))
                    },
                    tags::DIGEST_VALUE => {
                        Ok(Digest::from_untagged_cbor(untagged_cbor)?.short_description().flanked_by("Digest(", ")"))
                    },
                    #[cfg(feature = "expression")]
                    tags::FUNCTION_VALUE => {
                        let f = Function::from_untagged_cbor(untagged_cbor)?;
                        Ok(FunctionsStore::name_for_function(&f, Some(context.functions())).flanked_by("«", "»"))
                    },
                    #[cfg(feature = "expression")]
                    tags::PARAMETER_VALUE => {
                        let p = Parameter::from_untagged_cbor(untagged_cbor)?;
                        Ok(ParametersStore::name_for_parameter(&p, Some(context.parameters())).flanked_by("❰", "❱"))
                    },
                    #[cfg(feature = "expression")]
                    tags::REQUEST_VALUE => {
                        Ok(Envelope::new(untagged_cbor).format_opt(Some(context)).flanked_by("request(", ")"))
                    },
                    #[cfg(feature = "expression")]
                    tags::RESPONSE_VALUE => {
                        Ok(Envelope::new(untagged_cbor).format_opt(Some(context)).flanked_by("response(", ")"))
                    },
                    _ => {
                        let name = context.name_for_tag(tag);
                        Ok(format!("{}({})", name, untagged_cbor.envelope_summary(max_length, context)?))
                    }
                }
            },
            CBORCase::Simple(v) => Ok(v.to_string()),
        }
    }
}
