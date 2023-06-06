use std::error::Error;
use std::rc::Rc;
use crate::{Envelope, known_value_registry, Enclosable};
use bc_components::{DigestProvider, SymmetricKey, Nonce, EncryptedMessage};
use hex_literal::hex;

fn basic_envelope() -> Rc<Envelope> {
    "Hello.".enclose()
}

fn known_value_envelope() -> Rc<Envelope> {
    known_value_registry::NOTE.enclose()
}

fn assertion_envelope() -> Rc<Envelope> {
    Envelope::new_assertion("knows".enclose(), "Bob".enclose())
}

fn single_assertion_envelope() -> Rc<Envelope> {
    "Alice".enclose()
        .add_assertion("knows".enclose(), "Bob".enclose())
}

fn double_assertion_envelope() -> Rc<Envelope> {
    single_assertion_envelope()
        .add_assertion("knows".enclose(), "Carol".enclose())
}

fn wrapped_envelope() -> Rc<Envelope> {
    basic_envelope().enclose()
}

fn double_wrapped_envelope() -> Rc<Envelope> {
    wrapped_envelope().enclose()
}

fn symmetric_key() -> SymmetricKey {
    SymmetricKey::from_data(hex!("38900719dea655e9a1bc1682aaccf0bfcd79a7239db672d39216e4acdd660dc0"))
}

fn fake_nonce() -> Nonce {
    Nonce::from_data(hex!("4d785658f36c22fb5aed3ac0"))
}

fn encrypted_test(e1: Rc<Envelope>) -> Result<(), Box<dyn Error>> {
    // println!("{}", e1.hex_opt(true, None));
    let e2 = e1
        .clone()
        .encrypt_subject_opt(&symmetric_key(), Some(fake_nonce()))?
        .check_encoding()?;

    // println!("{}", e2.hex_opt(true, None));

    assert!(e1.clone().is_equivalent_to(e2.clone()));
    assert!(e1.clone().subject().is_equivalent_to(e2.clone().subject()));

    let encrypted_message = e2.extract_subject::<EncryptedMessage>()?;
    assert_eq!(encrypted_message.digest(), e1.clone().subject().digest());

    let e3 = e2
        .decrypt_subject(&symmetric_key())?;

    assert!(e1.is_equivalent_to(e3));

    Ok(())
}

#[test]
fn test_encrypted() {
    encrypted_test(basic_envelope()).unwrap();
    encrypted_test(wrapped_envelope()).unwrap();
    encrypted_test(double_wrapped_envelope()).unwrap();
    encrypted_test(known_value_envelope()).unwrap();
    encrypted_test(assertion_envelope()).unwrap();
    encrypted_test(single_assertion_envelope()).unwrap();
    encrypted_test(double_assertion_envelope()).unwrap();
}

// #[test]
// fn test_sign_wrap_encrypt() {
//     let e1 = basic_envelope();
//     let e2 =
//         e1.sign(alice_private_keys())
// }
