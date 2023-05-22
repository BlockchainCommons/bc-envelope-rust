use std::rc::Rc;

use crate::{Envelope, with_format_context, KnownValue, known_value_registry};
use bc_components::DigestProvider;
use indoc::indoc;

fn basic_envelope() -> Rc<Envelope> {
    Envelope::new("Hello.")
}

fn known_value_envelope() -> Rc<Envelope> {
    Envelope::new(known_value_registry::NOTE)
}

fn assertion_envelope() -> Rc<Envelope> {
    Envelope::new_assertion(&"knows", &"Bob")
}

fn single_assertion_envelope() -> Rc<Envelope> {
    Envelope::new("Alice")
        .add_assertion_predicate_object("knows", "Bob")
        .unwrap()
}

#[test]
fn test_int_subject() {
    let e = Envelope::new_leaf(42).check_encoding().unwrap();

    with_format_context!(|context| {
        assert_eq!(e.diagnostic_opt(true, Some(context)),
        indoc! {r#"
        200(   ; envelope
           24(42)   ; leaf
        )
        "#}.trim()
        );
    });

    assert_eq!("Digest(7f83f7bda2d63959d34767689f06d47576683d378d9eb8d09386c9a020395c53)", e.digest().to_string());

    assert_eq!(e.format(),
    indoc! {r#"
    42
    "#}.trim()
    );

    assert_eq!(*e.extract_subject::<i32>().unwrap(), 42);
}

#[test]
fn test_negative_int_subject() {
    let e = Envelope::new_leaf(-42).check_encoding().unwrap();

    with_format_context!(|context| {
        assert_eq!(e.diagnostic_opt(true, Some(context)),
        indoc! {r#"
        200(   ; envelope
           24(-42)   ; leaf
        )
        "#}.trim()
        );
    });

    assert_eq!("Digest(9e0ad272780de7aa1dbdfbc99058bb81152f623d3b95b5dfb0a036badfcc9055)", e.digest().to_string());

    assert_eq!(e.format(),
    indoc! {r#"
    -42
    "#}.trim()
    );

    assert_eq!(*e.extract_subject::<i32>().unwrap(), -42);
}

#[test]
fn test_cbor_encodable_subject() {
    let e = basic_envelope().check_encoding().unwrap();
    with_format_context!(|context| {
        assert_eq!(e.diagnostic_opt(true, Some(context)),
        indoc! {r#"
        200(   ; envelope
           24("Hello.")   ; leaf
        )
        "#}.trim()
        );
    });

    assert_eq!("Digest(8cc96cdb771176e835114a0f8936690b41cfed0df22d014eedd64edaea945d59)", e.digest().to_string());

    assert_eq!(e.format(),
    indoc! {r#"
    "Hello."
    "#}.trim()
    );

    assert_eq!(*e.extract_subject::<String>().unwrap(), "Hello.");
}

#[test]
fn test_known_value_subject() {
    let e = known_value_envelope().check_encoding().unwrap();

    with_format_context!(|context| {
        assert_eq!(e.diagnostic_opt(true, Some(context)),
        indoc! {r#"
        200(   ; envelope
           202(4)   ; known-value
        )
        "#}.trim()
        );
    });

    assert_eq!("Digest(49a5f41b242e76fa4ed7083f4fb3b9cab117f3437b38083b7375d6f19f199508)", e.digest().to_string());

    assert_eq!(e.format(),
    indoc! {r#"
    note
    "#}.trim()
    );

    assert_eq!(*e.extract_subject::<KnownValue>().unwrap(), known_value_registry::NOTE);
}

#[test]
fn test_assertion_subject() {
    let e = assertion_envelope().check_encoding().unwrap();

    assert_eq!(e.clone().predicate().unwrap().digest().to_string(), "Digest(db7dd21c5169b4848d2a1bcb0a651c9617cdd90bae29156baaefbb2a8abef5ba)");
    assert_eq!(e.clone().object().unwrap().digest().to_string(), "Digest(13b741949c37b8e09cc3daa3194c58e4fd6b2f14d4b1d0f035a46d6d5a1d3f11)");
    assert_eq!(e.clone().subject().digest().to_string(), "Digest(78d666eb8f4c0977a0425ab6aa21ea16934a6bc97c6f0c3abaefac951c1714a2)");
    assert_eq!(e.clone().digest().to_string(), "Digest(78d666eb8f4c0977a0425ab6aa21ea16934a6bc97c6f0c3abaefac951c1714a2)");

    with_format_context!(|context| {
        assert_eq!(e.diagnostic_opt(true, Some(context)),
        indoc! {r#"
        200(   ; envelope
           201(   ; assertion
              [
                 200(   ; envelope
                    24("knows")   ; leaf
                 ),
                 200(   ; envelope
                    24("Bob")   ; leaf
                 )
              ]
           )
        )
        "#}.trim()
        );
    });

    assert_eq!(e.format(),
    indoc! {r#"
    "knows": "Bob"
    "#}.trim()
    );

    assert_eq!(e.digest(), Envelope::new_assertion(&"knows", &"Bob").digest());
}
