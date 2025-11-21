//! Functions for traversing and manipulating the envelope hierarchy.
//!
//! This module provides functionality for traversing the hierarchical structure
//! of envelopes, allowing for operations such as inspection, transformation,
//! and extraction of specific elements. It implements a visitor pattern that
//! enables executing arbitrary code on each element of an envelope in a
//! structured way.
//!
//! The traversal can be performed in two modes:
//! - Structure-based traversal: Visits every element in the envelope hierarchy
//! - Tree-based traversal: Skips node elements and focuses on the semantic
//!   content
//!
//! # Examples
//!
//! ```
//! use std::{cell::RefCell, collections::HashSet};
//!
//! use bc_envelope::prelude::*;
//!
//! // Create an envelope with nested structure
//! let envelope = Envelope::new("Alice")
//!     .add_assertion("knows", "Bob")
//!     .add_assertion("email", "alice@example.com");
//!
//! // Collect all digests in the envelope by walking its structure
//! let digests = RefCell::new(HashSet::new());
//! let visitor = |env: &Envelope,
//!                _level: usize,
//!                _edge: EdgeType,
//!                state: ()|
//!  -> ((), bool) {
//!     digests.borrow_mut().insert(env.digest());
//!     (state, false)
//! };
//!
//! // Walk the entire envelope structure
//! envelope.walk(false, (), &visitor);
//!
//! // All elements of the envelope will have their digests collected
//! assert!(digests.borrow().len() > 0);
//! ```

use super::envelope::EnvelopeCase;
use crate::Envelope;

/// The type of incoming edge provided to the visitor.
///
/// This enum identifies how an envelope element is connected to its parent in
/// the hierarchy during traversal. It helps the visitor function understand the
/// semantic relationship between elements.
///
/// Each edge type represents a specific relationship within the envelope
/// structure:
/// - `None`: Root or no connection
/// - `Subject`: Element is the subject of its parent node
/// - `Assertion`: Element is an assertion on its parent node
/// - `Predicate`: Element is the predicate of an assertion
/// - `Object`: Element is the object of an assertion
/// - `Wrapped`: Element is wrapped by its parent
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum EdgeType {
    /// No incoming edge (root)
    None,
    /// Element is the subject of a node
    Subject,
    /// Element is an assertion on a node
    Assertion,
    /// Element is the predicate of an assertion
    Predicate,
    /// Element is the object of an assertion
    Object,
    /// Element is the content wrapped by another envelope
    Content,
}

/// Provides a label for the edge type in tree formatting.
impl EdgeType {
    /// Returns a short text label for the edge type, or None if no label is
    /// needed.
    ///
    /// This is primarily used for tree formatting to identify relationships
    /// between elements.
    ///
    /// # Examples
    ///
    /// ```
    /// # use bc_envelope::prelude::*;
    /// assert_eq!(EdgeType::Subject.label(), Some("subj"));
    /// assert_eq!(EdgeType::Content.label(), Some("cont"));
    /// assert_eq!(EdgeType::Predicate.label(), Some("pred"));
    /// assert_eq!(EdgeType::Object.label(), Some("obj"));
    /// assert_eq!(EdgeType::Assertion.label(), None);
    /// ```
    pub fn label(&self) -> Option<&'static str> {
        match self {
            EdgeType::Subject => Some("subj"),
            EdgeType::Content => Some("cont"),
            EdgeType::Predicate => Some("pred"),
            EdgeType::Object => Some("obj"),
            _ => None,
        }
    }
}

/// A visitor function that is called for each element in the envelope.
///
/// The visitor function takes the following parameters:
/// - `envelope`: The current envelope element being visited
/// - `level`: The depth level in the hierarchy (0 for root)
/// - `incoming_edge`: The type of edge connecting this element to its parent
/// - `state`: Optional context passed down from the parent's visitor call
///
/// The visitor returns a state that will be passed to child elements.
/// This enables accumulating state or passing context during traversal.
///
/// # Type Parameters
///
/// * `State` - The type of context passed between parent and child elements
pub type Visitor<'a, State> =
    dyn Fn(&Envelope, usize, EdgeType, State) -> (State, bool) + 'a;

