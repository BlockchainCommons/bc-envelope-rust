use std::collections::HashMap;

use bc_components::{Digest, DigestProvider};

use crate::{Envelope, Result};

/// A container for edge envelopes on a document.
///
/// `Edges` stores pre-constructed edge envelopes keyed by their digest,
/// mirroring the `Attachments` container but for edges as defined in
/// BCR-2026-003.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Edges {
    envelopes: HashMap<Digest, Envelope>,
}

impl Default for Edges {
    fn default() -> Self { Self::new() }
}

impl Edges {
    /// Creates a new empty edges container.
    pub fn new() -> Self { Self { envelopes: HashMap::new() } }

    /// Adds a pre-constructed edge envelope.
    pub fn add(&mut self, edge_envelope: Envelope) {
        let digest = edge_envelope.digest();
        self.envelopes.insert(digest, edge_envelope);
    }

    /// Retrieves an edge by its digest.
    pub fn get(&self, digest: Digest) -> Option<&Envelope> {
        self.envelopes.get(&digest)
    }

    /// Removes an edge by its digest.
    pub fn remove(&mut self, digest: Digest) -> Option<Envelope> {
        self.envelopes.remove(&digest)
    }

    /// Removes all edges from the container.
    pub fn clear(&mut self) { self.envelopes.clear(); }

    /// Returns whether the container has any edges.
    pub fn is_empty(&self) -> bool { self.envelopes.is_empty() }

    /// Returns the number of edges in the container.
    pub fn len(&self) -> usize { self.envelopes.len() }

    /// Returns an iterator over all edge envelopes.
    pub fn iter(&self) -> impl Iterator<Item = (&Digest, &Envelope)> {
        self.envelopes.iter()
    }

    /// Adds all edges as `'edge'` assertion envelopes to the given envelope.
    pub fn add_to_envelope(&self, envelope: Envelope) -> Envelope {
        let mut new_envelope = envelope;
        for (_digest, edge_envelope) in self.envelopes.iter() {
            new_envelope = new_envelope
                .add_assertion(known_values::EDGE, edge_envelope.clone());
        }
        new_envelope
    }

    /// Extracts edges from an envelope's `'edge'` assertions.
    pub fn try_from_envelope(envelope: &Envelope) -> Result<Edges> {
        let edge_envelopes = envelope.edges()?;
        let mut edges = Edges::new();
        for edge in edge_envelopes {
            let digest = edge.digest();
            edges.envelopes.insert(digest, edge);
        }
        Ok(edges)
    }
}

/// A trait for types that can have edges.
///
/// `Edgeable` provides a consistent interface for working with edges.
/// Types implementing this trait can store and retrieve edge envelopes
/// representing verifiable claims as defined in BCR-2026-003.
#[allow(dead_code)]
pub trait Edgeable {
    /// Returns a reference to the edges container.
    fn edges(&self) -> &Edges;

    /// Returns a mutable reference to the edges container.
    fn edges_mut(&mut self) -> &mut Edges;

    /// Adds a pre-constructed edge envelope.
    fn add_edge(&mut self, edge_envelope: Envelope) {
        self.edges_mut().add(edge_envelope);
    }

    /// Retrieves an edge by its digest.
    fn get_edge(&self, digest: Digest) -> Option<&Envelope> {
        self.edges().get(digest)
    }

    /// Removes an edge by its digest.
    fn remove_edge(&mut self, digest: Digest) -> Option<Envelope> {
        self.edges_mut().remove(digest)
    }

    /// Removes all edges.
    fn clear_edges(&mut self) { self.edges_mut().clear(); }

    /// Returns whether the object has any edges.
    fn has_edges(&self) -> bool { !self.edges().is_empty() }
}

use crate::known_values;
