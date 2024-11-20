use std::{collections::HashSet, cell::RefCell};

use bc_components::{Digest, DigestProvider};

use crate::{Envelope, with_format_context, FormatContext};
#[cfg(feature = "known_value")]
use crate::{string_utils::StringUtils, extension::KnownValuesStore};

use super::{walk::EdgeType, EnvelopeSummary, envelope::EnvelopeCase};

/// Support for tree-formatting envelopes.
impl Envelope {
    pub fn tree_format_opt(&self, hide_nodes: bool, context: Option<&FormatContext>) -> String {
        self.tree_format_with_target_opt(hide_nodes, &HashSet::new(), context)
    }

    pub fn tree_format(&self, hide_nodes: bool) -> String {
        with_format_context!(|context| {
            self.tree_format_opt(hide_nodes, Some(context))
        })
    }

    pub fn tree_format_with_target_opt(&self, hide_nodes: bool, highlighting_target: &HashSet<Digest>, context: Option<&FormatContext>) -> String {
        let elements: RefCell<Vec<TreeElement>> = RefCell::new(Vec::new());
        let visitor = |envelope: Self, level: usize, incoming_edge: EdgeType, _: Option<&()>| -> _ {
            let elem = TreeElement::new(
                level,
                envelope.clone(),
                incoming_edge,
                !hide_nodes,
                highlighting_target.contains(&envelope.digest()),
            );
            elements.borrow_mut().push(elem);
            None
        };
        let s = self.clone();
        s.walk(hide_nodes, &visitor);
        let elements = elements.borrow();
        elements.iter().map(|e| e.string(context.unwrap_or(&FormatContext::default()))).collect::<Vec<_>>().join("\n")
    }

    pub fn tree_format_with_target(&self, hide_nodes: bool, highlighting_target: &HashSet<Digest>) -> String {
        with_format_context!(|context| {
            self.tree_format_with_target_opt(hide_nodes, highlighting_target, Some(context))
        })
    }
}

impl Envelope {
    pub fn short_id(&self) -> String {
        self.digest().short_description()
    }

    pub fn summary(&self, max_length: usize, context: &FormatContext) -> String {
        match self.case() {
            EnvelopeCase::Node { .. } => "NODE".to_string(),
            EnvelopeCase::Leaf { cbor, .. } => cbor.envelope_summary(max_length, context).unwrap(),
            EnvelopeCase::Wrapped { .. } => "WRAPPED".to_string(),
            EnvelopeCase::Assertion(_) => "ASSERTION".to_string(),
            EnvelopeCase::Elided(_) => "ELIDED".to_string(),
            #[cfg(feature = "known_value")]
            EnvelopeCase::KnownValue { value, .. } => {
                let known_value = KnownValuesStore::known_value_for_raw_value(value.value(), Some(context.known_values()));
                known_value.to_string().flanked_by("'", "'",)
            },
            #[cfg(feature = "encrypt")]
            EnvelopeCase::Encrypted(_) => "ENCRYPTED".to_string(),
            #[cfg(feature = "compress")]
            EnvelopeCase::Compressed(_) => "COMPRESSED".to_string(),
        }
    }
}

#[derive(Debug)]
struct TreeElement {
    level: usize,
    envelope: Envelope,
    incoming_edge: EdgeType,
    show_id: bool,
    is_highlighted: bool,
}

impl TreeElement {
    fn new(level: usize, envelope: Envelope, incoming_edge: EdgeType, show_id: bool, is_highlighted: bool) -> Self {
        Self { level, envelope, incoming_edge, show_id, is_highlighted }
    }

    fn string(&self, context: &FormatContext) -> String {
        let line = vec![
            if self.is_highlighted { Some("*".to_string()) } else { None },
            if self.show_id { Some(self.envelope.short_id()) } else { None },
            self.incoming_edge.label().map(|s| s.to_string()),
            Some(self.envelope.summary(40, context)),
        ].into_iter().flatten().collect::<Vec<_>>().join(" ");
        let indent = " ".repeat(self.level * 4);
        format!("{}{}", indent, line)
    }
}
