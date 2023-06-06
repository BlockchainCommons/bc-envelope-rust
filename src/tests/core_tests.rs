use std::error::Error;
use std::rc::Rc;
use crate::{Envelope, with_format_context, KnownValue, known_value_registry, Enclosable, enclose_cbor};
use bc_components::DigestProvider;
use indoc::indoc;

fn basic_envelope() -> Rc<Envelope> {
    "Hello.".enclose()
}

fn known_value_envelope() -> Rc<Envelope> {
    known_value_registry::NOTE.enclose()
}

fn assertion_envelope() -> Rc<Envelope> {
    Envelope::new_assertion_with_predobj("knows".enclose(), "Bob".enclose())
}

fn single_assertion_envelope() -> Rc<Envelope> {
    "Alice".enclose()
        .add_assertion_with_predobj("knows".enclose(), "Bob".enclose())
}

fn double_assertion_envelope() -> Rc<Envelope> {
    single_assertion_envelope()
        .add_assertion_with_predobj("knows".enclose(), "Carol".enclose())
}

fn wrapped_envelope() -> Rc<Envelope> {
    basic_envelope().enclose()
}

fn double_wrapped_envelope() -> Rc<Envelope> {
    wrapped_envelope().enclose()
}

#[test]
fn test_int_subject() -> Result<(), Box<dyn Error>> {
    let e = 42.enclose().check_encoding()?;

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

    assert_eq!(*e.extract_subject::<i32>()?, 42);

    Ok(())
}

#[test]
fn test_negative_int_subject() -> Result<(), Box<dyn Error>> {
    let e = (-42).enclose().check_encoding()?;

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

    assert_eq!(*e.extract_subject::<i32>()?, -42);

    Ok(())
}

#[test]
fn test_cbor_encodable_subject() -> Result<(), Box<dyn Error>> {
    let e = basic_envelope().check_encoding()?;
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

    assert_eq!(*e.extract_subject::<String>()?, "Hello.");

    Ok(())
}

#[test]
fn test_known_value_subject() -> Result<(), Box<dyn Error>> {
    let e = known_value_envelope().check_encoding()?;

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

    assert_eq!(*e.extract_subject::<KnownValue>()?, known_value_registry::NOTE);

    Ok(())
}

#[test]
fn test_assertion_subject() -> Result<(), Box<dyn Error>> {
    let e = assertion_envelope().check_encoding()?;

    assert_eq!(e.clone().predicate().unwrap().digest().to_string(), "Digest(db7dd21c5169b4848d2a1bcb0a651c9617cdd90bae29156baaefbb2a8abef5ba)");
    assert_eq!(e.clone().object().unwrap().digest().to_string(), "Digest(13b741949c37b8e09cc3daa3194c58e4fd6b2f14d4b1d0f035a46d6d5a1d3f11)");
    assert_eq!(e.clone().subject().digest().to_string(), "Digest(78d666eb8f4c0977a0425ab6aa21ea16934a6bc97c6f0c3abaefac951c1714a2)");
    assert_eq!(e.digest().to_string(), "Digest(78d666eb8f4c0977a0425ab6aa21ea16934a6bc97c6f0c3abaefac951c1714a2)");

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

    assert_eq!(e.digest(), Envelope::new_assertion_with_predobj("knows".enclose(), "Bob".enclose()).digest());

    Ok(())
}

