use crate::{Envelope, Error, Result, known_values};

/// Methods for working with edge envelopes on documents.
impl Envelope {
    /// Returns a new envelope with an added `'edge': <edge>` assertion.
    pub fn add_edge_envelope(&self, edge: Self) -> Self {
        self.add_assertion(known_values::EDGE, edge)
    }

    /// Returns all edge object envelopes (assertions with predicate `'edge'`).
    pub fn edges(&self) -> Result<Vec<Self>> {
        Ok(self.objects_for_predicate(known_values::EDGE))
    }

    /// Validates an edge envelope's structure per BCR-2026-003.
    ///
    /// An edge may be wrapped (signed) or unwrapped. The inner envelope
    /// must have exactly three assertion predicates: `'isA'`, `'source'`,
    /// and `'target'`.
    pub fn validate_edge(&self) -> Result<()> {
        let inner = if self.subject().is_wrapped() {
            self.subject().try_unwrap()?
        } else {
            self.clone()
        };

        let is_a_count =
            inner.assertions_with_predicate(known_values::IS_A).len();
        let source_count =
            inner.assertions_with_predicate(known_values::SOURCE).len();
        let target_count =
            inner.assertions_with_predicate(known_values::TARGET).len();

        if is_a_count == 0 {
            return Err(Error::EdgeMissingIsA);
        }
        if source_count == 0 {
            return Err(Error::EdgeMissingSource);
        }
        if target_count == 0 {
            return Err(Error::EdgeMissingTarget);
        }
        if is_a_count > 1 {
            return Err(Error::EdgeDuplicateIsA);
        }
        if source_count > 1 {
            return Err(Error::EdgeDuplicateSource);
        }
        if target_count > 1 {
            return Err(Error::EdgeDuplicateTarget);
        }

        Ok(())
    }

    /// Extracts the `'isA'` assertion object from an edge envelope.
    pub fn edge_is_a(&self) -> Result<Self> {
        let inner = if self.subject().is_wrapped() {
            self.subject().try_unwrap()?
        } else {
            self.clone()
        };
        inner.object_for_predicate(known_values::IS_A)
    }

    /// Extracts the `'source'` assertion object from an edge envelope.
    pub fn edge_source(&self) -> Result<Self> {
        let inner = if self.subject().is_wrapped() {
            self.subject().try_unwrap()?
        } else {
            self.clone()
        };
        inner.object_for_predicate(known_values::SOURCE)
    }

    /// Extracts the `'target'` assertion object from an edge envelope.
    pub fn edge_target(&self) -> Result<Self> {
        let inner = if self.subject().is_wrapped() {
            self.subject().try_unwrap()?
        } else {
            self.clone()
        };
        inner.object_for_predicate(known_values::TARGET)
    }

    /// Extracts the edge's subject identifier (the inner envelope's subject).
    pub fn edge_subject(&self) -> Result<Self> {
        let inner = if self.subject().is_wrapped() {
            self.subject().try_unwrap()?
        } else {
            self.clone()
        };
        Ok(inner.subject())
    }

    /// Filters edges by optional criteria.
    ///
    /// Each parameter is optional. When provided, only edges matching
    /// all specified criteria are returned.
    pub fn edges_matching(
        &self,
        is_a: Option<&Self>,
        source: Option<&Self>,
        target: Option<&Self>,
        subject: Option<&Self>,
    ) -> Result<Vec<Self>> {
        let all_edges = self.edges()?;
        let mut matching = Vec::new();

        for edge in all_edges {
            if let Some(is_a_filter) = is_a {
                if let Ok(edge_is_a) = edge.edge_is_a() {
                    if !edge_is_a.is_equivalent_to(is_a_filter) {
                        continue;
                    }
                } else {
                    continue;
                }
            }

            if let Some(source_filter) = source {
                if let Ok(edge_source) = edge.edge_source() {
                    if !edge_source.is_equivalent_to(source_filter) {
                        continue;
                    }
                } else {
                    continue;
                }
            }

            if let Some(target_filter) = target {
                if let Ok(edge_target) = edge.edge_target() {
                    if !edge_target.is_equivalent_to(target_filter) {
                        continue;
                    }
                } else {
                    continue;
                }
            }

            if let Some(subject_filter) = subject {
                if let Ok(edge_subject) = edge.edge_subject() {
                    if !edge_subject.is_equivalent_to(subject_filter) {
                        continue;
                    }
                } else {
                    continue;
                }
            }

            matching.push(edge);
        }

        Ok(matching)
    }
}
