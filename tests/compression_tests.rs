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
    assert_eq!(original.cbor_data().len(), 369);
    let compressed = original.clone().compress().unwrap().check_encoding().unwrap();
    assert_eq!(compressed.cbor_data().len(), 282);

    assert_eq!(original.digest(), compressed.digest());
    let uncompressed = compressed.uncompress().unwrap().check_encoding().unwrap();
    assert_eq!(uncompressed.digest(), original.digest());
    assert_eq!(uncompressed.structural_digest(), original.structural_digest());
}

#[cfg(feature = "signature")]
#[test]
fn test_compress_subject() {
    let mut rng = make_fake_random_number_generator();
    let original = Envelope::new("Alice")
        .add_assertion(known_values::NOTE, SOURCE)
        .wrap_envelope()
        .sign_with_using(&alice_private_keys(), &mut rng);
    assert_eq!(original.cbor_data().len(), 456);
    with_format_context!(|context| {
        let s = original.clone().tree_format_opt(false, Some(context));
        assert_eq!(s, indoc! {r#"
        9ed291b0 NODE
            d7183f04 subj WRAPPED
                7f35e345 subj NODE
                    13941b48 subj "Alice"
                    9fb69539 ASSERTION
                        0fcd6a39 pred 'note'
                        e343c9b4 obj "Lorem ipsum dolor sit amet consectetur a…"
            2f87ba42 ASSERTION
                d0e39e78 pred 'verifiedBy'
                dd386db5 obj Signature
        "#}.trim());
    });
    let compressed = original.clone().compress_subject().unwrap().check_encoding().unwrap();
    assert_eq!(compressed.cbor_data().len(), 373);
    with_format_context!(|context| {
        let s = compressed.clone().tree_format_opt(false, Some(context));
        assert_eq!(s, indoc! {r#"
        9ed291b0 NODE
            d7183f04 subj COMPRESSED
            2f87ba42 ASSERTION
                d0e39e78 pred 'verifiedBy'
                dd386db5 obj Signature
        "#}.trim());
    });
    let uncompressed = compressed.uncompress_subject().unwrap().check_encoding().unwrap();
    assert_eq!(uncompressed.digest(), original.digest());
    assert_eq!(uncompressed.structural_digest(), original.structural_digest());
}
