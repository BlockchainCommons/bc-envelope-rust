use std::{collections::HashSet, rc::Rc};

use bc_envelope::{IntoEnvelope, Envelope, known_values};

use bc_components::{SymmetricKey, Digest, DigestProvider, ARID};
use bc_rand::make_fake_random_number_generator;
use dcbor::{CBOREncodable, Date};
use indoc::indoc;
use hex_literal::hex;

include!("test_data.rs.inc");
use test_data::*;

#[test]
fn test_plaintext() {
    let envelope = Envelope::new(PLAINTEXT_HELLO);
    assert_eq!(envelope.format(), indoc! {r#"
    "Hello."
    "#}.trim());
    assert_eq!(envelope.clone().tree_format(false, None), indoc! {r#"
    8cc96cdb "Hello."
    "#}.trim());
    assert_eq!(envelope.clone().tree_format(true, None), indoc! {r#"
    "Hello."
    "#}.trim());
    assert_eq!(envelope.elements_count(), envelope.tree_format(false, None).split('\n').count());
}

#[test]
fn test_signed_plaintext() {
    let mut rng = make_fake_random_number_generator();
    let envelope = Envelope::new(PLAINTEXT_HELLO)
        .sign_with_using(&alice_private_keys(), &mut rng);
    assert_eq!(envelope.format(), indoc! {r#"
    "Hello." [
        'verifiedBy': Signature
    ]
    "#}.trim());
    assert_eq!(envelope.clone().tree_format(false, None), indoc! {r#"
    509d8ab2 NODE
        8cc96cdb subj "Hello."
        f761face ASSERTION
            d0e39e78 pred 'verifiedBy'
            0d868228 obj Signature
    "#}.trim());
    assert_eq!(envelope.clone().tree_format(true, None), indoc! {r#"
    "Hello."
        ASSERTION
            'verifiedBy'
            Signature
    "#}.trim());
    assert_eq!(envelope.elements_count(), envelope.tree_format(false, None).split('\n').count());
}

#[test]
fn test_encrypt_subject() {
    let envelope = Envelope::new("Alice")
        .add_assertion("knows", "Bob")
        .encrypt_subject(&SymmetricKey::new())
        .unwrap();
    assert_eq!(envelope.format(), indoc! {r#"
    ENCRYPTED [
        "knows": "Bob"
    ]
    "#}.trim());
    assert_eq!(envelope.clone().tree_format(false, None), indoc! {r#"
    8955db5e NODE
        13941b48 subj ENCRYPTED
        78d666eb ASSERTION
            db7dd21c pred "knows"
            13b74194 obj "Bob"
    "#}.trim());
    assert_eq!(envelope.clone().tree_format(true, None), indoc! {r#"
    ENCRYPTED
        ASSERTION
            "knows"
            "Bob"
    "#}.trim());
    assert_eq!(envelope.elements_count(), envelope.tree_format(false, None).split('\n').count());
}

#[test]
fn test_top_level_assertion() {
    let envelope = Envelope::new_assertion("knows", "Bob");
    assert_eq!(envelope.format(), indoc! {r#"
    "knows": "Bob"
    "#}.trim());
    assert_eq!(envelope.clone().tree_format(false, None), indoc! {r#"
    78d666eb ASSERTION
        db7dd21c pred "knows"
        13b74194 obj "Bob"
    "#}.trim());
    assert_eq!(envelope.clone().tree_format(true, None), indoc! {r#"
    ASSERTION
        "knows"
        "Bob"
    "#}.trim());
    assert_eq!(envelope.elements_count(), envelope.tree_format(false, None).split('\n').count());
}

#[test]
fn test_elided_object() {
    let envelope = Envelope::new("Alice")
        .add_assertion("knows", "Bob");
    let elided = envelope.elide_removing_target(&"Bob".into_envelope());
    assert_eq!(elided.format(), indoc! {r#"
    "Alice" [
        "knows": ELIDED
    ]
    "#}.trim());
    assert_eq!(elided.clone().tree_format(false, None), indoc! {r#"
    8955db5e NODE
        13941b48 subj "Alice"
        78d666eb ASSERTION
            db7dd21c pred "knows"
            13b74194 obj ELIDED
    "#}.trim());
    assert_eq!(elided.clone().tree_format(true, None), indoc! {r#"
    "Alice"
        ASSERTION
            "knows"
            ELIDED
    "#}.trim());
    assert_eq!(elided.elements_count(), elided.tree_format(false, None).split('\n').count());
}

#[test]
fn test_signed_subject() {
    let mut rng = make_fake_random_number_generator();
    let envelope = Envelope::new("Alice")
        .add_assertion("knows", "Bob")
        .add_assertion("knows", "Carol")
        .sign_with_using(&alice_private_keys(), &mut rng);
    assert_eq!(envelope.format(), indoc! {r#"
    "Alice" [
        "knows": "Bob"
        "knows": "Carol"
        'verifiedBy': Signature
    ]
    "#}.trim());
    assert_eq!(envelope.clone().tree_format(false, None), indoc! {r#"
    98457ac2 NODE
        13941b48 subj "Alice"
        4012caf2 ASSERTION
            db7dd21c pred "knows"
            afb8122e obj "Carol"
        6c31d926 ASSERTION
            d0e39e78 pred 'verifiedBy'
            4ab15d8b obj Signature
        78d666eb ASSERTION
            db7dd21c pred "knows"
            13b74194 obj "Bob"
    "#}.trim());
    assert_eq!(envelope.clone().tree_format(true, None), indoc! {r#"
    "Alice"
        ASSERTION
            "knows"
            "Carol"
        ASSERTION
            'verifiedBy'
            Signature
        ASSERTION
            "knows"
            "Bob"
    "#}.trim());
    assert_eq!(envelope.elements_count(), envelope.clone().tree_format(false, None).split('\n').count());

    // Elided assertions
    let mut target: HashSet<Digest> = HashSet::new();
    target.insert(envelope.digest().into_owned());
    target.insert(envelope.clone().subject().digest().into_owned());
    let elided = envelope.elide_revealing_set(&target);
    assert_eq!(elided.format(), indoc! {r#"
    "Alice" [
        ELIDED (3)
    ]
    "#}.trim());
    assert_eq!(elided.clone().tree_format(false, None), indoc! {r#"
    98457ac2 NODE
        13941b48 subj "Alice"
        4012caf2 ELIDED
        6c31d926 ELIDED
        78d666eb ELIDED
"#}.trim());
    assert_eq!(elided.clone().tree_format(true, None), indoc! {r#"
    "Alice"
        ELIDED
        ELIDED
        ELIDED
    "#}.trim());
    assert_eq!(elided.elements_count(), elided.clone().tree_format(false, None).split('\n').count());
}

#[test]
fn test_wrap_then_signed() {
    let mut rng = make_fake_random_number_generator();
    let envelope = Envelope::new("Alice")
        .add_assertion("knows", "Bob")
        .add_assertion("knows", "Carol")
        .wrap_envelope()
        .sign_with_using(&alice_private_keys(), &mut rng);
    assert_eq!(envelope.format(), indoc! {r#"
    {
        "Alice" [
            "knows": "Bob"
            "knows": "Carol"
        ]
    } [
        'verifiedBy': Signature
    ]
    "#}.trim());
    assert_eq!(envelope.clone().tree_format(false, None), indoc! {r#"
    410ba52c NODE
        9e3b0673 subj WRAPPED
            b8d857f6 subj NODE
                13941b48 subj "Alice"
                4012caf2 ASSERTION
                    db7dd21c pred "knows"
                    afb8122e obj "Carol"
                78d666eb ASSERTION
                    db7dd21c pred "knows"
                    13b74194 obj "Bob"
        73600f24 ASSERTION
            d0e39e78 pred 'verifiedBy'
            d08ddb0d obj Signature
    "#}.trim());
    assert_eq!(envelope.clone().tree_format(true, None), indoc! {r#"
    WRAPPED
        "Alice"
            ASSERTION
                "knows"
                "Carol"
            ASSERTION
                "knows"
                "Bob"
        ASSERTION
            'verifiedBy'
            Signature
    "#}.trim());
    assert_eq!(envelope.elements_count(), envelope.clone().tree_format(false, None).split('\n').count());
}

#[test]
fn test_encrypt_to_recipients() {
    let envelope = Envelope::new(PLAINTEXT_HELLO)
        .encrypt_subject_opt(&fake_content_key(), Some(fake_nonce())).unwrap().check_encoding().unwrap()
        .add_recipient_opt(&bob_public_keys(), &fake_content_key(), Some(fake_content_key().data()), Some(&fake_nonce())).check_encoding().unwrap()
        .add_recipient_opt(&carol_public_keys(), &fake_content_key(), Some(fake_content_key().data()), Some(&fake_nonce())).check_encoding().unwrap();
    assert_eq!(envelope.format(), indoc! {r#"
    ENCRYPTED [
        'hasRecipient': SealedMessage
        'hasRecipient': SealedMessage
    ]
    "#}.trim());
    assert_eq!(envelope.clone().tree_format(false, None), indoc! {r#"
    310c90f6 NODE
        8cc96cdb subj ENCRYPTED
        93f3bf1d ASSERTION
            943a35d1 pred 'hasRecipient'
            d3250f01 obj SealedMessage
        bf724d53 ASSERTION
            943a35d1 pred 'hasRecipient'
            78a28897 obj SealedMessage
    "#}.trim());
    assert_eq!(envelope.clone().tree_format(true, None), indoc! {r#"
    ENCRYPTED
        ASSERTION
            'hasRecipient'
            SealedMessage
        ASSERTION
            'hasRecipient'
            SealedMessage
    "#}.trim());
    assert_eq!(envelope.elements_count(), envelope.clone().tree_format(false, None).split('\n').count());
}

#[test]
fn test_assertion_positions() {
    let predicate = Envelope::new("predicate")
        .add_assertion("predicate-predicate", "predicate-object");
    let object = Envelope::new("object")
        .add_assertion("object-predicate", "object-object");
    let envelope = Envelope::new("subject")
        .add_assertion(predicate, object)
        .check_encoding().unwrap();
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
    assert_eq!(envelope.clone().tree_format(false, None), indoc! {r#"
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
    assert_eq!(envelope.clone().tree_format(true, None), indoc! {r#"
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
    assert_eq!(envelope.elements_count(), envelope.clone().tree_format(false, None).split('\n').count());
}

#[test]
fn test_complex_metadata() {
    // Assertions made about an ARID are considered part of a distributed set. Which
    // assertions are returned depends on who resolves the ARID and when it is
    // resolved. In other words, the referent of an ARID is mutable.
    let author = Envelope::new(ARID::from_data(hex!("9c747ace78a4c826392510dd6285551e7df4e5164729a1b36198e56e017666c8")))
        .add_assertion(known_values::DEREFERENCE_VIA, "LibraryOfCongress")
        .add_assertion(known_values::HAS_NAME, "Ayn Rand")
        .check_encoding().unwrap();

    // Assertions made on a literal value are considered part of the same set of
    // assertions made on the digest of that value.
    let name_en = Envelope::new("Atlas Shrugged")
        .add_assertion(known_values::LANGUAGE, "en");

    let name_es = Envelope::new("La rebelión de Atlas")
        .add_assertion(known_values::LANGUAGE, "es");

    let work = Envelope::new(ARID::from_data(hex!("7fb90a9d96c07f39f75ea6acf392d79f241fac4ec0be2120f7c82489711e3e80")))
        .add_assertion(known_values::IS_A, "novel")
        .add_assertion("isbn", "9780451191144")
        .add_assertion("author", author)
        .add_assertion(known_values::DEREFERENCE_VIA, "LibraryOfCongress")
        .add_assertion(known_values::HAS_NAME, name_en)
        .add_assertion(known_values::HAS_NAME, name_es)
        .check_encoding().unwrap();

    let book_data = "This is the entire book “Atlas Shrugged” in EPUB format.";
    // Assertions made on a digest are considered associated with that specific binary
    // object and no other. In other words, the referent of a Digest is immutable.
    let book_metadata = Envelope::new(Digest::from_image(&book_data))
        .add_assertion("work", work)
        .add_assertion("format", "EPUB")
        .add_assertion(known_values::DEREFERENCE_VIA, "IPFS")
        .check_encoding().unwrap();

    assert_eq!(book_metadata.format(), indoc! {r#"
    Digest(26d05af5) [
        "format": "EPUB"
        "work": ARID(7fb90a9d) [
            "author": ARID(9c747ace) [
                'dereferenceVia': "LibraryOfCongress"
                'hasName': "Ayn Rand"
            ]
            "isbn": "9780451191144"
            'dereferenceVia': "LibraryOfCongress"
            'hasName': "Atlas Shrugged" [
                'language': "en"
            ]
            'hasName': "La rebelión de Atlas" [
                'language': "es"
            ]
            'isA': "novel"
        ]
        'dereferenceVia': "IPFS"
    ]
    "#}.trim());

    assert_eq!(book_metadata.clone().tree_format(false, None), indoc! {r#"
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
                    14ff9eac pred 'hasName'
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
                            14ff9eac pred 'hasName'
                            98985bd5 obj "Ayn Rand"
                9c10d60f ASSERTION
                    cdb6a696 pred 'dereferenceVia'
                    34a04547 obj "LibraryOfCongress"
                b722c07c ASSERTION
                    14ff9eac pred 'hasName'
                    0cfacc06 obj NODE
                        e84c3091 subj "Atlas Shrugged"
                        b80d3b05 ASSERTION
                            60dfb783 pred 'language'
                            6700869c obj "en"
    "#}.trim());

    assert_eq!(book_metadata.clone().tree_format(true, None), indoc! {r#"
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
                    'hasName'
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
                            'hasName'
                            "Ayn Rand"
                ASSERTION
                    'dereferenceVia'
                    "LibraryOfCongress"
                ASSERTION
                    'hasName'
                    "Atlas Shrugged"
                        ASSERTION
                            'language'
                            "en"
    "#}.trim());
    assert_eq!(book_metadata.elements_count(), book_metadata.clone().tree_format(false, None).split('\n').count());
}

fn credential() -> Rc<Envelope> {
    let mut rng = make_fake_random_number_generator();
    Envelope::new(ARID::from_data(hex!("4676635a6e6068c2ef3ffd8ff726dd401fd341036e920f136a1d8af5e829496d")))
        .add_assertion(known_values::IS_A, "Certificate of Completion")
        .add_assertion(known_values::ISSUER, "Example Electrical Engineering Board")
        .add_assertion(known_values::CONTROLLER, "Example Electrical Engineering Board")
        .add_assertion("firstName", "James")
        .add_assertion("lastName", "Maxwell")
        .add_assertion("issueDate", Date::new_from_string("2020-01-01").unwrap())
        .add_assertion("expirationDate", Date::new_from_string("2028-01-01").unwrap())
        .add_assertion("photo", "This is James Maxwell's photo.")
        .add_assertion("certificateNumber", "123-456-789")
        .add_assertion("subject", "RF and Microwave Engineering")
        .add_assertion("continuingEducationUnits", 1)
        .add_assertion("professionalDevelopmentHours", 15)
        .add_assertion("topics", vec!["Subject 1", "Subject 2"].cbor())
        .wrap_envelope()
        .sign_with_using(&alice_private_keys(), &mut rng)
        .add_assertion(known_values::NOTE, "Signed by Example Electrical Engineering Board")
        .check_encoding().unwrap()
}

#[test]
fn test_credential() {
    let credential = credential();
    assert_eq!(credential.format(), indoc! {r#"
    {
        ARID(4676635a) [
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
            'isA': "Certificate of Completion"
            'issuer': "Example Electrical Engineering Board"
        ]
    } [
        'note': "Signed by Example Electrical Engineering Board"
        'verifiedBy': Signature
    ]
    "#}.trim());
    assert_eq!(credential.clone().tree_format(false, None), indoc! {r#"
    11d52de3 NODE
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
        52b10e0b ASSERTION
            d0e39e78 pred 'verifiedBy'
            039ef97a obj Signature
        e6d7fca0 ASSERTION
            0fcd6a39 pred 'note'
            f106bad1 obj "Signed by Example Electrical Engineering…"
    "#}.trim());
    assert_eq!(credential.clone().tree_format(true, None), indoc! {r#"
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
            'verifiedBy'
            Signature
        ASSERTION
            'note'
            "Signed by Example Electrical Engineering…"
    "#}.trim());
    assert_eq!(credential.elements_count(), credential.tree_format(false, None).split('\n').count());
}

#[test]
fn test_redacted_credential() {
    let credential = credential();
    let mut target = HashSet::new();
    target.insert(credential.digest().into_owned());
    for assertion in credential.clone().assertions() {
        target.extend(assertion.deep_digests());
    }
    target.insert(credential.clone().subject().digest().into_owned());
    let content = credential.clone().subject().unwrap_envelope().unwrap();
    target.insert(content.digest().into_owned());
    target.insert(content.clone().subject().digest().into_owned());

    target.extend(content.clone().assertion_with_predicate("firstName").unwrap().shallow_digests());
    target.extend(content.clone().assertion_with_predicate("lastName").unwrap().shallow_digests());
    target.extend(content.clone().assertion_with_predicate(known_values::IS_A).unwrap().shallow_digests());
    target.extend(content.clone().assertion_with_predicate(known_values::ISSUER).unwrap().shallow_digests());
    target.extend(content.clone().assertion_with_predicate("subject").unwrap().shallow_digests());
    target.extend(content.assertion_with_predicate("expirationDate").unwrap().shallow_digests());
    let redacted_credential = credential.elide_revealing_set(&target);
    let mut rng = make_fake_random_number_generator();
    let warranty = redacted_credential
        .wrap_envelope()
        .add_assertion("employeeHiredDate", Date::new_from_string("2022-01-01").unwrap())
        .add_assertion("employeeStatus", "active")
        .wrap_envelope()
        .add_assertion(known_values::NOTE, "Signed by Employer Corp.")
        .sign_with_using(&bob_private_keys(), &mut rng)
        .check_encoding().unwrap();
    assert_eq!(warranty.format(), indoc! {r#"
    {
        {
            {
                ARID(4676635a) [
                    "expirationDate": 2028-01-01
                    "firstName": "James"
                    "lastName": "Maxwell"
                    "subject": "RF and Microwave Engineering"
                    'isA': "Certificate of Completion"
                    'issuer': "Example Electrical Engineering Board"
                    ELIDED (7)
                ]
            } [
                'note': "Signed by Example Electrical Engineering Board"
                'verifiedBy': Signature
            ]
        } [
            "employeeHiredDate": 2022-01-01
            "employeeStatus": "active"
        ]
    } [
        'note': "Signed by Employer Corp."
        'verifiedBy': Signature
    ]
    "#}.trim());
    assert_eq!(warranty.clone().tree_format(false, None), indoc! {r#"
    a816c8ce NODE
        d4a527ac subj WRAPPED
            3e2b6cab subj NODE
                7506ccc6 subj WRAPPED
                    11d52de3 subj NODE
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
                        52b10e0b ASSERTION
                            d0e39e78 pred 'verifiedBy'
                            039ef97a obj Signature
                        e6d7fca0 ASSERTION
                            0fcd6a39 pred 'note'
                            f106bad1 obj "Signed by Example Electrical Engineering…"
                4c159c16 ASSERTION
                    e1ae011e pred "employeeHiredDate"
                    13b5a817 obj 2022-01-01
                e071508b ASSERTION
                    d03e7352 pred "employeeStatus"
                    1d7a790d obj "active"
        4054359a ASSERTION
            d0e39e78 pred 'verifiedBy'
            0ea27c28 obj Signature
        874aa7e1 ASSERTION
            0fcd6a39 pred 'note'
            f59806d2 obj "Signed by Employer Corp."
    "#}.trim());
    assert_eq!(warranty.clone().tree_format(true, None), indoc! {r#"
    WRAPPED
        WRAPPED
            WRAPPED
                ARID(4676635a)
                    ELIDED
                    ASSERTION
                        "expirationDate"
                        2028-01-01
                    ASSERTION
                        "lastName"
                        "Maxwell"
                    ELIDED
                    ASSERTION
                        'isA'
                        "Certificate of Completion"
                    ELIDED
                    ELIDED
                    ASSERTION
                        "firstName"
                        "James"
                    ELIDED
                    ELIDED
                    ELIDED
                    ASSERTION
                        "subject"
                        "RF and Microwave Engineering"
                    ASSERTION
                        'issuer'
                        "Example Electrical Engineering Board"
                ASSERTION
                    'verifiedBy'
                    Signature
                ASSERTION
                    'note'
                    "Signed by Example Electrical Engineering…"
            ASSERTION
                "employeeHiredDate"
                2022-01-01
            ASSERTION
                "employeeStatus"
                "active"
        ASSERTION
            'verifiedBy'
            Signature
        ASSERTION
            'note'
            "Signed by Employer Corp."
    "#}.trim());
    assert_eq!(warranty.elements_count(), warranty.tree_format(false, None).split('\n').count());
}
