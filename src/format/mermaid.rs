use std::{cell::RefCell, collections::HashSet, rc::Rc};

use bc_components::{Digest, DigestProvider};

use super::FormatContextOpt;
use crate::{base::envelope::EnvelopeCase, with_format_context, EdgeType, Envelope, FormatContext, GLOBAL_FORMAT_CONTEXT};

#[derive(Clone)]
pub struct MermaidFormatOpts<'a> {
    hide_nodes: bool,
    color: bool,
    highlighting_target: HashSet<Digest>,
    context: FormatContextOpt<'a>,
}

impl Default for MermaidFormatOpts<'_> {
    fn default() -> Self {
        Self {
            hide_nodes: false,
            color: true,
            highlighting_target: HashSet::new(),
            context: FormatContextOpt::Global,
        }
    }
}

impl<'a> MermaidFormatOpts<'a> {
    /// Sets whether to hide NODE identifiers in the tree representation (default is false).
    pub fn hide_nodes(mut self, hide: bool) -> Self {
        self.hide_nodes = hide;
        self
    }

    /// Sets whether to use color in the tree representation (default is true).
    pub fn color(mut self, color: bool) -> Self {
        self.color = color;
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
}

/// Support for tree-formatting envelopes.
impl Envelope {
    pub fn mermaid_format(&self) -> String {
        self.mermaid_format_opt(&MermaidFormatOpts::default())
    }

    pub fn mermaid_format_opt<'a>(
        &self,
        opts: &MermaidFormatOpts<'a>,
    ) -> String {
        let elements: RefCell<Vec<Rc<MermaidElement>>> =
            RefCell::new(Vec::new());
        let next_id = RefCell::new(0);
        let visitor = |envelope: Self,
                       level: usize,
                       incoming_edge: EdgeType,
                       parent: Option<Rc<MermaidElement>>|
         -> _ {
            let id = *next_id.borrow_mut();
            *next_id.borrow_mut() += 1;
            let elem = Rc::new(MermaidElement::new(
                id,
                level,
                envelope.clone(),
                incoming_edge,
                !opts.hide_nodes,
                opts.highlighting_target.contains(&envelope.digest()),
                parent.clone(),
            ));
            elements.borrow_mut().push(elem.clone());
            Some(elem)
        };
        let s = self.clone();
        s.walk(opts.hide_nodes, &visitor);

        let elements = elements.borrow();

        let mut element_ids: HashSet<usize> =
            elements.iter().map(|e| e.id).collect();

        let mut lines = vec![
            "%%{ init: { 'flowchart': { 'curve': 'basis' } } }%%".to_string(),
            "graph LR".to_string()
        ];

        let mut node_styles: Vec<String> = Vec::new();
        let mut link_styles: Vec<String> = Vec::new();
        let mut link_index = 0;

        for element in elements.iter() {
            let indent = "  ".repeat(element.level);
            let mut this_node_styles = Vec::new();
            let content = if element.parent.is_some() {
                let mut this_link_styles = Vec::new();
                if opts.color {
                    if let Some(color) = element.incoming_edge.link_stroke_color() {
                        this_link_styles.push(format!("stroke:{}", color));
                    }
                }
                this_link_styles.push("stroke-width:2px".to_string());
                if !this_link_styles.is_empty() {
                    link_styles.push(format!("linkStyle {} {}", link_index, this_link_styles.join(",")));
                }
                link_index += 1;
                element.format_edge(&mut element_ids)
            } else {
                element.format_node(&mut element_ids)
            };
            if opts.color {
                let (stroke_color, fill_color) = element.envelope.node_color();
                this_node_styles.push(format!("stroke:{}", stroke_color));
                this_node_styles.push(format!("fill:{}", fill_color));
            }
            this_node_styles.push("stroke-width:4px".to_string());
            if !this_node_styles.is_empty() {
                node_styles.push(format!("style {} {}", element.id, this_node_styles.join(",")));
            }
            lines.push(format!("{}{}", indent, content));
        }

        for style in node_styles {
            lines.push(style);
        }

        for style in link_styles {
            lines.push(style);
        }

        lines.join("\n")
    }
}

