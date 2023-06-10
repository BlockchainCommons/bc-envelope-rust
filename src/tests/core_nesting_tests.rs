use crate::{Envelope, IntoEnvelope};
use indoc::indoc;

#[test]
fn test_predicate_enclosures() {
    let alice = "Alice".into_envelope();
    let knows = "knows".into_envelope();
    let bob = "Bob".into_envelope();

    let a = "A".into_envelope();
    let b = "B".into_envelope();

    let knows_bob = Envelope::new_assertion(knows.clone(), bob.clone());
    assert_eq!(knows_bob.format(),
    indoc! {r#"
    "knows": "Bob"
    "#}.trim()
    );

    let ab = Envelope::new_assertion(a, b);
    assert_eq!(ab.format(),
    indoc! {r#"
    "A": "B"
    "#}.trim()
    );

    let knows_ab_bob = Envelope::new_assertion(knows.clone().add_assertion_envelope(ab.clone()).unwrap(), bob.clone()).check_encoding().unwrap();
    assert_eq!(knows_ab_bob.format(),
    indoc! {r#"
    "knows" [
        "A": "B"
    ]
    : "Bob"
    "#}.trim()
    );

    let knows_bob_ab = Envelope::new_assertion(knows.clone(), bob.clone().add_assertion_envelope(ab.clone()).unwrap()).check_encoding().unwrap();
    assert_eq!(knows_bob_ab.format(),
    indoc! {r#"
    "knows": "Bob" [
        "A": "B"
    ]
    "#}.trim()
    );

    let knows_bob_enclose_ab = knows_bob.clone()
        .add_assertion_envelope(ab.clone()).unwrap()
        .check_encoding().unwrap();
    assert_eq!(knows_bob_enclose_ab.format(),
    indoc! {r#"
    {
        "knows": "Bob"
    } [
        "A": "B"
    ]
    "#}.trim()
    );

    let alice_knows_bob = alice.clone()
        .add_assertion_envelope(knows_bob).unwrap()
        .check_encoding().unwrap();
    assert_eq!(alice_knows_bob.format(),
    indoc! {r#"
    "Alice" [
        "knows": "Bob"
    ]
    "#}.trim()
    );

    let alice_ab_knows_bob = alice_knows_bob
        .add_assertion_envelope(ab.clone()).unwrap()
        .check_encoding().unwrap();
    assert_eq!(alice_ab_knows_bob.format(),
    indoc! {r#"
    "Alice" [
        "A": "B"
        "knows": "Bob"
    ]
    "#}.trim()
    );

    let alice_knows_ab_bob = alice.clone()
        .add_assertion_envelope(Envelope::new_assertion(knows.clone().add_assertion_envelope(ab.clone()).unwrap(), bob.clone())).unwrap()
        .check_encoding().unwrap();
    assert_eq!(alice_knows_ab_bob.format(),
    indoc! {r#"
    "Alice" [
        "knows" [
            "A": "B"
        ]
        : "Bob"
    ]
    "#}.trim()
    );

    let alice_knows_bob_ab = alice.clone()
        .add_assertion_envelope(Envelope::new_assertion(knows.clone(), bob.clone().add_assertion_envelope(ab.clone()).unwrap())).unwrap()
        .check_encoding().unwrap();
    assert_eq!(alice_knows_bob_ab.format(),
    indoc! {r#"
    "Alice" [
        "knows": "Bob" [
            "A": "B"
        ]
    ]
    "#}.trim()
    );

    let alice_knows_ab_bob_ab = alice.clone()
        .add_assertion_envelope(Envelope::new_assertion(knows.clone().add_assertion_envelope(ab.clone()).unwrap(), bob.clone().add_assertion_envelope(ab.clone()).unwrap())).unwrap()
        .check_encoding().unwrap();
    assert_eq!(alice_knows_ab_bob_ab.format(),
    indoc! {r#"
    "Alice" [
        "knows" [
            "A": "B"
        ]
        : "Bob" [
            "A": "B"
        ]
    ]
    "#}.trim()
    );

    let alice_ab_knows_ab_bob_ab = alice.clone()
        .add_assertion_envelope(ab.clone()).unwrap()
        .add_assertion_envelope(Envelope::new_assertion(knows.clone().add_assertion_envelope(ab.clone()).unwrap(), bob.clone().add_assertion_envelope(ab.clone()).unwrap())).unwrap()
        .check_encoding().unwrap();
    assert_eq!(alice_ab_knows_ab_bob_ab.format(),
    indoc! {r#"
    "Alice" [
        "A": "B"
        "knows" [
            "A": "B"
        ]
        : "Bob" [
            "A": "B"
        ]
    ]
    "#}.trim()
    );

    let alice_ab_knows_ab_bob_ab_enclose_ab = alice
        .add_assertion_envelope(ab.clone()).unwrap()
        .add_assertion_envelope(Envelope::new_assertion(knows.add_assertion_envelope(ab.clone()).unwrap(), bob.add_assertion_envelope(ab.clone()).unwrap()).add_assertion_envelope(ab).unwrap()).unwrap()
        .check_encoding().unwrap();
    assert_eq!(alice_ab_knows_ab_bob_ab_enclose_ab.format(),
    indoc! {r#"
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
    "#}.trim()
    );
}

#[test]
fn test_nesting_plaintext() {
    let envelope = "Hello.".into_envelope();

    let expected_format = indoc! {r#"
    "Hello."
    "#}.trim();
    assert_eq!(envelope.format(), expected_format);

    let elided_envelope = envelope.clone().elide();
    assert!(elided_envelope.clone().is_equivalent_to(envelope));

    let expected_elided_format = indoc! {r#"
    ELIDED
    "#}.trim();
    assert_eq!(elided_envelope.format(), expected_elided_format);
}

#[test]
fn test_nesting_once() {
    let envelope = "Hello.".into_envelope()
        .wrap_envelope()
        .check_encoding().unwrap();

    let expected_format = indoc! {r#"
    {
        "Hello."
    }
    "#}.trim();
    assert_eq!(envelope.format(), expected_format);

    let elided_envelope = "Hello.".into_envelope()
        .elide()
        .wrap_envelope()
        .check_encoding().unwrap();

    assert!(elided_envelope.clone().is_equivalent_to(envelope));

    let expected_elided_format = indoc! {r#"
    {
        ELIDED
    }
    "#}.trim();
    assert_eq!(elided_envelope.format(), expected_elided_format);
}

#[test]
fn test_nesting_twice() {
    let envelope = "Hello.".into_envelope()
        .wrap_envelope()
        .wrap_envelope()
        .check_encoding().unwrap();

    let expected_format = indoc! {r#"
    {
        {
            "Hello."
        }
    }
    "#}.trim();
    assert_eq!(envelope.format(), expected_format);

    let target = envelope.clone()
        .unwrap_envelope().unwrap()
        .unwrap_envelope().unwrap();
    let elided_envelope = envelope.clone().elide_removing_target(&target);

    let expected_elided_format = indoc! {r#"
    {
        {
            ELIDED
        }
    }
    "#}.trim();
    assert_eq!(elided_envelope.format(), expected_elided_format);
    assert!(envelope.clone().is_equivalent_to(elided_envelope.clone()));
    assert!(envelope.is_equivalent_to(elided_envelope));
}

#[test]
fn test_assertions_on_all_parts_of_envelope() {
    let predicate = "predicate".into_envelope()
        .add_assertion("predicate-predicate", "predicate-object");
    let object = "object".into_envelope()
        .add_assertion("object-predicate", "object-object");
    let envelope = "subject".into_envelope()
        .add_assertion(predicate, object)
        .check_encoding().unwrap();

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
    assert_eq!(envelope.format(), expected_format);
}

#[test]
fn test_assertion_on_bare_assertion() {
    let envelope = Envelope::new_assertion("predicate", "object")
        .add_assertion("assertion-predicate", "assertion-object");
    let expected_format = indoc! {r#"
    {
        "predicate": "object"
    } [
        "assertion-predicate": "assertion-object"
    ]
    "#}.trim();
    assert_eq!(envelope.format(), expected_format);
}
