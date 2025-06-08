#[cfg(feature = "encrypt")]
use bc_components::SymmetricKey;
#[cfg(feature = "known_value")]
use bc_components::{ARID, Digest};
use bc_envelope::prelude::*;
#[cfg(feature = "known_value")]
use hex_literal::hex;
use indoc::indoc;

mod common;
#[cfg(feature = "signature")]
use std::collections::HashSet;
#[cfg(feature = "signature")]
use std::{cell::RefCell, rc::Rc};

#[cfg(feature = "signature")]
use bc_components::DigestProvider;
#[cfg(feature = "signature")]
use bc_components::SigningOptions;
#[cfg(feature = "signature")]
use bc_rand::make_fake_random_number_generator;

use crate::common::{check_encoding::*, test_data::*};

#[test]
fn test_plaintext() {
    let envelope = Envelope::new(PLAINTEXT_HELLO);

    #[rustfmt::skip]
    assert_eq!(envelope.format(), indoc! {r#"
        "Hello."
    "#}.trim());

    assert_eq!(envelope.format_flat(), r#""Hello.""#);

    #[rustfmt::skip]
    assert_eq!(envelope.tree_format(), indoc! {r#"
        8cc96cdb "Hello."
    "#}.trim());

    #[rustfmt::skip]
    assert_eq!(envelope.tree_format_opt(&TreeFormatOpts::default().context(FormatContextOpt::None)), indoc! {r#"
        8cc96cdb "Hello."
    "#}.trim());

    #[rustfmt::skip]
    assert_eq!(envelope.tree_format_opt(&TreeFormatOpts::default().digest_display(DigestDisplayFormat::Full)), indoc! {r#"
        8cc96cdb771176e835114a0f8936690b41cfed0df22d014eedd64edaea945d59 "Hello."
    "#}.trim());

    #[rustfmt::skip]
    assert_eq!(envelope.tree_format_opt(&TreeFormatOpts::default().digest_display(DigestDisplayFormat::UR)), indoc! {r#"
        ur:digest/hdcxlksojzuyktbykovsecbygebsldeninbdfptkwebtwzdpadglwetbgltnwdmwhlhksbbthtpy "Hello."
    "#}.trim());

    #[rustfmt::skip]
    assert_eq!(envelope.tree_format_opt(&TreeFormatOpts::default().hide_nodes(true)), indoc! {r#"
        "Hello."
    "#}.trim());

    assert_eq!(
        envelope.elements_count(),
        envelope.tree_format().split('\n').count()
    );
}

#[cfg(feature = "signature")]
#[test]
fn test_signed_plaintext() {
    let rng = Rc::new(RefCell::new(make_fake_random_number_generator()));
    let options = SigningOptions::Schnorr { rng };
    let envelope = Envelope::new(PLAINTEXT_HELLO).add_signature_opt(
        &alice_private_key(),
        Some(options),
        None,
    );

    #[rustfmt::skip]
    assert_eq!(envelope.format(), indoc! {r#"
        "Hello." [
            'signed': Signature
        ]
    "#}.trim());

    assert_eq!(
        envelope.format_flat(),
        r#""Hello." [ 'signed': Signature ]"#
    );

    let s = envelope.tree_format();
    #[rustfmt::skip]
    assert_eq!(s, indoc! {r#"
        949a991e NODE
            8cc96cdb subj "Hello."
            fcb4e2be ASSERTION
                d0e39e78 pred 'signed'
                b8bb043f obj Signature
    "#}.trim());

    let s = envelope.tree_format_opt(
        &TreeFormatOpts::default().context(FormatContextOpt::None),
    );
    #[rustfmt::skip]
    assert_eq!(s, indoc! {r#"
        949a991e NODE
            8cc96cdb subj "Hello."
            fcb4e2be ASSERTION
                d0e39e78 pred '3'
                b8bb043f obj 40020(h'd0f6b2577edb3f4b0f533e21577bc12a58aaca2604bc71e84bd4e2c81421900bca361a1a8de3b7dbfe1cb5c16e34cb8c9a78fe6f7a387e959bbb15f6f3d898d3')
    "#}.trim());

    #[rustfmt::skip]
    assert_eq!(envelope.tree_format_opt(&TreeFormatOpts::default().hide_nodes(true)), indoc! {r#"
        "Hello."
            ASSERTION
                'signed'
                Signature
    "#}.trim());

    let s = envelope.tree_format_opt(
        &TreeFormatOpts::default()
            .hide_nodes(true)
            .context(FormatContextOpt::None),
    );
    #[rustfmt::skip]
    assert_eq!(s, indoc! {r#"
        "Hello."
            ASSERTION
                '3'
                40020(h'd0f6b2577edb3f4b0f533e21577bc12a58aaca2604bc71e84bd4e2c81421900bca361a1a8de3b7dbfe1cb5c16e34cb8c9a78fe6f7a387e959bbb15f6f3d898d3')
    "#}.trim());

    assert_eq!(
        envelope.elements_count(),
        envelope.tree_format().split('\n').count()
    );
}

#[cfg(feature = "encrypt")]
#[test]
fn test_encrypt_subject() {
    let envelope = Envelope::new("Alice")
        .add_assertion("knows", "Bob")
        .encrypt_subject(&SymmetricKey::new())
        .unwrap();
    #[rustfmt::skip]
    assert_eq!(envelope.format(), indoc! {r#"
        ENCRYPTED [
            "knows": "Bob"
        ]
    "#}.trim());

    assert_eq!(envelope.format_flat(), r#"ENCRYPTED [ "knows": "Bob" ]"#);

    #[rustfmt::skip]
    assert_eq!(envelope.tree_format(), indoc! {r#"
        8955db5e NODE
            13941b48 subj ENCRYPTED
            78d666eb ASSERTION
                db7dd21c pred "knows"
                13b74194 obj "Bob"
    "#}.trim());

    #[rustfmt::skip]
    assert_eq!(envelope.tree_format_opt(&TreeFormatOpts::default().hide_nodes(true)), indoc! {r#"
        ENCRYPTED
            ASSERTION
                "knows"
                "Bob"
    "#}.trim());

    assert_eq!(
        envelope.elements_count(),
        envelope.tree_format().split('\n').count()
    );
}

#[test]
fn test_top_level_assertion() {
    let envelope = Envelope::new_assertion("knows", "Bob");
    #[rustfmt::skip]
    assert_eq!(envelope.format(), indoc! {r#"
        "knows": "Bob"
    "#}.trim());

    assert_eq!(envelope.format_flat(), r#""knows": "Bob""#);

    #[rustfmt::skip]
    assert_eq!(envelope.tree_format(), indoc! {r#"
        78d666eb ASSERTION
            db7dd21c pred "knows"
            13b74194 obj "Bob"
    "#}.trim());

    #[rustfmt::skip]
    assert_eq!(envelope.tree_format_opt(&TreeFormatOpts::default().hide_nodes(true)), indoc! {r#"
        ASSERTION
            "knows"
            "Bob"
    "#}.trim());

    assert_eq!(
        envelope.elements_count(),
        envelope.tree_format().split('\n').count()
    );
}

#[test]
fn test_elided_object() {
    let envelope = Envelope::new("Alice").add_assertion("knows", "Bob");
    let elided = envelope.elide_removing_target(&"Bob".to_envelope());
    #[rustfmt::skip]
    assert_eq!(elided.format(), indoc! {r#"
        "Alice" [
            "knows": ELIDED
        ]
    "#}.trim());

    assert_eq!(elided.format_flat(), r#""Alice" [ "knows": ELIDED ]"#);

    #[rustfmt::skip]
    assert_eq!(elided.tree_format(), indoc! {r#"
        8955db5e NODE
            13941b48 subj "Alice"
            78d666eb ASSERTION
                db7dd21c pred "knows"
                13b74194 obj ELIDED
    "#}.trim());

    #[rustfmt::skip]
    assert_eq!(elided.tree_format_opt(&TreeFormatOpts::default().hide_nodes(true)), indoc! {r#"
        "Alice"
            ASSERTION
                "knows"
                ELIDED
    "#}.trim());

    assert_eq!(
        elided.elements_count(),
        elided.tree_format().split('\n').count()
    );
}

#[cfg(feature = "signature")]
#[test]
fn test_signed_subject() {
    let rng = Rc::new(RefCell::new(make_fake_random_number_generator()));
    let options = SigningOptions::Schnorr { rng };
    let envelope = Envelope::new("Alice")
        .add_assertion("knows", "Bob")
        .add_assertion("knows", "Carol")
        .add_signature_opt(&alice_private_key(), Some(options), None);
    #[rustfmt::skip]
    assert_eq!(envelope.format(), indoc! {r#"
        "Alice" [
            "knows": "Bob"
            "knows": "Carol"
            'signed': Signature
        ]
    "#}.trim());

    assert_eq!(
        envelope.format_flat(),
        r#""Alice" [ "knows": "Bob", "knows": "Carol", 'signed': Signature ]"#
    );

    let s = envelope.tree_format();
    // println!("{}", s);
    #[rustfmt::skip]
    assert_eq!(s, indoc! {r#"
        d595106e NODE
            13941b48 subj "Alice"
            399c974c ASSERTION
                d0e39e78 pred 'signed'
                ff10427c obj Signature
            4012caf2 ASSERTION
                db7dd21c pred "knows"
                afb8122e obj "Carol"
            78d666eb ASSERTION
                db7dd21c pred "knows"
                13b74194 obj "Bob"
    "#}.trim());

    let s =
        envelope.tree_format_opt(&TreeFormatOpts::default().hide_nodes(true));
    // println!("{}", s);
    #[rustfmt::skip]
    assert_eq!(s, indoc! {r#"
        "Alice"
            ASSERTION
                'signed'
                Signature
            ASSERTION
                "knows"
                "Carol"
            ASSERTION
                "knows"
                "Bob"
    "#}.trim());

    assert_eq!(
        envelope.elements_count(),
        envelope.tree_format().split('\n').count()
    );

    // Elided assertions
    let mut target: HashSet<Digest> = HashSet::new();
    target.insert(envelope.digest().into_owned());
    target.insert(envelope.subject().digest().into_owned());
    let elided = envelope.elide_revealing_set(&target);
    #[rustfmt::skip]
    assert_eq!(elided.format(), indoc! {r#"
        "Alice" [
            ELIDED (3)
        ]
    "#}.trim());

    assert_eq!(elided.format_flat(), r#""Alice" [ ELIDED (3) ]"#);

    let s = elided.tree_format();
    // println!("{}", s);
    #[rustfmt::skip]
    assert_eq!(s, indoc! {r#"
        d595106e NODE
            13941b48 subj "Alice"
            399c974c ELIDED
            4012caf2 ELIDED
            78d666eb ELIDED
    "#}.trim());

    #[rustfmt::skip]
    assert_eq!(elided.tree_format_opt(&TreeFormatOpts::default().hide_nodes(true)), indoc! {r#"
        "Alice"
            ELIDED
            ELIDED
            ELIDED
    "#}.trim());

    assert_eq!(
        elided.elements_count(),
        elided.tree_format().split('\n').count()
    );
}

#[cfg(feature = "signature")]
#[test]
fn test_wrap_then_signed() {
    let rng = Rc::new(RefCell::new(make_fake_random_number_generator()));
    let options = SigningOptions::Schnorr { rng };
    let envelope = Envelope::new("Alice")
        .add_assertion("knows", "Bob")
        .add_assertion("knows", "Carol")
        .wrap_envelope()
        .add_signature_opt(&alice_private_key(), Some(options), None);
    #[rustfmt::skip]
    assert_eq!(envelope.format(), indoc! {r#"
        {
            "Alice" [
                "knows": "Bob"
                "knows": "Carol"
            ]
        } [
            'signed': Signature
        ]
    "#}.trim());

    assert_eq!(
        envelope.format_flat(),
        r#"{ "Alice" [ "knows": "Bob", "knows": "Carol" ] } [ 'signed': Signature ]"#
    );

    let s = envelope.tree_format();
    #[rustfmt::skip]
    assert_eq!(s, indoc! {r#"
        66c9d594 NODE
            9e3b0673 subj WRAPPED
                b8d857f6 subj NODE
                    13941b48 subj "Alice"
                    4012caf2 ASSERTION
                        db7dd21c pred "knows"
                        afb8122e obj "Carol"
                    78d666eb ASSERTION
                        db7dd21c pred "knows"
                        13b74194 obj "Bob"
            f13623da ASSERTION
                d0e39e78 pred 'signed'
                e30a727c obj Signature
    "#}.trim());

    let s =
        envelope.tree_format_opt(&TreeFormatOpts::default().hide_nodes(true));
    #[rustfmt::skip]
    assert_eq!(s, indoc! {r#"
        WRAPPED
            "Alice"
                ASSERTION
                    "knows"
                    "Carol"
                ASSERTION
                    "knows"
                    "Bob"
            ASSERTION
                'signed'
                Signature
    "#}.trim());

    let s = envelope.tree_format_opt(
        &TreeFormatOpts::default().digest_display(DigestDisplayFormat::Full),
    );
    #[rustfmt::skip]
    assert_eq!(s, indoc! {r#"
        66c9d5944eaedc418d8acf52df7842f50c2aa2d0da2857ad1048412cd070c3e8 NODE
            9e3b06737407b10cac0b9353dd978c4a68537709554dabdd66a8b68b8bd36cf6 subj WRAPPED
                b8d857f6e06a836fbc68ca0ce43e55ceb98eefd949119dab344e11c4ba5a0471 subj NODE
                    13941b487c1ddebce827b6ec3f46d982938acdc7e3b6a140db36062d9519dd2f subj "Alice"
                    4012caf2d96bf3962514bcfdcf8dd70c351735dec72c856ec5cdcf2ee35d6a91 ASSERTION
                        db7dd21c5169b4848d2a1bcb0a651c9617cdd90bae29156baaefbb2a8abef5ba pred "knows"
                        afb8122e3227657b415f9f1c930d4891fb040b3e23c1f7770f185e2d0396c737 obj "Carol"
                    78d666eb8f4c0977a0425ab6aa21ea16934a6bc97c6f0c3abaefac951c1714a2 ASSERTION
                        db7dd21c5169b4848d2a1bcb0a651c9617cdd90bae29156baaefbb2a8abef5ba pred "knows"
                        13b741949c37b8e09cc3daa3194c58e4fd6b2f14d4b1d0f035a46d6d5a1d3f11 obj "Bob"
            f13623dac926c57e2ac128868dfaa22fb8e434a7e4a552029992d6f6283da533 ASSERTION
                d0e39e788c0d8f0343af4588db21d3d51381db454bdf710a9a1891aaa537693c pred 'signed'
                e30a727cc1f43fbe3c9fd228447c34faaf6b54101bf7bcd766e280f8449ceade obj Signature
    "#}.trim());

    let s = envelope.tree_format_opt(
        &TreeFormatOpts::default().digest_display(DigestDisplayFormat::UR),
    );
    #[rustfmt::skip]
    assert_eq!(s, indoc! {r#"
        ur:digest/hdcxiysotlmwglpluofplgletkgmurksfwykbndroetitndehgpmbefdfpdwtijosrvsbsdlsndm NODE
            ur:digest/hdcxnnframjkjyatpabnpsbdmuguutmslkgeisguktasgogtpyutiypdrplulutejzynmygrnlly subj WRAPPED
                ur:digest/hdcxrotphgynvtimlsjlrfissgbnvefmgotorhmnwstagabyntpyeeglbyssrdhtaajsaetafrbw subj NODE
                    ur:digest/hdcxbwmwcwfdkecauerfvsdirpwpfhfgtalfmulesnstvlrpoyfzuyenamdpmdcfutdlstyaqzrk subj "Alice"
                    ur:digest/hdcxfzbgsgwztajewfmtdabbrfzctklgtsbnecchecuestdwlpjtsksntkdmvlhlimmetlcpiyms ASSERTION
                        ur:digest/hdcxuykitdcegyinqzlrlgdrcwsbbkihcemtchsntabdpldtbzjepkwsrkdrlernykrddpjtgdfh pred "knows"
                        ur:digest/hdcxperobgdmeydiihkgfphenecemubtfdmezoaabdfmcnseylktbscshydpaxmtstemtarhmngd obj "Carol"
                    ur:digest/hdcxkstbiywmmygsasktnbfwhtrppkclwdcmmugejesokejlbnftrdwspsmdcechbboerhzebtws ASSERTION
                        ur:digest/hdcxuykitdcegyinqzlrlgdrcwsbbkihcemtchsntabdpldtbzjepkwsrkdrlernykrddpjtgdfh pred "knows"
                        ur:digest/hdcxbwrlfpmwnsemrovtnssrtnotcfgshdvezcjedlbbtypatiwtecoxjnjnhtcafhbysptsnsnl obj "Bob"
            ur:digest/hdcxwnencntnsodsskkbdrsedelnlgzsoedlroveeeosveongmaonlmotbyndefsoneorfutayas ASSERTION
                ur:digest/hdcxtivlnnkslkbtmyaxfxpefelouycltetlbwlyuyfegrurjsbknycsmepkoneminfnrpjpssla pred 'signed'
                ur:digest/hdcxvlbkjpkesewkfhrnfnnetddefykeeezspejeghbecwylrftsiyvolayafynswduefytsgaos obj Signature
    "#}.trim());

    assert_eq!(
        envelope.elements_count(),
        envelope.tree_format().split('\n').count()
    );
}

#[cfg(feature = "recipient")]
#[test]
fn test_encrypt_to_recipients() {
    let envelope = Envelope::new(PLAINTEXT_HELLO)
        .encrypt_subject_opt(&fake_content_key(), Some(fake_nonce()))
        .unwrap()
        .check_encoding()
        .unwrap()
        .add_recipient_opt(
            &bob_public_key(),
            &fake_content_key(),
            Some(&fake_nonce()),
        )
        .check_encoding()
        .unwrap()
        .add_recipient_opt(
            &carol_public_key(),
            &fake_content_key(),
            Some(&fake_nonce()),
        )
        .check_encoding()
        .unwrap();
    #[rustfmt::skip]
    assert_eq!(envelope.format(), indoc! {r#"
        ENCRYPTED [
            'hasRecipient': SealedMessage
            'hasRecipient': SealedMessage
        ]
    "#}.trim());

    assert_eq!(
        envelope.format_flat(),
        r#"ENCRYPTED [ 'hasRecipient': SealedMessage, 'hasRecipient': SealedMessage ]"#
    );

    #[rustfmt::skip]
    assert_eq!(envelope.tree_format_opt(&TreeFormatOpts::default().hide_nodes(true)), indoc! {r#"
        ENCRYPTED
            ASSERTION
                'hasRecipient'
                SealedMessage
            ASSERTION
                'hasRecipient'
                SealedMessage
    "#}.trim());

    assert_eq!(
        envelope.elements_count(),
        envelope.tree_format().split('\n').count()
    );
}

#[test]
fn test_assertion_positions() {
    let predicate = Envelope::new("predicate")
        .add_assertion("predicate-predicate", "predicate-object");
    let object = Envelope::new("object")
        .add_assertion("object-predicate", "object-object");
    let envelope = Envelope::new("subject")
        .add_assertion(predicate, object)
        .check_encoding()
        .unwrap();
    #[rustfmt::skip]
    assert_eq!(envelope.format(), indoc! {r#"
        "subject" [
            "predicate" [
                "predicate-predicate": "predicate-object"
            ]
            : "object" [
                "object-predicate": "object-object"
            ]
        ]
    "#}.trim());

    assert_eq!(
        envelope.format_flat(),
        r#""subject" [ "predicate" [ "predicate-predicate": "predicate-object" ] : "object" [ "object-predicate": "object-object" ] ]"#
    );

    #[rustfmt::skip]
    assert_eq!(envelope.tree_format(), indoc! {r#"
        e06d7003 NODE
            8e4e62eb subj "subject"
            91a436e0 ASSERTION
                cece8b2c pred NODE
                    d21efb76 subj "predicate"
                    66a0c92b ASSERTION
                        ab829e9f pred "predicate-predicate"
                        f1098628 obj "predicate-object"
                03a99a27 obj NODE
                    fda63155 subj "object"
                    d1878aea ASSERTION
                        88bb262f pred "object-predicate"
                        0bdb89a6 obj "object-object"
    "#}.trim());

    #[rustfmt::skip]
    assert_eq!(envelope.tree_format_opt(&TreeFormatOpts::default().hide_nodes(true)), indoc! {r#"
        "subject"
            ASSERTION
                "predicate"
                    ASSERTION
                        "predicate-predicate"
                        "predicate-object"
                "object"
                    ASSERTION
                        "object-predicate"
                        "object-object"
    "#}.trim());

    assert_eq!(
        envelope.elements_count(),
        envelope.tree_format().split('\n').count()
    );
}

#[cfg(feature = "known_value")]
#[test]
fn test_complex_metadata() {
    // Assertions made about an ARID are considered part of a distributed set.
    // Which assertions are returned depends on who resolves the ARID and
    // when it is resolved. In other words, the referent of an ARID is
    // mutable.
    let author = Envelope::new(ARID::from_data(hex!(
        "9c747ace78a4c826392510dd6285551e7df4e5164729a1b36198e56e017666c8"
    )))
    .add_assertion(known_values::DEREFERENCE_VIA, "LibraryOfCongress")
    .add_assertion(known_values::NAME, "Ayn Rand")
    .check_encoding()
    .unwrap();

    // Assertions made on a literal value are considered part of the same set of
    // assertions made on the digest of that value.
    let name_en = Envelope::new("Atlas Shrugged")
        .add_assertion(known_values::LANGUAGE, "en");

    let name_es = Envelope::new("La rebelión de Atlas")
        .add_assertion(known_values::LANGUAGE, "es");

    let work = Envelope::new(ARID::from_data(hex!(
        "7fb90a9d96c07f39f75ea6acf392d79f241fac4ec0be2120f7c82489711e3e80"
    )))
    .add_assertion(known_values::IS_A, "novel")
    .add_assertion("isbn", "9780451191144")
    .add_assertion("author", author)
    .add_assertion(known_values::DEREFERENCE_VIA, "LibraryOfCongress")
    .add_assertion(known_values::NAME, name_en)
    .add_assertion(known_values::NAME, name_es)
    .check_encoding()
    .unwrap();

    let book_data = "This is the entire book “Atlas Shrugged” in EPUB format.";
    // Assertions made on a digest are considered associated with that specific
    // binary object and no other. In other words, the referent of a Digest
    // is immutable.
    let book_metadata = Envelope::new(Digest::from_image(book_data))
        .add_assertion("work", work)
        .add_assertion("format", "EPUB")
        .add_assertion(known_values::DEREFERENCE_VIA, "IPFS")
        .check_encoding()
        .unwrap();

    #[rustfmt::skip]
    assert_eq!(book_metadata.format(), indoc! {r#"
        Digest(26d05af5) [
            "format": "EPUB"
            "work": ARID(7fb90a9d) [
                'isA': "novel"
                "author": ARID(9c747ace) [
                    'dereferenceVia': "LibraryOfCongress"
                    'name': "Ayn Rand"
                ]
                "isbn": "9780451191144"
                'dereferenceVia': "LibraryOfCongress"
                'name': "Atlas Shrugged" [
                    'language': "en"
                ]
                'name': "La rebelión de Atlas" [
                    'language': "es"
                ]
            ]
            'dereferenceVia': "IPFS"
        ]
    "#}.trim());
    assert_eq!(
        book_metadata.format_flat(),
        r#"Digest(26d05af5) [ "format": "EPUB", "work": ARID(7fb90a9d) [ 'isA': "novel", "author": ARID(9c747ace) [ 'dereferenceVia': "LibraryOfCongress", 'name': "Ayn Rand" ], "isbn": "9780451191144", 'dereferenceVia': "LibraryOfCongress", 'name': "Atlas Shrugged" [ 'language': "en" ], 'name': "La rebelión de Atlas" [ 'language': "es" ] ], 'dereferenceVia': "IPFS" ]"#
    );

    #[rustfmt::skip]
    assert_eq!(book_metadata.tree_format(), indoc! {r#"
        c93370e7 NODE
            0c1e45b9 subj Digest(26d05af5)
            83b00bef ASSERTION
                cdb6a696 pred 'dereferenceVia'
                15eac58f obj "IPFS"
            953cdab2 ASSERTION
                a9a86b03 pred "format"
                9536cfe0 obj "EPUB"
            eec25a61 ASSERTION
                2ddb0b05 pred "work"
                26681136 obj NODE
                    0c69be6e subj ARID(7fb90a9d)
                    1786d8b5 ASSERTION
                        4019420b pred "isbn"
                        69ff76b1 obj "9780451191144"
                    5355d973 ASSERTION
                        2be2d79b pred 'isA'
                        6d7c7189 obj "novel"
                    63cd143a ASSERTION
                        14ff9eac pred 'name'
                        29fa40b1 obj NODE
                            5e825721 subj "La rebelión de Atlas"
                            c8db157b ASSERTION
                                60dfb783 pred 'language'
                                b33e79c2 obj "es"
                    7d6d5c1d ASSERTION
                        29c09059 pred "author"
                        1ba13788 obj NODE
                            3c47e105 subj ARID(9c747ace)
                            9c10d60f ASSERTION
                                cdb6a696 pred 'dereferenceVia'
                                34a04547 obj "LibraryOfCongress"
                            bff8435a ASSERTION
                                14ff9eac pred 'name'
                                98985bd5 obj "Ayn Rand"
                    9c10d60f ASSERTION
                        cdb6a696 pred 'dereferenceVia'
                        34a04547 obj "LibraryOfCongress"
                    b722c07c ASSERTION
                        14ff9eac pred 'name'
                        0cfacc06 obj NODE
                            e84c3091 subj "Atlas Shrugged"
                            b80d3b05 ASSERTION
                                60dfb783 pred 'language'
                                6700869c obj "en"
    "#}.trim());

    #[rustfmt::skip]
    assert_eq!(book_metadata.tree_format_opt(&TreeFormatOpts::default().hide_nodes(true)), indoc! {r#"
        Digest(26d05af5)
            ASSERTION
                'dereferenceVia'
                "IPFS"
            ASSERTION
                "format"
                "EPUB"
            ASSERTION
                "work"
                ARID(7fb90a9d)
                    ASSERTION
                        "isbn"
                        "9780451191144"
                    ASSERTION
                        'isA'
                        "novel"
                    ASSERTION
                        'name'
                        "La rebelión de Atlas"
                            ASSERTION
                                'language'
                                "es"
                    ASSERTION
                        "author"
                        ARID(9c747ace)
                            ASSERTION
                                'dereferenceVia'
                                "LibraryOfCongress"
                            ASSERTION
                                'name'
                                "Ayn Rand"
                    ASSERTION
                        'dereferenceVia'
                        "LibraryOfCongress"
                    ASSERTION
                        'name'
                        "Atlas Shrugged"
                            ASSERTION
                                'language'
                                "en"
    "#}.trim());

    assert_eq!(
        book_metadata.elements_count(),
        book_metadata.tree_format().split('\n').count()
    );
}

#[cfg(feature = "signature")]
fn credential() -> Envelope {
    let rng = Rc::new(RefCell::new(make_fake_random_number_generator()));
    let options = SigningOptions::Schnorr { rng };
    Envelope::new(ARID::from_data(hex!(
        "4676635a6e6068c2ef3ffd8ff726dd401fd341036e920f136a1d8af5e829496d"
    )))
    .add_assertion(known_values::IS_A, "Certificate of Completion")
    .add_assertion(known_values::ISSUER, "Example Electrical Engineering Board")
    .add_assertion(
        known_values::CONTROLLER,
        "Example Electrical Engineering Board",
    )
    .add_assertion("firstName", "James")
    .add_assertion("lastName", "Maxwell")
    .add_assertion("issueDate", dcbor::Date::from_string("2020-01-01").unwrap())
    .add_assertion(
        "expirationDate",
        dcbor::Date::from_string("2028-01-01").unwrap(),
    )
    .add_assertion("photo", "This is James Maxwell's photo.")
    .add_assertion("certificateNumber", "123-456-789")
    .add_assertion("subject", "RF and Microwave Engineering")
    .add_assertion("continuingEducationUnits", 1)
    .add_assertion("professionalDevelopmentHours", 15)
    .add_assertion("topics", vec!["Subject 1", "Subject 2"].to_cbor())
    .wrap_envelope()
    .add_signature_opt(&alice_private_key(), Some(options), None)
    .add_assertion(
        known_values::NOTE,
        "Signed by Example Electrical Engineering Board",
    )
    .check_encoding()
    .unwrap()
}

#[cfg(feature = "signature")]
#[test]
fn test_credential() {
    let credential = credential();
    #[rustfmt::skip]
    assert_eq!(credential.format(), indoc! {r#"
        {
            ARID(4676635a) [
                'isA': "Certificate of Completion"
                "certificateNumber": "123-456-789"
                "continuingEducationUnits": 1
                "expirationDate": 2028-01-01
                "firstName": "James"
                "issueDate": 2020-01-01
                "lastName": "Maxwell"
                "photo": "This is James Maxwell's photo."
                "professionalDevelopmentHours": 15
                "subject": "RF and Microwave Engineering"
                "topics": ["Subject 1", "Subject 2"]
                'controller': "Example Electrical Engineering Board"
                'issuer': "Example Electrical Engineering Board"
            ]
        } [
            'note': "Signed by Example Electrical Engineering Board"
            'signed': Signature
        ]
    "#}.trim());

    assert_eq!(
        credential.format_flat(),
        r#"{ ARID(4676635a) [ 'isA': "Certificate of Completion", "certificateNumber": "123-456-789", "continuingEducationUnits": 1, "expirationDate": 2028-01-01, "firstName": "James", "issueDate": 2020-01-01, "lastName": "Maxwell", "photo": "This is James Maxwell's photo.", "professionalDevelopmentHours": 15, "subject": "RF and Microwave Engineering", "topics": ["Subject 1", "Subject 2"], 'controller': "Example Electrical Engineering Board", 'issuer': "Example Electrical Engineering Board" ] } [ 'note': "Signed by Example Electrical Engineering Board", 'signed': Signature ]"#
    );

    let s = credential.tree_format();
    // println!("{}", s);
    #[rustfmt::skip]
    assert_eq!(s, indoc! {r#"
        0b721f78 NODE
            397a2d4c subj WRAPPED
                8122ffa9 subj NODE
                    10d3de01 subj ARID(4676635a)
                    1f9ff098 ASSERTION
                        9e3bff3a pred "certificateNumber"
                        21c21808 obj "123-456-789"
                    36c254d0 ASSERTION
                        6e5d379f pred "expirationDate"
                        639ae9bf obj 2028-01-01
                    3c114201 ASSERTION
                        5f82a16a pred "lastName"
                        fe4d5230 obj "Maxwell"
                    4a9b2e4d ASSERTION
                        222afe69 pred "issueDate"
                        cb67f31d obj 2020-01-01
                    4d67bba0 ASSERTION
                        2be2d79b pred 'isA'
                        051beee6 obj "Certificate of Completion"
                    5171cbaf ASSERTION
                        3976ef74 pred "photo"
                        231b8527 obj "This is James Maxwell's photo."
                    54b3e1e7 ASSERTION
                        f13aa855 pred "professionalDevelopmentHours"
                        dc0e9c36 obj 15
                    5dc6d4e3 ASSERTION
                        4395643b pred "firstName"
                        d6d0b768 obj "James"
                    68895d8e ASSERTION
                        e6bf4dd3 pred "topics"
                        543fcc09 obj ["Subject 1", "Subject 2"]
                    8ec5e912 ASSERTION
                        2b191589 pred "continuingEducationUnits"
                        4bf5122f obj 1
                    9b3d4785 ASSERTION
                        af10ee92 pred 'controller'
                        f8489ac1 obj "Example Electrical Engineering Board"
                    caf5ced3 ASSERTION
                        8e4e62eb pred "subject"
                        202c10ef obj "RF and Microwave Engineering"
                    d3e0cc15 ASSERTION
                        6dd16ba3 pred 'issuer'
                        f8489ac1 obj "Example Electrical Engineering Board"
            46a02aaf ASSERTION
                d0e39e78 pred 'signed'
                34c14941 obj Signature
            e6d7fca0 ASSERTION
                0fcd6a39 pred 'note'
                f106bad1 obj "Signed by Example Electrical Engineering…"
    "#}.trim());

    #[rustfmt::skip]
    assert_eq!(credential.tree_format_opt(&TreeFormatOpts::default().hide_nodes(true)), indoc! {r#"
        WRAPPED
            ARID(4676635a)
                ASSERTION
                    "certificateNumber"
                    "123-456-789"
                ASSERTION
                    "expirationDate"
                    2028-01-01
                ASSERTION
                    "lastName"
                    "Maxwell"
                ASSERTION
                    "issueDate"
                    2020-01-01
                ASSERTION
                    'isA'
                    "Certificate of Completion"
                ASSERTION
                    "photo"
                    "This is James Maxwell's photo."
                ASSERTION
                    "professionalDevelopmentHours"
                    15
                ASSERTION
                    "firstName"
                    "James"
                ASSERTION
                    "topics"
                    ["Subject 1", "Subject 2"]
                ASSERTION
                    "continuingEducationUnits"
                    1
                ASSERTION
                    'controller'
                    "Example Electrical Engineering Board"
                ASSERTION
                    "subject"
                    "RF and Microwave Engineering"
                ASSERTION
                    'issuer'
                    "Example Electrical Engineering Board"
            ASSERTION
                'signed'
                Signature
            ASSERTION
                'note'
                "Signed by Example Electrical Engineering…"
    "#}.trim());

    assert_eq!(
        credential.elements_count(),
        credential.tree_format().split('\n').count()
    );
}

#[cfg(feature = "signature")]
#[test]
fn test_redacted_credential() {
    let credential = credential();
    let mut target = HashSet::new();
    target.insert(credential.digest().into_owned());
    for assertion in credential.assertions() {
        target.extend(assertion.deep_digests());
    }
    target.insert(credential.subject().digest().into_owned());
    let content = credential.subject().unwrap_envelope().unwrap();
    target.insert(content.digest().into_owned());
    target.insert(content.subject().digest().into_owned());

    target.extend(
        content
            .assertion_with_predicate("firstName")
            .unwrap()
            .shallow_digests(),
    );
    target.extend(
        content
            .assertion_with_predicate("lastName")
            .unwrap()
            .shallow_digests(),
    );
    target.extend(
        content
            .assertion_with_predicate(known_values::IS_A)
            .unwrap()
            .shallow_digests(),
    );
    target.extend(
        content
            .assertion_with_predicate(known_values::ISSUER)
            .unwrap()
            .shallow_digests(),
    );
    target.extend(
        content
            .assertion_with_predicate("subject")
            .unwrap()
            .shallow_digests(),
    );
    target.extend(
        content
            .assertion_with_predicate("expirationDate")
            .unwrap()
            .shallow_digests(),
    );
    let redacted_credential = credential.elide_revealing_set(&target);
    let rng = Rc::new(RefCell::new(make_fake_random_number_generator()));
    let options = SigningOptions::Schnorr { rng };
    let warranty = redacted_credential
        .wrap_envelope()
        .add_assertion(
            "employeeHiredDate",
            dcbor::Date::from_string("2022-01-01").unwrap(),
        )
        .add_assertion("employeeStatus", "active")
        .wrap_envelope()
        .add_assertion(known_values::NOTE, "Signed by Employer Corp.")
        .add_signature_opt(&bob_private_key(), Some(options), None)
        .check_encoding()
        .unwrap();

    #[rustfmt::skip]
    assert_eq!(warranty.format(), indoc! {r#"
        {
            {
                {
                    ARID(4676635a) [
                        'isA': "Certificate of Completion"
                        "expirationDate": 2028-01-01
                        "firstName": "James"
                        "lastName": "Maxwell"
                        "subject": "RF and Microwave Engineering"
                        'issuer': "Example Electrical Engineering Board"
                        ELIDED (7)
                    ]
                } [
                    'note': "Signed by Example Electrical Engineering Board"
                    'signed': Signature
                ]
            } [
                "employeeHiredDate": 2022-01-01
                "employeeStatus": "active"
            ]
        } [
            'note': "Signed by Employer Corp."
            'signed': Signature
        ]
    "#}.trim());

    println!("{}", warranty.mermaid_format_opt(&MermaidFormatOpts::default().hide_nodes(true)));

    assert_eq!(
        warranty.format_flat(),
        r#"{ { { ARID(4676635a) [ 'isA': "Certificate of Completion", "expirationDate": 2028-01-01, "firstName": "James", "lastName": "Maxwell", "subject": "RF and Microwave Engineering", 'issuer': "Example Electrical Engineering Board", ELIDED (7) ] } [ 'note': "Signed by Example Electrical Engineering Board", 'signed': Signature ] } [ "employeeHiredDate": 2022-01-01, "employeeStatus": "active" ] } [ 'note': "Signed by Employer Corp.", 'signed': Signature ]"#
    );

    let s = warranty.tree_format();
    // println!("{}", s);
    #[rustfmt::skip]
    assert_eq!(s, indoc! {r#"
        7ab3e6b1 NODE
            3907ee6f subj WRAPPED
                719d5955 subj NODE
                    10fb2e18 subj WRAPPED
                        0b721f78 subj NODE
                            397a2d4c subj WRAPPED
                                8122ffa9 subj NODE
                                    10d3de01 subj ARID(4676635a)
                                    1f9ff098 ELIDED
                                    36c254d0 ASSERTION
                                        6e5d379f pred "expirationDate"
                                        639ae9bf obj 2028-01-01
                                    3c114201 ASSERTION
                                        5f82a16a pred "lastName"
                                        fe4d5230 obj "Maxwell"
                                    4a9b2e4d ELIDED
                                    4d67bba0 ASSERTION
                                        2be2d79b pred 'isA'
                                        051beee6 obj "Certificate of Completion"
                                    5171cbaf ELIDED
                                    54b3e1e7 ELIDED
                                    5dc6d4e3 ASSERTION
                                        4395643b pred "firstName"
                                        d6d0b768 obj "James"
                                    68895d8e ELIDED
                                    8ec5e912 ELIDED
                                    9b3d4785 ELIDED
                                    caf5ced3 ASSERTION
                                        8e4e62eb pred "subject"
                                        202c10ef obj "RF and Microwave Engineering"
                                    d3e0cc15 ASSERTION
                                        6dd16ba3 pred 'issuer'
                                        f8489ac1 obj "Example Electrical Engineering Board"
                            46a02aaf ASSERTION
                                d0e39e78 pred 'signed'
                                34c14941 obj Signature
                            e6d7fca0 ASSERTION
                                0fcd6a39 pred 'note'
                                f106bad1 obj "Signed by Example Electrical Engineering…"
                    4c159c16 ASSERTION
                        e1ae011e pred "employeeHiredDate"
                        13b5a817 obj 2022-01-01
                    e071508b ASSERTION
                        d03e7352 pred "employeeStatus"
                        1d7a790d obj "active"
            874aa7e1 ASSERTION
                0fcd6a39 pred 'note'
                f59806d2 obj "Signed by Employer Corp."
            d21d2033 ASSERTION
                d0e39e78 pred 'signed'
                5ba600c9 obj Signature
    "#}.trim());

    let s =
        warranty.tree_format_opt(&TreeFormatOpts::default().hide_nodes(true));
    // println!("{}", s);
    #[rustfmt::skip]
    assert_eq!(s, indoc! {r#"
        subj WRAPPED
            subj WRAPPED
                subj WRAPPED
                    subj ARID(4676635a)
                        ELIDED
                        ASSERTION
                            pred "expirationDate"
                            obj 2028-01-01
                        ASSERTION
                            pred "lastName"
                            obj "Maxwell"
                        ELIDED
                        ASSERTION
                            pred 'isA'
                            obj "Certificate of Completion"
                        ELIDED
                        ELIDED
                        ASSERTION
                            pred "firstName"
                            obj "James"
                        ELIDED
                        ELIDED
                        ELIDED
                        ASSERTION
                            pred "subject"
                            obj "RF and Microwave Engineering"
                        ASSERTION
                            pred 'issuer'
                            obj "Example Electrical Engineering Board"
                    ASSERTION
                        pred 'signed'
                        obj Signature
                    ASSERTION
                        pred 'note'
                        obj "Signed by Example Electrical Engineering…"
                ASSERTION
                    pred "employeeHiredDate"
                    obj 2022-01-01
                ASSERTION
                    pred "employeeStatus"
                    obj "active"
            ASSERTION
                pred 'note'
                obj "Signed by Employer Corp."
            ASSERTION
                pred 'signed'
                obj Signature
    "#}.trim());

    assert_eq!(
        warranty.elements_count(),
        warranty.tree_format().split('\n').count()
    );
}
