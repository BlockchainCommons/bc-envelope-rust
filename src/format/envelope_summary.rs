use anyhow::Result;
use dcbor::prelude::*;

use crate::{FormatContextOpt, string_utils::StringUtils};

pub trait EnvelopeSummary {
    fn envelope_summary<'a>(
        &self,
        max_length: usize,
        context: FormatContextOpt<'a>,
    ) -> Result<String>;
}

impl EnvelopeSummary for CBOR {
    fn envelope_summary<'a>(
        &self,
        max_length: usize,
        context: FormatContextOpt<'a>,
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

            CBORCase::Array(_) => Ok(self.diagnostic_opt(
                DiagFormatOpts::default()
                    .summarize(true)
                    .tags(context.into()),
            )),
            CBORCase::Map(_) => Ok(self.diagnostic_opt(
                DiagFormatOpts::default()
                    .summarize(true)
                    .tags(context.into()),
            )),
            CBORCase::Tagged(_, _) => Ok(self.diagnostic_opt(
                DiagFormatOpts::default()
                    .summarize(true)
                    .tags(context.into()),
            )),
        }
    }
}
