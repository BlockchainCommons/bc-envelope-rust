use anyhow::Result;
use bc_components::{tags, Digest, ARID, URI, UUID, XID};
use dcbor::prelude::*;

use crate::{FormatContext, string_utils::StringUtils};
#[cfg(feature = "known_value")]
use crate::extension::known_values::KnownValuesStore;

#[cfg(feature = "expression")]
use crate::{Envelope, extension::expressions::{Function, FunctionsStore, Parameter, ParametersStore}};

pub trait EnvelopeSummary {
    fn envelope_summary(&self, max_length: usize, context: &FormatContext) -> Result<String>;
}

impl EnvelopeSummary for CBOR {
    fn envelope_summary(&self, max_length: usize, context: &FormatContext) -> Result<String> {
        match self.as_case() {
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
                    .collect::<Result<Vec<String>>>()?
                    .join(", ")
                    .flanked_by("[", "]")
                )
            },
            CBORCase::Map(_) => Ok("Map".to_string()),
            CBORCase::Tagged(tag, untagged_cbor) => {
                let untagged_cbor = untagged_cbor.clone();
                match tag.value() {
                    tags::TAG_ENVELOPE => Ok("Envelope".to_string()),
                    #[cfg(feature = "known_value")]
                    tags::TAG_KNOWN_VALUE => {
                        if let CBORCase::Unsigned(raw_value) = untagged_cbor.as_case() {
                            Ok(KnownValuesStore::known_value_for_raw_value(*raw_value, Some(context.known_values()))
                                .to_string().flanked_by("'", "'",))
                        } else {
                            Ok("<not a known value>".to_string())
                        }
                    },
                    tags::TAG_SIGNATURE => Ok("Signature".to_string()),
                    tags::TAG_NONCE => Ok("Nonce".to_string()),
                    tags::TAG_SALT => Ok("Salt".to_string()),
                    tags::TAG_SEALED_MESSAGE => Ok("SealedMessage".to_string()),
                    tags::TAG_SSKR_SHARE => Ok("SSKRShare".to_string()),
                    tags::TAG_PUBLIC_KEY_BASE => Ok("PublicKeyBase".to_string()),
                    tags::TAG_SSH_TEXT_PRIVATE_KEY => Ok("SSHPrivateKey".to_string()),
                    tags::TAG_SSH_TEXT_PUBLIC_KEY => Ok("SSHPublicKey".to_string()),
                    tags::TAG_SSH_TEXT_SIGNATURE => Ok("SSHSignature".to_string()),
                    tags::TAG_SSH_TEXT_CERTIFICATE => Ok("SSHCertificate".to_string()),
                    tags::TAG_DATE => {
                        let date = dcbor::Date::from_untagged_cbor(untagged_cbor)?;
                        let s = date.to_string();
                        if s.len() == 20 && s.ends_with("T00:00:00Z") {
                            Ok(s.chars().take(10).collect::<String>())
                        } else {
                            Ok(s)
                        }
                    },
                    tags::TAG_ARID => {
                        Ok(ARID::from_untagged_cbor(untagged_cbor)?.short_description().flanked_by("ARID(", ")"))
                    },
                    tags::TAG_URI => {
                        Ok(URI::from_untagged_cbor(untagged_cbor)?.to_string().flanked_by("URI(", ")"))
                    },
                    tags::TAG_UUID => {
                        Ok(UUID::from_untagged_cbor(untagged_cbor)?.to_string().flanked_by("UUID(", ")"))
                    },
                    tags::TAG_DIGEST => {
                        Ok(Digest::from_untagged_cbor(untagged_cbor)?.short_description().flanked_by("Digest(", ")"))
                    },
                    tags::TAG_XID => {
                        Ok(XID::from_untagged_cbor(untagged_cbor)?.short_description().flanked_by("XID(", ")"))
                    },
                    #[cfg(feature = "expression")]
                    tags::TAG_FUNCTION => {
                        let f = Function::from_untagged_cbor(untagged_cbor)?;
                        Ok(FunctionsStore::name_for_function(&f, Some(context.functions())).flanked_by("«", "»"))
                    },
                    #[cfg(feature = "expression")]
                    tags::TAG_PARAMETER => {
                        let p = Parameter::from_untagged_cbor(untagged_cbor)?;
                        Ok(ParametersStore::name_for_parameter(&p, Some(context.parameters())).flanked_by("❰", "❱"))
                    },
                    #[cfg(feature = "expression")]
                    tags::TAG_REQUEST => {
                        Ok(Envelope::new(untagged_cbor).format_opt(Some(context)).flanked_by("request(", ")"))
                    },
                    #[cfg(feature = "expression")]
                    tags::TAG_RESPONSE => {
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
