use std::collections::HashSet;

use bc_envelope::prelude::*;
use indoc::indoc;

mod common;
use crate::common::check_encoding::*;

fn basic_envelope() -> Envelope { Envelope::new("Hello.") }

fn assertion_envelope() -> Envelope { Envelope::new_assertion("knows", "Bob") }

fn single_assertion_envelope() -> Envelope {
    Envelope::new("Alice").add_assertion("knows", "Bob")
}

fn double_assertion_envelope() -> Envelope {
    Envelope::new("Alice")
        .add_assertion("knows", "Bob")
        .add_assertion("knows", "Carol")
}

#[test]
fn test_envelope_elision() -> EnvelopeResult<()> {
    let e1 = basic_envelope();

    let e2 = e1.elide();
    assert!(e1.is_equivalent_to(&e2));
    assert!(!e1.is_identical_to(&e2));

    // expected-text-output-rubric:
    #[rustfmt::skip]
    assert_actual_expected!(e2.format(), indoc! {r#"
        ELIDED
    "#}.trim());

    // expected-text-output-rubric:
    #[rustfmt::skip]
    assert_actual_expected!(e2.diagnostic_annotated(), indoc! {r#"
        200(   / envelope /
            h'8cc96cdb771176e835114a0f8936690b41cfed0df22d014eedd64edaea945d59'
        )
    "#}.trim());

    let e3 = e2.unelide(&e1)?;
    assert!(e3.is_equivalent_to(&e1));
    // expected-text-output-rubric:
    #[rustfmt::skip]
    assert_actual_expected!(e3.format(), indoc! {r#"
        "Hello."
    "#}.trim());

    Ok(())
}

#[test]
fn test_single_assertion_remove_elision() -> EnvelopeResult<()> {
    // The original Envelope
    let e1 = single_assertion_envelope();
    // expected-text-output-rubric:
    #[rustfmt::skip]
    assert_actual_expected!(e1.format(), indoc! {r#"
        "Alice" [
            "knows": "Bob"
        ]
    "#}.trim());

    // Elide the entire envelope
    let e2 = e1.elide_removing_target(&e1).check_encoding()?;
    // expected-text-output-rubric:
    #[rustfmt::skip]
    assert_actual_expected!(e2.format(), indoc! {r#"
        ELIDED
    "#}.trim());

    // Elide just the envelope's subject
    let e3 = e1
        .elide_removing_target(&"Alice".to_envelope())
        .check_encoding()?;
    // expected-text-output-rubric:
    #[rustfmt::skip]
    assert_actual_expected!(e3.format(), indoc! {r#"
        ELIDED [
            "knows": "Bob"
        ]
    "#}.trim());

    // Elide just the assertion's predicate
    let e4 = e1
        .elide_removing_target(&"knows".to_envelope())
        .check_encoding()?;
    // expected-text-output-rubric:
    #[rustfmt::skip]
    assert_actual_expected!(e4.format(), indoc! {r#"
        "Alice" [
            ELIDED: "Bob"
        ]
    "#}.trim());

    // Elide just the assertion's object
    let e5 = e1
        .elide_removing_target(&"Bob".to_envelope())
        .check_encoding()?;
    // expected-text-output-rubric:
    #[rustfmt::skip]
    assert_actual_expected!(e5.format(), indoc! {r#"
        "Alice" [
            "knows": ELIDED
        ]
    "#}.trim());

    // Elide the entire assertion
    let e6 = e1
        .elide_removing_target(&assertion_envelope())
        .check_encoding()?;
    // expected-text-output-rubric:
    #[rustfmt::skip]
    assert_actual_expected!(e6.format(), indoc! {r#"
        "Alice" [
            ELIDED
        ]
    "#}.trim());

    Ok(())
}

#[test]
fn test_double_assertion_remove_elision() -> EnvelopeResult<()> {
    // The original Envelope
    let e1 = double_assertion_envelope();
    // expected-text-output-rubric:
    #[rustfmt::skip]
    assert_actual_expected!(e1.format(), indoc! {r#"
        "Alice" [
            "knows": "Bob"
            "knows": "Carol"
        ]
    "#}.trim());

    // Elide the entire envelope
    let e2 = e1.elide_removing_target(&e1).check_encoding()?;
    // expected-text-output-rubric:
    #[rustfmt::skip]
    assert_actual_expected!(e2.format(), indoc! {r#"
        ELIDED
    "#}.trim());

    // Elide just the envelope's subject
    let e3 = e1
        .elide_removing_target(&"Alice".to_envelope())
        .check_encoding()?;
    // expected-text-output-rubric:
    #[rustfmt::skip]
    assert_actual_expected!(e3.format(), indoc! {r#"
        ELIDED [
            "knows": "Bob"
            "knows": "Carol"
        ]
    "#}.trim());

    // Elide just the assertion's predicate
    let e4 = e1
        .elide_removing_target(&"knows".to_envelope())
        .check_encoding()?;
    // expected-text-output-rubric:
    #[rustfmt::skip]
    assert_actual_expected!(e4.format(), indoc! {r#"
        "Alice" [
            ELIDED: "Bob"
            ELIDED: "Carol"
        ]
    "#}.trim());

    // Elide just the assertion's object
    let e5 = e1
        .elide_removing_target(&"Bob".to_envelope())
        .check_encoding()?;
    // expected-text-output-rubric:
    #[rustfmt::skip]
    assert_actual_expected!(e5.format(), indoc! {r#"
        "Alice" [
            "knows": "Carol"
            "knows": ELIDED
        ]
    "#}.trim());

    // Elide the entire assertion
    let e6 = e1
        .elide_removing_target(&assertion_envelope())
        .check_encoding()?;
    // expected-text-output-rubric:
    #[rustfmt::skip]
    assert_actual_expected!(e6.format(), indoc! {r#"
        "Alice" [
            "knows": "Carol"
            ELIDED
        ]
    "#}.trim());

    Ok(())
}

#[test]
fn test_single_assertion_reveal_elision() -> EnvelopeResult<()> {
    // The original Envelope
    let e1 = single_assertion_envelope();
    // expected-text-output-rubric:
    #[rustfmt::skip]
    assert_actual_expected!(e1.format(), indoc! {r#"
        "Alice" [
            "knows": "Bob"
        ]
    "#}.trim());

    // Elide revealing nothing
    let e2 = e1.elide_revealing_array(&[]).check_encoding()?;
    // expected-text-output-rubric:
    #[rustfmt::skip]
    assert_actual_expected!(e2.format(), indoc! {r#"
        ELIDED
    "#}.trim());

    // Reveal just the envelope's structure
    let e3 = e1.elide_revealing_array(&[&e1]).check_encoding()?;
    #[rustfmt::skip]
    assert_actual_expected!(e3.format(), indoc! {r#"
        ELIDED [
            ELIDED
        ]
    "#}.trim());

    // Reveal just the envelope's subject
    let e4 = e1
        .elide_revealing_array(&[&e1, &"Alice".to_envelope()])
        .check_encoding()?;
    // expected-text-output-rubric:
    #[rustfmt::skip]
    assert_actual_expected!(e4.format(), indoc! {r#"
        "Alice" [
            ELIDED
        ]
    "#}.trim());

    // Reveal just the assertion's structure.
    let e5 = e1
        .elide_revealing_array(&[&e1, &assertion_envelope()])
        .check_encoding()?;
    // expected-text-output-rubric:
    #[rustfmt::skip]
    assert_actual_expected!(e5.format(), indoc! {r#"
        ELIDED [
            ELIDED: ELIDED
        ]
    "#}.trim());

    // Reveal just the assertion's predicate
    let e6 = e1
        .elide_revealing_array(&[
            &e1,
            &assertion_envelope(),
            &"knows".to_envelope(),
        ])
        .check_encoding()?;
    // expected-text-output-rubric:
    #[rustfmt::skip]
    assert_actual_expected!(e6.format(), indoc! {r#"
        ELIDED [
            "knows": ELIDED
        ]
    "#}.trim());

    // Reveal just the assertion's object
    let e7 = e1
        .elide_revealing_array(&[
            &e1,
            &assertion_envelope(),
            &"Bob".to_envelope(),
        ])
        .check_encoding()?;
    // expected-text-output-rubric:
    #[rustfmt::skip]
    assert_actual_expected!(e7.format(), indoc! {r#"
        ELIDED [
            ELIDED: "Bob"
        ]
    "#}.trim());

    Ok(())
}

#[test]
fn test_double_assertion_reveal_elision() -> EnvelopeResult<()> {
    // The original Envelope
    let e1 = double_assertion_envelope();
    // expected-text-output-rubric:
    #[rustfmt::skip]
    assert_actual_expected!(e1.format(), indoc! {r#"
        "Alice" [
            "knows": "Bob"
            "knows": "Carol"
        ]
    "#}.trim());

    // Elide revealing nothing
    let e2 = e1.elide_revealing_array(&[]).check_encoding()?;
    // expected-text-output-rubric:
    #[rustfmt::skip]
    assert_actual_expected!(e2.format(), indoc! {r#"
        ELIDED
    "#}.trim());

    // Reveal just the envelope's structure
    let e3 = e1.elide_revealing_array(&[&e1]).check_encoding()?;
    // expected-text-output-rubric:
    #[rustfmt::skip]
    assert_actual_expected!(e3.format(), indoc! {r#"
        ELIDED [
            ELIDED (2)
        ]
    "#}.trim());

    // Reveal just the envelope's subject
    let e4 = e1
        .elide_revealing_array(&[&e1, &"Alice".to_envelope()])
        .check_encoding()?;
    // expected-text-output-rubric:
    #[rustfmt::skip]
    assert_actual_expected!(e4.format(), indoc! {r#"
        "Alice" [
            ELIDED (2)
        ]
    "#}.trim());

    // Reveal just the assertion's structure.
    let e5 = e1
        .elide_revealing_array(&[&e1, &assertion_envelope()])
        .check_encoding()?;
    // expected-text-output-rubric:
    #[rustfmt::skip]
    assert_actual_expected!(e5.format(), indoc! {r#"
        ELIDED [
            ELIDED: ELIDED
            ELIDED
        ]
    "#}.trim());

    // Reveal just the assertion's predicate
    let e6 = e1
        .elide_revealing_array(&[
            &e1,
            &assertion_envelope(),
            &"knows".to_envelope(),
        ])
        .check_encoding()?;
    // expected-text-output-rubric:
    #[rustfmt::skip]
    assert_actual_expected!(e6.format(), indoc! {r#"
        ELIDED [
            "knows": ELIDED
            ELIDED
        ]
    "#}.trim());

    // Reveal just the assertion's object
    let e7 = e1
        .elide_revealing_array(&[
            &e1,
            &assertion_envelope(),
            &"Bob".to_envelope(),
        ])
        .check_encoding()?;
    // expected-text-output-rubric:
    #[rustfmt::skip]
    assert_actual_expected!(e7.format(), indoc! {r#"
        ELIDED [
            ELIDED: "Bob"
            ELIDED
        ]
    "#}.trim());

    Ok(())
}

#[test]
fn test_digests() -> EnvelopeResult<()> {
    let e1 = double_assertion_envelope();
    // expected-text-output-rubric:
    #[rustfmt::skip]
    assert_actual_expected!(e1.format(), indoc! {r#"
        "Alice" [
            "knows": "Bob"
            "knows": "Carol"
        ]
    "#}.trim());

    let e2 = e1.elide_revealing_set(&e1.digests(0)).check_encoding()?;
    // expected-text-output-rubric:
    #[rustfmt::skip]
    assert_actual_expected!(e2.format(), indoc! {r#"
        ELIDED
    "#}.trim());

    let e3 = e1.elide_revealing_set(&e1.digests(1)).check_encoding()?;
    // expected-text-output-rubric:
    #[rustfmt::skip]
    assert_actual_expected!(e3.format(), indoc! {r#"
        "Alice" [
            ELIDED (2)
        ]
    "#}.trim());

    let e4 = e1.elide_revealing_set(&e1.digests(2)).check_encoding()?;
    // expected-text-output-rubric:
    #[rustfmt::skip]
    assert_actual_expected!(e4.format(), indoc! {r#"
        "Alice" [
            ELIDED: ELIDED
            ELIDED: ELIDED
        ]
    "#}.trim());

    let e5 = e1.elide_revealing_set(&e1.digests(3)).check_encoding()?;
    // expected-text-output-rubric:
    #[rustfmt::skip]
    assert_actual_expected!(e5.format(), indoc! {r#"
        "Alice" [
            "knows": "Bob"
            "knows": "Carol"
        ]
    "#}.trim());

    Ok(())
}

#[test]
fn test_target_reveal() -> EnvelopeResult<()> {
    let e1 =
        double_assertion_envelope().add_assertion("livesAt", "123 Main St.");
    // expected-text-output-rubric:
    #[rustfmt::skip]
    assert_actual_expected!(e1.format(), indoc! {r#"
        "Alice" [
            "knows": "Bob"
            "knows": "Carol"
            "livesAt": "123 Main St."
        ]
    "#}.trim());

    let mut target = HashSet::new();
    // Reveal the Envelope structure
    target.extend(e1.digests(1));
    // Reveal everything about the subject
    target.extend(e1.subject().deep_digests());
    // Reveal everything about one of the assertions
    target.extend(assertion_envelope().deep_digests());
    // Reveal the specific `livesAt` assertion
    target.extend(e1.assertion_with_predicate("livesAt")?.deep_digests());
    let e2 = e1.elide_revealing_set(&target).check_encoding()?;
    // expected-text-output-rubric:
    #[rustfmt::skip]
    assert_actual_expected!(e2.format(), indoc! {r#"
        "Alice" [
            "knows": "Bob"
            "livesAt": "123 Main St."
            ELIDED
        ]
    "#}.trim());

    Ok(())
}

#[test]
fn test_targeted_remove() -> EnvelopeResult<()> {
    let e1 =
        double_assertion_envelope().add_assertion("livesAt", "123 Main St.");
    // expected-text-output-rubric:
    #[rustfmt::skip]
    assert_actual_expected!(e1.format(), indoc! {r#"
        "Alice" [
            "knows": "Bob"
            "knows": "Carol"
            "livesAt": "123 Main St."
        ]
    "#}.trim());

    let mut target2 = HashSet::new();
    // Hide one of the assertions
    target2.extend(assertion_envelope().digests(1));
    let e2 = e1.elide_removing_set(&target2).check_encoding()?;
    // expected-text-output-rubric:
    #[rustfmt::skip]
    assert_actual_expected!(e2.format(), indoc! {r#"
        "Alice" [
            "knows": "Carol"
            "livesAt": "123 Main St."
            ELIDED
        ]
    "#}.trim());

    let mut target3 = HashSet::new();
    // Hide one of the assertions by finding its predicate
    target3.extend(e1.assertion_with_predicate("livesAt")?.deep_digests());
    let e3 = e1.elide_removing_set(&target3).check_encoding()?;
    // expected-text-output-rubric:
    #[rustfmt::skip]
    assert_actual_expected!(e3.format(), indoc! {r#"
        "Alice" [
            "knows": "Bob"
            "knows": "Carol"
            ELIDED
        ]
    "#}.trim());

    // Semantically equivalent
    assert!(e1.is_equivalent_to(&e3));

    // Structurally different
    assert!(!e1.is_identical_to(&e3));

    Ok(())
}

#[test]
fn test_walk_replace_basic() -> EnvelopeResult<()> {
    // Create envelopes
    let alice = Envelope::new("Alice");
    let bob = Envelope::new("Bob");
    let charlie = Envelope::new("Charlie");

    // Create an envelope with Bob referenced multiple times
    let envelope = alice
        .clone()
        .add_assertion("knows", bob.clone())
        .add_assertion("likes", bob.clone());

    // expected-text-output-rubric:
    #[rustfmt::skip]
    assert_actual_expected!(envelope.format(), indoc! {r#"
        "Alice" [
            "knows": "Bob"
            "likes": "Bob"
        ]
    "#}.trim());

    // Replace all instances of Bob with Charlie
    let mut target = HashSet::new();
    target.insert(bob.digest());

    let modified = envelope.walk_replace(&target, &charlie)?;

    // expected-text-output-rubric:
    #[rustfmt::skip]
    assert_actual_expected!(modified.format(), indoc! {r#"
        "Alice" [
            "knows": "Charlie"
            "likes": "Charlie"
        ]
    "#}.trim());

    // The structure is different (different content)
    assert!(!modified.is_equivalent_to(&envelope));

    Ok(())
}

#[test]
fn test_walk_replace_subject() -> EnvelopeResult<()> {
    let alice = Envelope::new("Alice");
    let bob = Envelope::new("Bob");
    let carol = Envelope::new("Carol");

    let envelope = alice.clone().add_assertion("knows", bob.clone());

    // expected-text-output-rubric:
    #[rustfmt::skip]
    assert_actual_expected!(envelope.format(), indoc! {r#"
        "Alice" [
            "knows": "Bob"
        ]
    "#}.trim());

    // Replace the subject (Alice) with Carol
    let mut target = HashSet::new();
    target.insert(alice.digest());

    let modified = envelope.walk_replace(&target, &carol)?;

    // expected-text-output-rubric:
    #[rustfmt::skip]
    assert_actual_expected!(modified.format(), indoc! {r#"
        "Carol" [
            "knows": "Bob"
        ]
    "#}.trim());

    Ok(())
}

#[test]
fn test_walk_replace_nested() -> EnvelopeResult<()> {
    let alice = Envelope::new("Alice");
    let bob = Envelope::new("Bob");
    let charlie = Envelope::new("Charlie");

    // Create a nested structure with Bob appearing at multiple levels
    let inner = bob.clone().add_assertion("friend", bob.clone());
    let envelope = alice.clone().add_assertion("knows", inner);

    // expected-text-output-rubric:
    #[rustfmt::skip]
    assert_actual_expected!(envelope.format(), indoc! {r#"
        "Alice" [
            "knows": "Bob" [
                "friend": "Bob"
            ]
        ]
    "#}.trim());

    // Replace all instances of Bob with Charlie
    let mut target = HashSet::new();
    target.insert(bob.digest());

    let modified = envelope.walk_replace(&target, &charlie)?;

    // expected-text-output-rubric:
    #[rustfmt::skip]
    assert_actual_expected!(modified.format(), indoc! {r#"
        "Alice" [
            "knows": "Charlie" [
                "friend": "Charlie"
            ]
        ]
    "#}.trim());

    Ok(())
}

#[test]
fn test_walk_replace_wrapped() -> EnvelopeResult<()> {
    let alice = Envelope::new("Alice");
    let bob = Envelope::new("Bob");
    let charlie = Envelope::new("Charlie");

    // Create a wrapped envelope containing Bob
    let wrapped = bob.clone().wrap();
    let envelope = alice.clone().add_assertion("data", wrapped);

    // expected-text-output-rubric:
    #[rustfmt::skip]
    assert_actual_expected!(envelope.format(), indoc! {r#"
        "Alice" [
            "data": {
                "Bob"
            }
        ]
    "#}.trim());

    // Replace Bob with Charlie
    let mut target = HashSet::new();
    target.insert(bob.digest());

    let modified = envelope.walk_replace(&target, &charlie)?;

    // expected-text-output-rubric:
    #[rustfmt::skip]
    assert_actual_expected!(modified.format(), indoc! {r#"
        "Alice" [
            "data": {
                "Charlie"
            }
        ]
    "#}.trim());

    Ok(())
}

#[test]
fn test_walk_replace_no_match() -> EnvelopeResult<()> {
    let alice = Envelope::new("Alice");
    let bob = Envelope::new("Bob");
    let charlie = Envelope::new("Charlie");
    let dave = Envelope::new("Dave");

    let envelope = alice.clone().add_assertion("knows", bob.clone());

    // expected-text-output-rubric:
    #[rustfmt::skip]
    assert_actual_expected!(envelope.format(), indoc! {r#"
        "Alice" [
            "knows": "Bob"
        ]
    "#}.trim());

    // Try to replace Dave (who doesn't exist in the envelope)
    let mut target = HashSet::new();
    target.insert(dave.digest());

    let modified = envelope.walk_replace(&target, &charlie)?;

    // Should be identical since nothing matched
    // expected-text-output-rubric:
    #[rustfmt::skip]
    assert_actual_expected!(modified.format(), indoc! {r#"
        "Alice" [
            "knows": "Bob"
        ]
    "#}.trim());

    assert!(modified.is_identical_to(&envelope));

    Ok(())
}

#[test]
fn test_walk_replace_multiple_targets() -> EnvelopeResult<()> {
    let alice = Envelope::new("Alice");
    let bob = Envelope::new("Bob");
    let carol = Envelope::new("Carol");
    let replacement = Envelope::new("REDACTED");

    let envelope = alice
        .clone()
        .add_assertion("knows", bob.clone())
        .add_assertion("likes", carol.clone());

    // expected-text-output-rubric:
    #[rustfmt::skip]
    assert_actual_expected!(envelope.format(), indoc! {r#"
        "Alice" [
            "knows": "Bob"
            "likes": "Carol"
        ]
    "#}.trim());

    // Replace both Bob and Carol with REDACTED
    let mut target = HashSet::new();
    target.insert(bob.digest());
    target.insert(carol.digest());

    let modified = envelope.walk_replace(&target, &replacement)?;

    // expected-text-output-rubric:
    #[rustfmt::skip]
    assert_actual_expected!(modified.format(), indoc! {r#"
        "Alice" [
            "knows": "REDACTED"
            "likes": "REDACTED"
        ]
    "#}.trim());

    Ok(())
}

#[test]
fn test_walk_replace_elided() -> EnvelopeResult<()> {
    let alice = Envelope::new("Alice");
    let bob = Envelope::new("Bob");
    let charlie = Envelope::new("Charlie");

    // Create an envelope with Bob, then elide Bob
    let envelope = alice
        .clone()
        .add_assertion("knows", bob.clone())
        .add_assertion("likes", bob.clone());

    // expected-text-output-rubric:
    #[rustfmt::skip]
    assert_actual_expected!(envelope.format(), indoc! {r#"
        "Alice" [
            "knows": "Bob"
            "likes": "Bob"
        ]
    "#}.trim());

    // Elide Bob
    let elided = envelope.elide_removing_target(&bob);

    // expected-text-output-rubric:
    #[rustfmt::skip]
    assert_actual_expected!(elided.format(), indoc! {r#"
        "Alice" [
            "knows": ELIDED
            "likes": ELIDED
        ]
    "#}.trim());

    // Replace the elided Bob with Charlie
    // This works because the elided node has Bob's digest
    let mut target = HashSet::new();
    target.insert(bob.digest());

    let modified = elided.walk_replace(&target, &charlie)?;

    // expected-text-output-rubric:
    #[rustfmt::skip]
    assert_actual_expected!(modified.format(), indoc! {r#"
        "Alice" [
            "knows": "Charlie"
            "likes": "Charlie"
        ]
    "#}.trim());

    // Verify that the elided nodes were replaced
    assert!(!modified.is_equivalent_to(&envelope));
    assert!(!modified.is_equivalent_to(&elided));

    Ok(())
}

#[test]
fn test_walk_replace_assertion_with_non_assertion_fails() -> EnvelopeResult<()>
{
    let alice = Envelope::new("Alice");
    let bob = Envelope::new("Bob");
    let charlie = Envelope::new("Charlie");

    let envelope = alice.clone().add_assertion("knows", bob.clone());

    // Get the assertion's digest
    let knows_assertion = envelope.assertion_with_predicate("knows")?;
    let assertion_digest = knows_assertion.digest();

    // Try to replace the entire assertion with Charlie (a non-assertion)
    let mut target = HashSet::new();
    target.insert(assertion_digest);

    let result = envelope.walk_replace(&target, &charlie);

    // This should fail because we're replacing an assertion with a
    // non-assertion
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().to_string(), "invalid format");

    Ok(())
}
