//! Creates a textual tree representation of an envelope for debugging and visualization.
//!
//! This module provides functionality for creating a textual tree representation of an envelope,
//! which is useful for debugging and visualizing the structure of complex envelopes.
//!
//! The tree format displays each component of an envelope (subject and assertions) as nodes in a tree,
//! making it easy to understand the hierarchical structure of nested envelopes. Each node includes:
//!
//! * The first 8 characters of the element's digest (for easy reference)
//! * The type of the element (NODE, ASSERTION, ELIDED, etc.)
//! * The content of the element (for leaf nodes)
//! 
//! # Examples
//!
//! ```
//! use bc_envelope::prelude::*;
//!
//! // Create a complex envelope with nested assertions
//! let envelope = Envelope::new("Alice")
//!     .add_assertion("knows", Envelope::new("Bob")
//!         .add_assertion("email", "bob@example.com"));
//! 
//! // Get a tree representation of the envelope
//! let tree = envelope.tree_format(false);
//! // Output will look like:
//! // 9e3b0673 NODE
//! //     13941b48 subj "Alice"
//! //     f45afd77 ASSERTION
//! //         db7dd21c pred "knows" 
//! //         76543210 obj NODE
//! //             13b74194 subj "Bob"
//! //             ee23dcba ASSERTION
//! //                 a9e85a47 pred "email"
//! //                 84fd6e57 obj "bob@example.com"
//! ```

use std::{collections::HashSet, cell::RefCell};

use bc_components::{Digest, DigestProvider};

use crate::{Envelope, with_format_context, FormatContext};
#[cfg(feature = "known_value")]
use crate::{string_utils::StringUtils, extension::KnownValuesStore};

use super::{walk::EdgeType, EnvelopeSummary, envelope::EnvelopeCase};

/// Support for tree-formatting envelopes.
impl Envelope {
    /// Returns a tree-formatted string representation of the envelope with optional context.
    ///
    /// # Arguments
    /// * `hide_nodes` - If true, hides NODE identifiers and only shows the semantic content
    /// * `context` - Optional formatting context
    pub fn tree_format_opt(&self, hide_nodes: bool, context: Option<&FormatContext>) -> String {
        self.tree_format_with_target_opt(hide_nodes, &HashSet::new(), context)
    }

    /// Returns a tree-formatted string representation of the envelope.
    ///
    /// # Arguments
    /// * `hide_nodes` - If true, hides NODE identifiers and only shows the semantic content
    pub fn tree_format(&self, hide_nodes: bool) -> String {
        with_format_context!(|context| {
            self.tree_format_opt(hide_nodes, Some(context))
        })
    }

    /// Returns a tree-formatted string representation of the envelope with highlighted digests.
    ///
    /// # Arguments
    /// * `hide_nodes` - If true, hides NODE identifiers and only shows the semantic content
    /// * `highlighting_target` - Set of digests to highlight in the tree representation
    /// * `context` - Optional formatting context
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

    /// Returns a tree-formatted string representation of the envelope with highlighted digests.
    ///
    /// # Arguments
    /// * `hide_nodes` - If true, hides NODE identifiers and only shows the semantic content
    /// * `highlighting_target` - Set of digests to highlight in the tree representation
    pub fn tree_format_with_target(&self, hide_nodes: bool, highlighting_target: &HashSet<Digest>) -> String {
        with_format_context!(|context| {
            self.tree_format_with_target_opt(hide_nodes, highlighting_target, Some(context))
        })
    }
}

impl Envelope {
    /// Returns a shortened hexadecimal representation of the envelope's digest.
    pub fn short_id(&self) -> String {
        self.digest().short_description()
    }

    /// Returns a short summary of the envelope's content with a maximum length.
    ///
    /// # Arguments
    /// * `max_length` - The maximum length of the summary
    /// * `context` - The formatting context
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

/// Represents an element in the tree representation of an envelope.
#[derive(Debug)]
struct TreeElement {
    /// Indentation level of the element in the tree
    level: usize,
    /// The envelope element
    envelope: Envelope,
    /// The type of edge connecting this element to its parent
    incoming_edge: EdgeType,
    /// Whether to show the element's ID (digest)
    show_id: bool,
    /// Whether this element should be highlighted in the output
    is_highlighted: bool,
}

impl TreeElement {
    /// Creates a new TreeElement.
    ///
    /// # Arguments
    /// * `level` - Indentation level of the element in the tree
    /// * `envelope` - The envelope element
    /// * `incoming_edge` - The type of edge connecting this element to its parent
    /// * `show_id` - Whether to show the element's ID (digest)
    /// * `is_highlighted` - Whether this element should be highlighted in the output
    fn new(level: usize, envelope: Envelope, incoming_edge: EdgeType, show_id: bool, is_highlighted: bool) -> Self {
        Self { level, envelope, incoming_edge, show_id, is_highlighted }
    }

        /// Formats the tree element as a string.
    ///
    /// # Arguments
    /// * `context` - The formatting context
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
