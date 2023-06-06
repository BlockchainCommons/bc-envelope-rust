use crate::{Envelope, Enclosable};
use std::error::Error;
use indoc::indoc;

#[test]
fn test_predicate_enclosures() -> Result<(), Box<dyn Error>> {
    let alice = "Alice".enclose();
    let knows = "knows".enclose();
    let bob = "Bob".enclose();

    let a = "A".enclose();
    let b = "B".enclose();

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

    let knows_ab_bob = Envelope::new_assertion(knows.clone().add_assertion_envelope(ab.clone()), bob.clone()).check_encoding()?;
    assert_eq!(knows_ab_bob.format(),
    indoc! {r#"
    "knows" [
        "A": "B"
    ]
    : "Bob"
    "#}.trim()
    );

    let knows_bob_ab = Envelope::new_assertion(knows.clone(), bob.clone().add_assertion_envelope(ab.clone())).check_encoding()?;
    assert_eq!(knows_bob_ab.format(),
    indoc! {r#"
    "knows": "Bob" [
        "A": "B"
    ]
    "#}.trim()
    );

    let knows_bob_enclose_ab = knows_bob.clone()
        .add_assertion_envelope(ab.clone())
        .check_encoding()?;
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
        .add_assertion_envelope(knows_bob)
        .check_encoding()?;
    assert_eq!(alice_knows_bob.format(),
    indoc! {r#"
    "Alice" [
        "knows": "Bob"
    ]
    "#}.trim()
    );

    let alice_ab_knows_bob = alice_knows_bob
        .add_assertion_envelope(ab.clone())
        .check_encoding()?;
    assert_eq!(alice_ab_knows_bob.format(),
    indoc! {r#"
    "Alice" [
        "A": "B"
        "knows": "Bob"
    ]
    "#}.trim()
    );

    let alice_knows_ab_bob = alice.clone()
        .add_assertion_envelope(Envelope::new_assertion(knows.clone().add_assertion_envelope(ab.clone()), bob.clone()))
        .check_encoding()?;
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
        .add_assertion_envelope(Envelope::new_assertion(knows.clone(), bob.clone().add_assertion_envelope(ab.clone())))
        .check_encoding()?;
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
        .add_assertion_envelope(Envelope::new_assertion(knows.clone().add_assertion_envelope(ab.clone()), bob.clone().add_assertion_envelope(ab.clone())))
        .check_encoding()?;
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
        .add_assertion_envelope(ab.clone())
        .add_assertion_envelope(Envelope::new_assertion(knows.clone().add_assertion_envelope(ab.clone()), bob.clone().add_assertion_envelope(ab.clone())))
        .check_encoding()?;
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
        .add_assertion_envelope(ab.clone())
        .add_assertion_envelope(Envelope::new_assertion(knows.add_assertion_envelope(ab.clone()), bob.add_assertion_envelope(ab.clone())).add_assertion_envelope(ab))
        .check_encoding()?;
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

    Ok(())
}

#[test]
fn test_nesting_plaintext() -> Result<(), Box<dyn Error>> {
    let envelope = "Hello.".enclose();

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

    Ok(())
}

#[test]
fn test_nesting_once() -> Result<(), Box<dyn Error>> {
    let envelope = "Hello.".enclose()
        .wrap_envelope()
        .check_encoding()?;

    let expected_format = indoc! {r#"
    {
        "Hello."
    }
    "#}.trim();
    assert_eq!(envelope.format(), expected_format);

    let elided_envelope = "Hello.".enclose()
        .elide()
        .wrap_envelope()
        .check_encoding()?;

    assert!(elided_envelope.clone().is_equivalent_to(envelope));

    let expected_elided_format = indoc! {r#"
    {
        ELIDED
    }
    "#}.trim();
    assert_eq!(elided_envelope.format(), expected_elided_format);

    Ok(())
}

#[test]
fn test_nesting_twice() -> Result<(), Box<dyn Error>> {
    let envelope = "Hello.".enclose()
        .wrap_envelope()
        .wrap_envelope()
        .check_encoding()?;

    let expected_format = indoc! {r#"
    {
        {
            "Hello."
        }
    }
    "#}.trim();
    assert_eq!(envelope.format(), expected_format);

    let target = envelope.clone()
        .unwrap_envelope()?
        .unwrap_envelope()?;
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

    Ok(())
}

#[test]
fn test_assertions_on_all_parts_of_envelope() -> Result<(), Box<dyn Error>> {
    let predicate = "predicate".enclose()
        .add_assertion("predicate-predicate".enclose(), "predicate-object".enclose());
    let object = "object".enclose()
        .add_assertion("object-predicate".enclose(), "object-object".enclose());
    let envelope = "subject".enclose()
        .add_assertion(predicate, object)
        .check_encoding()?;

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

    Ok(())
}

#[test]
fn test_assertion_on_bare_assertion() -> Result<(), Box<dyn Error>> {
    let envelope = Envelope::new_assertion("predicate".enclose(), "object".enclose())
        .add_assertion("assertion-predicate".enclose(), "assertion-object".enclose());
    let expected_format = indoc! {r#"
    {
        "predicate": "object"
    } [
        "assertion-predicate": "assertion-object"
    ]
    "#}.trim();
    assert_eq!(envelope.format(), expected_format);

    Ok(())
}
