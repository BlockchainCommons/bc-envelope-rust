// use std::error::Error;
// use std::rc::Rc;
// use crate::{Envelope, with_format_context, KnownValue, known_value_registry};
// use bc_components::{DigestProvider, SymmetricKey};
// use indoc::indoc;
// use hex_literal::hex;

// fn basic_envelope() -> Rc<Envelope> {
//     Envelope::new("Hello.")
// }

// fn known_value_envelope() -> Rc<Envelope> {
//     Envelope::new(known_value_registry::NOTE)
// }

// fn assertion_envelope() -> Rc<Envelope> {
//     Envelope::new_assertion_with_predobj("knows", "Bob")
// }

// fn single_assertion_envelope() -> Rc<Envelope> {
//     Envelope::new("Alice")
//         .add_assertion_with_predobj("knows", "Bob")
// }

// fn double_assertion_envelope() -> Rc<Envelope> {
//     single_assertion_envelope()
//         .add_assertion_with_predobj("knows", "Carol")
// }

// fn wrapped_envelope() -> Rc<Envelope> {
//     Envelope::new(basic_envelope())
// }

// fn double_wrapped_envelope() -> Rc<Envelope> {
//     Envelope::new(wrapped_envelope())
// }

// fn symmetric_key() -> SymmetricKey {
//     SymmetricKey::from_data(hex!("38900719dea655e9a1bc1682aaccf0bfcd79a7239db672d39216e4acdd660dc0"))
// }

// fn encrypted_test(e1: Rc<Envelope>) {
//     let e2 = e1
//         .encrypt_subject(symmetric_key(), Some(fake_nonce));
// }