#[test]
fn test_subject_with_assertion() -> Result<(), Box<dyn Error>> {
    let e = single_assertion_envelope().check_encoding()?;

    with_format_context!(|context| {
        assert_eq!(e.diagnostic_opt(true, Some(context)),
        indoc! {r#"
        200(   ; envelope
           [
              200(   ; envelope
                 24("Alice")   ; leaf
              ),
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
           ]
        )
        "#}.trim()
        );
    });

    assert_eq!(e.digest().to_string(), "Digest(8955db5e016affb133df56c11fe6c5c82fa3036263d651286d134c7e56c0e9f2)");

    assert_eq!(e.format(),
    indoc! {r#"
    "Alice" [
        "knows": "Bob"
    ]
    "#}.trim()
    );

    assert_eq!(*e.extract_subject::<String>()?, "Alice");

    Ok(())
}

#[test]
fn test_subject_with_two_assertions() -> Result<(), Box<dyn Error>> {
    let e = double_assertion_envelope().check_encoding()?;

    with_format_context!(|context| {
        assert_eq!(e.diagnostic_opt(true, Some(context)),
        indoc! {r#"
        200(   ; envelope
           [
              200(   ; envelope
                 24("Alice")   ; leaf
              ),
              200(   ; envelope
                 201(   ; assertion
                    [
                       200(   ; envelope
                          24("knows")   ; leaf
                       ),
                       200(   ; envelope
                          24("Carol")   ; leaf
                       )
                    ]
                 )
              ),
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
           ]
        )
        "#}.trim()
        );
    });

    assert_eq!(e.digest().to_string(), "Digest(b8d857f6e06a836fbc68ca0ce43e55ceb98eefd949119dab344e11c4ba5a0471)");

    assert_eq!(e.format(),
    indoc! {r#"
    "Alice" [
        "knows": "Bob"
        "knows": "Carol"
    ]
    "#}.trim()
    );

    assert_eq!(*e.extract_subject::<String>()?, "Alice");

    Ok(())
}

#[test]
fn test_wrapped() -> Result<(), Box<dyn Error>> {
    let e = wrapped_envelope().check_encoding()?;

    with_format_context!(|context| {
        assert_eq!(e.diagnostic_opt(true, Some(context)),
        indoc! {r#"
        200(   ; envelope
           203(   ; wrapped-envelope
              24("Hello.")   ; leaf
           )
        )
        "#}.trim()
        );
    });

    assert_eq!(e.digest().to_string(), "Digest(172a5e51431062e7b13525cbceb8ad8475977444cf28423e21c0d1dcbdfcaf47)");

    assert_eq!(e.format(),
    indoc! {r#"
    {
        "Hello."
    }
    "#}.trim()
    );

    Ok(())
}

#[test]
fn test_double_wrapped() -> Result<(), Box<dyn Error>> {
    let e = double_wrapped_envelope().check_encoding()?;

    with_format_context!(|context| {
        assert_eq!(e.diagnostic_opt(true, Some(context)),
        indoc! {r#"
        200(   ; envelope
           203(   ; wrapped-envelope
              203(   ; wrapped-envelope
                 24("Hello.")   ; leaf
              )
           )
        )
        "#}.trim()
        );
    });

    assert_eq!(e.digest().to_string(), "Digest(8b14f3bcd7c05aac8f2162e7047d7ef5d5eab7d82ee3f9dc4846c70bae4d200b)");

    assert_eq!(e.format(),
    indoc! {r#"
    {
        {
            "Hello."
        }
    }
    "#}.trim()
    );

    Ok(())
}

#[test]
fn test_assertion_with_assertions() -> Result<(), Box<dyn Error>> {
    let a = Envelope::new_assertion_with_predobj(1.enclose(), 2.enclose())
        .add_assertion_with_predobj(3.enclose(), 4.enclose())
        .add_assertion_with_predobj(5.enclose(), 6.enclose());
    let e = 7.enclose()
        .add_assertion(a);
    assert_eq!(e.format(),
    indoc! {r#"
    7 [
        {
            1: 2
        } [
            3: 4
            5: 6
        ]
    ]
    "#}.trim()
    );

    Ok(())
}

#[test]
fn test_digest_leaf() -> Result<(), Box<dyn Error>> {
    let digest = basic_envelope().digest().into_owned();
    let e = enclose_cbor(&digest).check_encoding()?;
    assert_eq!(e.format(),
    indoc! {r#"
    Digest(8cc96cdb)
    "#}.trim()
    );

    assert_eq!(e.digest().to_string(), "Digest(17db10e567ceb05522f0074c27c7d7796cac1d5ce20e45f405ab9063fdeeff1a)");

    with_format_context!(|context| {
        assert_eq!(e.diagnostic_opt(true, Some(context)),
        indoc! {r#"
        200(   ; envelope
           24(   ; leaf
              204(   ; digest
                 h'8cc96cdb771176e835114a0f8936690b41cfed0df22d014eedd64edaea945d59'
              )
           )
        )
        "#}.trim()
        );
    });

    Ok(())
}