/// Represents an element in the tree representation of an envelope.
#[derive(Debug)]
struct MermaidElement {
    id: usize,
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
    parent: Option<Rc<MermaidElement>>,
}

impl MermaidElement {
    fn new(
        id: usize,
        level: usize,
        envelope: Envelope,
        incoming_edge: EdgeType,
        show_id: bool,
        is_highlighted: bool,
        parent: Option<Rc<MermaidElement>>,
    ) -> Self {
        Self {
            id,
            level,
            envelope,
            incoming_edge,
            show_id,
            is_highlighted,
            parent,
        }
    }

    fn format_node(&self, element_ids: &mut HashSet<usize>) -> String {
        if element_ids.contains(&self.id) {
            element_ids.remove(&self.id);
            let summary = with_format_context!(|ctx| {
                format!(r#"{}"#, self.envelope.summary(20, ctx).replace('"', "&quot;"))
            });
            let lines = vec![
                self.envelope.digest().short_description(),
                summary,
            ].join("<br>");
            let (frame_l, frame_r) = self.envelope.mermaid_frame();
            let id = self.id;
            format!(r#"{id}{frame_l}"{lines}"{frame_r}"#)
        } else {
            format!("{}", self.id)
        }
    }

    fn format_edge(&self, element_ids: &mut HashSet<usize>) -> String {
        let parent = self.parent.as_ref().unwrap();
        let arrow = if let Some(label) = self.incoming_edge.label() {
            format!("-- {} -->", label)
        } else {
            "-->".to_string()
        };
        format!(
            "{} {} {}",
            parent.format_node(element_ids),
            arrow,
            self.format_node(element_ids)
        )
    }
}

impl Envelope {
    #[rustfmt::skip]
    fn mermaid_frame(&self) -> (&str, &str) {
        match self.case() {
            EnvelopeCase::Node { .. }       => ("((", "))"),
            EnvelopeCase::Leaf { .. }       => ("[",  "]"),
            EnvelopeCase::Wrapped { .. }    => ("[/", "\\]"),
            EnvelopeCase::Assertion(..)     => ("([", "])"),
            EnvelopeCase::Elided(..)        => ("{{", "}}"),
            EnvelopeCase::KnownValue { .. } => ("[/", "/]"),
            EnvelopeCase::Encrypted(..)     => (">",  "]"),
            EnvelopeCase::Compressed(..)    => ("[[", "]]"),
        }
    }
    #[rustfmt::skip]
    fn node_color(&self) -> (&'static str, &'static str) {
        match self.case() {
            EnvelopeCase::Node { .. }       => ("#b62128", "#4e2325"),
            EnvelopeCase::Leaf { .. }       => ("#0c7883", "#1c383a"),
            EnvelopeCase::Wrapped { .. }    => ("#4554f8", "#2a2e57"),
            EnvelopeCase::Assertion(..)     => ("#8abe5e", "#3c4930"),
            EnvelopeCase::Elided(..)        => ("#a6aaa9", "#434443"),
            EnvelopeCase::KnownValue { .. } => ("#d7ac48", "#584c2e"),
            EnvelopeCase::Encrypted(..)     => ("#ff9d35", "#594026"),
            EnvelopeCase::Compressed(..)    => ("#a01d76", "#412036"),
        }
    }
}

impl EdgeType {
    pub fn link_stroke_color(&self) -> Option<&'static str> {
        match self {
            EdgeType::Subject | EdgeType::Wrapped => Some("#f66"),
            EdgeType::Predicate => Some("#6f6"),
            EdgeType::Object => Some("#66f"),
            _ => None,
        }
    }
}
