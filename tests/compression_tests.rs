#![cfg(feature = "compress")]
use bc_components::DigestProvider;

use dcbor::prelude::*;
use bc_envelope::prelude::*;

mod common;
use crate::common::check_encoding::*;

#[cfg(feature = "signature")]
use crate::common::test_data::*;
#[cfg(feature = "signature")]
use bc_rand::make_fake_random_number_generator;
#[cfg(feature = "signature")]
use indoc::indoc;

static SOURCE: &str = "Lorem ipsum dolor sit amet consectetur adipiscing elit mi nibh ornare proin blandit diam ridiculus, faucibus mus dui eu vehicula nam donec dictumst sed vivamus bibendum aliquet efficitur. Felis imperdiet sodales dictum morbi vivamus augue dis duis aliquet velit ullamcorper porttitor, lobortis dapibus hac purus aliquam natoque iaculis blandit montes nunc pretium.";

#[test]
fn test_compress() {
    let original = Envelope::new(SOURCE);
    assert_eq!(original.to_cbor_data().len(), 371);
    let compressed = original.compress().unwrap().check_encoding().unwrap();
    assert_eq!(compressed.to_cbor_data().len(), 283);

    assert_eq!(original.digest(), compressed.digest());
    let uncompressed = compressed.uncompress().unwrap().check_encoding().unwrap();
    assert_eq!(uncompressed.digest(), original.digest());
    assert_eq!(uncompressed.structural_digest(), original.structural_digest());
}

#[cfg(feature = "signature")]
#[test]
fn test_compress_subject() {
    use std::{cell::RefCell, rc::Rc};

    use bc_components::SigningOptions;
    use bc_envelope::known_values;

    let rng = Rc::new(RefCell::new(make_fake_random_number_generator()));
    let options = SigningOptions::Schnorr { rng };
    let original = Envelope::new("Alice")
        .add_assertion(known_values::NOTE, SOURCE)
        .wrap_envelope()
        .add_signature_opt(&alice_private_key(), Some(options), None);
    assert_eq!(original.to_cbor_data().len(), 458);
    let s = original.tree_format(false);
    // println!("{}", s);
    assert_eq!(s, indoc! {r#"
    ec608f27 NODE
        d7183f04 subj WRAPPED
            7f35e345 subj NODE
                13941b48 subj "Alice"
                9fb69539 ASSERTION
                    0fcd6a39 pred 'note'
                    e343c9b4 obj "Lorem ipsum dolor sit amet consectetur a…"
        0db2ee20 ASSERTION
            d0e39e78 pred 'signed'
            f0d3ce4c obj Signature
    "#}.trim());
    let compressed = original.compress_subject().unwrap().check_encoding().unwrap();
    assert_eq!(compressed.clone().to_cbor_data().len(), 374);
    let s = compressed.tree_format(false);
    // println!("{}", s);
    assert_eq!(s, indoc! {r#"
    ec608f27 NODE
        d7183f04 subj COMPRESSED
        0db2ee20 ASSERTION
            d0e39e78 pred 'signed'
            f0d3ce4c obj Signature
    "#}.trim());
    let uncompressed = compressed.uncompress_subject().unwrap().check_encoding().unwrap();
    assert_eq!(uncompressed.digest(), original.digest());
    assert_eq!(uncompressed.structural_digest(), original.structural_digest());
}
