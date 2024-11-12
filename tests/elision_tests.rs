use std::collections::HashSet;
use bc_envelope::prelude::*;
use indoc::indoc;

mod common;
use crate::common::check_encoding::*;

fn basic_envelope() -> Envelope {
    Envelope::new("Hello.")
}

fn assertion_envelope() -> Envelope {
    Envelope::new_assertion("knows", "Bob")
}

fn single_assertion_envelope() -> Envelope {
    Envelope::new("Alice")
        .add_assertion("knows", "Bob")
}

fn double_assertion_envelope() -> Envelope {
    Envelope::new("Alice")
        .add_assertion("knows", "Bob")
        .add_assertion("knows", "Carol")
}

#[test]
fn test_envelope_elision() -> anyhow::Result<()> {
    let e1 = basic_envelope();

    let e2 = e1.elide();
    assert!(e1.is_equivalent_to(&e2));
    assert!(!e1.is_identical_to(&e2));

    assert_eq!(e2.format(),
    indoc! {r#"
    ELIDED
    "#}.trim()
    );

    assert_eq!(e2.diagnostic_annotated(),
    indoc! {r#"
    200(   / envelope /
        h'8cc96cdb771176e835114a0f8936690b41cfed0df22d014eedd64edaea945d59'
    )
    "#}.trim()
    );

    let e3 = e2.unelide(&e1)?;
    assert!(e3.is_equivalent_to(&e1));
    assert_eq!(e3.format(),
    indoc! {r#"
    "Hello."
    "#}.trim()
    );

    Ok(())
}

#[test]
fn test_single_assertion_remove_elision() -> anyhow::Result<()> {
    // The original Envelope
    let e1 = single_assertion_envelope();
    assert_eq!(e1.format(),
    indoc! {r#"
    "Alice" [
        "knows": "Bob"
    ]
    "#}.trim()
    );

    // Elide the entire envelope
    let e2 = e1.elide_removing_target(&e1).check_encoding()?;
    assert_eq!(e2.format(),
    indoc! {r#"
    ELIDED
    "#}.trim()
    );

    // Elide just the envelope's subject
    let e3 = e1.elide_removing_target(&"Alice".to_envelope()).check_encoding()?;
    assert_eq!(e3.format(),
    indoc! {r#"
    ELIDED [
        "knows": "Bob"
    ]
    "#}.trim()
    );

    // Elide just the assertion's predicate
    let e4 = e1.elide_removing_target(&"knows".to_envelope()).check_encoding()?;
    assert_eq!(e4.format(),
    indoc! {r#"
    "Alice" [
        ELIDED: "Bob"
    ]
    "#}.trim()
    );

    // Elide just the assertion's object
    let e5 = e1.elide_removing_target(&"Bob".to_envelope()).check_encoding()?;
    assert_eq!(e5.format(),
    indoc! {r#"
    "Alice" [
        "knows": ELIDED
    ]
    "#}.trim()
    );

    // Elide the entire assertion
    let e6 = e1.elide_removing_target(&assertion_envelope()).check_encoding()?;
    assert_eq!(e6.format(),
    indoc! {r#"
    "Alice" [
        ELIDED
    ]
    "#}.trim()
    );

    Ok(())
}

#[test]
fn test_double_assertion_remove_elision() -> anyhow::Result<()> {
    // The original Envelope
    let e1 = double_assertion_envelope();
    assert_eq!(e1.format(),
    indoc! {r#"
    "Alice" [
        "knows": "Bob"
        "knows": "Carol"
    ]
    "#}.trim()
    );

    // Elide the entire envelope
    let e2 = e1.elide_removing_target(&e1).check_encoding()?;
    assert_eq!(e2.format(),
    indoc! {r#"
    ELIDED
    "#}.trim()
    );

    // Elide just the envelope's subject
    let e3 = e1.elide_removing_target(&"Alice".to_envelope()).check_encoding()?;
    assert_eq!(e3.format(),
    indoc! {r#"
    ELIDED [
        "knows": "Bob"
        "knows": "Carol"
    ]
    "#}.trim()
    );

    // Elide just the assertion's predicate
    let e4 = e1.elide_removing_target(&"knows".to_envelope()).check_encoding()?;
    assert_eq!(e4.format(),
    indoc! {r#"
    "Alice" [
        ELIDED: "Bob"
        ELIDED: "Carol"
    ]
    "#}.trim()
    );

    // Elide just the assertion's object
    let e5 = e1.elide_removing_target(&"Bob".to_envelope()).check_encoding()?;
    assert_eq!(e5.format(),
    indoc! {r#"
    "Alice" [
        "knows": "Carol"
        "knows": ELIDED
    ]
    "#}.trim()
    );

    // Elide the entire assertion
    let e6 = e1.elide_removing_target(&assertion_envelope()).check_encoding()?;
    assert_eq!(e6.format(),
    indoc! {r#"
    "Alice" [
        "knows": "Carol"
        ELIDED
    ]
    "#}.trim()
    );

    Ok(())
}

#[test]
fn test_single_assertion_reveal_elision() -> anyhow::Result<()> {
    // The original Envelope
    let e1 = single_assertion_envelope();
    assert_eq!(e1.format(),
    indoc! {r#"
    "Alice" [
        "knows": "Bob"
    ]
    "#}.trim()
    );

    // Elide revealing nothing
    let e2 = e1.elide_revealing_array(&[]).check_encoding()?;
    assert_eq!(e2.format(),
    indoc! {r#"
    ELIDED
    "#}.trim()
    );

    // Reveal just the envelope's structure
    let e3 = e1.elide_revealing_array(&[&e1]).check_encoding()?;
    assert_eq!(e3.format(),
    indoc! {r#"
    ELIDED [
        ELIDED
    ]
    "#}.trim()
    );

    // Reveal just the envelope's subject
    let e4 = e1.elide_revealing_array(&[&e1, &"Alice".to_envelope()]).check_encoding()?;
    assert_eq!(e4.format(),
    indoc! {r#"
    "Alice" [
        ELIDED
    ]
    "#}.trim()
    );

    // Reveal just the assertion's structure.
    let e5 = e1.elide_revealing_array(&[&e1, &assertion_envelope()]).check_encoding()?;
    assert_eq!(e5.format(),
    indoc! {r#"
    ELIDED [
        ELIDED: ELIDED
    ]
    "#}.trim()
    );

    // Reveal just the assertion's predicate
    let e6 = e1.elide_revealing_array(&[&e1, &assertion_envelope(), &"knows".to_envelope()]).check_encoding()?;
    assert_eq!(e6.format(),
    indoc! {r#"
    ELIDED [
        "knows": ELIDED
    ]
    "#}.trim()
    );

    // Reveal just the assertion's object
    let e7 = e1.elide_revealing_array(&[&e1, &assertion_envelope(), &"Bob".to_envelope()]).check_encoding()?;
    assert_eq!(e7.format(),
    indoc! {r#"
    ELIDED [
        ELIDED: "Bob"
    ]
    "#}.trim()
    );

    Ok(())
}

#[test]
fn test_double_assertion_reveal_elision() -> anyhow::Result<()> {
    // The original Envelope
    let e1 = double_assertion_envelope();
    assert_eq!(e1.format(),
    indoc! {r#"
    "Alice" [
        "knows": "Bob"
        "knows": "Carol"
    ]
    "#}.trim()
    );

    // Elide revealing nothing
    let e2 = e1.elide_revealing_array(&[]).check_encoding()?;
    assert_eq!(e2.format(),
    indoc! {r#"
    ELIDED
    "#}.trim()
    );

    // Reveal just the envelope's structure
    let e3 = e1.elide_revealing_array(&[&e1]).check_encoding()?;
    assert_eq!(e3.format(),
    indoc! {r#"
    ELIDED [
        ELIDED (2)
    ]
    "#}.trim()
    );

    // Reveal just the envelope's subject
    let e4 = e1.elide_revealing_array(&[&e1, &"Alice".to_envelope()]).check_encoding()?;
    assert_eq!(e4.format(),
    indoc! {r#"
    "Alice" [
        ELIDED (2)
    ]
    "#}.trim()
    );

    // Reveal just the assertion's structure.
    let e5 = e1.elide_revealing_array(&[&e1, &assertion_envelope()]).check_encoding()?;
    assert_eq!(e5.format(),
    indoc! {r#"
    ELIDED [
        ELIDED: ELIDED
        ELIDED
    ]
    "#}.trim()
    );

    // Reveal just the assertion's predicate
    let e6 = e1.elide_revealing_array(&[&e1, &assertion_envelope(), &"knows".to_envelope()]).check_encoding()?;
    assert_eq!(e6.format(),
    indoc! {r#"
    ELIDED [
        "knows": ELIDED
        ELIDED
    ]
    "#}.trim()
    );

    // Reveal just the assertion's object
    let e7 = e1.elide_revealing_array(&[&e1, &assertion_envelope(), &"Bob".to_envelope()]).check_encoding()?;
    assert_eq!(e7.format(),
    indoc! {r#"
    ELIDED [
        ELIDED: "Bob"
        ELIDED
    ]
    "#}.trim()
    );

    Ok(())
}

#[test]
fn test_digests() -> anyhow::Result<()> {
    let e1 = double_assertion_envelope();
    assert_eq!(e1.format(),
    indoc! {r#"
    "Alice" [
        "knows": "Bob"
        "knows": "Carol"
    ]
    "#}.trim()
    );

    let e2 = e1.elide_revealing_set(&e1.digests(0)).check_encoding()?;
    assert_eq!(e2.format(),
    indoc! {r#"
    ELIDED
    "#}.trim()
    );

    let e3 = e1.elide_revealing_set(&e1.digests(1)).check_encoding()?;
    assert_eq!(e3.format(),
    indoc! {r#"
    "Alice" [
        ELIDED (2)
    ]
    "#}.trim()
    );

    let e4 = e1.elide_revealing_set(&e1.digests(2)).check_encoding()?;
    assert_eq!(e4.format(),
    indoc! {r#"
    "Alice" [
        ELIDED: ELIDED
        ELIDED: ELIDED
    ]
    "#}.trim()
    );

    let e5 = e1.elide_revealing_set(&e1.digests(3)).check_encoding()?;
    assert_eq!(e5.format(),
    indoc! {r#"
    "Alice" [
        "knows": "Bob"
        "knows": "Carol"
    ]
    "#}.trim()
    );

    Ok(())
}

#[test]
fn test_target_reveal() -> anyhow::Result<()> {
    let e1 = double_assertion_envelope()
        .add_assertion("livesAt", "123 Main St.");
    assert_eq!(e1.format(),
    indoc! {r#"
    "Alice" [
        "knows": "Bob"
        "knows": "Carol"
        "livesAt": "123 Main St."
    ]
    "#}.trim()
    );

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
    assert_eq!(e2.format(),
    indoc! {r#"
    "Alice" [
        "knows": "Bob"
        "livesAt": "123 Main St."
        ELIDED
    ]
    "#}.trim()
    );

    Ok(())
}

#[test]
fn test_targeted_remove() -> anyhow::Result<()> {
    let e1 = double_assertion_envelope()
        .add_assertion("livesAt", "123 Main St.");
    assert_eq!(e1.format(),
    indoc! {r#"
    "Alice" [
        "knows": "Bob"
        "knows": "Carol"
        "livesAt": "123 Main St."
    ]
    "#}.trim()
    );

    let mut target2 = HashSet::new();
    // Hide one of the assertions
    target2.extend(assertion_envelope().digests(1));
    let e2 = e1.elide_removing_set(&target2).check_encoding()?;
    assert_eq!(e2.format(),
    indoc! {r#"
    "Alice" [
        "knows": "Carol"
        "livesAt": "123 Main St."
        ELIDED
    ]
    "#}.trim()
    );

    let mut target3 = HashSet::new();
    // Hide one of the assertions by finding its predicate
    target3.extend(e1.assertion_with_predicate("livesAt")?.deep_digests());
    let e3 = e1.elide_removing_set(&target3).check_encoding()?;
    assert_eq!(e3.format(),
    indoc! {r#"
    "Alice" [
        "knows": "Bob"
        "knows": "Carol"
        ELIDED
    ]
    "#}.trim()
    );

    // Semantically equivalent
    assert!(e1.is_equivalent_to(&e3));

    // Structurally different
    assert!(!e1.is_identical_to(&e3));

    Ok(())
}
