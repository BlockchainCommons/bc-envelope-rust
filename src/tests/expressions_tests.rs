use std::rc::Rc;

use bc_components::ARID;
use indoc::indoc;
use hex_literal::hex;
use crate::{Envelope, with_format_context, expressions::{functions, parameters, Function, Parameter}};

fn two_plus_three() -> Rc<Envelope> {
    Envelope::new(functions::ADD)
        .add_parameter(parameters::LHS, 2)
        .add_parameter(parameters::RHS, 3)
}

#[test]
fn test_known() {
    let envelope = two_plus_three();
    with_format_context!(|context| {
        assert_eq!(envelope.format_opt(Some(context)), indoc! {r#"
        «add» [
            ❰lhs❱: 2
            ❰rhs❱: 3
        ]
        "#}.trim());
    });
}

#[test]
fn test_named() {
    let envelope = Envelope::new(Function::new_named("foo"))
        .add_parameter(Parameter::new_named("bar"), 2)
        .add_parameter(Parameter::new_named("baz"), 3);
    with_format_context!(|context| {
        assert_eq!(envelope.format_opt(Some(context)), indoc! {r#"
        «"foo"» [
            ❰"bar"❱: 2
            ❰"baz"❱: 3
        ]
        "#}.trim());
    });
}

#[test]
fn test_request() {
    with_format_context!(|context| {
        let request_id = ARID::from_data(hex!("c66be27dbad7cd095ca77647406d07976dc0f35f0d4d654bb0e96dd227a1e9fc"));

        let request_envelope = Envelope::new_request(&request_id, two_plus_three());
            assert_eq!(request_envelope.format_opt(Some(context)), indoc! {r#"
            request(ARID(c66be27d)) [
                body: «add» [
                    ❰lhs❱: 2
                    ❰rhs❱: 3
                ]
            ]
            "#}.trim());

        let response_envelope = Envelope::new_response(&request_id, 5);
        assert_eq!(response_envelope.format_opt(Some(context)), indoc! {r#"
            response(ARID(c66be27d)) [
                result: 5
            ]
            "#}.trim());

        let error_response = Envelope::new_error_response_with_id(request_id, "Internal Server Error");
        assert_eq!(error_response.format_opt(Some(context)), indoc! {r#"
            response(ARID(c66be27d)) [
                error: "Internal Server Error"
            ]
            "#}.trim());

        let unknown_error_response = Envelope::new_error_response(Some("Decryption failure"));
        assert_eq!(unknown_error_response.format_opt(Some(context)), indoc! {r#"
            response("unknown") [
                error: "Decryption failure"
            ]
            "#}.trim());
    });
}
