use std::error::Error;
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
    Envelope::new_assertion_with_predobj("knows", "Bob")
}

fn single_assertion_envelope() -> Rc<Envelope> {
    Envelope::new("Alice")
        .add_assertion_with_predobj("knows", "Bob")
}

fn double_assertion_envelope() -> Rc<Envelope> {
    single_assertion_envelope()
        .add_assertion_with_predobj("knows", "Carol")
}

fn wrapped_envelope() -> Rc<Envelope> {
    Envelope::new(basic_envelope())
}

fn double_wrapped_envelope() -> Rc<Envelope> {
    Envelope::new(wrapped_envelope())
}
