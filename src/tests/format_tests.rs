use std::{collections::HashSet, rc::Rc};

use crate::{IntoEnvelope, Envelope, known_value_registry};

use bc_components::{SymmetricKey, Digest, DigestProvider, CID};
use bc_crypto::make_fake_random_number_generator;
use dcbor::{CBOREncodable, Date};
use indoc::indoc;

use super::test_data::*;
use hex_literal::hex;

#[test]
fn test_plaintext() {
    let envelope = PLAINTEXT_HELLO.into_envelope();
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
    let envelope = PLAINTEXT_HELLO.into_envelope()
        .sign_with_using(&alice_private_keys(), &mut rng);
    assert_eq!(envelope.format(), indoc! {r#"
    "Hello." [
        verifiedBy: Signature
    ]
    "#}.trim());
    assert_eq!(envelope.clone().tree_format(false, None), indoc! {r#"
    f987a100 NODE
        8cc96cdb subj "Hello."
        823a7958 ASSERTION
            9d7ba9eb pred verifiedBy
            93387377 obj Signature
    "#}.trim());
    assert_eq!(envelope.clone().tree_format(true, None), indoc! {r#"
    "Hello."
        ASSERTION
            verifiedBy
            Signature
    "#}.trim());
    assert_eq!(envelope.elements_count(), envelope.tree_format(false, None).split('\n').count());
}

#[test]
fn test_encrypt_subject() {
    let envelope = "Alice".into_envelope()
        .add_assertion("knows".into_envelope(), "Bob".into_envelope())
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
    let envelope = Envelope::new_assertion("knows".into_envelope(), "Bob".into_envelope());
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
    let envelope = "Alice".into_envelope()
        .add_assertion("knows".into_envelope(), "Bob".into_envelope());
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
    let envelope = "Alice".into_envelope()
        .add_assertion("knows".into_envelope(), "Bob".into_envelope())
        .add_assertion("knows".into_envelope(), "Carol".into_envelope())
        .sign_with_using(&alice_private_keys(), &mut rng);
    assert_eq!(envelope.format(), indoc! {r#"
    "Alice" [
        "knows": "Bob"
        "knows": "Carol"
        verifiedBy: Signature
    ]
    "#}.trim());
    assert_eq!(envelope.clone().tree_format(false, None), indoc! {r#"
    6cd79bd6 NODE
        13941b48 subj "Alice"
        4012caf2 ASSERTION
            db7dd21c pred "knows"
            afb8122e obj "Carol"
        78d666eb ASSERTION
            db7dd21c pred "knows"
            13b74194 obj "Bob"
        7cd7252e ASSERTION
            9d7ba9eb pred verifiedBy
            a83508b8 obj Signature
    "#}.trim());
    assert_eq!(envelope.clone().tree_format(true, None), indoc! {r#"
    "Alice"
        ASSERTION
            "knows"
            "Carol"
        ASSERTION
            "knows"
            "Bob"
        ASSERTION
            verifiedBy
            Signature
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
    6cd79bd6 NODE
        13941b48 subj "Alice"
        4012caf2 ELIDED
        78d666eb ELIDED
        7cd7252e ELIDED
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
    let envelope = "Alice".into_envelope()
        .add_assertion("knows".into_envelope(), "Bob".into_envelope())
        .add_assertion("knows".into_envelope(), "Carol".into_envelope())
        .wrap_envelope()
        .sign_with_using(&alice_private_keys(), &mut rng);
    assert_eq!(envelope.format(), indoc! {r#"
    {
        "Alice" [
            "knows": "Bob"
            "knows": "Carol"
        ]
    } [
        verifiedBy: Signature
    ]
    "#}.trim());
    assert_eq!(envelope.clone().tree_format(false, None), indoc! {r#"
    6a388c1d NODE
        9e3b0673 subj WRAPPED
            b8d857f6 subj NODE
                13941b48 subj "Alice"
                4012caf2 ASSERTION
                    db7dd21c pred "knows"
                    afb8122e obj "Carol"
                78d666eb ASSERTION
                    db7dd21c pred "knows"
                    13b74194 obj "Bob"
        27435202 ASSERTION
            9d7ba9eb pred verifiedBy
            0bdbcecd obj Signature
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
            verifiedBy
            Signature
    "#}.trim());
    assert_eq!(envelope.elements_count(), envelope.clone().tree_format(false, None).split('\n').count());
}

#[test]
fn test_encrypt_to_recipients() {
    let envelope = PLAINTEXT_HELLO.into_envelope()
        .encrypt_subject_opt(&fake_content_key(), Some(fake_nonce())).unwrap().check_encoding().unwrap()
        .add_recipient_opt(&bob_public_keys(), &fake_content_key(), Some(fake_content_key().data()), Some(&fake_nonce())).check_encoding().unwrap()
        .add_recipient_opt(&carol_public_keys(), &fake_content_key(), Some(fake_content_key().data()), Some(&fake_nonce())).check_encoding().unwrap();
    assert_eq!(envelope.format(), indoc! {r#"
    ENCRYPTED [
        hasRecipient: SealedMessage
        hasRecipient: SealedMessage
    ]
    "#}.trim());
    assert_eq!(envelope.clone().tree_format(false, None), indoc! {r#"
    9f130c09 NODE
        8cc96cdb subj ENCRYPTED
        7137c3c2 ASSERTION
            e41178b8 pred hasRecipient
            b0925349 obj SealedMessage
        fdb08145 ASSERTION
            e41178b8 pred hasRecipient
            b0e86e20 obj SealedMessage
    "#}.trim());
    assert_eq!(envelope.clone().tree_format(true, None), indoc! {r#"
    ENCRYPTED
        ASSERTION
            hasRecipient
            SealedMessage
        ASSERTION
            hasRecipient
            SealedMessage
    "#}.trim());
    assert_eq!(envelope.elements_count(), envelope.clone().tree_format(false, None).split('\n').count());
}

#[test]
fn test_assertion_positions() {
    let predicate = "predicate".into_envelope()
        .add_assertion("predicate-predicate".into_envelope(), "predicate-object".into_envelope());
    let object = "object".into_envelope()
        .add_assertion("object-predicate".into_envelope(), "object-object".into_envelope());
    let envelope = "subject".into_envelope()
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
    // Assertions made about a CID are considered part of a distributed set. Which
    // assertions are returned depends on who resolves the CID and when it is
    // resolved. In other words, the referent of a CID is mutable.
    let author = CID::from_data(hex!("9c747ace78a4c826392510dd6285551e7df4e5164729a1b36198e56e017666c8")).into_envelope()
        .add_assertion(known_value_registry::DEREFERENCE_VIA.into_envelope(), "LibraryOfCongress".into_envelope())
        .add_assertion(known_value_registry::HAS_NAME.into_envelope(), "Ayn Rand".into_envelope())
        .check_encoding().unwrap();

    // Assertions made on a literal value are considered part of the same set of
    // assertions made on the digest of that value.
    let name_en = "Atlas Shrugged".into_envelope()
        .add_assertion(known_value_registry::LANGUAGE.into_envelope(), "en".into_envelope());

    let name_es = "La rebelión de Atlas".into_envelope()
        .add_assertion(known_value_registry::LANGUAGE.into_envelope(), "es".into_envelope());

    let work = CID::from_data(hex!("7fb90a9d96c07f39f75ea6acf392d79f241fac4ec0be2120f7c82489711e3e80")).into_envelope()
        .add_assertion(known_value_registry::IS_A.into_envelope(), "novel".into_envelope())
        .add_assertion("isbn".into_envelope(), "9780451191144".into_envelope())
        .add_assertion("author".into_envelope(), author)
        .add_assertion(known_value_registry::DEREFERENCE_VIA.into_envelope(), "LibraryOfCongress".into_envelope())
        .add_assertion(known_value_registry::HAS_NAME.into_envelope(), name_en)
        .add_assertion(known_value_registry::HAS_NAME.into_envelope(), name_es)
        .check_encoding().unwrap();

    let book_data = "This is the entire book “Atlas Shrugged” in EPUB format.";
    // Assertions made on a digest are considered associated with that specific binary
    // object and no other. In other words, the referent of a Digest is immutable.
    let book_metadata = Digest::from_image(&book_data).into_envelope()
        .add_assertion("work".into_envelope(), work)
        .add_assertion("format".into_envelope(), "EPUB".into_envelope())
        .add_assertion(known_value_registry::DEREFERENCE_VIA.into_envelope(), "IPFS".into_envelope())
        .check_encoding().unwrap();

    assert_eq!(book_metadata.format(), indoc! {r#"
    Digest(26d05af5) [
        "format": "EPUB"
        "work": CID(7fb90a9d) [
            "author": CID(9c747ace) [
                dereferenceVia: "LibraryOfCongress"
                hasName: "Ayn Rand"
            ]
            "isbn": "9780451191144"
            dereferenceVia: "LibraryOfCongress"
            hasName: "Atlas Shrugged" [
                language: "en"
            ]
            hasName: "La rebelión de Atlas" [
                language: "es"
            ]
            isA: "novel"
        ]
        dereferenceVia: "IPFS"
    ]
    "#}.trim());

    assert_eq!(book_metadata.clone().tree_format(false, None), indoc! {r#"
    f41cbb59 NODE
        5d3e9195 subj Digest(26d05af5)
        05edf8ca ASSERTION
            e25b9baf pred dereferenceVia
            15eac58f obj "IPFS"
        953cdab2 ASSERTION
            a9a86b03 pred "format"
            9536cfe0 obj "EPUB"
        a3400534 ASSERTION
            2ddb0b05 pred "work"
            3d2c3a7f obj NODE
                d8304d46 subj CID(7fb90a9d)
                1786d8b5 ASSERTION
                    4019420b pred "isbn"
                    69ff76b1 obj "9780451191144"
                1903fe89 ASSERTION
                    9d0480e0 pred hasName
                    61a11981 obj NODE
                        5e825721 subj "La rebelión de Atlas"
                        62ea333c ASSERTION
                            65fa1c25 pred language
                            b33e79c2 obj "es"
                212af9fb ASSERTION
                    96f0167d pred isA
                    6d7c7189 obj "novel"
                21af9ce9 ASSERTION
                    29c09059 pred "author"
                    71bf6c35 obj NODE
                        f4f77a81 subj CID(9c747ace)
                        050a4539 ASSERTION
                            9d0480e0 pred hasName
                            98985bd5 obj "Ayn Rand"
                        24b5a41b ASSERTION
                            e25b9baf pred dereferenceVia
                            34a04547 obj "LibraryOfCongress"
                24b5a41b ASSERTION
                    e25b9baf pred dereferenceVia
                    34a04547 obj "LibraryOfCongress"
                3d1f0148 ASSERTION
                    9d0480e0 pred hasName
                    fb15ce3e obj NODE
                        e84c3091 subj "Atlas Shrugged"
                        e6bd65c8 ASSERTION
                            65fa1c25 pred language
                            6700869c obj "en"
    "#}.trim());

    assert_eq!(book_metadata.clone().tree_format(true, None), indoc! {r#"
    Digest(26d05af5)
        ASSERTION
            dereferenceVia
            "IPFS"
        ASSERTION
            "format"
            "EPUB"
        ASSERTION
            "work"
            CID(7fb90a9d)
                ASSERTION
                    "isbn"
                    "9780451191144"
                ASSERTION
                    hasName
                    "La rebelión de Atlas"
                        ASSERTION
                            language
                            "es"
                ASSERTION
                    isA
                    "novel"
                ASSERTION
                    "author"
                    CID(9c747ace)
                        ASSERTION
                            hasName
                            "Ayn Rand"
                        ASSERTION
                            dereferenceVia
                            "LibraryOfCongress"
                ASSERTION
                    dereferenceVia
                    "LibraryOfCongress"
                ASSERTION
                    hasName
                    "Atlas Shrugged"
                        ASSERTION
                            language
                            "en"
    "#}.trim());
    assert_eq!(book_metadata.elements_count(), book_metadata.clone().tree_format(false, None).split('\n').count());
}

fn credential() -> Rc<Envelope> {
    let mut rng = make_fake_random_number_generator();
    CID::from_data(hex!("4676635a6e6068c2ef3ffd8ff726dd401fd341036e920f136a1d8af5e829496d")).into_envelope()
        .add_assertion(known_value_registry::IS_A.into_envelope(), "Certificate of Completion".into_envelope())
        .add_assertion(known_value_registry::ISSUER.into_envelope(), "Example Electrical Engineering Board".into_envelope())
        .add_assertion(known_value_registry::CONTROLLER.into_envelope(), "Example Electrical Engineering Board".into_envelope())
        .add_assertion("firstName".into_envelope(), "James".into_envelope())
        .add_assertion("lastName".into_envelope(), "Maxwell".into_envelope())
        .add_assertion("issueDate".into_envelope(), Date::new_from_string("2020-01-01").unwrap().into_envelope())
        .add_assertion("expirationDate".into_envelope(), Date::new_from_string("2028-01-01").unwrap().into_envelope())
        .add_assertion("photo".into_envelope(), "This is James Maxwell's photo.".into_envelope())
        .add_assertion("certificateNumber".into_envelope(), "123-456-789".into_envelope())
        .add_assertion("subject".into_envelope(), "RF and Microwave Engineering".into_envelope())
        .add_assertion("continuingEducationUnits".into_envelope(), 1.into_envelope())
        .add_assertion("professionalDevelopmentHours".into_envelope(), 15.into_envelope())
        .add_assertion("topics".into_envelope(), vec!["Subject 1", "Subject 2"].cbor().into_envelope())
        .wrap_envelope()
        .sign_with_using(&alice_private_keys(), &mut rng)
        .add_assertion(known_value_registry::NOTE.into_envelope(), "Signed by Example Electrical Engineering Board".into_envelope())
        .check_encoding().unwrap()
}

#[test]
fn test_credential() {
    let credential = credential();
    assert_eq!(credential.format(), indoc! {r#"
    {
        CID(4676635a) [
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
            controller: "Example Electrical Engineering Board"
            isA: "Certificate of Completion"
            issuer: "Example Electrical Engineering Board"
        ]
    } [
        note: "Signed by Example Electrical Engineering Board"
        verifiedBy: Signature
    ]
    "#}.trim());
    assert_eq!(credential.clone().tree_format(false, None), indoc! {r#"
    31ff6d6b NODE
        5886e784 subj WRAPPED
            c9860567 subj NODE
                5fb45cf1 subj CID(4676635a)
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
                922c859a ASSERTION
                    96f0167d pred isA
                    051beee6 obj "Certificate of Completion"
                caf5ced3 ASSERTION
                    8e4e62eb pred "subject"
                    202c10ef obj "RF and Microwave Engineering"
                d61e0984 ASSERTION
                    e665c567 pred controller
                    f8489ac1 obj "Example Electrical Engineering Board"
                ebcbf71f ASSERTION
                    fde30b5c pred issuer
                    f8489ac1 obj "Example Electrical Engineering Board"
        55b14b17 ASSERTION
            49a5f41b pred note
            f106bad1 obj "Signed by Example Electrical Engineering…"
        9902b6e6 ASSERTION
            9d7ba9eb pred verifiedBy
            4b4c0a2e obj Signature
    "#}.trim());
    assert_eq!(credential.clone().tree_format(true, None), indoc! {r#"
    WRAPPED
        CID(4676635a)
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
                isA
                "Certificate of Completion"
            ASSERTION
                "subject"
                "RF and Microwave Engineering"
            ASSERTION
                controller
                "Example Electrical Engineering Board"
            ASSERTION
                issuer
                "Example Electrical Engineering Board"
        ASSERTION
            note
            "Signed by Example Electrical Engineering…"
        ASSERTION
            verifiedBy
            Signature
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

    target.extend(content.clone().assertion_with_predicate("firstName".into_envelope()).unwrap().shallow_digests());
    target.extend(content.clone().assertion_with_predicate("lastName".into_envelope()).unwrap().shallow_digests());
    target.extend(content.clone().assertion_with_predicate(known_value_registry::IS_A).unwrap().shallow_digests());
    target.extend(content.clone().assertion_with_predicate(known_value_registry::ISSUER).unwrap().shallow_digests());
    target.extend(content.clone().assertion_with_predicate("subject".into_envelope()).unwrap().shallow_digests());
    target.extend(content.assertion_with_predicate("expirationDate".into_envelope()).unwrap().shallow_digests());
    let redacted_credential = credential.elide_revealing_set(&target);
    let mut rng = make_fake_random_number_generator();
    let warranty = redacted_credential
        .wrap_envelope()
        .add_assertion("employeeHiredDate".into_envelope(), Date::new_from_string("2022-01-01").unwrap().into_envelope())
        .add_assertion("employeeStatus".into_envelope(), "active".into_envelope())
        .wrap_envelope()
        .add_assertion(known_value_registry::NOTE.into_envelope(), "Signed by Employer Corp.".into_envelope())
        .sign_with_using(&bob_private_keys(), &mut rng)
        .check_encoding().unwrap();
    assert_eq!(warranty.format(), indoc! {r#"
    {
        {
            {
                CID(4676635a) [
                    "expirationDate": 2028-01-01
                    "firstName": "James"
                    "lastName": "Maxwell"
                    "subject": "RF and Microwave Engineering"
                    isA: "Certificate of Completion"
                    issuer: "Example Electrical Engineering Board"
                    ELIDED (7)
                ]
            } [
                note: "Signed by Example Electrical Engineering Board"
                verifiedBy: Signature
            ]
        } [
            "employeeHiredDate": 2022-01-01
            "employeeStatus": "active"
        ]
    } [
        note: "Signed by Employer Corp."
        verifiedBy: Signature
    ]
    "#}.trim());
    assert_eq!(warranty.clone().tree_format(false, None), indoc! {r#"
    ed7529fe NODE
        9222bb56 subj WRAPPED
            2d6a7024 subj NODE
                5063b615 subj WRAPPED
                    31ff6d6b subj NODE
                        5886e784 subj WRAPPED
                            c9860567 subj NODE
                                5fb45cf1 subj CID(4676635a)
                                1f9ff098 ELIDED
                                36c254d0 ASSERTION
                                    6e5d379f pred "expirationDate"
                                    639ae9bf obj 2028-01-01
                                3c114201 ASSERTION
                                    5f82a16a pred "lastName"
                                    fe4d5230 obj "Maxwell"
                                4a9b2e4d ELIDED
                                5171cbaf ELIDED
                                54b3e1e7 ELIDED
                                5dc6d4e3 ASSERTION
                                    4395643b pred "firstName"
                                    d6d0b768 obj "James"
                                68895d8e ELIDED
                                8ec5e912 ELIDED
                                922c859a ASSERTION
                                    96f0167d pred isA
                                    051beee6 obj "Certificate of Completion"
                                caf5ced3 ASSERTION
                                    8e4e62eb pred "subject"
                                    202c10ef obj "RF and Microwave Engineering"
                                d61e0984 ELIDED
                                ebcbf71f ASSERTION
                                    fde30b5c pred issuer
                                    f8489ac1 obj "Example Electrical Engineering Board"
                        55b14b17 ASSERTION
                            49a5f41b pred note
                            f106bad1 obj "Signed by Example Electrical Engineering…"
                        9902b6e6 ASSERTION
                            9d7ba9eb pred verifiedBy
                            4b4c0a2e obj Signature
                4c159c16 ASSERTION
                    e1ae011e pred "employeeHiredDate"
                    13b5a817 obj 2022-01-01
                e071508b ASSERTION
                    d03e7352 pred "employeeStatus"
                    1d7a790d obj "active"
        16639289 ASSERTION
            9d7ba9eb pred verifiedBy
            c82b0b88 obj Signature
        8f255569 ASSERTION
            49a5f41b pred note
            f59806d2 obj "Signed by Employer Corp."
    "#}.trim());
    assert_eq!(warranty.clone().tree_format(true, None), indoc! {r#"
    WRAPPED
        WRAPPED
            WRAPPED
                CID(4676635a)
                    ELIDED
                    ASSERTION
                        "expirationDate"
                        2028-01-01
                    ASSERTION
                        "lastName"
                        "Maxwell"
                    ELIDED
                    ELIDED
                    ELIDED
                    ASSERTION
                        "firstName"
                        "James"
                    ELIDED
                    ELIDED
                    ASSERTION
                        isA
                        "Certificate of Completion"
                    ASSERTION
                        "subject"
                        "RF and Microwave Engineering"
                    ELIDED
                    ASSERTION
                        issuer
                        "Example Electrical Engineering Board"
                ASSERTION
                    note
                    "Signed by Example Electrical Engineering…"
                ASSERTION
                    verifiedBy
                    Signature
            ASSERTION
                "employeeHiredDate"
                2022-01-01
            ASSERTION
                "employeeStatus"
                "active"
        ASSERTION
            verifiedBy
            Signature
        ASSERTION
            note
            "Signed by Employer Corp."
    "#}.trim());
    assert_eq!(warranty.elements_count(), warranty.tree_format(false, None).split('\n').count());
}
