use bc_components::Digest;
use bc_envelope::prelude::*;
use indoc::indoc;

mod common;
use crate::common::check_encoding::*;

#[test]
fn test_digest() {
    Envelope::new(Digest::from_image("Hello.".as_bytes()))
        .check_encoding()
        .unwrap();
}

#[test]
fn test_1() -> EnvelopeResult<()> {
    let e = Envelope::new("Hello.");

    // expected-text-output-rubric:
    #[rustfmt::skip]
    assert_actual_expected!(e.diagnostic_annotated(), indoc! {r#"
        200(   / envelope /
            201("Hello.")   / leaf /
        )
    "#}.trim());

    Ok(())
}

#[test]
fn test_2() -> EnvelopeResult<()> {
    let array: Vec<u64> = vec![1, 2, 3];
    let e = Envelope::new(Into::<CBOR>::into(array));

    // expected-text-output-rubric:
    #[rustfmt::skip]
    assert_actual_expected!(e.diagnostic_annotated(), indoc! {r#"
        200(   / envelope /
            201(   / leaf /
                [1, 2, 3]
            )
        )
    "#}.trim());

    Ok(())
}

#[test]
fn test_3() -> EnvelopeResult<()> {
    let e1 = Envelope::new_assertion("A", "B").check_encoding()?;
    let e2 = Envelope::new_assertion("C", "D").check_encoding()?;
    let e3 = Envelope::new_assertion("E", "F").check_encoding()?;

    let e4 = e2.add_assertion_envelope(e3).unwrap();
    // expected-text-output-rubric:
    #[rustfmt::skip]
    assert_actual_expected!(e4.format(), indoc! {r#"
        {
            "C": "D"
        } [
            "E": "F"
        ]
    "#}.trim());

    // println!("{}", e4.diagnostic_annotated());

    // expected-text-output-rubric:
    #[rustfmt::skip]
    assert_actual_expected!(e4.diagnostic_annotated(), indoc! {r#"
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
    "#}.trim());

    e4.check_encoding()?;

    let e5 = e1.add_assertion_envelope(e4).unwrap().check_encoding()?;

    // expected-text-output-rubric:
    #[rustfmt::skip]
    assert_actual_expected!(e5.format(), indoc! {r#"
        {
            "A": "B"
        } [
            {
                "C": "D"
            } [
                "E": "F"
            ]
        ]
    "#}.trim());

    // expected-text-output-rubric:
    #[rustfmt::skip]
    assert_actual_expected!(e5.diagnostic_annotated(), indoc! {r#"
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
    "#}.trim());

    Ok(())
}
