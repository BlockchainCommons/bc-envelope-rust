#![cfg(feature = "edge")]

use bc_components::DigestProvider;
use bc_envelope::prelude::*;
use indoc::indoc;

mod common;
use crate::common::test_data::*;

/// Helper to create a basic edge envelope with the three required assertions.
fn make_edge(
    subject: &str,
    is_a: &str,
    source: &Envelope,
    target: &Envelope,
) -> Envelope {
    Envelope::new(subject)
        .add_assertion(known_values::IS_A, is_a)
        .add_assertion(known_values::SOURCE, source.clone())
        .add_assertion(known_values::TARGET, target.clone())
}

/// Helper to create an XID-like identifier envelope.
fn xid_like(name: &str) -> Envelope {
    Envelope::new(name)
}

// -------------------------------------------------------------------
// Edge construction and format
// -------------------------------------------------------------------

#[test]
fn test_edge_basic_format() {
    let alice = xid_like("Alice");
    let edge = make_edge("credential-1", "foaf:Person", &alice, &alice);

    #[rustfmt::skip]
    assert_actual_expected!(edge.format(), indoc! {r#"
        "credential-1" [
            'isA': "foaf:Person"
            'source': "Alice"
            'target': "Alice"
        ]
    "#}.trim());
}

#[test]
fn test_edge_relationship_format() {
    let alice = xid_like("Alice");
    let bob = xid_like("Bob");
    let edge = make_edge("knows-bob", "schema:colleague", &alice, &bob);

    #[rustfmt::skip]
    assert_actual_expected!(edge.format(), indoc! {r#"
        "knows-bob" [
            'isA': "schema:colleague"
            'source': "Alice"
            'target': "Bob"
        ]
    "#}.trim());
}

// -------------------------------------------------------------------
// Edge validation
// -------------------------------------------------------------------

#[test]
fn test_validate_edge_valid() {
    let alice = xid_like("Alice");
    let edge = make_edge("cred-1", "foaf:Person", &alice, &alice);
    assert!(edge.validate_edge().is_ok());
}

#[test]
fn test_validate_edge_missing_is_a() {
    let alice = xid_like("Alice");
    let edge = Envelope::new("cred-1")
        .add_assertion(known_values::SOURCE, alice.clone())
        .add_assertion(known_values::TARGET, alice.clone());
    let result = edge.validate_edge();
    assert!(matches!(result, Err(EnvelopeError::EdgeMissingIsA)));
}

#[test]
fn test_validate_edge_missing_source() {
    let alice = xid_like("Alice");
    let edge = Envelope::new("cred-1")
        .add_assertion(known_values::IS_A, "foaf:Person")
        .add_assertion(known_values::TARGET, alice.clone());
    let result = edge.validate_edge();
    assert!(matches!(result, Err(EnvelopeError::EdgeMissingSource)));
}

#[test]
fn test_validate_edge_missing_target() {
    let alice = xid_like("Alice");
    let edge = Envelope::new("cred-1")
        .add_assertion(known_values::IS_A, "foaf:Person")
        .add_assertion(known_values::SOURCE, alice.clone());
    let result = edge.validate_edge();
    assert!(matches!(result, Err(EnvelopeError::EdgeMissingTarget)));
}

#[test]
fn test_validate_edge_no_assertions() {
    let edge = Envelope::new("cred-1");
    let result = edge.validate_edge();
    assert!(matches!(result, Err(EnvelopeError::EdgeMissingIsA)));
}

#[test]
fn test_validate_edge_duplicate_is_a() {
    let alice = xid_like("Alice");
    let edge = Envelope::new("cred-1")
        .add_assertion(known_values::IS_A, "foaf:Person")
        .add_assertion(known_values::IS_A, "schema:Thing")
        .add_assertion(known_values::SOURCE, alice.clone())
        .add_assertion(known_values::TARGET, alice.clone());
    let result = edge.validate_edge();
    assert!(matches!(result, Err(EnvelopeError::EdgeDuplicateIsA)));
}

#[test]
fn test_validate_edge_duplicate_source() {
    let alice = xid_like("Alice");
    let bob = xid_like("Bob");
    let edge = Envelope::new("cred-1")
        .add_assertion(known_values::IS_A, "foaf:Person")
        .add_assertion(known_values::SOURCE, alice.clone())
        .add_assertion(known_values::SOURCE, bob.clone())
        .add_assertion(known_values::TARGET, alice.clone());
    let result = edge.validate_edge();
    assert!(matches!(result, Err(EnvelopeError::EdgeDuplicateSource)));
}

#[test]
fn test_validate_edge_duplicate_target() {
    let alice = xid_like("Alice");
    let bob = xid_like("Bob");
    let edge = Envelope::new("cred-1")
        .add_assertion(known_values::IS_A, "foaf:Person")
        .add_assertion(known_values::SOURCE, alice.clone())
        .add_assertion(known_values::TARGET, alice.clone())
        .add_assertion(known_values::TARGET, bob.clone());
    let result = edge.validate_edge();
    assert!(matches!(result, Err(EnvelopeError::EdgeDuplicateTarget)));
}

#[test]
fn test_validate_edge_wrapped_signed() {
    let alice = xid_like("Alice");
    let edge = make_edge("cred-1", "foaf:Person", &alice, &alice);

    // Wrap and sign the edge
    let signed_edge = edge.wrap().add_signature(&alice_private_key());

    // Signed (wrapped) edge should still validate
    assert!(signed_edge.validate_edge().is_ok());
}

// -------------------------------------------------------------------
// Edge accessor methods
// -------------------------------------------------------------------

#[test]
fn test_edge_is_a() -> Result<(), EnvelopeError> {
    let alice = xid_like("Alice");
    let edge = make_edge("cred-1", "foaf:Person", &alice, &alice);

    let is_a = edge.edge_is_a()?;
    assert_actual_expected!(is_a.format(), r#""foaf:Person""#);
    Ok(())
}

#[test]
fn test_edge_source() -> Result<(), EnvelopeError> {
    let alice = xid_like("Alice");
    let edge = make_edge("cred-1", "foaf:Person", &alice, &alice);

    let source = edge.edge_source()?;
    assert_actual_expected!(source.format(), r#""Alice""#);
    Ok(())
}

#[test]
fn test_edge_target() -> Result<(), EnvelopeError> {
    let alice = xid_like("Alice");
    let bob = xid_like("Bob");
    let edge = make_edge("knows-bob", "schema:colleague", &alice, &bob);

    let target = edge.edge_target()?;
    assert_actual_expected!(target.format(), r#""Bob""#);
    Ok(())
}

#[test]
fn test_edge_subject() -> Result<(), EnvelopeError> {
    let alice = xid_like("Alice");
    let edge = make_edge("my-credential", "foaf:Person", &alice, &alice);

    let subject = edge.edge_subject()?;
    assert_actual_expected!(subject.format(), r#""my-credential""#);
    Ok(())
}

#[test]
fn test_edge_accessors_on_signed_edge() -> Result<(), EnvelopeError> {
    let alice = xid_like("Alice");
    let bob = xid_like("Bob");
    let edge = make_edge("cred-1", "foaf:Person", &alice, &bob);

    let signed_edge = edge.wrap().add_signature(&alice_private_key());

    // Accessors should work through the wrapped/signed layer
    let is_a = signed_edge.edge_is_a()?;
    assert_actual_expected!(is_a.format(), r#""foaf:Person""#);

    let source = signed_edge.edge_source()?;
    assert_actual_expected!(source.format(), r#""Alice""#);

    let target = signed_edge.edge_target()?;
    assert_actual_expected!(target.format(), r#""Bob""#);

    let subject = signed_edge.edge_subject()?;
    assert_actual_expected!(subject.format(), r#""cred-1""#);

    Ok(())
}

// -------------------------------------------------------------------
// Adding edges to envelopes
// -------------------------------------------------------------------

#[test]
fn test_add_edge_envelope() {
    let alice = xid_like("Alice");
    let edge = make_edge("cred-1", "foaf:Person", &alice, &alice);

    let doc = Envelope::new("Alice").add_edge_envelope(edge);

    #[rustfmt::skip]
    assert_actual_expected!(doc.format(), indoc! {r#"
        "Alice" [
            'edge': "cred-1" [
                'isA': "foaf:Person"
                'source': "Alice"
                'target': "Alice"
            ]
        ]
    "#}.trim());
}

#[test]
fn test_add_multiple_edges() {
    let alice = xid_like("Alice");
    let bob = xid_like("Bob");
    let edge1 = make_edge("self-desc", "foaf:Person", &alice, &alice);
    let edge2 = make_edge("knows-bob", "schema:colleague", &alice, &bob);

    let doc = Envelope::new("Alice")
        .add_edge_envelope(edge1)
        .add_edge_envelope(edge2);

    let edges = doc.edges().unwrap();
    assert_eq!(edges.len(), 2);

    let formatted = doc.format();
    assert!(formatted.contains("'edge'"));
    assert!(formatted.contains("\"self-desc\""));
    assert!(formatted.contains("\"knows-bob\""));
}

// -------------------------------------------------------------------
// Edges retrieval via envelope
// -------------------------------------------------------------------

#[test]
fn test_edges_empty() -> Result<(), EnvelopeError> {
    let doc = Envelope::new("Alice");
    let edges = doc.edges()?;
    assert_eq!(edges.len(), 0);
    Ok(())
}

#[test]
fn test_edges_retrieval() -> Result<(), EnvelopeError> {
    let alice = xid_like("Alice");
    let edge1 = make_edge("cred-1", "foaf:Person", &alice, &alice);
    let edge2 = make_edge("cred-2", "schema:Thing", &alice, &alice);

    let doc = Envelope::new("Alice")
        .add_edge_envelope(edge1)
        .add_edge_envelope(edge2);

    let edges = doc.edges()?;
    assert_eq!(edges.len(), 2);

    // Each retrieved edge should be a valid edge
    for edge in &edges {
        edge.validate_edge()?;
    }

    Ok(())
}

// -------------------------------------------------------------------
// Edges container (add / get / remove / clear / len)
// -------------------------------------------------------------------

#[test]
fn test_edges_container_new_is_empty() {
    let edges = Edges::new();
    assert!(edges.is_empty());
    assert_eq!(edges.len(), 0);
}

#[test]
fn test_edges_container_add_and_get() {
    let alice = xid_like("Alice");
    let edge = make_edge("cred-1", "foaf:Person", &alice, &alice);
    let digest = edge.digest();

    let mut edges = Edges::new();
    edges.add(edge.clone());

    assert!(!edges.is_empty());
    assert_eq!(edges.len(), 1);
    assert!(edges.get(digest).is_some());
    assert!(edges.get(digest).unwrap().is_equivalent_to(&edge));
}

#[test]
fn test_edges_container_remove() {
    let alice = xid_like("Alice");
    let edge = make_edge("cred-1", "foaf:Person", &alice, &alice);
    let digest = edge.digest();

    let mut edges = Edges::new();
    edges.add(edge);

    let removed = edges.remove(digest);
    assert!(removed.is_some());
    assert!(edges.is_empty());
}

#[test]
fn test_edges_container_remove_nonexistent() {
    let alice = xid_like("Alice");
    let edge = make_edge("cred-1", "foaf:Person", &alice, &alice);

    let mut edges = Edges::new();
    let removed = edges.remove(edge.digest());
    assert!(removed.is_none());
}

#[test]
fn test_edges_container_clear() {
    let alice = xid_like("Alice");
    let edge1 = make_edge("cred-1", "foaf:Person", &alice, &alice);
    let edge2 = make_edge("cred-2", "schema:Thing", &alice, &alice);

    let mut edges = Edges::new();
    edges.add(edge1);
    edges.add(edge2);
    assert_eq!(edges.len(), 2);

    edges.clear();
    assert!(edges.is_empty());
    assert_eq!(edges.len(), 0);
}

#[test]
fn test_edges_container_iter() {
    let alice = xid_like("Alice");
    let edge1 = make_edge("cred-1", "foaf:Person", &alice, &alice);
    let edge2 = make_edge("cred-2", "schema:Thing", &alice, &alice);

    let mut edges = Edges::new();
    edges.add(edge1);
    edges.add(edge2);

    let count = edges.iter().count();
    assert_eq!(count, 2);
}

// -------------------------------------------------------------------
// Edges container round-trip: add_to_envelope / try_from_envelope
// -------------------------------------------------------------------

#[test]
fn test_edges_container_roundtrip() -> Result<(), EnvelopeError> {
    let alice = xid_like("Alice");
    let edge1 = make_edge("cred-1", "foaf:Person", &alice, &alice);
    let edge2 = make_edge("cred-2", "schema:Thing", &alice, &alice);

    let mut edges = Edges::new();
    edges.add(edge1.clone());
    edges.add(edge2.clone());

    // Serialize to envelope
    let doc = Envelope::new("Alice");
    let doc_with_edges = edges.add_to_envelope(doc);

    // Deserialize back
    let recovered = Edges::try_from_envelope(&doc_with_edges)?;
    assert_eq!(recovered.len(), 2);
    assert!(recovered.get(edge1.digest()).is_some());
    assert!(recovered.get(edge2.digest()).is_some());

    Ok(())
}

#[test]
fn test_edges_container_roundtrip_empty() -> Result<(), EnvelopeError> {
    let edges = Edges::new();
    let doc = Envelope::new("Alice");
    let doc_with_edges = edges.add_to_envelope(doc);

    let recovered = Edges::try_from_envelope(&doc_with_edges)?;
    assert!(recovered.is_empty());

    Ok(())
}

#[test]
fn test_edges_container_roundtrip_preserves_format() -> Result<(), EnvelopeError> {
    let alice = xid_like("Alice");
    let bob = xid_like("Bob");
    let edge = make_edge("knows-bob", "schema:colleague", &alice, &bob);

    let mut edges = Edges::new();
    edges.add(edge);

    let doc = edges.add_to_envelope(Envelope::new("Alice"));

    #[rustfmt::skip]
    assert_actual_expected!(doc.format(), indoc! {r#"
        "Alice" [
            'edge': "knows-bob" [
                'isA': "schema:colleague"
                'source': "Alice"
                'target': "Bob"
            ]
        ]
    "#}.trim());

    let recovered = Edges::try_from_envelope(&doc)?;
    assert_eq!(recovered.len(), 1);

    Ok(())
}

// -------------------------------------------------------------------
// Edgeable trait
// -------------------------------------------------------------------

#[test]
fn test_edgeable_default_methods() {
    // Use a struct that we can manually wrap — in practice this is used
    // via impl_edgeable! on XIDDocument, but we test the trait methods
    // directly on the Edges container to verify behavior.
    let alice = xid_like("Alice");
    let edge = make_edge("cred-1", "foaf:Person", &alice, &alice);
    let digest = edge.digest();

    let mut edges = Edges::new();
    edges.add(edge);

    assert!(!edges.is_empty());
    assert_eq!(edges.len(), 1);
    assert!(edges.get(digest).is_some());

    let removed = edges.remove(digest);
    assert!(removed.is_some());
    assert!(edges.is_empty());
}

// -------------------------------------------------------------------
// edges_matching — filtering by criteria
// -------------------------------------------------------------------

#[test]
fn test_edges_matching_no_filters() -> Result<(), EnvelopeError> {
    let alice = xid_like("Alice");
    let bob = xid_like("Bob");
    let edge1 = make_edge("self-desc", "foaf:Person", &alice, &alice);
    let edge2 = make_edge("knows-bob", "schema:colleague", &alice, &bob);

    let doc = Envelope::new("Alice")
        .add_edge_envelope(edge1)
        .add_edge_envelope(edge2);

    // No filters => all edges
    let matching = doc.edges_matching(None, None, None, None)?;
    assert_eq!(matching.len(), 2);

    Ok(())
}

#[test]
fn test_edges_matching_by_is_a() -> Result<(), EnvelopeError> {
    let alice = xid_like("Alice");
    let bob = xid_like("Bob");
    let edge1 = make_edge("self-desc", "foaf:Person", &alice, &alice);
    let edge2 = make_edge("knows-bob", "schema:colleague", &alice, &bob);
    let edge3 = make_edge("self-thing", "foaf:Person", &alice, &alice);

    let doc = Envelope::new("Alice")
        .add_edge_envelope(edge1)
        .add_edge_envelope(edge2)
        .add_edge_envelope(edge3);

    let is_a_person = Envelope::new("foaf:Person");
    let matching = doc.edges_matching(Some(&is_a_person), None, None, None)?;
    assert_eq!(matching.len(), 2);

    let is_a_colleague = Envelope::new("schema:colleague");
    let matching = doc.edges_matching(Some(&is_a_colleague), None, None, None)?;
    assert_eq!(matching.len(), 1);

    let is_a_none = Envelope::new("nonexistent");
    let matching = doc.edges_matching(Some(&is_a_none), None, None, None)?;
    assert_eq!(matching.len(), 0);

    Ok(())
}

#[test]
fn test_edges_matching_by_source() -> Result<(), EnvelopeError> {
    let alice = xid_like("Alice");
    let bob = xid_like("Bob");
    let edge1 = make_edge("alice-claim", "foaf:Person", &alice, &alice);
    let edge2 = make_edge("bob-claim", "foaf:Person", &bob, &alice);

    let doc = Envelope::new("Alice")
        .add_edge_envelope(edge1)
        .add_edge_envelope(edge2);

    let matching = doc.edges_matching(None, Some(&alice), None, None)?;
    assert_eq!(matching.len(), 1);

    let matching = doc.edges_matching(None, Some(&bob), None, None)?;
    assert_eq!(matching.len(), 1);

    let carol = xid_like("Carol");
    let matching = doc.edges_matching(None, Some(&carol), None, None)?;
    assert_eq!(matching.len(), 0);

    Ok(())
}

#[test]
fn test_edges_matching_by_target() -> Result<(), EnvelopeError> {
    let alice = xid_like("Alice");
    let bob = xid_like("Bob");
    let edge1 = make_edge("self-desc", "foaf:Person", &alice, &alice);
    let edge2 = make_edge("knows-bob", "schema:colleague", &alice, &bob);

    let doc = Envelope::new("Alice")
        .add_edge_envelope(edge1)
        .add_edge_envelope(edge2);

    let matching = doc.edges_matching(None, None, Some(&alice), None)?;
    assert_eq!(matching.len(), 1);

    let matching = doc.edges_matching(None, None, Some(&bob), None)?;
    assert_eq!(matching.len(), 1);

    Ok(())
}

#[test]
fn test_edges_matching_by_subject() -> Result<(), EnvelopeError> {
    let alice = xid_like("Alice");
    let edge1 = make_edge("self-desc", "foaf:Person", &alice, &alice);
    let edge2 = make_edge("cred-2", "schema:Thing", &alice, &alice);

    let doc = Envelope::new("Alice")
        .add_edge_envelope(edge1)
        .add_edge_envelope(edge2);

    let subject_filter = Envelope::new("self-desc");
    let matching = doc.edges_matching(None, None, None, Some(&subject_filter))?;
    assert_eq!(matching.len(), 1);

    let subject_filter = Envelope::new("nonexistent");
    let matching = doc.edges_matching(None, None, None, Some(&subject_filter))?;
    assert_eq!(matching.len(), 0);

    Ok(())
}

#[test]
fn test_edges_matching_combined_filters() -> Result<(), EnvelopeError> {
    let alice = xid_like("Alice");
    let bob = xid_like("Bob");
    let edge1 = make_edge("self-desc", "foaf:Person", &alice, &alice);
    let edge2 = make_edge("self-thing", "foaf:Person", &alice, &alice);
    let edge3 = make_edge("knows-bob", "foaf:Person", &alice, &bob);

    let doc = Envelope::new("Alice")
        .add_edge_envelope(edge1)
        .add_edge_envelope(edge2)
        .add_edge_envelope(edge3);

    // All three are foaf:Person
    let is_a = Envelope::new("foaf:Person");
    let matching = doc.edges_matching(Some(&is_a), None, None, None)?;
    assert_eq!(matching.len(), 3);

    // foaf:Person + target Alice => 2 (self-desc, self-thing)
    let matching = doc.edges_matching(Some(&is_a), None, Some(&alice), None)?;
    assert_eq!(matching.len(), 2);

    // foaf:Person + target Bob => 1 (knows-bob)
    let matching = doc.edges_matching(Some(&is_a), None, Some(&bob), None)?;
    assert_eq!(matching.len(), 1);

    // foaf:Person + target Alice + subject "self-desc" => 1
    let subj = Envelope::new("self-desc");
    let matching = doc.edges_matching(Some(&is_a), None, Some(&alice), Some(&subj))?;
    assert_eq!(matching.len(), 1);

    // foaf:Person + source Alice + target Bob + subject "knows-bob" => 1
    let subj = Envelope::new("knows-bob");
    let matching = doc.edges_matching(
        Some(&is_a),
        Some(&alice),
        Some(&bob),
        Some(&subj),
    )?;
    assert_eq!(matching.len(), 1);

    // All filters that match nothing
    let subj = Envelope::new("nonexistent");
    let matching = doc.edges_matching(Some(&is_a), Some(&alice), Some(&alice), Some(&subj))?;
    assert_eq!(matching.len(), 0);

    Ok(())
}

// -------------------------------------------------------------------
// Signed edges with format verification
// -------------------------------------------------------------------

#[test]
fn test_signed_edge_format() {
    let alice = xid_like("Alice");
    let edge = make_edge("cred-1", "foaf:Person", &alice, &alice);

    let signed_edge = edge.wrap().add_signature(&alice_private_key());

    #[rustfmt::skip]
    assert_actual_expected!(signed_edge.format(), indoc! {r#"
        {
            "cred-1" [
                'isA': "foaf:Person"
                'source': "Alice"
                'target': "Alice"
            ]
        } [
            'signed': Signature
        ]
    "#}.trim());
}

#[test]
fn test_signed_edge_on_document_format() {
    let alice = xid_like("Alice");
    let edge = make_edge("cred-1", "foaf:Person", &alice, &alice);
    let signed_edge = edge.wrap().add_signature(&alice_private_key());

    let doc = Envelope::new("Alice")
        .add_assertion("knows", "Bob")
        .add_edge_envelope(signed_edge);

    let formatted = doc.format();
    assert!(formatted.contains("'edge': {"));
    assert!(formatted.contains("'signed': Signature"));
    assert!(formatted.contains("'isA': \"foaf:Person\""));
}

// -------------------------------------------------------------------
// Edge coexistence with attachments
// -------------------------------------------------------------------

#[test]
fn test_edges_coexist_with_attachments() -> Result<(), EnvelopeError> {
    let alice = xid_like("Alice");
    let edge = make_edge("cred-1", "foaf:Person", &alice, &alice);

    let doc = Envelope::new("Alice")
        .add_attachment("Metadata", "com.example", Some("https://example.com/v1"))
        .add_edge_envelope(edge);

    // Both should be present
    assert_eq!(doc.edges()?.len(), 1);
    assert_eq!(doc.attachments()?.len(), 1);

    let formatted = doc.format();
    assert!(formatted.contains("'edge'"));
    assert!(formatted.contains("'attachment'"));

    Ok(())
}

// -------------------------------------------------------------------
// Edge UR round-trip
// -------------------------------------------------------------------

#[test]
fn test_edge_ur_roundtrip() -> Result<(), EnvelopeError> {
    let alice = xid_like("Alice");
    let edge = make_edge("cred-1", "foaf:Person", &alice, &alice);

    let doc = Envelope::new("Alice").add_edge_envelope(edge.clone());

    // Round-trip through UR
    let ur = doc.ur();
    let recovered = Envelope::from_ur(&ur).unwrap();
    assert!(recovered.is_equivalent_to(&doc));

    let recovered_edges = recovered.edges()?;
    assert_eq!(recovered_edges.len(), 1);
    assert!(recovered_edges[0].is_equivalent_to(&edge));

    Ok(())
}

#[test]
fn test_multiple_edges_ur_roundtrip() -> Result<(), EnvelopeError> {
    let alice = xid_like("Alice");
    let bob = xid_like("Bob");
    let edge1 = make_edge("self-desc", "foaf:Person", &alice, &alice);
    let edge2 = make_edge("knows-bob", "schema:colleague", &alice, &bob);
    let edge3 = make_edge("project", "schema:CreativeWork", &alice, &bob);

    let doc = Envelope::new("Alice")
        .add_edge_envelope(edge1)
        .add_edge_envelope(edge2)
        .add_edge_envelope(edge3);

    let ur = doc.ur();
    let recovered = Envelope::from_ur(&ur).unwrap();
    assert!(recovered.is_equivalent_to(&doc));

    let recovered_edges = recovered.edges()?;
    assert_eq!(recovered_edges.len(), 3);

    Ok(())
}

// -------------------------------------------------------------------
// Edge with extra assertions beyond the required three
// -------------------------------------------------------------------

#[test]
fn test_edge_with_additional_assertions() -> Result<(), EnvelopeError> {
    let alice = xid_like("Alice");
    let bob = xid_like("Bob");

    // An edge with extra detail assertions beyond isA/source/target
    let edge = Envelope::new("knows-bob")
        .add_assertion(known_values::IS_A, "schema:colleague")
        .add_assertion(known_values::SOURCE, alice.clone())
        .add_assertion(known_values::TARGET, bob.clone())
        .add_assertion("department", "Engineering")
        .add_assertion("since", "2024-01-15");

    // Should still validate — has the three required assertions
    assert!(edge.validate_edge().is_ok());

    // Accessors work
    let is_a = edge.edge_is_a()?;
    assert_actual_expected!(is_a.format(), r#""schema:colleague""#);

    let source = edge.edge_source()?;
    assert_actual_expected!(source.format(), r#""Alice""#);

    let target = edge.edge_target()?;
    assert_actual_expected!(target.format(), r#""Bob""#);

    Ok(())
}
