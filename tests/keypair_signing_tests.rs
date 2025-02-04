#![cfg(feature = "signature")]

use bc_components::SignatureScheme;
use bc_components::SigningOptions;
use bc_envelope::prelude::*;

mod common;
use crate::common::test_data::*;
use crate::common::check_encoding::*;

fn test_scheme(scheme: SignatureScheme, options: Option<SigningOptions>) {
    let (private_key, public_key) = scheme.keypair();
    let envelope = hello_envelope()
        .sign_opt(&private_key, options)
        .check_encoding().unwrap();
    // println!("{}", envelope.format());
    envelope.verify(&public_key).unwrap();
}

#[test]
fn test_keypair_signing() {
    bc_components::register_tags();

    test_scheme(SignatureScheme::Schnorr, None);
    test_scheme(SignatureScheme::Ecdsa, None);
    test_scheme(SignatureScheme::Ed25519, None);
    test_scheme(SignatureScheme::Dilithium2, None);
    test_scheme(SignatureScheme::Dilithium3, None);
    test_scheme(SignatureScheme::Dilithium5, None);
}

#[cfg(feature = "ssh")]
#[test]
fn test_keypair_signing_ssh() {
    let options = Some(SigningOptions::Ssh { namespace: "test".to_string(), hash_alg: ssh_key::HashAlg::Sha512 });
    test_scheme(SignatureScheme::SshEd25519, options.clone());
    test_scheme(SignatureScheme::SshDsa, options.clone());
    test_scheme(SignatureScheme::SshEcdsaP256, options.clone());
    test_scheme(SignatureScheme::SshEcdsaP384, options.clone());
}
