use anyhow::Result;
use dcbor::prelude::*;

use crate::{FormatContext, FormatContextOpt, string_utils::StringUtils};

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
