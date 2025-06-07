use anyhow::Result;
use dcbor::prelude::*;

use crate::{ FormatContext, string_utils::StringUtils };

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
            CBORCase::Array(_) => Ok(self.diagnostic_opt(DiagFormatOpts::default().summarize(true).tags(TagsStoreOpt::Custom(context.tags())))),

            CBORCase::Map(_) => Ok(self.diagnostic_opt(DiagFormatOpts::default().summarize(true).tags(TagsStoreOpt::Custom(context.tags())))),
            CBORCase::Simple(v) => Ok(v.to_string()),
            CBORCase::Tagged(_, _) => Ok(self.diagnostic_opt(DiagFormatOpts::default().summarize(true).tags(TagsStoreOpt::Custom(context.tags())))),
        }
    }
}
