use std::rc::Rc;
use crate::{Envelope, with_format_context, KnownValue, known_value_registry};
use bc_components::DigestProvider;
use indoc::indoc;

#[test]
pub fn test_predicate_enclosures() {
    let alice = Envelope::new("Alice");
    let knows = Envelope::new("knows");
    let bob = Envelope::new("Bob");

    let a = Envelope::new("A");
    let b = Envelope::new("B");

    let knows_bob = Envelope::new_assertion_with_predobj(knows.clone(), bob.clone());
    assert_eq!(knows_bob.format(),
    indoc! {r#"
    "knows": "Bob"
    "#}.trim()
    );

    let ab = Envelope::new_assertion_with_predobj(a.clone(), b.clone());
    assert_eq!(ab.format(),
    indoc! {r#"
    "A": "B"
    "#}.trim()
    );

    let knows_ab_bob = Envelope::new_assertion_with_predobj(knows.clone().add_assertion(ab.clone()), bob.clone()).check_encoding().unwrap();
    assert_eq!(knows_ab_bob.format(),
    indoc! {r#"
    "knows" [
        "A": "B"
    ]
    : "Bob"
    "#}.trim()
    );

    let knows_bob_ab = Envelope::new_assertion_with_predobj(knows.clone(), bob.clone().add_assertion(ab.clone())).check_encoding().unwrap();
    assert_eq!(knows_bob_ab.format(),
    indoc! {r#"
    "knows": "Bob" [
        "A": "B"
    ]
    "#}.trim()
    );

    let knows_bob_enclose_ab = knows_bob.clone()
        .add_assertion(ab.clone())
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
        .add_assertion(knows_bob.clone())
        .check_encoding().unwrap();
    assert_eq!(alice_knows_bob.format(),
    indoc! {r#"
    "Alice" [
        "knows": "Bob"
    ]
    "#}.trim()
    );

    let alice_ab_knows_bob = alice_knows_bob.clone()
        .add_assertion(ab.clone())
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
        .add_assertion(Envelope::new_assertion_with_predobj(knows.clone().add_assertion(ab.clone()), bob.clone()))
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
        .add_assertion(Envelope::new_assertion_with_predobj(knows.clone(), bob.clone().add_assertion(ab.clone())))
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
        .add_assertion(Envelope::new_assertion_with_predobj(knows.clone().add_assertion(ab.clone()), bob.clone().add_assertion(ab.clone())))
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
        .add_assertion(ab.clone())
        .add_assertion(Envelope::new_assertion_with_predobj(knows.clone().add_assertion(ab.clone()), bob.clone().add_assertion(ab.clone())))
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

    let alice_ab_knows_ab_bob_ab_enclose_ab = alice.clone()
        .add_assertion(ab.clone())
        .add_assertion(Envelope::new_assertion_with_predobj(knows.clone().add_assertion(ab.clone()), bob.clone().add_assertion(ab.clone())).add_assertion(ab.clone()))
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

/*
```swift
    func testNestingPlaintext() {
        let envelope = Envelope(plaintextHello)

        let expectedFormat =
        """
        "Hello."
        """
        XCTAssertEqual(envelope.format(), expectedFormat)

        let elidedEnvelope = envelope.elide()
        XCTAssert(elidedEnvelope.isEquivalent(to: envelope))

        let expectedElidedFormat =
        """
        ELIDED
        """
        XCTAssertEqual(elidedEnvelope.format(), expectedElidedFormat)
    }

    func testNestingOnce() throws {
        let envelope = try Envelope(plaintextHello)
            .wrap()
            .checkEncoding()

        let expectedFormat =
        """
        {
            "Hello."
        }
        """
        XCTAssertEqual(envelope.format(), expectedFormat)

        let elidedEnvelope = try Envelope(plaintextHello)
            .elide()
            .wrap()
            .checkEncoding()

        XCTAssert(elidedEnvelope.isEquivalent(to: envelope))

        let expectedElidedFormat =
        """
        {
            ELIDED
        }
        """
        XCTAssertEqual(elidedEnvelope.format(), expectedElidedFormat)
    }

    func testNestingTwice() throws {
        let envelope = try Envelope(plaintextHello)
            .wrap()
            .wrap()
            .checkEncoding()

        let expectedFormat =
        """
        {
            {
                "Hello."
            }
        }
        """
        XCTAssertEqual(envelope.format(), expectedFormat)

        let target = try envelope
            .unwrap()
            .unwrap()
        let elidedEnvelope = try envelope.elideRemoving(target)

        let expectedElidedFormat =
        """
        {
            {
                ELIDED
            }
        }
        """
        XCTAssertEqual(elidedEnvelope.format(), expectedElidedFormat)
        XCTAssert(envelope.isEquivalent(to: elidedEnvelope))
        XCTAssert(envelope.isEquivalent(to: elidedEnvelope))
    }

    func testAssertionsOnAllPartsOfEnvelope() throws {
        let predicate = Envelope("predicate")
            .addAssertion("predicate-predicate", "predicate-object")
        let object = Envelope("object")
            .addAssertion("object-predicate", "object-object")
        let envelope = try Envelope("subject")
            .addAssertion(predicate, object)
            .checkEncoding()

        let expectedFormat =
        """
        "subject" [
            "predicate" [
                "predicate-predicate": "predicate-object"
            ]
            : "object" [
                "object-predicate": "object-object"
            ]
        ]
        """
        XCTAssertEqual(envelope.format(), expectedFormat)
    }

    func testAssertionOnBareAssertion() throws {
        let envelope = try Envelope("predicate", "object")
            .addAssertion(Envelope("assertion-predicate", "assertion-object"))
        let expectedFormat =
        """
        {
            "predicate": "object"
        } [
            "assertion-predicate": "assertion-object"
        ]
        """
        XCTAssertEqual(envelope.format(), expectedFormat)
    }
```
 */

#[test]
fn test_nesting_plaintext() {
    let envelope = Envelope::new("Hello.");

    let expected_format = indoc! {r#"
    "Hello."
    "#}.trim();
    assert_eq!(envelope.format(), expected_format);

    let elided_envelope = envelope.clone().elide();
    assert!(elided_envelope.clone().is_equivalent_to(envelope.clone()));

    let expected_elided_format = indoc! {r#"
    ELIDED
    "#}.trim();
    assert_eq!(elided_envelope.format(), expected_elided_format);
}

#[test]
fn test_nesting_once() {
    let envelope = Envelope::new("Hello.")
        .wrap()
        .check_encoding().unwrap();

    let expected_format = indoc! {r#"
    {
        "Hello."
    }
    "#}.trim();
    assert_eq!(envelope.format(), expected_format);

    let elided_envelope = Envelope::new("Hello.")
        .elide()
        .wrap()
        .check_encoding().unwrap();

    assert!(elided_envelope.clone().is_equivalent_to(envelope.clone()));

    let expected_elided_format = indoc! {r#"
    {
        ELIDED
    }
    "#}.trim();
    assert_eq!(elided_envelope.format(), expected_elided_format);
}

#[test]
fn test_nesting_twice() {
    let envelope = Envelope::new("Hello.")
        .wrap()
        .wrap()
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
        .unwrap()
        .unwrap();
    let elided_envelope = envelope.clone().elide_removing(target).unwrap();

    let expected_elided_format = indoc! {r#"
    {
        {
            ELIDED
        }
    }
    "#}.trim();
    assert_eq!(elided_envelope.format(), expected_elided_format);
    assert!(envelope.clone().is_equivalent_to(elided_envelope.clone()));
    assert!(envelope.clone().is_equivalent_to(elided_envelope.clone()));
}
