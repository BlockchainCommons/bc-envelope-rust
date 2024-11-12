use dcbor::prelude::*;
use indoc::indoc;
use bc_components::Digest;
use bc_envelope::prelude::*;

mod common;
use crate::common::check_encoding::*;

#[test]
fn test_digest() {
   Envelope::new(Digest::from_image("Hello.".as_bytes())).check_encoding().unwrap();
}

#[test]
fn test_1() -> anyhow::Result<()> {
    let e = Envelope::new("Hello.");

    assert_eq!(e.diagnostic_annotated(),
    indoc! {r#"
    200(   / envelope /
        201("Hello.")   / leaf /
    )
    "#}.trim()
    );

    Ok(())
}

#[test]
fn test_2() -> anyhow::Result<()> {
    let array: Vec<u64> = vec![1, 2, 3];
    let e = Envelope::new(Into::<CBOR>::into(array));

    assert_eq!(e.diagnostic_annotated(),
    indoc! {r#"
    200(   / envelope /
        201(   / leaf /
            [1, 2, 3]
        )
    )
    "#}.trim()
    );

    Ok(())
}

#[test]
fn test_3() -> anyhow::Result<()> {
    let e1 = Envelope::new_assertion("A", "B").check_encoding()?;
    let e2 = Envelope::new_assertion("C", "D").check_encoding()?;
    let e3 = Envelope::new_assertion("E", "F").check_encoding()?;

    let e4 = e2.add_assertion_envelope(e3).unwrap();
    assert_eq!(e4.format(),
        indoc! {r#"
        {
            "C": "D"
        } [
            "E": "F"
        ]
        "#}.trim()
    );

    // println!("{}", e4.diagnostic_annotated());

    assert_eq!(e4.diagnostic_annotated(),
    indoc! {r#"
    200(   / envelope /
        [
            {
                201("C"):   / leaf /
                201("D")   / leaf /
            },
            {
                201("E"):   / leaf /
                201("F")   / leaf /
            }
        ]
    )
    "#}.trim()
    );

    e4.check_encoding()?;

    let e5 = e1.add_assertion_envelope(e4).unwrap().check_encoding()?;

    assert_eq!(e5.format(),
        indoc! {r#"
        {
            "A": "B"
        } [
            {
                "C": "D"
            } [
                "E": "F"
            ]
        ]
        "#}.trim()
    );

    assert_eq!(e5.diagnostic_annotated(),
    indoc! {r#"
    200(   / envelope /
        [
            {
                201("A"):   / leaf /
                201("B")   / leaf /
            },
            [
                {
                    201("C"):   / leaf /
                    201("D")   / leaf /
                },
                {
                    201("E"):   / leaf /
                    201("F")   / leaf /
                }
            ]
        ]
    )
    "#}.trim()
    );

    Ok(())
}
