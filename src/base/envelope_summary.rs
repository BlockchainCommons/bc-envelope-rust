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
            CBORCase::Array(elements) => {
                Ok(
                    elements
                        .iter()
                        .map(|element| element.envelope_summary(max_length, context))
                        .collect::<Result<Vec<String>>>()?
                        .join(", ")
                        .flanked_by("[", "]")
                )
            }
            CBORCase::Map(_) => Ok("Map".to_string()),
            CBORCase::Simple(v) => Ok(v.to_string()),
            CBORCase::Tagged(_, _) => Ok(self.summary_opt(context)),
        }
    }
}
