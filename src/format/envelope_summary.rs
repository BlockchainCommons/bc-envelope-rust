use anyhow::Result;
use dcbor::prelude::*;

#[cfg(feature = "known_value")]
use crate::extension::KnownValuesStore;
use crate::{
    Envelope, FormatContext, FormatContextOpt, base::envelope::EnvelopeCase,
    string_utils::StringUtils,
};

impl Envelope {
    /// Returns a short summary of the envelope's content with a maximum length.
    ///
    /// # Arguments
    /// * `max_length` - The maximum length of the summary
    /// * `context` - The formatting context
    pub fn summary(
        &self,
        max_length: usize,
        context: &FormatContext,
    ) -> String {
        match self.case() {
            EnvelopeCase::Node { .. } => "NODE".to_string(),
            EnvelopeCase::Leaf { cbor, .. } => cbor
                .envelope_summary(
                    max_length,
                    &FormatContextOpt::Custom(context),
                )
                .unwrap(),
            EnvelopeCase::Wrapped { .. } => "WRAPPED".to_string(),
            EnvelopeCase::Assertion(_) => "ASSERTION".to_string(),
            EnvelopeCase::Elided(_) => "ELIDED".to_string(),
            #[cfg(feature = "known_value")]
            EnvelopeCase::KnownValue { value, .. } => {
                let known_value = KnownValuesStore::known_value_for_raw_value(
                    value.value(),
                    Some(context.known_values()),
                );
                known_value.to_string().flanked_by("'", "'")
            }
            #[cfg(feature = "encrypt")]
            EnvelopeCase::Encrypted(_) => "ENCRYPTED".to_string(),
            #[cfg(feature = "compress")]
            EnvelopeCase::Compressed(_) => "COMPRESSED".to_string(),
        }
    }
}

pub trait EnvelopeSummary {
    fn envelope_summary<'a>(
        &self,
        max_length: usize,
        context: &FormatContextOpt<'a>,
    ) -> Result<String>;
}

impl EnvelopeSummary for CBOR {
    fn envelope_summary<'a>(
        &self,
        max_length: usize,
        context: &FormatContextOpt<'a>,
    ) -> Result<String> {
        match self.as_case() {
            CBORCase::Unsigned(n) => Ok(n.to_string()),
            CBORCase::Negative(n) => Ok((-1 - (*n as i128)).to_string()),
            CBORCase::ByteString(data) => Ok(format!("Bytes({})", data.len())),
            CBORCase::Text(string) => {
                let string = if string.len() > max_length {
                    format!(
                        "{}…",
                        string.chars().take(max_length).collect::<String>()
                    )
                } else {
                    string.clone()
                };
                Ok(string.replace('\n', "\\n").flanked_by("\"", "\""))
            }
            CBORCase::Simple(v) => Ok(v.to_string()),

            CBORCase::Array(_) => match context {
                FormatContextOpt::None => Ok(self.diagnostic_opt(
                    &DiagFormatOpts::default()
                        .summarize(true)
                        .tags(TagsStoreOpt::None),
                )),
                FormatContextOpt::Global => {
                    crate::with_format_context!(|ctx: &FormatContext| {
                        Ok(self.diagnostic_opt(
                            &DiagFormatOpts::default()
                                .summarize(true)
                                .tags(TagsStoreOpt::Custom(ctx.tags())),
                        ))
                    })
                }
                FormatContextOpt::Custom(format_context) => Ok(self
                    .diagnostic_opt(
                        &DiagFormatOpts::default()
                            .summarize(true)
                            .tags(TagsStoreOpt::Custom(format_context.tags())),
                    )),
            },
            CBORCase::Map(_) => match context {
                FormatContextOpt::None => Ok(self.diagnostic_opt(
                    &DiagFormatOpts::default()
                        .summarize(true)
                        .tags(TagsStoreOpt::None),
                )),
                FormatContextOpt::Global => {
                    crate::with_format_context!(|ctx: &FormatContext| {
                        Ok(self.diagnostic_opt(
                            &DiagFormatOpts::default()
                                .summarize(true)
                                .tags(TagsStoreOpt::Custom(ctx.tags())),
                        ))
                    })
                }
                FormatContextOpt::Custom(format_context) => Ok(self
                    .diagnostic_opt(
                        &DiagFormatOpts::default()
                            .summarize(true)
                            .tags(TagsStoreOpt::Custom(format_context.tags())),
                    )),
            },
            CBORCase::Tagged(_, _) => match context {
                FormatContextOpt::None => Ok(self.diagnostic_opt(
                    &DiagFormatOpts::default()
                        .summarize(true)
                        .tags(TagsStoreOpt::None),
                )),
                FormatContextOpt::Global => {
                    crate::with_format_context!(|ctx: &FormatContext| {
                        Ok(self.diagnostic_opt(
                            &DiagFormatOpts::default()
                                .summarize(true)
                                .tags(TagsStoreOpt::Custom(ctx.tags())),
                        ))
                    })
                }
                FormatContextOpt::Custom(format_context) => Ok(self
                    .diagnostic_opt(
                        &DiagFormatOpts::default()
                            .summarize(true)
                            .tags(TagsStoreOpt::Custom(format_context.tags())),
                    )),
            },
        }
    }
}
