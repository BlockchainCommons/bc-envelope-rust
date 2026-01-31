use bc_components::DigestProvider;
use bc_envelope::prelude::*;
use indoc::indoc;

mod common;
use crate::common::{check_encoding::*, test_data::*};

// A previous version of the Envelope spec used tag #6.24 ("Encoded CBOR Item")
// as the header for the Envelope `leaf` case. Unfortunately, this was not a
// correct use of the tag, as the contents of #6.24 (RFC8949 §3.4.5.1) MUST
// always be a byte string, while we were simply using it as a wrapper/header
// for any dCBOR data item.
//
// https://www.rfc-editor.org/rfc/rfc8949.html#name-encoded-cbor-data-item
//
// The new leaf tag is #6.201, but we will still recognize #6.24 for backwards
// compatibility.
//
// This test ensures that Envelopes encoded with the old tag are still
// correctly decoded as `leaf` cases.
#[test]
fn test_read_legacy_leaf() {
    let legacy_envelope =
        Envelope::from_tagged_cbor_data(hex_literal::hex!("d8c8d818182a"))
            .unwrap();
    let e = Envelope::new(42);
    assert!(legacy_envelope.is_identical_to(&e));
    assert!(legacy_envelope.is_equivalent_to(&e));
}

#[test]
fn test_int_subject() {
    let e = Envelope::new(42).check_encoding().unwrap();

    // expected-text-output-rubric:
    #[rustfmt::skip]
    assert_actual_expected!(e.diagnostic_annotated(), indoc! {r#"
        200(   / envelope /
            201(42)   / leaf /
        )
    "#}.trim());

    assert_actual_expected!(
        e.digest().to_string(),
        "Digest(7f83f7bda2d63959d34767689f06d47576683d378d9eb8d09386c9a020395c53)"
    );

    // expected-text-output-rubric:
    #[rustfmt::skip]
    assert_actual_expected!(e.format(), indoc! {r#"
        42
    "#}.trim());

    assert_eq!(e.extract_subject::<i32>().unwrap(), 42);
}

#[test]
fn test_negative_int_subject() {
    let e = Envelope::new(-42).check_encoding().unwrap();

    // expected-text-output-rubric:
    #[rustfmt::skip]
    assert_actual_expected!(e.diagnostic_annotated(), indoc! {r#"
        200(   / envelope /
            201(-42)   / leaf /
        )
    "#}.trim());

    assert_actual_expected!(
        e.digest().to_string(),
        "Digest(9e0ad272780de7aa1dbdfbc99058bb81152f623d3b95b5dfb0a036badfcc9055)"
    );

    // expected-text-output-rubric:
    #[rustfmt::skip]
    assert_actual_expected!(e.format(), indoc! {r#"
        -42
    "#}.trim());

    assert_eq!(e.extract_subject::<i32>().unwrap(), -42);
}

#[test]
fn test_cbor_encodable_subject() {
    let e = hello_envelope().check_encoding().unwrap();

    // expected-text-output-rubric:
    #[rustfmt::skip]
    assert_actual_expected!(e.diagnostic_annotated(), indoc! {r#"
        200(   / envelope /
            201("Hello.")   / leaf /
        )
    "#}.trim());

    assert_actual_expected!(
        e.digest().to_string(),
        "Digest(8cc96cdb771176e835114a0f8936690b41cfed0df22d014eedd64edaea945d59)"
    );

    // expected-text-output-rubric:
    #[rustfmt::skip]
    assert_actual_expected!(e.format(), indoc! {r#"
        "Hello."
    "#}.trim());

    assert_eq!(e.extract_subject::<String>().unwrap(), PLAINTEXT_HELLO);
}

#[cfg(feature = "known_value")]
#[test]
fn test_known_value_subject() {
    let e = known_value_envelope().check_encoding().unwrap();

    // expected-text-output-rubric:
    #[rustfmt::skip]
    assert_actual_expected!(e.diagnostic_annotated(), indoc! {r#"
        200(4)   / envelope /
    "#}.trim());

    assert_actual_expected!(
        e.digest().to_string(),
        "Digest(0fcd6a39d6ed37f2e2efa6a96214596f1b28a5cd42a5a27afc32162aaf821191)"
    );

    // expected-text-output-rubric:
    #[rustfmt::skip]
    assert_actual_expected!(e.format(), indoc! {r#"
        'note'
    "#}.trim());

    assert_eq!(
        e.extract_subject::<KnownValue>().unwrap(),
        known_values::NOTE
    );
}

#[test]
fn test_assertion_subject() {
    let e = assertion_envelope().check_encoding().unwrap();

    assert_actual_expected!(
        e.as_predicate().unwrap().digest().to_string(),
        "Digest(db7dd21c5169b4848d2a1bcb0a651c9617cdd90bae29156baaefbb2a8abef5ba)"
    );
    assert_actual_expected!(
        e.as_object().unwrap().digest().to_string(),
        "Digest(13b741949c37b8e09cc3daa3194c58e4fd6b2f14d4b1d0f035a46d6d5a1d3f11)"
    );
    assert_actual_expected!(
        e.subject().digest().to_string(),
        "Digest(78d666eb8f4c0977a0425ab6aa21ea16934a6bc97c6f0c3abaefac951c1714a2)"
    );
    assert_actual_expected!(
        e.digest().to_string(),
        "Digest(78d666eb8f4c0977a0425ab6aa21ea16934a6bc97c6f0c3abaefac951c1714a2)"
    );

    // expected-text-output-rubric:
    #[rustfmt::skip]
    assert_actual_expected!(e.diagnostic_annotated(), indoc! {r#"
        200(   / envelope /
            {
                201("knows"):   / leaf /
                201("Bob")   / leaf /
            }
        )
    "#}.trim());

    // expected-text-output-rubric:
    #[rustfmt::skip]
    assert_actual_expected!(e.format(), indoc! {r#"
        "knows": "Bob"
    "#}.trim());

    assert_eq!(e.digest(), Envelope::new_assertion("knows", "Bob").digest());
}

#[test]
fn test_subject_with_assertion() {
    let e = single_assertion_envelope().check_encoding().unwrap();

    // expected-text-output-rubric:
    #[rustfmt::skip]
    assert_actual_expected!(e.diagnostic_annotated(), indoc! {r#"
        200(   / envelope /
            [
                201("Alice"),   / leaf /
                {
                    201("knows"):   / leaf /
                    201("Bob")   / leaf /
                }
            ]
        )
    "#}.trim());

    assert_actual_expected!(
        e.digest().to_string(),
        "Digest(8955db5e016affb133df56c11fe6c5c82fa3036263d651286d134c7e56c0e9f2)"
    );

    // expected-text-output-rubric:
    #[rustfmt::skip]
    assert_actual_expected!(e.format(), indoc! {r#"
        "Alice" [
            "knows": "Bob"
        ]
    "#}.trim());

    assert_eq!(e.extract_subject::<String>().unwrap(), "Alice");
}

#[test]
fn test_subject_with_two_assertions() {
    let e = double_assertion_envelope().check_encoding().unwrap();

    // expected-text-output-rubric:
    #[rustfmt::skip]
    assert_actual_expected!(e.diagnostic_annotated(), indoc! {r#"
        200(   / envelope /
            [
                201("Alice"),   / leaf /
                {
                    201("knows"):   / leaf /
                    201("Carol")   / leaf /
                },
                {
                    201("knows"):   / leaf /
                    201("Bob")   / leaf /
                }
            ]
        )
    "#}.trim());

    assert_actual_expected!(
        e.digest().to_string(),
        "Digest(b8d857f6e06a836fbc68ca0ce43e55ceb98eefd949119dab344e11c4ba5a0471)"
    );

    // expected-text-output-rubric:
    #[rustfmt::skip]
    assert_actual_expected!(e.format(), indoc! {r#"
        "Alice" [
            "knows": "Bob"
            "knows": "Carol"
        ]
    "#}.trim());

    assert_eq!(e.extract_subject::<String>().unwrap(), "Alice");
}

#[test]
fn test_wrapped() {
    let e = wrapped_envelope().check_encoding().unwrap();

    // expected-text-output-rubric:
    #[rustfmt::skip]
    assert_actual_expected!(e.diagnostic_annotated(), indoc! {r#"
        200(   / envelope /
            200(   / envelope /
                201("Hello.")   / leaf /
            )
        )
    "#}.trim());

    assert_actual_expected!(
        e.digest().to_string(),
        "Digest(172a5e51431062e7b13525cbceb8ad8475977444cf28423e21c0d1dcbdfcaf47)"
    );

    // expected-text-output-rubric:
    #[rustfmt::skip]
    assert_actual_expected!(e.format(), indoc! {r#"
        {
            "Hello."
        }
    "#}.trim());
}

#[test]
fn test_double_wrapped() {
    let e = double_wrapped_envelope().check_encoding().unwrap();

    // expected-text-output-rubric:
    #[rustfmt::skip]
    assert_actual_expected!(e.diagnostic_annotated(), indoc! {r#"
        200(   / envelope /
            200(   / envelope /
                200(   / envelope /
                    201("Hello.")   / leaf /
                )
            )
        )
    "#}.trim());

    assert_actual_expected!(
        e.digest().to_string(),
        "Digest(8b14f3bcd7c05aac8f2162e7047d7ef5d5eab7d82ee3f9dc4846c70bae4d200b)"
    );

    // expected-text-output-rubric:
    #[rustfmt::skip]
    assert_actual_expected!(e.format(), indoc! {r#"
        {
            {
                "Hello."
            }
        }
    "#}.trim());
}

#[test]
fn test_assertion_with_assertions() {
    let a = Envelope::new_assertion(1, 2)
        .add_assertion(3, 4)
        .add_assertion(5, 6);
    let e = Envelope::new(7).add_assertion_envelope(a).unwrap();
    // expected-text-output-rubric:
    #[rustfmt::skip]
    assert_actual_expected!(e.format(), indoc! {r#"
        7 [
            {
                1: 2
            } [
                3: 4
                5: 6
            ]
        ]
    "#}.trim());
}

#[test]
fn test_digest_leaf() {
    let digest = hello_envelope().digest();
    let e = Envelope::new(digest).check_encoding().unwrap();
    // expected-text-output-rubric:
    #[rustfmt::skip]
    assert_actual_expected!(e.format(), indoc! {r#"
        Digest(8cc96cdb)
    "#}.trim());

    assert_actual_expected!(
        e.digest().to_string(),
        "Digest(07b518af92a6196bc153752aabefedb34ff8e1a7d820c01ef978dfc3e7e52e05)"
    );

    // expected-text-output-rubric:
    #[rustfmt::skip]
    assert_actual_expected!(e.diagnostic_annotated(), indoc! {r#"
        200(   / envelope /
            201(   / leaf /
                40001(   / digest /
                    h'8cc96cdb771176e835114a0f8936690b41cfed0df22d014eedd64edaea945d59'
                )
            )
        )
    "#}.trim());
}

#[test]
fn test_unknown_leaf() {
    crate::register_tags();

    let unknown_ur =
        "ur:envelope/tpsotaaodnoyadgdjlssmkcklgoskseodnyteofwwfylkiftaydpdsjz";
    let e = Envelope::from_ur_string(unknown_ur).unwrap();
    let expected = "555({1: h'6fc4981e8da778332bf93342f3f77d3a'})";
    assert_actual_expected!(e.format(), expected);
}

#[test]
fn test_true() {
    crate::register_tags();
    let e = Envelope::new(true).check_encoding().unwrap();
    assert!(e.is_bool());
    assert!(e.is_true());
    assert!(!e.is_false());
    assert_eq!(e, Envelope::r#true());
    // expected-text-output-rubric:
    #[rustfmt::skip]
    assert_actual_expected!(e.format(), indoc! {r#"
        true
    "#}.trim());
}

#[test]
fn test_false() {
    crate::register_tags();
    let e = Envelope::new(false).check_encoding().unwrap();
    assert!(e.is_bool());
    assert!(!e.is_true());
    assert!(e.is_false());
    assert_eq!(e, Envelope::r#false());
    // expected-text-output-rubric:
    #[rustfmt::skip]
    assert_actual_expected!(e.format(), indoc! {r#"
        false
    "#}.trim());
}

#[cfg(feature = "known_value")]
#[test]
fn test_unit() {
    crate::register_tags();
    let mut e = Envelope::unit().check_encoding().unwrap();
    assert!(e.is_subject_unit());
    // expected-text-output-rubric:
    #[rustfmt::skip]
    assert_actual_expected!(e.format(), indoc! {r#"
        ''
    "#}.trim());

    e = e.add_assertion("foo", "bar");
    assert!(e.is_subject_unit());
    // expected-text-output-rubric:
    #[rustfmt::skip]
    assert_actual_expected!(e.format(), indoc! {r#"
        '' [
            "foo": "bar"
        ]
    "#}.trim());

    let subject = e.extract_subject::<KnownValue>().unwrap();
    assert_eq!(subject, known_values::UNIT);
}

#[cfg(feature = "known_value")]
#[test]
fn test_position() {
    crate::register_tags();

    let mut e = Envelope::new("Hello");
    assert!(e.position().is_err());

    e = e.set_position(42).unwrap();
    assert_eq!(e.position().unwrap(), 42);
    // expected-text-output-rubric:
    #[rustfmt::skip]
    assert_actual_expected!(e.format(), indoc! {r#"
        "Hello" [
            'position': 42
        ]
    "#}.trim());

    e = e.set_position(0).unwrap();
    assert_eq!(e.position().unwrap(), 0);
    // expected-text-output-rubric:
    #[rustfmt::skip]
    assert_actual_expected!(e.format(), indoc! {r#"
        "Hello" [
            'position': 0
        ]
    "#}.trim());

    e = e.remove_position().unwrap();
    assert!(e.position().is_err());
    // expected-text-output-rubric:
    #[rustfmt::skip]
    assert_actual_expected!(e.format(), indoc! {r#"
        "Hello"
    "#}.trim());
}
