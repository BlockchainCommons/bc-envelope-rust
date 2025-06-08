//! Creates a textual tree representation of an envelope for debugging and
//! visualization.
//!
//! This module provides functionality for creating a textual tree
//! representation of an envelope, which is useful for debugging and visualizing
//! the structure of complex envelopes.
//!
//! The tree format displays each component of an envelope (subject and
//! assertions) as nodes in a tree, making it easy to understand the
//! hierarchical structure of nested envelopes. Each node includes:
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
//! let envelope = Envelope::new("Alice").add_assertion(
//!     "knows",
//!     Envelope::new("Bob").add_assertion("email", "bob@example.com"),
//! );
//!
//! // Get a tree representation of the envelope
//! let tree = envelope.tree_format();
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

use std::{cell::RefCell, collections::HashSet};

use bc_components::{Digest, DigestProvider};
use bc_ur::UREncodable;

use super::FormatContextOpt;
use crate::{
    EdgeType, Envelope, FormatContext,
    with_format_context,
};

#[derive(Clone, Copy)]
pub enum DigestDisplayFormat {
    /// Default: Display a shortened version of the digest (first 8 characters).
    Short,
    /// Display the full digest for each element in the tree.
    Full,
    /// Display a `ur:digest` UR for each element in the tree.
    UR,
}

impl Default for DigestDisplayFormat {
    fn default() -> Self { DigestDisplayFormat::Short }
}

#[derive(Clone, Default)]
pub struct TreeFormatOpts<'a> {
    hide_nodes: bool,
    highlighting_target: HashSet<Digest>,
    context: FormatContextOpt<'a>,
    digest_display: DigestDisplayFormat,
}

impl<'a> TreeFormatOpts<'a> {
    /// Sets whether to hide NODE identifiers in the tree representation.
    pub fn hide_nodes(mut self, hide: bool) -> Self {
        self.hide_nodes = hide;
        self
    }

    /// Sets the set of digests to highlight in the tree representation.
    pub fn highlighting_target(mut self, target: HashSet<Digest>) -> Self {
        self.highlighting_target = target;
        self
    }

    /// Sets the formatting context for the tree representation.
    pub fn context(mut self, context: FormatContextOpt<'a>) -> Self {
        self.context = context;
        self
    }

    /// Sets the digest display option for the tree representation.
    pub fn digest_display(mut self, opt: DigestDisplayFormat) -> Self {
        self.digest_display = opt;
        self
    }
}

/// Support for tree-formatting envelopes.
impl Envelope {
    /// Returns a tree-formatted string representation of the envelope with
    /// default options.
    pub fn tree_format(&self) -> String {
        self.tree_format_opt(&TreeFormatOpts::default())
    }

    /// Returns a tree-formatted string representation of the envelope with the
    /// specified options.
    ///
    /// # Options
    /// * `hide_nodes` - If true, hides NODE identifiers and only shows the
    ///   semantic content. Default is `false`.
    /// * `highlighting_target` - Set of digests to highlight in the tree
    ///   representation. Default is an empty set.
    /// * `context` - Formatting context. Default is
    ///   `TreeFormatContext::Global`.
    pub fn tree_format_opt<'a>(&self, opts: &TreeFormatOpts<'a>) -> String {
        let elements: RefCell<Vec<TreeElement>> = RefCell::new(Vec::new());
        let visitor = |envelope: Self,
                       level: usize,
                       incoming_edge: EdgeType,
                       _: Option<&()>|
         -> _ {
            let elem = TreeElement::new(
                level,
                envelope.clone(),
                incoming_edge,
                !opts.hide_nodes,
                opts.highlighting_target.contains(&envelope.digest()),
            );
            elements.borrow_mut().push(elem);
            None
        };
        let s = self.clone();
        s.walk(opts.hide_nodes, &visitor);

        let elements = elements.borrow();

        // Closure to format elements with a given context and digest option
        let format_elements =
            |elements: &[TreeElement], context: &FormatContext| -> String {
                elements
                    .iter()
                    .map(|e| e.string(context, opts.digest_display))
                    .collect::<Vec<_>>()
                    .join("\n")
            };

        match &opts.context {
            FormatContextOpt::None => {
                let context_ref = &FormatContext::default();
                format_elements(&elements, context_ref)
            }
            FormatContextOpt::Global => {
                with_format_context!(|context| {
                    format_elements(&elements, context)
                })
            }
            FormatContextOpt::Custom(ctx) => format_elements(&elements, ctx),
        }
    }
}

impl Envelope {
    /// Returns a text representation of the envelope's digest.
    pub fn short_id(&self, opt: DigestDisplayFormat) -> String {
        match opt {
            DigestDisplayFormat::Short => self.digest().short_description(),
            DigestDisplayFormat::Full => self.digest().hex(),
            DigestDisplayFormat::UR => self.digest().ur_string(),
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
    /// * `incoming_edge` - The type of edge connecting this element to its
    ///   parent
    /// * `show_id` - Whether to show the element's ID (digest)
    /// * `is_highlighted` - Whether this element should be highlighted in the
    ///   output
    fn new(
        level: usize,
        envelope: Envelope,
        incoming_edge: EdgeType,
        show_id: bool,
        is_highlighted: bool,
    ) -> Self {
        Self {
            level,
            envelope,
            incoming_edge,
            show_id,
            is_highlighted,
        }
    }

    /// Formats the tree element as a string.
    fn string(
        &self,
        context: &FormatContext,
        digest_display: DigestDisplayFormat,
    ) -> String {
        let line = vec![
            if self.is_highlighted {
                Some("*".to_string())
            } else {
                None
            },
            if self.show_id {
                Some(self.envelope.short_id(digest_display))
            } else {
                None
            },
            self.incoming_edge.label().map(|s| s.to_string()),
            Some(self.envelope.summary(40, context)),
        ]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>()
        .join(" ");
        let indent = " ".repeat(self.level * 4);
        format!("{}{}", indent, line)
    }
}
