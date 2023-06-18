use std::{rc::Rc, collections::HashSet, cell::RefCell};

use bc_components::{Digest, DigestProvider};

use crate::{Envelope, FormatContext, walk::EdgeType};

impl Envelope {
    pub fn tree_format(self: Rc<Envelope>, hide_nodes: bool, context: Option<&FormatContext>) -> String {
        self.tree_format_with_target(hide_nodes, &HashSet::new(), context)
    }

    pub fn tree_format_with_target(self: Rc<Envelope>, hide_nodes: bool, highlighting_target: &HashSet<Digest>, context: Option<&FormatContext>) -> String {
        let elements: RefCell<Vec<TreeElement>> = RefCell::new(Vec::new());
        let visitor = |envelope: Rc<Self>, level: usize, incoming_edge: EdgeType, _: Option<&()>| -> _ {
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
        self.walk(hide_nodes, &visitor);
        let elements = elements.borrow();
        elements.iter().map(|e| e.string(context.unwrap_or(&FormatContext::default()))).collect::<Vec<_>>().join("\n")
    }
}

struct TreeElement {
    level: usize,
    envelope: Rc<Envelope>,
    incoming_edge: EdgeType,
    show_id: bool,
    is_highlighted: bool,
}

impl TreeElement {
    fn new(level: usize, envelope: Rc<Envelope>, incoming_edge: EdgeType, show_id: bool, is_highlighted: bool) -> Self {
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
