use std::error::Error;

mod format_context;
pub use format_context::{FormatContext, GLOBAL_FORMAT_CONTEXT};

mod tree_format;

use bc_components::{Digest, ARID, URI, UUID};
use dcbor::preamble::*;
use crate::{Envelope, Assertion, string_utils::StringUtils, expressions::{Function, FunctionsStore, ParametersStore, Parameter}, known_values::{KnownValuesStore, KnownValue, self}};
use bc_components::tags;

/// Support for the various text output formats for ``Envelope``.
impl Envelope {
    /// Returns the envelope notation for this envelope.
    pub fn format_opt(&self, context: Option<&FormatContext>) -> String {
        self.format_item(context.unwrap_or(&FormatContext::default())).format().trim().to_string()
    }

    /// Returns the envelope notation for this envelope.
    pub fn format(&self) -> String {
        self.format_opt(None)
    }

    /// Returns the CBOR diagnostic notation for this envelope.
    ///
    /// See [RFC-8949 §8](https://www.rfc-editor.org/rfc/rfc8949.html#name-diagnostic-notation)
    /// for information on CBOR diagnostic notation.
    pub fn diagnostic_opt(&self, annotate: bool, context: Option<&FormatContext>) -> String {
        self.tagged_cbor().diagnostic_opt(annotate, Some(context.unwrap_or(&FormatContext::default()).tags()))
    }

    /// Returns the CBOR diagnostic notation for this envelope.
    ///
    /// See [RFC-8949 §8](https://www.rfc-editor.org/rfc/rfc8949.html#name-diagnostic-notation)
    /// for information on CBOR diagnostic notation.
    pub fn diagnostic(&self) -> String {
        self.diagnostic_opt(false, None)
    }

    /// Returns the CBOR hex dump of this envelope.
    ///
    /// See [RFC-8949](https://www.rfc-editor.org/rfc/rfc8949.html) for information on
    /// the CBOR binary format.
    pub fn hex_opt(&self, annotate: bool, context: Option<&FormatContext>) -> String {
        self.cbor().hex_opt(annotate, Some(context.unwrap_or(&FormatContext::default()).tags()))
    }

    /// Returns the CBOR hex dump of this envelope.
    ///
    /// See [RFC-8949](https://www.rfc-editor.org/rfc/rfc8949.html) for information on
    /// the CBOR binary format.
    pub fn hex(&self) -> String {
        self.hex_opt(false, None)
    }
}

/// Implementers of this trait define how to be formatted in when output in envelope notation.
pub trait EnvelopeFormat {
    fn format_item(&self, context: &FormatContext) -> EnvelopeFormatItem;
}

/// This type is returned by implementers of the [`EnvelopeFormat`] trait.
#[derive(Debug, Clone, Eq)]
pub enum EnvelopeFormatItem {
    Begin(String),
    End(String),
    Item(String),
    Separator,
    List(Vec<EnvelopeFormatItem>),
}

impl EnvelopeFormatItem {
    fn flatten(&self) -> Vec<EnvelopeFormatItem> {
        match self {
            EnvelopeFormatItem::List(items) => items.iter().flat_map(|i| i.flatten()).collect(),
            _ => vec![self.clone()],
        }
    }

    fn nicen(&self, items: &[EnvelopeFormatItem]) -> Vec<EnvelopeFormatItem> {
        let mut input = items.to_vec();
        let mut result: Vec<EnvelopeFormatItem> = vec![];

        while !input.is_empty() {
            let current = input.remove(0);
            if input.is_empty() {
                result.push(current);
                break;
            }
            if let EnvelopeFormatItem::End(end_string) = current.clone() {
                if let EnvelopeFormatItem::Begin(begin_string) = input[0].clone() {
                    result.push(EnvelopeFormatItem::End(format!("{} {}", end_string, begin_string)));
                    result.push(EnvelopeFormatItem::Begin("".to_string()));
                    input.remove(0);
                } else {
                    result.push(current);
                }
            } else {
                result.push(current);
            }
        }

        result
    }

    fn indent(&self, level: usize) -> String {
        " ".repeat(level * 4)
    }

    fn add_space_at_end_if_needed(&self, s: &str) -> String {
        if s.is_empty() {
            " ".to_string()
        } else if s.ends_with(' ') {
            s.to_string()
        } else {
            s.to_string() + " "
        }
    }

