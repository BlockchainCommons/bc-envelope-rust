use bc_envelope::prelude::*;
use indoc::indoc;

mod common;
use crate::common::check_encoding::*;

#[test]
fn test_predicate_enclosures() {
    let alice = Envelope::new("Alice");
    let knows = Envelope::new("knows");
    let bob = Envelope::new("Bob");

    let a = Envelope::new("A");
    let b = Envelope::new("B");

    let knows_bob = Envelope::new_assertion(&knows, &bob);
    #[rustfmt::skip]
    assert_actual_expected!(knows_bob.format(), indoc! {r#"
        "knows": "Bob"
    "#}.trim());

    let ab = Envelope::new_assertion(a, b);
    #[rustfmt::skip]
    assert_actual_expected!(ab.format(),indoc! {r#"
        "A": "B"
    "#}.trim());

    let knows_ab_bob = Envelope::new_assertion(
        knows.add_assertion_envelope(&ab).unwrap(),
        &bob,
    )
    .check_encoding()
    .unwrap();
    #[rustfmt::skip]
    assert_actual_expected!(knows_ab_bob.format(), indoc! {r#"
        "knows" [
            "A": "B"
        ]
        : "Bob"
    "#}.trim());

    let knows_bob_ab = Envelope::new_assertion(
        &knows,
        bob.add_assertion_envelope(&ab).unwrap(),
    )
    .check_encoding()
    .unwrap();
    #[rustfmt::skip]
    assert_actual_expected!(knows_bob_ab.format(), indoc! {r#"
        "knows": "Bob" [
            "A": "B"
        ]
    "#}.trim());

    let knows_bob_enclose_ab = knows_bob
        .add_assertion_envelope(&ab)
        .unwrap()
        .check_encoding()
        .unwrap();
    #[rustfmt::skip]
    assert_actual_expected!(knows_bob_enclose_ab.format(), indoc! {r#"
        {
            "knows": "Bob"
        } [
            "A": "B"
        ]
    "#}.trim());

    let alice_knows_bob = alice
        .add_assertion_envelope(knows_bob)
        .unwrap()
        .check_encoding()
        .unwrap();
    #[rustfmt::skip]
    assert_actual_expected!(alice_knows_bob.format(), indoc! {r#"
        "Alice" [
            "knows": "Bob"
        ]
    "#}.trim());

    let alice_ab_knows_bob = alice_knows_bob
        .add_assertion_envelope(&ab)
        .unwrap()
        .check_encoding()
        .unwrap();
    #[rustfmt::skip]
    assert_actual_expected!(alice_ab_knows_bob.format(), indoc! {r#"
        "Alice" [
            "A": "B"
            "knows": "Bob"
        ]
    "#}.trim());

    let alice_knows_ab_bob = alice
        .add_assertion_envelope(Envelope::new_assertion(
            knows.add_assertion_envelope(&ab).unwrap(),
            &bob,
        ))
        .unwrap()
        .check_encoding()
        .unwrap();
    #[rustfmt::skip]
    assert_actual_expected!(alice_knows_ab_bob.format(), indoc! {r#"
        "Alice" [
            "knows" [
                "A": "B"
            ]
            : "Bob"
        ]
    "#}.trim());

    let alice_knows_bob_ab = alice
        .add_assertion_envelope(Envelope::new_assertion(
            &knows,
            bob.add_assertion_envelope(&ab).unwrap(),
        ))
        .unwrap()
        .check_encoding()
        .unwrap();
    #[rustfmt::skip]
    assert_actual_expected!(alice_knows_bob_ab.format(), indoc! {r#"
        "Alice" [
            "knows": "Bob" [
                "A": "B"
            ]
        ]
    "#}.trim());

    let alice_knows_ab_bob_ab = alice
        .add_assertion_envelope(Envelope::new_assertion(
            knows.add_assertion_envelope(&ab).unwrap(),
            bob.add_assertion_envelope(&ab).unwrap(),
        ))
        .unwrap()
        .check_encoding()
        .unwrap();
    #[rustfmt::skip]
    assert_actual_expected!(alice_knows_ab_bob_ab.format(), indoc! {r#"
        "Alice" [
            "knows" [
                "A": "B"
            ]
            : "Bob" [
                "A": "B"
            ]
        ]
    "#}.trim());

    let alice_ab_knows_ab_bob_ab = alice
        .add_assertion_envelope(&ab)
        .unwrap()
        .add_assertion_envelope(Envelope::new_assertion(
            knows.add_assertion_envelope(&ab).unwrap(),
            bob.add_assertion_envelope(&ab).unwrap(),
        ))
        .unwrap()
        .check_encoding()
        .unwrap();
    #[rustfmt::skip]
    assert_actual_expected!(alice_ab_knows_ab_bob_ab.format(), indoc! {r#"
        "Alice" [
            "A": "B"
            "knows" [
                "A": "B"
            ]
            : "Bob" [
                "A": "B"
            ]
        ]
    "#}.trim());

    let alice_ab_knows_ab_bob_ab_enclose_ab = alice
        .add_assertion_envelope(&ab)
        .unwrap()
        .add_assertion_envelope(
            Envelope::new_assertion(
                knows.add_assertion_envelope(&ab).unwrap(),
                bob.add_assertion_envelope(&ab).unwrap(),
            )
            .add_assertion_envelope(ab)
            .unwrap(),
        )
        .unwrap()
        .check_encoding()
        .unwrap();
    #[rustfmt::skip]
    assert_actual_expected!(alice_ab_knows_ab_bob_ab_enclose_ab.format(), indoc! {r#"
        "Alice" [
            {
                "knows" [
                    "A": "B"
                ]
                : "Bob" [
                    "A": "B"
                ]
            } [
                "A": "B"
            ]
            "A": "B"
        ]
    "#}.trim());
}

#[test]
fn test_nesting_plaintext() {
    let envelope = Envelope::new("Hello.");

    #[rustfmt::skip]
    let expected_format = indoc! {r#"
        "Hello."
    "#}.trim();
    assert_actual_expected!(envelope.format(), expected_format);

    let elided_envelope = envelope.elide();
    assert!(elided_envelope.is_equivalent_to(&envelope));

    #[rustfmt::skip]
    let expected_elided_format = indoc! {r#"
        ELIDED
    "#}.trim();
    assert_actual_expected!(elided_envelope.format(), expected_elided_format);
}

#[test]
fn test_nesting_once() {
    let envelope = Envelope::new("Hello.").wrap().check_encoding().unwrap();

    #[rustfmt::skip]
    let expected_format = indoc! {r#"
        {
            "Hello."
        }
    "#}.trim();
    assert_actual_expected!(envelope.format(), expected_format);

    let elided_envelope = Envelope::new("Hello.")
        .elide()
        .wrap()
        .check_encoding()
        .unwrap();

    assert!(elided_envelope.is_equivalent_to(&envelope));

    #[rustfmt::skip]
    let expected_elided_format = indoc! {r#"
        {
            ELIDED
        }
    "#}.trim();
    assert_actual_expected!(elided_envelope.format(), expected_elided_format);
}

#[test]
fn test_nesting_twice() {
    let envelope = Envelope::new("Hello.")
        .wrap()
        .wrap()
        .check_encoding()
        .unwrap();

    #[rustfmt::skip]
    let expected_format = indoc! {r#"
        {
            {
                "Hello."
            }
        }
    "#}.trim();
    assert_actual_expected!(envelope.format(), expected_format);

    let target = envelope.try_unwrap().unwrap().try_unwrap().unwrap();
    let elided_envelope = envelope.elide_removing_target(&target);

    #[rustfmt::skip]
    let expected_elided_format = indoc! {r#"
        {
            {
                ELIDED
            }
        }
    "#}.trim();
    assert_actual_expected!(elided_envelope.format(), expected_elided_format);
    assert!(envelope.is_equivalent_to(&elided_envelope));
    assert!(envelope.is_equivalent_to(&elided_envelope));
}

#[test]
fn test_assertions_on_all_parts_of_envelope() {
    let predicate = Envelope::new("predicate")
        .add_assertion("predicate-predicate", "predicate-object");
    let object = Envelope::new("object")
        .add_assertion("object-predicate", "object-object");
    let envelope = Envelope::new("subject")
        .add_assertion(predicate, object)
        .check_encoding()
        .unwrap();

    #[rustfmt::skip]
    let expected_format = indoc! {r#"
        "subject" [
            "predicate" [
                "predicate-predicate": "predicate-object"
            ]
            : "object" [
                "object-predicate": "object-object"
            ]
        ]
    "#}.trim();
    assert_actual_expected!(envelope.format(), expected_format);
}

#[test]
fn test_assertion_on_bare_assertion() {
    let envelope = Envelope::new_assertion("predicate", "object")
        .add_assertion("assertion-predicate", "assertion-object");
    #[rustfmt::skip]
    let expected_format = indoc! {r#"
        {
            "predicate": "object"
        } [
            "assertion-predicate": "assertion-object"
        ]
    "#}.trim();
    assert_actual_expected!(envelope.format(), expected_format);
}