/// Functions for traversing and manipulating the envelope hierarchy.
impl Envelope {
    /// Walks the envelope structure, calling the visitor function for each
    /// element.
    ///
    /// This function traverses the entire envelope hierarchy and calls the
    /// visitor function on each element. The traversal can be performed in
    /// two modes:
    ///
    /// - Structure-based traversal (`hide_nodes = false`): Visits every element
    ///   including node containers
    /// - Tree-based traversal (`hide_nodes = true`): Skips node elements and
    ///   focuses on semantic content
    ///
    /// The visitor function can optionally return a context value that is
    /// passed to child elements, enabling state to be accumulated or passed
    /// down during traversal.
    ///
    /// # Type Parameters
    ///
    /// * `Parent` - The type of context passed between parent and child
    ///   elements
    ///
    /// # Arguments
    ///
    /// * `hide_nodes` - If true, the visitor will not be called for node
    ///   containers
    /// * `visit` - The visitor function called for each element
    ///
    /// # Examples
    ///
    /// ```
    /// use std::cell::RefCell;
    ///
    /// use bc_envelope::prelude::*;
    ///
    /// // Create an envelope with nested structure
    /// let envelope = Envelope::new("Alice").add_assertion("knows", "Bob");
    ///
    /// // Count the number of elements in the envelope
    /// let count = RefCell::new(0);
    /// let visitor = |_env: &Envelope,
    ///                _level: usize,
    ///                _edge: EdgeType,
    ///                state: ()|
    ///  -> ((), bool) {
    ///     *count.borrow_mut() += 1;
    ///     (state, false)
    /// };
    ///
    /// // Walk the entire envelope structure
    /// envelope.walk(false, (), &visitor);
    /// assert!(*count.borrow() > 0);
    /// ```
    pub fn walk<State: Clone>(
        &self,
        hide_nodes: bool,
        state: State,
        visit: &Visitor<'_, State>,
    ) {
        if hide_nodes {
            self.walk_tree(state, visit)
        } else {
            self.walk_structure(state, visit)
        }
    }

    /// Walks the complete structure of the envelope, visiting every element.
    ///
    /// This is an internal method that begins a structure-based traversal from
    /// the root level. Use the public `walk` method with `hide_nodes =
    /// false` instead of calling this directly.
    fn walk_structure<State: Clone>(
        &self,
        state: State,
        visit: &Visitor<'_, State>,
    ) {
        self._walk_structure(0, EdgeType::None, state, visit)
    }

    /// Recursive implementation of structure-based traversal.
    ///
    /// This internal method performs the actual recursive traversal of the
    /// envelope structure, visiting every element and maintaining the
    /// correct level and edge relationships.
    fn _walk_structure<State: Clone>(
        &self,
        level: usize,
        incoming_edge: EdgeType,
        state: State,
        visit: &Visitor<'_, State>,
    ) {
        let mut state = state;
        let stop;
        (state, stop) = visit(self, level, incoming_edge, state);
        if stop {
            return;
        }
        let next_level = level + 1;
        match self.case() {
            EnvelopeCase::Node { subject, assertions, .. } => {
                subject._walk_structure(
                    next_level,
                    EdgeType::Subject,
                    state.clone(),
                    visit,
                );
                for assertion in assertions {
                    assertion._walk_structure(
                        next_level,
                        EdgeType::Assertion,
                        state.clone(),
                        visit,
                    );
                }
            }
            EnvelopeCase::Wrapped { envelope, .. } => {
                envelope._walk_structure(
                    next_level,
                    EdgeType::Content,
                    state,
                    visit,
                );
            }
            EnvelopeCase::Assertion(assertion) => {
                assertion.predicate()._walk_structure(
                    next_level,
                    EdgeType::Predicate,
                    state.clone(),
                    visit,
                );
                assertion.object()._walk_structure(
                    next_level,
                    EdgeType::Object,
                    state,
                    visit,
                );
            }
            _ => {}
        }
    }

    /// Walks the envelope's semantic tree, skipping node containers.
    ///
    /// This is an internal method that begins a tree-based traversal from the
    /// root level. Use the public `walk` method with `hide_nodes = true`
    /// instead of calling this directly.
    fn walk_tree<State: Clone>(
        &self,
        state: State,
        visit: &Visitor<'_, State>,
    ) {
        _ = self._walk_tree(0, EdgeType::None, state, visit)
    }

    /// Recursive implementation of tree-based traversal.
    ///
    /// This internal method performs the actual recursive traversal of the
    /// envelope's semantic tree, skipping node containers and focusing on
    /// the semantic content elements. It maintains the correct level and
    /// edge relationships while skipping structural elements.
    fn _walk_tree<State: Clone>(
        &self,
        level: usize,
        incoming_edge: EdgeType,
        state: State,
        visit: &Visitor<'_, State>,
    ) -> State {
        let mut state = state;
        let mut subject_level = level;
        if !self.is_node() {
            let stop;
            (state, stop) = visit(self, level, incoming_edge, state);
            if stop {
                return state;
            }
            subject_level = level + 1;
        }
        match self.case() {
            EnvelopeCase::Node { subject, assertions, .. } => {
                let assertion_state = subject._walk_tree(
                    subject_level,
                    EdgeType::Subject,
                    state.clone(),
                    visit,
                );
                let assertion_level = subject_level + 1;
                for assertion in assertions {
                    assertion._walk_tree(
                        assertion_level,
                        EdgeType::Assertion,
                        assertion_state.clone(),
                        visit,
                    );
                }
            }
            EnvelopeCase::Wrapped { envelope, .. } => {
                envelope._walk_tree(
                    subject_level,
                    EdgeType::Content,
                    state.clone(),
                    visit,
                );
            }
            EnvelopeCase::Assertion(assertion) => {
                assertion.predicate()._walk_tree(
                    subject_level,
                    EdgeType::Predicate,
                    state.clone(),
                    visit,
                );
                assertion.object()._walk_tree(
                    subject_level,
                    EdgeType::Object,
                    state.clone(),
                    visit,
                );
            }
            _ => {}
        }
        state
    }
}
