use indoc::indoc;
use std::error::Error;
use bc_components::Digest;
use crate::{Envelope, with_format_context, IntoEnvelope};

#[test]
fn test_digest() -> Result<(), Box<dyn Error>> {
   Envelope::cbor_into_envelope(&Digest::from_image(&"Hello.".as_bytes())).check_encoding()?;

    Ok(())
}

#[test]
fn test_1() -> Result<(), Box<dyn Error>> {
    let e = "Hello.".into_envelope();

    with_format_context!(|context| {
        assert_eq!(e.diagnostic_opt(true, Some(context)),
        indoc! {r#"
        200(   ; envelope
           24("Hello.")   ; leaf
        )
        "#}.trim()
        );
    });

    Ok(())
}

#[test]
fn test_2() -> Result<(), Box<dyn Error>> {
    let array: Vec<u64> = vec![1, 2, 3];
    let e = Envelope::cbor_into_envelope(&array);

    with_format_context!(|context| {
        assert_eq!(e.diagnostic_opt(true, Some(context)),
        indoc! {r#"
        200(   ; envelope
           24(   ; leaf
              [1, 2, 3]
           )
        )
        "#}.trim()
        );
    });

    Ok(())
}

#[test]
fn test_3() -> Result<(), Box<dyn Error>> {
    let e1 = Envelope::new_assertion("A", "B");
    let e2 = Envelope::new_assertion("C", "D");
    let e3 = Envelope::new_assertion("E", "F");

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

    with_format_context!(|context| {
        assert_eq!(e4.diagnostic_opt(true, Some(context)),
        indoc! {r#"
        200(   ; envelope
           [
              200(   ; envelope
                 201(   ; assertion
                    [
                       200(   ; envelope
                          24("C")   ; leaf
                       ),
                       200(   ; envelope
                          24("D")   ; leaf
                       )
                    ]
                 )
              ),
              200(   ; envelope
                 201(   ; assertion
                    [
                       200(   ; envelope
                          24("E")   ; leaf
                       ),
                       200(   ; envelope
                          24("F")   ; leaf
                       )
                    ]
                 )
              )
           ]
        )
        "#}.trim()
        );
    });

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

    with_format_context!(|context| {
        assert_eq!(e5.diagnostic_opt(true, Some(context)),
        indoc! {r#"
        200(   ; envelope
           [
              200(   ; envelope
                 201(   ; assertion
                    [
                       200(   ; envelope
                          24("A")   ; leaf
                       ),
                       200(   ; envelope
                          24("B")   ; leaf
                       )
                    ]
                 )
              ),
              200(   ; envelope
                 [
                    200(   ; envelope
                       201(   ; assertion
                          [
                             200(   ; envelope
                                24("C")   ; leaf
                             ),
                             200(   ; envelope
                                24("D")   ; leaf
                             )
                          ]
                       )
                    ),
                    200(   ; envelope
                       201(   ; assertion
                          [
                             200(   ; envelope
                                24("E")   ; leaf
                             ),
                             200(   ; envelope
                                24("F")   ; leaf
                             )
                          ]
                       )
                    )
                 ]
              )
           ]
        )
        "#}.trim()
        );
    });

    Ok(())
}
