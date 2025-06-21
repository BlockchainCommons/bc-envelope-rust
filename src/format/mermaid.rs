use std::{cell::RefCell, collections::HashSet, rc::Rc};

use bc_components::{Digest, DigestProvider};

use super::FormatContextOpt;
use crate::{
    EdgeType, Envelope, base::envelope::EnvelopeCase, with_format_context,
};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Default)]
pub enum MermaidOrientation {
    #[default]
    LeftToRight,
    TopToBottom,
    RightToLeft,
    BottomToTop,
}

impl std::fmt::Display for MermaidOrientation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MermaidOrientation::LeftToRight => write!(f, "LR"),
            MermaidOrientation::TopToBottom => write!(f, "TB"),
            MermaidOrientation::RightToLeft => write!(f, "RL"),
            MermaidOrientation::BottomToTop => write!(f, "BT"),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Default)]
pub enum MermaidTheme {
    #[default]
    Default,
    Neutral,
    Dark,
    Forest,
    Base,
}

impl std::fmt::Display for MermaidTheme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MermaidTheme::Default => write!(f, "default"),
            MermaidTheme::Neutral => write!(f, "neutral"),
            MermaidTheme::Dark => write!(f, "dark"),
            MermaidTheme::Forest => write!(f, "forest"),
            MermaidTheme::Base => write!(f, "base"),
        }
    }
}

#[derive(Clone)]
pub struct MermaidFormatOpts<'a> {
    hide_nodes: bool,
    monochrome: bool,
    theme: MermaidTheme,
    orientation: MermaidOrientation,
    highlighting_target: HashSet<Digest>,
    context: FormatContextOpt<'a>,
}

impl Default for MermaidFormatOpts<'_> {
    fn default() -> Self {
        Self {
            hide_nodes: false,
            monochrome: false,
            theme: MermaidTheme::default(),
            orientation: MermaidOrientation::default(),
            highlighting_target: HashSet::new(),
            context: FormatContextOpt::Global,
        }
    }
}

impl<'a> MermaidFormatOpts<'a> {
    /// Sets whether to hide NODE identifiers in the tree representation
    /// (default is false).
    pub fn hide_nodes(mut self, hide: bool) -> Self {
        self.hide_nodes = hide;
        self
    }

    /// When set to true, the tree representation will use a monochrome
    /// color scheme (default is false).
    pub fn monochrome(mut self, monochrome: bool) -> Self {
        self.monochrome = monochrome;
        self
    }

    /// Sets the theme for the tree representation (default is Default).
    pub fn theme(mut self, theme: MermaidTheme) -> Self {
        self.theme = theme;
        self
    }

    /// Sets the orientation of the tree representation (default is
    /// LeftToRight).
    pub fn orientation(mut self, orientation: MermaidOrientation) -> Self {
        self.orientation = orientation;
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

    pub fn mermaid_format_opt(&self, opts: &MermaidFormatOpts<'_>) -> String {
        let elements: RefCell<Vec<Rc<MermaidElement>>> =
            RefCell::new(Vec::new());
        let next_id = RefCell::new(0);
        let visitor = |envelope: &Envelope,
                       level: usize,
                       incoming_edge: EdgeType,
                       parent: Option<Rc<MermaidElement>>|
         -> (_, bool) {
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
            (Some(elem), false)
        };
        let s = self.clone();
        s.walk(opts.hide_nodes, None, &visitor);

        let elements = elements.borrow();

        let mut element_ids: HashSet<usize> =
            elements.iter().map(|e| e.id).collect();

        let mut lines = vec![
            format!(
                "%%{{ init: {{ 'theme': '{}', 'flowchart': {{ 'curve': 'basis' }} }} }}%%",
                opts.theme
            ),
            format!("graph {}", opts.orientation),
        ];

        let mut node_styles: Vec<String> = Vec::new();
        let mut link_styles: Vec<String> = Vec::new();
        let mut link_index = 0;

        for element in elements.iter() {
            let indent = "    ".repeat(element.level);
            let content = if let Some(parent) = element.parent.as_ref() {
                let mut this_link_styles = Vec::new();
                if !opts.monochrome {
                    if let Some(color) =
                        element.incoming_edge.link_stroke_color()
                    {
                        this_link_styles.push(format!("stroke:{}", color));
                    }
                }
                if element.is_highlighted && parent.is_highlighted {
                    this_link_styles.push("stroke-width:4px".to_string());
                } else {
                    this_link_styles.push("stroke-width:2px".to_string());
                }
                if !this_link_styles.is_empty() {
                    link_styles.push(format!(
                        "linkStyle {} {}",
                        link_index,
                        this_link_styles.join(",")
                    ));
                }
                link_index += 1;
                element.format_edge(&mut element_ids)
            } else {
                element.format_node(&mut element_ids)
            };
            let mut this_node_styles = Vec::new();
            if !opts.monochrome {
                let stroke_color = element.envelope.node_color();
                this_node_styles.push(format!("stroke:{}", stroke_color));
            }
            if element.is_highlighted {
                this_node_styles.push("stroke-width:6px".to_string());
            } else {
                this_node_styles.push("stroke-width:4px".to_string());
            }
            if !this_node_styles.is_empty() {
                node_styles.push(format!(
                    "style {} {}",
                    element.id,
                    this_node_styles.join(",")
                ));
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
    /// The parent element in the tree, if any
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
            let mut lines: Vec<String> = Vec::new();
            let summary = with_format_context!(|ctx| {
                self.envelope
                    .summary(20, ctx)
                    .replace('"', "&quot;")
                    .to_string()
            });
            lines.push(summary);
            if self.show_id {
                let id = self.envelope.digest().short_description();
                lines.push(id);
            }
            let lines = lines.join("<br>");
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
            #[cfg(feature = "known_value")]
            EnvelopeCase::KnownValue { .. } => ("[/", "/]"),
            #[cfg(feature = "encrypt")]
            EnvelopeCase::Encrypted(..)     => (">",  "]"),
            #[cfg(feature = "compress")]
            EnvelopeCase::Compressed(..)    => ("[[", "]]"),
        }
    }

    #[rustfmt::skip]
    fn node_color(&self) -> &'static str {
        match self.case() {
            EnvelopeCase::Node { .. }       => "red",
            EnvelopeCase::Leaf { .. }       => "teal",
            EnvelopeCase::Wrapped { .. }    => "blue",
            EnvelopeCase::Assertion(..)     => "green",
            EnvelopeCase::Elided(..)        => "gray",
            #[cfg(feature = "known_value")]
            EnvelopeCase::KnownValue { .. } => "goldenrod",
            #[cfg(feature = "encrypt")]
            EnvelopeCase::Encrypted(..)     => "coral",
            #[cfg(feature = "compress")]
            EnvelopeCase::Compressed(..)    => "purple",
        }
    }
}

impl EdgeType {
    pub fn link_stroke_color(&self) -> Option<&'static str> {
        match self {
            EdgeType::Subject  => Some("red"),
            EdgeType::Content => Some("blue"),
            EdgeType::Predicate => Some("cyan"),
            EdgeType::Object => Some("magenta"),
            _ => None,
        }
    }
}
