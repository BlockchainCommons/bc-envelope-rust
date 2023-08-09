use bc_components::DigestProvider;
use bc_crypto::make_fake_random_number_generator;
use dcbor::CBOREncodable;
use indoc::indoc;
use crate::{known_values::NOTE, with_format_context, Envelope};

use super::test_data::alice_private_keys;


fn source() -> &'static str {
    "Lorem ipsum dolor sit amet consectetur adipiscing elit mi nibh ornare proin blandit diam ridiculus, faucibus mus dui eu vehicula nam donec dictumst sed vivamus bibendum aliquet efficitur. Felis imperdiet sodales dictum morbi vivamus augue dis duis aliquet velit ullamcorper porttitor, lobortis dapibus hac purus aliquam natoque iaculis blandit montes nunc pretium."
}

#[test]
fn test_compress() {
    let original = Envelope::new(source());
    assert_eq!(original.cbor_data().len(), 371);
    let compressed = original.clone().compress().unwrap().check_encoding().unwrap();
    assert_eq!(compressed.cbor_data().len(), 284);

    assert_eq!(original.digest(), compressed.digest());
    let uncompressed = compressed.uncompress().unwrap().check_encoding().unwrap();
    assert_eq!(uncompressed.digest(), original.digest());
    assert_eq!(uncompressed.structural_digest(), original.structural_digest());
}

#[test]
fn test_compress_subject() {
    let mut rng = make_fake_random_number_generator();
    let original = Envelope::new("Alice")
        .add_assertion(NOTE, source())
        .wrap_envelope()
        .sign_with_using(&alice_private_keys(), &mut rng);
    assert_eq!(original.cbor_data().len(), 474);
    with_format_context!(|context| {
        let s = original.clone().tree_format(false, Some(context));
        assert_eq!(s, indoc! {r#"
        9ed291b0 NODE
            d7183f04 subj WRAPPED
                7f35e345 subj NODE
                    13941b48 subj "Alice"
                    9fb69539 ASSERTION
                        0fcd6a39 pred note
                        e343c9b4 obj "Lorem ipsum dolor sit amet consectetur a…"
            2f87ba42 ASSERTION
                d0e39e78 pred verifiedBy
                dd386db5 obj Signature
        "#}.trim());
    });
    let compresssed = original.clone().compress_subject().unwrap().check_encoding().unwrap();
    assert_eq!(compresssed.cbor_data().len(), 388);
    with_format_context!(|context| {
        let s = compresssed.clone().tree_format(false, Some(context));
        assert_eq!(s, indoc! {r#"
        9ed291b0 NODE
            d7183f04 subj COMPRESSED
            2f87ba42 ASSERTION
                d0e39e78 pred verifiedBy
                dd386db5 obj Signature
        "#}.trim());
    });
    let uncompressed = compresssed.uncompress_subject().unwrap().check_encoding().unwrap();
    assert_eq!(uncompressed.digest(), original.digest());
    assert_eq!(uncompressed.structural_digest(), original.structural_digest());
}
