/// Types dealing with formatting envelopes.
mod notation;
mod format_context;
pub use format_context::{
    FormatContext, FormatContextOpt, GLOBAL_FORMAT_CONTEXT, *,
};

mod tree;
pub use tree::{DigestDisplayFormat, TreeFormatOpts};

mod mermaid;
pub use mermaid::{MermaidFormatOpts, MermaidTheme, MermaidOrientation};

mod envelope_summary;
pub use envelope_summary::EnvelopeSummary;

mod hex;
mod diagnostic;