    fn format(&self) -> String {
        let mut lines: Vec<String> = vec![];
        let mut level = 0;
        let mut current_line = "".to_string();
        let items = self.nicen(&self.flatten());
        for item in items {
            match item {
                EnvelopeFormatItem::Begin(string) => {
                    if !string.is_empty() {
                        let c = if current_line.is_empty() {
                            string
                        } else {
                            self.add_space_at_end_if_needed(&current_line) + &string
                        };
                        lines.push(self.indent(level) + &c + "\n");
                    }
                    level += 1;
                    current_line = "".to_string();
                }
                EnvelopeFormatItem::End(string) => {
                    if !current_line.is_empty() {
                        lines.push(self.indent(level) + &current_line + "\n");
                        current_line = "".to_string();
                    }
                    level -= 1;
                    lines.push(self.indent(level) + &string + "\n");
                }
                EnvelopeFormatItem::Item(string) => {
                    current_line += &string;
                }
                EnvelopeFormatItem::Separator => {
                    if !current_line.is_empty() {
                        lines.push(self.indent(level) + &current_line + "\n");
                        current_line = "".to_string();
                    }
                }
                EnvelopeFormatItem::List(_) => {
                    lines.push("<list>".to_string());
                }
            }
        }
        if !current_line.is_empty() {
            lines.push(current_line);
        }
        lines.join("")
    }
}

impl From<&str> for EnvelopeFormatItem {
    fn from(s: &str) -> Self {
        Self::Item(s.to_string())
    }
}

impl std::fmt::Display for EnvelopeFormatItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EnvelopeFormatItem::Begin(s) => write!(f, ".begin({})", s),
            EnvelopeFormatItem::End(s) => write!(f, ".end({})", s),
            EnvelopeFormatItem::Item(s) => write!(f, ".item({})", s),
            EnvelopeFormatItem::Separator => write!(f, ".separator"),
            EnvelopeFormatItem::List(items) => write!(f, ".list({:?})", items),
        }
    }
}

impl PartialEq for EnvelopeFormatItem {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (EnvelopeFormatItem::Begin(s1), EnvelopeFormatItem::Begin(s2)) => s1 == s2,
            (EnvelopeFormatItem::End(s1), EnvelopeFormatItem::End(s2)) => s1 == s2,
            (EnvelopeFormatItem::Item(s1), EnvelopeFormatItem::Item(s2)) => s1 == s2,
            (EnvelopeFormatItem::Separator, EnvelopeFormatItem::Separator) => true,
            (EnvelopeFormatItem::List(items1), EnvelopeFormatItem::List(items2)) => items1 == items2,
            _ => false,
        }
    }
}

impl EnvelopeFormatItem {
    fn index(&self) -> u32 {
        match self {
            EnvelopeFormatItem::Begin(_) => 1,
            EnvelopeFormatItem::End(_) => 2,
            EnvelopeFormatItem::Item(_) => 3,
            EnvelopeFormatItem::Separator => 4,
            EnvelopeFormatItem::List(_) => 5,
        }
    }
}

impl PartialOrd for EnvelopeFormatItem {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        let l_index = self.index();
        let r_index = other.index();
        match l_index.cmp(&r_index) {
            std::cmp::Ordering::Less => return Some(std::cmp::Ordering::Less),
            std::cmp::Ordering::Greater => return Some(std::cmp::Ordering::Greater),
            _ => {}
        }
        match (self, other) {
            (EnvelopeFormatItem::Begin(l), EnvelopeFormatItem::Begin(r)) => l.partial_cmp(r),
            (EnvelopeFormatItem::End(l), EnvelopeFormatItem::End(r)) => l.partial_cmp(r),
            (EnvelopeFormatItem::Item(l), EnvelopeFormatItem::Item(r)) => l.partial_cmp(r),
            (EnvelopeFormatItem::Separator, EnvelopeFormatItem::Separator) => Some(std::cmp::Ordering::Equal),
            (EnvelopeFormatItem::List(l), EnvelopeFormatItem::List(r)) => l.partial_cmp(r),
            _ => None,
        }
    }
}


