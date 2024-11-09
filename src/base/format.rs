use dcbor::prelude::*;
use crate::{Envelope, Assertion, string_utils::StringUtils, FormatContext, with_format_context};
#[cfg(feature = "known_value")]
use crate::extension::{KnownValue, known_values};

use super::{EnvelopeSummary, envelope::EnvelopeCase};

/// Support for the various text output formats for ``Envelope``.
impl Envelope {
    /// Returns the envelope notation for this envelope.
    pub fn format_opt(&self, context: Option<&FormatContext>) -> String {
        let context = context.cloned().unwrap_or(FormatContext::default());
        self.format_item(&context).format(context.is_flat()).trim().to_string()
    }

    /// Returns the envelope notation for this envelope.
    ///
    /// Uses the current format context.
    pub fn format(&self) -> String {
        with_format_context!(|context| {
            self.format_opt(Some(context))
        })
    }

    /// Returns the envelope notation for this envelope in flat format.
    ///
    /// In flat format, the envelope is printed on a single line.
    pub fn format_flat(&self) -> String {
        with_format_context!(|context: &FormatContext| {
            let context = context.clone().set_flat(true);
            self.format_opt(Some(&context))
        })
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
    /// Uses the current format context.
    ///
    /// See [RFC-8949 §8](https://www.rfc-editor.org/rfc/rfc8949.html#name-diagnostic-notation)
    /// for information on CBOR diagnostic notation.
    pub fn diagnostic(&self) -> String {
        with_format_context!(|context| {
            self.diagnostic_opt(true, Some(context))
        })
    }

    /// Returns the CBOR hex dump of this envelope.
    ///
    /// See [RFC-8949](https://www.rfc-editor.org/rfc/rfc8949.html) for information on
    /// the CBOR binary format.
    pub fn hex_opt(&self, annotate: bool, context: Option<&FormatContext>) -> String {
        let cbor: CBOR = self.clone().into();
        cbor.hex_opt(annotate, Some(context.unwrap_or(&FormatContext::default()).tags()))
    }

    /// Returns the CBOR hex dump of this envelope.
    ///
    /// Uses the current format context.
    ///
    /// See [RFC-8949](https://www.rfc-editor.org/rfc/rfc8949.html) for information on
    /// the CBOR binary format.
    pub fn hex(&self) -> String {
        with_format_context!(|context| {
            self.hex_opt(true, Some(context))
        })
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

    fn nicen(items: &[EnvelopeFormatItem]) -> Vec<EnvelopeFormatItem> {
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

    fn indent(level: usize) -> String {
        " ".repeat(level * 4)
    }

    fn add_space_at_end_if_needed(s: &str) -> String {
        if s.is_empty() {
            " ".to_string()
        } else if s.ends_with(' ') {
            s.to_string()
        } else {
            s.to_string() + " "
        }
    }

    fn format(&self, is_flat: bool) -> String {
        if is_flat {
            return self.format_flat();
        }
        self.format_hierarchical()
    }

    fn format_flat(&self) -> String {
        let mut line: String = "".to_string();
        let items = self.flatten();
        for item in items {
            match item {
                EnvelopeFormatItem::Begin(s) => {
                    if !line.ends_with(' ') {
                        line += " ";
                    }
                    line += &s;
                    line += " ";
                },
                EnvelopeFormatItem::End(s) => {
                    if !line.ends_with(' ') {
                        line += " ";
                    }
                    line += &s;
                    line += " ";
                },
                EnvelopeFormatItem::Item(s) => line += &s,
                EnvelopeFormatItem::Separator => {
                    line = line.trim_end().to_string() + ", ";
                },
                EnvelopeFormatItem::List(items) => {
                    for item in items {
                        line += &item.format_flat();
                    }
                }
            }
        }
        line
    }

    fn format_hierarchical(&self) -> String {
        let mut lines: Vec<String> = vec![];
        let mut level = 0;
        let mut current_line = "".to_string();
        let items = Self::nicen(&self.flatten());
        for item in items {
            match item {
                EnvelopeFormatItem::Begin(delimiter) => {
                    if !delimiter.is_empty() {
                        let c = if current_line.is_empty() {
                            delimiter
                        } else {
                            Self::add_space_at_end_if_needed(&current_line) + &delimiter
                        };
                        lines.push(Self::indent(level) + &c + "\n");
                    }
                    level += 1;
                    current_line = "".to_string();
                }
                EnvelopeFormatItem::End(delimiter) => {
                    if !current_line.is_empty() {
                        lines.push(Self::indent(level) + &current_line + "\n");
                        current_line = "".to_string();
                    }
                    level -= 1;
                    lines.push(Self::indent(level) + &delimiter + "\n");
                }
                EnvelopeFormatItem::Item(string) => {
                    current_line += &string;
                }
                EnvelopeFormatItem::Separator => {
                    if !current_line.is_empty() {
                        lines.push(Self::indent(level) + &current_line + "\n");
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
        Some(self.cmp(other))
    }
}


impl Ord for EnvelopeFormatItem {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let l_index = self.index();
        let r_index = other.index();
        match l_index.cmp(&r_index) {
            std::cmp::Ordering::Less => return std::cmp::Ordering::Less,
            std::cmp::Ordering::Greater => return std::cmp::Ordering::Greater,
            _ => {}
        }
        match (self, other) {
            (EnvelopeFormatItem::Begin(l), EnvelopeFormatItem::Begin(r)) => l.cmp(r),
            (EnvelopeFormatItem::End(l), EnvelopeFormatItem::End(r)) => l.cmp(r),
            (EnvelopeFormatItem::Item(l), EnvelopeFormatItem::Item(r)) => l.cmp(r),
            (EnvelopeFormatItem::Separator, EnvelopeFormatItem::Separator) => std::cmp::Ordering::Equal,
            (EnvelopeFormatItem::List(l), EnvelopeFormatItem::List(r)) => l.cmp(r),
            _ => std::cmp::Ordering::Equal,
        }
    }
}

// impl EnvelopeFormat for Digest {
//     fn format_item(&self, _context: &FormatContext) -> EnvelopeFormatItem {
//         EnvelopeFormatItem::Item(hex::encode(self.data()))
//     }
// }

// impl EnvelopeFormat for ARID {
//     fn format_item(&self, _context: &FormatContext) -> EnvelopeFormatItem {
//         EnvelopeFormatItem::Item(hex::encode(self.data()))
//     }
// }

impl EnvelopeFormat for CBOR {
    fn format_item(&self, context: &FormatContext) -> EnvelopeFormatItem {
        match self.as_case() {
            CBORCase::Tagged(tag, cbor) if tag == &Envelope::cbor_tags()[0] => {
                Envelope::from_untagged_cbor(cbor.clone())
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
        match self.case() {
            EnvelopeCase::Leaf { cbor, .. } => cbor.format_item(context),
            EnvelopeCase::Wrapped { envelope, .. } => EnvelopeFormatItem::List(vec![
                EnvelopeFormatItem::Begin("{".to_string()),
                envelope.format_item(context),
                EnvelopeFormatItem::End("}".to_string()),
            ]),
            EnvelopeCase::Assertion(assertion) => assertion.format_item(context),
            #[cfg(feature = "known_value")]
            EnvelopeCase::KnownValue { value, .. } => value.format_item(context),
            #[cfg(feature = "encrypt")]
            EnvelopeCase::Encrypted(_) => EnvelopeFormatItem::Item("ENCRYPTED".to_string()),
            #[cfg(feature = "compress")]
            EnvelopeCase::Compressed(_) => EnvelopeFormatItem::Item("COMPRESSED".to_string()),
            EnvelopeCase::Node { subject, assertions, .. } => {
                let mut items: Vec<EnvelopeFormatItem> = Vec::new();

                let subject_item = subject.format_item(context);
                let mut elided_count = 0;
                #[cfg(feature = "encrypt")]
                let mut encrypted_count = 0;
                #[cfg(feature = "compress")]
                let mut compressed_count = 0;
                #[cfg(feature = "known_value")]
                let mut type_assertion_items: Vec<Vec<EnvelopeFormatItem>> = Vec::new();
                let mut assertion_items: Vec<Vec<EnvelopeFormatItem>> = Vec::new();

                for assertion in assertions {
                    match assertion.case() {
                        EnvelopeCase::Elided(_) => {
                            elided_count += 1;
                        },
                        #[cfg(feature = "encrypt")]
                        EnvelopeCase::Encrypted(_) => {
                            encrypted_count += 1;
                        },
                        #[cfg(feature = "compress")]
                        EnvelopeCase::Compressed(_) => {
                            compressed_count += 1;
                        },
                        _ => {
                            let item = vec![assertion.format_item(context)];
                            #[cfg(feature = "known_value")]
                            {
                                let mut is_type_assertion = false;
                                if let Some(predicate) = assertion.as_predicate() {
                                    if let Some(known_value) = predicate.subject().as_known_value() {
                                        if *known_value == known_values::IS_A {
                                            is_type_assertion = true;
                                        }
                                    }
                                }
                                if is_type_assertion {
                                    type_assertion_items.push(item);
                                } else {
                                    assertion_items.push(item);
                                }
                            }
                            #[cfg(not(feature = "known_value"))]
                            assertion_items.push(item);
                        },
                    }
                }
                #[cfg(feature = "known_value")]
                type_assertion_items.sort();
                assertion_items.sort();
                #[cfg(feature = "known_value")]
                assertion_items.splice(0..0, type_assertion_items);
                #[cfg(feature = "compress")]
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
                #[cfg(feature = "encrypt")]
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
            EnvelopeCase::Elided(_) => EnvelopeFormatItem::Item("ELIDED".to_string()),
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

#[cfg(feature = "known_value")]
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
        match self.case() {
            EnvelopeCase::Node { subject, assertions, .. } => {
                let assertions = assertions
                    .iter()
                    .map(|a| a.to_string())
                    .collect::<Vec<String>>()
                    .join(", ")
                    .flanked_by("[", "]");
                format!(".node({}, {})", subject, assertions)
            }
            EnvelopeCase::Leaf { cbor, .. } => format!(".cbor({})", cbor.format_item(context.unwrap_or(&FormatContext::default()))),
            EnvelopeCase::Wrapped { envelope, .. } => format!(".wrapped({})", envelope),
            EnvelopeCase::Assertion(assertion) => format!(".assertion({}, {})", assertion.predicate(), assertion.object()),
            EnvelopeCase::Elided(_) => ".elided".to_string(),
            #[cfg(feature = "known_value")]
            EnvelopeCase::KnownValue { value, .. } => format!(".knownValue({})", value),
            #[cfg(feature = "encrypt")]
            EnvelopeCase::Encrypted(_) => ".encrypted".to_string(),
            #[cfg(feature = "compress")]
            EnvelopeCase::Compressed(_) => ".compressed".to_string(),
        }
    }
}

impl std::fmt::Display for Envelope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.description(None))
    }
}
