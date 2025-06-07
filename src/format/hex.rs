use dcbor::prelude::*;

use crate::{Envelope, FormatContextOpt};

#[derive(Clone, Default)]
pub struct HexFormatOpts<'a> {
    annotate: bool,
    context: FormatContextOpt<'a>,
}

impl<'a> From<HexFormatOpts<'a>> for dcbor::HexFormatOpts<'a> {
    fn from(opts: HexFormatOpts<'a>) -> Self {
        dcbor::HexFormatOpts::default()
            .annotate(opts.annotate)
            .context(opts.context.into())
    }
}

impl<'a> HexFormatOpts<'a> {
    /// Sets whether to annotate the hex dump with tags.
    pub fn annotate(mut self, annotate: bool) -> Self {
        self.annotate = annotate;
        self
    }

    /// Sets the formatting context for the hex dump.
    pub fn context(mut self, context: FormatContextOpt<'a>) -> Self {
        self.context = context;
        self
    }
}

impl Envelope {
    /// Returns the CBOR hex dump of this envelope.
    ///
    /// See [RFC-8949](https://www.rfc-editor.org/rfc/rfc8949.html) for information on
    /// the CBOR binary format.
    pub fn hex_opt<'a>(&self, opts: HexFormatOpts<'a>) -> String {
        let cbor: CBOR = self.clone().into();
        let hex_opts: dcbor::HexFormatOpts<'a> = opts.into();
        cbor.hex_opt(hex_opts)
    }

    /// Returns the CBOR hex dump of this envelope.
    ///
    /// Uses the current format context.
    ///
    /// See [RFC-8949](https://www.rfc-editor.org/rfc/rfc8949.html) for information on
    /// the CBOR binary format.
    pub fn hex(&self) -> String {
        self.hex_opt(HexFormatOpts::default().annotate(true))
    }
}
