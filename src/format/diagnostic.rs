use dcbor::prelude::*;

use crate::{Envelope, FormatContext, with_format_context};

impl Envelope {
    /// Returns the CBOR diagnostic notation for this envelope, with
    /// annotations.
    ///
    /// See [RFC-8949 §8](https://www.rfc-editor.org/rfc/rfc8949.html#name-diagnostic-notation)
    /// for information on CBOR diagnostic notation.
    pub fn diagnostic_annotated(&self) -> String {
        with_format_context!(|context: &FormatContext| {
            self.tagged_cbor().diagnostic_opt(
                &DiagFormatOpts::default().annotate(true).tags(TagsStoreOpt::Custom(context.tags())),
            )
        })
    }

    /// Returns the CBOR diagnostic notation for this envelope.
    ///
    /// Uses the current format context.
    ///
    /// See [RFC-8949 §8](https://www.rfc-editor.org/rfc/rfc8949.html#name-diagnostic-notation)
    /// for information on CBOR diagnostic notation.
    pub fn diagnostic(&self) -> String { self.tagged_cbor().diagnostic() }
}