impl Ord for EnvelopeFormatItem {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl EnvelopeFormat for Digest {
    fn format_item(&self, _context: &FormatContext) -> EnvelopeFormatItem {
        EnvelopeFormatItem::Item(hex::encode(self.data()))
    }
}

impl EnvelopeFormat for ARID {
    fn format_item(&self, _context: &FormatContext) -> EnvelopeFormatItem {
        EnvelopeFormatItem::Item(hex::encode(self.data()))
    }
}

trait EnvelopeSummary {
    fn envelope_summary(&self, max_length: usize, context: &FormatContext) -> Result<String, Box<dyn Error>>;
}

impl EnvelopeSummary for CBOR {
    fn envelope_summary(&self, max_length: usize, context: &FormatContext) -> Result<String, Box<dyn Error>> {
        match self {
            CBOR::Unsigned(n) => Ok(n.to_string()),
            CBOR::Negative(n) => Ok(n.to_string()),
            CBOR::ByteString(data) => Ok(format!("Bytes({})", data.len())),
            CBOR::Text(string) => {
                let string = if string.len() > max_length {
                    format!("{}…", string.chars().take(max_length).collect::<String>())
                } else {
                    string.clone()
                };
                Ok(string.replace('\n', "\\n").flanked_by("\"", "\""))
            }
            CBOR::Array(elements) => {
                Ok(elements
                    .iter()
                    .map(|element| element.envelope_summary(max_length, context))
                    .collect::<Result<Vec<String>, Box<dyn Error>>>()?
                    .join(", ")
                    .flanked_by("[", "]")
                )
            },
            CBOR::Map(_) => Ok("Map".to_string()),
            CBOR::Tagged(tag, untagged_cbor) => {
                match tag.value() {
                    tags::ENVELOPE_VALUE => Ok("Envelope".to_string()),
                    tags::KNOWN_VALUE_VALUE => {
                        if let CBOR::Unsigned(raw_value) = &**untagged_cbor {
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
                    tags::PUBLIC_KEYBASE_VALUE => Ok("PublicKeyBase".to_string()),
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
                    tags::FUNCTION_VALUE => {
                        let f = Function::from_untagged_cbor(untagged_cbor)?;
                        Ok(FunctionsStore::name_for_function(&f, Some(context.functions())).flanked_by("«", "»"))
                    },
                    tags::PARAMETER_VALUE => {
                        let p = Parameter::from_untagged_cbor(untagged_cbor)?;
                        Ok(ParametersStore::name_for_parameter(&p, Some(context.parameters())).flanked_by("❰", "❱"))
                    },
                    tags::REQUEST_VALUE => {
                        Ok(Envelope::new(untagged_cbor).format_opt(Some(context)).flanked_by("request(", ")"))
                    },
                    tags::RESPONSE_VALUE => {
                        Ok(Envelope::new(untagged_cbor).format_opt(Some(context)).flanked_by("response(", ")"))
                    },
                    _ => {
                        let name = context.name_for_tag(tag);
                        Ok(format!("{}({})", name, untagged_cbor.envelope_summary(max_length, context)?))
                    }
                }
            },
            CBOR::Simple(v) => Ok(v.to_string()),
        }
    }
}

impl EnvelopeFormat for CBOR {
    fn format_item(&self, context: &FormatContext) -> EnvelopeFormatItem {
        match self {
            CBOR::Tagged(tag, cbor) if tag == &Envelope::CBOR_TAG => {
                Envelope::from_untagged_cbor(cbor)
                    .map(|envelope| envelope.format_item(context))
                    .unwrap_or_else(|_| "<error>".into())
            }
            _ => EnvelopeFormatItem::Item(
                self.envelope_summary(usize::MAX, context)
                    .unwrap_or_else(|_| "<error>".into())
            ),
        }
    }
}

impl EnvelopeFormat for Envelope {
    fn format_item(&self, context: &FormatContext) -> EnvelopeFormatItem {
        match self {
            Envelope::Leaf { cbor, .. } => cbor.format_item(context),
            Envelope::KnownValue { value, .. } => value.format_item(context),
            Envelope::Wrapped { envelope, .. } => EnvelopeFormatItem::List(vec![
                EnvelopeFormatItem::Begin("{".to_string()),
                envelope.format_item(context),
                EnvelopeFormatItem::End("}".to_string()),
            ]),
            Envelope::Assertion(assertion) => assertion.format_item(context),
            Envelope::Encrypted(_) => EnvelopeFormatItem::Item("ENCRYPTED".to_string()),
            Envelope::Compressed(_) => EnvelopeFormatItem::Item("COMPRESSED".to_string()),
            Envelope::Node { subject, assertions, .. } => {
                let mut items: Vec<EnvelopeFormatItem> = Vec::new();

                let subject_item = subject.format_item(context);
                let mut elided_count = 0;
                let mut encrypted_count = 0;
                let mut compressed_count = 0;
                let mut type_assertion_items: Vec<Vec<EnvelopeFormatItem>> = Vec::new();
                let mut assertion_items: Vec<Vec<EnvelopeFormatItem>> = Vec::new();

                for assertion in assertions {
                    match &**assertion {
                        Envelope::Elided(_) => {
                            elided_count += 1;
                        },
                        Envelope::Encrypted(_) => {
                            encrypted_count += 1;
                        },
                        Envelope::Compressed(_) => {
                            compressed_count += 1;
                        },
                        _ => {
                            let mut is_type_assertion = false;
                            if let Some(predicate) = assertion.clone().predicate() {
                                if let Some(known_value) = predicate.subject().known_value() {
                                    if *known_value == known_values::IS_A {
                                        is_type_assertion = true;
                                    }
                                }
                            }
                            let item = vec![assertion.format_item(context)];
                            if is_type_assertion {
                                type_assertion_items.push(item);
                            } else {
                                assertion_items.push(item);
                            }
                        },
                    }
                }
                type_assertion_items.sort();
                assertion_items.sort();
                assertion_items.splice(0..0, type_assertion_items);
                if compressed_count > 1 {
                    assertion_items.push(vec![EnvelopeFormatItem::Item(format!("COMPRESSED ({})", compressed_count))]);
                } else if compressed_count > 0 {
                    assertion_items.push(vec![EnvelopeFormatItem::Item("COMPRESSED".to_string())]);
                }
                if elided_count > 1 {
                    assertion_items.push(vec![EnvelopeFormatItem::Item(format!("ELIDED ({})", elided_count))]);
                } else if elided_count > 0 {
                    assertion_items.push(vec![EnvelopeFormatItem::Item("ELIDED".to_string())]);
                }
                if encrypted_count > 1 {
                    assertion_items.push(vec![EnvelopeFormatItem::Item(format!("ENCRYPTED ({})", encrypted_count))]);
                } else if encrypted_count > 0 {
                    assertion_items.push(vec![EnvelopeFormatItem::Item("ENCRYPTED".to_string())]);
                }
                let joined_assertions_items: Vec<Vec<EnvelopeFormatItem>> =
                    itertools::intersperse_with(assertion_items, || vec![EnvelopeFormatItem::Separator]).collect();

                let needs_braces = subject.is_subject_assertion();

                if needs_braces {
                    items.push(EnvelopeFormatItem::Begin("{".to_string()));
                }
                items.push(subject_item);
                if needs_braces {
                    items.push(EnvelopeFormatItem::End("}".to_string()));
                }
                items.push(EnvelopeFormatItem::Begin("[".to_string()));
                items.extend(joined_assertions_items.into_iter().flatten());
                items.push(EnvelopeFormatItem::End("]".to_string()));
                EnvelopeFormatItem::List(items)
            },
            Envelope::Elided(_) => EnvelopeFormatItem::Item("ELIDED".to_string()),
        }
    }
}

impl EnvelopeFormat for Assertion {
    fn format_item(&self, context: &FormatContext) -> EnvelopeFormatItem {
        EnvelopeFormatItem::List(vec![
            self.predicate().format_item(context),
            EnvelopeFormatItem::Item(": ".to_string()),
            self.object().format_item(context),
        ])
    }
}

impl EnvelopeFormat for KnownValue {
    fn format_item(&self, context: &FormatContext) -> EnvelopeFormatItem {
        EnvelopeFormatItem::Item(context
            .known_values()
            .assigned_name(self)
            .map(|s| s.to_string())
            .unwrap_or_else(|| self.name())
            .flanked_by("'", "'")
        )
    }
}

 impl Envelope {
    fn description(&self, context: Option<&FormatContext>) -> String {
        match self {
            Self::Node { subject, assertions, .. } => {
                let assertions = assertions
                    .iter()
                    .map(|a| a.to_string())
                    .collect::<Vec<String>>()
                    .join(", ")
                    .flanked_by("[", "]");
                format!(".node({}, {})", subject, assertions)
            }
            Self::Leaf { cbor, .. } => format!(".cbor({})", cbor.format_item(context.unwrap_or(&FormatContext::default()))),
            Self::Wrapped { envelope, .. } => format!(".wrapped({})", envelope),
            Self::KnownValue { value, .. } => format!(".knownValue({})", value),
            Self::Assertion(assertion) => format!(".assertion({}, {})", assertion.predicate(), assertion.object()),
            Self::Encrypted(_) => ".encrypted".to_string(),
            Self::Compressed(_) => ".compressed".to_string(),
            Self::Elided(_) => ".elided".to_string()
        }
    }
}

impl std::fmt::Display for Envelope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.description(None))
    }
}
