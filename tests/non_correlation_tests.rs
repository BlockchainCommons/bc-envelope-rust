#![cfg(feature = "salt")]
use bc_envelope::prelude::*;
use bc_rand::make_fake_random_number_generator;
use indoc::indoc;

mod common;
use crate::common::check_encoding::*;

#[test]
fn test_envelope_non_correlation() {
    let e1 = Envelope::new("Hello.");

    // e1 correlates with its elision
    assert!(e1.is_equivalent_to(&e1.elide()));

    // e2 is the same message, but with random salt
    let mut rng = make_fake_random_number_generator();
    let e2 = e1.add_salt_using(&mut rng).check_encoding().unwrap();

    #[rustfmt::skip]
    assert_eq!(e2.format(), indoc! {r#"
        "Hello." [
            'salt': Salt
        ]
    "#}.trim());

    #[rustfmt::skip]
    assert_eq!(e2.diagnostic_annotated(), indoc! {r#"
        200(   / envelope /
            [
                201("Hello."),   / leaf /
                {
                    15:
                    201(   / leaf /
                        40018(h'b559bbbf6cce2632')   / salt /
                    )
                }
            ]
        )
    "#}.trim());

    #[rustfmt::skip]
    assert_eq!(e2.tree_format(), indoc! {r#"
        4f0f2d55 NODE
            8cc96cdb subj "Hello."
            dd412f1d ASSERTION
                618975ce pred 'salt'
                7915f200 obj Salt
    "#}.trim());

    // So even though its content is the same, it doesn't correlate.
    assert!(!e1.is_equivalent_to(&e2));

    // And of course, neither does its elision.
    assert!(!e1.is_equivalent_to(&e2.elide()));
}

#[test]
fn test_predicate_correlation() {
    let e1 = Envelope::new("Foo")
        .add_assertion("note", "Bar")
        .check_encoding().unwrap();
    let e2 = Envelope::new("Baz")
        .add_assertion("note", "Quux")
        .check_encoding().unwrap();

    #[rustfmt::skip]
    let e1_expected_format = indoc! {r#"
        "Foo" [
            "note": "Bar"
        ]
    "#}.trim();
    assert_eq!(e1.format(), e1_expected_format);

    // e1 and e2 have the same predicate
    assert!(e1.assertions().first().unwrap().as_predicate().unwrap()
        .is_equivalent_to(&e2.assertions().first().unwrap().as_predicate().unwrap()));

    // Redact the entire contents of e1 without
    // redacting the envelope itself.
    let e1_elided = e1.elide_revealing_target(&e1).check_encoding().unwrap();

    #[rustfmt::skip]
    let redacted_expected_format = indoc! {r#"
        ELIDED [
            ELIDED
        ]
    "#}.trim();
    assert_eq!(e1_elided.format(), redacted_expected_format);
}

#[test]
fn test_add_salt() {
    let source = "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.";
    let e1 = Envelope::new("Alpha")
        .add_salt().check_encoding().unwrap()
        .wrap_envelope().check_encoding().unwrap()
        .add_assertion(
            Envelope::new(known_values::NOTE).add_salt().check_encoding().unwrap(),
            Envelope::new(source).add_salt().check_encoding().unwrap()
        ).check_encoding().unwrap();
    #[rustfmt::skip]
    let e1_expected_format = indoc! {r#"
        {
            "Alpha" [
                'salt': Salt
            ]
        } [
            'note' [
                'salt': Salt
            ]
            : "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum." [
                'salt': Salt
            ]
        ]
    "#}.trim();
    assert_eq!(e1.format(), e1_expected_format);

    let e1_elided = e1.elide_revealing_target(&e1).check_encoding().unwrap();

    #[rustfmt::skip]
    let redacted_expected_format = indoc! {r#"
        ELIDED [
            ELIDED
        ]
    "#}.trim();
    assert_eq!(e1_elided.format(), redacted_expected_format);
}
