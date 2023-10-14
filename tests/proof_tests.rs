#![cfg(feature = "proof")]

// ```swift
// class ProofTests: XCTestCase {
//     func testFriendsList() throws {
//         /// This document contains a list of people Alice knows. Each "knows" assertion has
//         /// been salted so if the assertions have been elided one can't merely guess at who
//         /// she knows by pairing the "knows" predicate with the names of possibly-known
//         /// associates and comparing the resulting digests to the elided digests in the
//         /// document.
//         let aliceFriends = Envelope("Alice")
//             .addAssertion("knows", "Bob", salted: true)
//             .addAssertion("knows", "Carol", salted: true)
//             .addAssertion("knows", "Dan", salted: true)
//         XCTAssertEqual(aliceFriends.format(),
//         """
//         "Alice" [
//             {
//                 "knows": "Bob"
//             } [
//                 'salt': Salt
//             ]
//             {
//                 "knows": "Carol"
//             } [
//                 'salt': Salt
//             ]
//             {
//                 "knows": "Dan"
//             } [
//                 'salt': Salt
//             ]
//         ]
//         """
//         )

//         /// Alice provides just the root digest of her document to a third party. This is
//         /// simply an envelope in which everything has been elided and nothing revealed.
//         let aliceFriendsRoot = aliceFriends.elideRevealing([])
//         XCTAssertEqual(aliceFriendsRoot.format(), "ELIDED")

//         /// Now Alice wants to prove to the third party that her document contains a "knows
//         /// Bob" assertion. To do this, she produces a proof that is an envelope with the
//         /// minimal structure of digests included so that the proof envelope has the same
//         /// digest as the completely elided envelope, but also exposes the digest of the
//         /// target of the proof.
//         ///
//         /// Note that in the proof the digests of the two other elided "knows" assertions
//         /// are present, but because they have been salted, the third party cannot easily
//         /// guess who else she knows.
//         let knowsBobAssertion = Envelope("knows", "Bob")
//         let aliceKnowsBobProof = try aliceFriends.proof(contains: knowsBobAssertion)!.checkEncoding()
//         XCTAssertEqual(aliceKnowsBobProof.format(),
//         """
//         ELIDED [
//             ELIDED [
//                 ELIDED
//             ]
//             ELIDED (2)
//         ]
//         """
//         )

//         /// The third party then uses the previously known and trusted root to confirm that
//         /// the envelope does indeed contain a "knows bob" assertion.
//         XCTAssertTrue(aliceFriendsRoot.confirm(contains: knowsBobAssertion, using: aliceKnowsBobProof))
//     }

//     func testMultiPosition() throws {
//         let aliceFriends = Envelope("Alice")
//             .addAssertion("knows", "Bob", salted: true)
//             .addAssertion("knows", "Carol", salted: true)
//             .addAssertion("knows", "Dan", salted: true)

//         /// In some cases the target of a proof might exist at more than one position in an
//         /// envelope. An example target from Alice's list of friends would be any envelope
//         /// containing "knows" as its subject. Since all three "knows" assertions use this
//         /// as their predicate, that identical envelope exists in three different positions
//         /// in the outer envelope.
//         ///
//         /// Note that revealing all positions of the "knows" predicate in this envelope also
//         /// reveals the digest of the salt for each assertion, which might make Alice's other
//         /// associates easier to guess.
//         let knowsProof = try aliceFriends.proof(contains: Envelope("knows"))!.checkEncoding()
//         XCTAssertEqual(knowsProof.format(),
//         """
//         ELIDED [
//             {
//                 ELIDED: ELIDED
//             } [
//                 ELIDED
//             ]
//             {
//                 ELIDED: ELIDED
//             } [
//                 ELIDED
//             ]
//             {
//                 ELIDED: ELIDED
//             } [
//                 ELIDED
//             ]
//         ]
//         """
//         )
//     }

//     func testVerifiableCredential() throws {
//         let arid = Envelope(ARID(‡"4676635a6e6068c2ef3ffd8ff726dd401fd341036e920f136a1d8af5e829496d")!)
//         let credential = try arid
//             .addAssertion("firstName", "John", salted: true)
//             .addAssertion("lastName", "Smith", salted: true)
//             .addAssertion("address", "123 Main St.", salted: true)
//             .addAssertion("birthDate", Date(iso8601: "1970-01-01"), salted: true)
//             .addAssertion("photo", "This is John Smith's photo.", salted: true)
//             .addAssertion("dlNumber", "123-456-789", salted: true)
//             .addAssertion("nonCommercialVehicleEndorsement", true, salted: true)
//             .addAssertion("motorocycleEndorsement", true, salted: true)
//             .addAssertion(.issuer, "State of Example")
//             .addAssertion(.controller, "State of Example")
//             .wrap()
//             .sign(with: alicePrivateKeys)
//             .addAssertion(.note, "Signed by the State of Example")

//         let credentialRoot = credential.elideRevealing([])

//         /// In this case the holder of a credential wants to prove a single assertion from it, the address.
//         let addressAssertion = Envelope("address", "123 Main St.")
//         let addressProof = try credential.proof(contains: addressAssertion)!.checkEncoding()
//         /// The proof includes digests from all the elided assertions.
//         XCTAssertEqual(addressProof.format(),
//         """
//         {
//             ELIDED [
//                 ELIDED [
//                     ELIDED
//                 ]
//                 ELIDED (9)
//             ]
//         } [
//             ELIDED (2)
//         ]
//         """
//         )

//         /// The proof confirms the address, as intended.
//         XCTAssertTrue(credentialRoot.confirm(contains: addressAssertion, using: addressProof))

//         /// Assertions without salt can also be confirmed.
//         let issuerAssertion = Envelope(.issuer, "State of Example")
//         XCTAssertTrue(credentialRoot.confirm(contains: issuerAssertion, using: addressProof))

//         /// The proof cannot be used to confirm salted assertions.
//         let firstNameAssertion = Envelope("firstName", "John")
//         XCTAssertFalse(credentialRoot.confirm(contains: firstNameAssertion, using: addressProof))
//     }
// }
// ```

use std::collections::HashSet;

use bc_components::{ARID, PrivateKeyBase};
use bc_envelope::prelude::*;
use dcbor::Date;
use indoc::indoc;
mod common;
use crate::common::{check_encoding::*, test_seed::Seed};
use hex_literal::hex;

#[test]
fn test_friends_list() {
    // This document contains a list of people Alice knows. Each "knows" assertion has
    // been salted so if the assertions have been elided one can't merely guess at who
    // she knows by pairing the "knows" predicate with the names of possibly-known
    // associates and comparing the resulting digests to the elided digests in the
    // document.
    let alice_friends = Envelope::new("Alice")
        .add_assertion_salted("knows", "Bob", true)
        .add_assertion_salted("knows", "Carol", true)
        .add_assertion_salted("knows", "Dan", true);
    assert_eq!(alice_friends.format(), indoc! {r#"
    "Alice" [
        {
            "knows": "Bob"
        } [
            'salt': Salt
        ]
        {
            "knows": "Carol"
        } [
            'salt': Salt
        ]
        {
            "knows": "Dan"
        } [
            'salt': Salt
        ]
    ]
    "#}.trim());

    // Alice provides just the root digest of her document to a third party. This is
    // simply an envelope in which everything has been elided and nothing revealed.
    let alice_friends_root = alice_friends.clone().elide_revealing_set(&HashSet::new());
    assert_eq!(alice_friends_root.format(), "ELIDED");

    // Now Alice wants to prove to the third party that her document contains a "knows
    // Bob" assertion. To do this, she produces a proof that is an envelope with the
    // minimal structure of digests included so that the proof envelope has the same
    // digest as the completely elided envelope, but also exposes the digest of the
    // target of the proof.
    //
    // Note that in the proof the digests of the two other elided "knows" assertions
    // are present, but because they have been salted, the third party cannot easily
    // guess who else she knows.
    let knows_bob_assertion = Envelope::new_assertion("knows", "Bob");
    let alice_knows_bob_proof = alice_friends.proof_contains_target(&knows_bob_assertion).unwrap().check_encoding().unwrap();
    assert_eq!(alice_knows_bob_proof.format(), indoc! {r#"
    ELIDED [
        ELIDED [
            ELIDED
        ]
        ELIDED (2)
    ]
    "#}.trim());

    // The third party then uses the previously known and trusted root to confirm that
    // the envelope does indeed contain a "knows bob" assertion.
    assert!(alice_friends_root.confirm_contains_target(&knows_bob_assertion, &alice_knows_bob_proof));
}

#[test]
fn test_multi_position() {
    let alice_friends = Envelope::new("Alice")
        .add_assertion_salted("knows", "Bob", true)
        .add_assertion_salted("knows", "Carol", true)
        .add_assertion_salted("knows", "Dan", true);

    // In some cases the target of a proof might exist at more than one position in an
    // envelope. An example target from Alice's list of friends would be any envelope
    // containing "knows" as its subject. Since all three "knows" assertions use this
    // as their predicate, that identical envelope exists in three different positions
    // in the outer envelope.
    //
    // Note that revealing all positions of the "knows" predicate in this envelope also
    // reveals the digest of the salt for each assertion, which might make Alice's other
    // associates easier to guess.
    let knows_proof = alice_friends.proof_contains_target(&Envelope::new("knows")).unwrap().check_encoding().unwrap();
    assert_eq!(knows_proof.format(), indoc! {r#"
    ELIDED [
        {
            ELIDED: ELIDED
        } [
            ELIDED
        ]
        {
            ELIDED: ELIDED
        } [
            ELIDED
        ]
        {
            ELIDED: ELIDED
        } [
            ELIDED
        ]
    ]
    "#}.trim());
}

#[test]
fn test_verifiable_credential() {
    // ```swift
    // let aliceSeed = Seed(data: ‡"82f32c855d3d542256180810797e0073")!
    // let alicePrivateKeys = PrivateKeyBase(aliceSeed)
    // ```

    let alice_seed = Seed::new(hex!("82f32c855d3d542256180810797e0073"));
    let alice_private_keys = PrivateKeyBase::from_data(alice_seed.data());
    let arid = Envelope::new(ARID::from_data_ref(&hex!("4676635a6e6068c2ef3ffd8ff726dd401fd341036e920f136a1d8af5e829496d")).unwrap());
    let credential = arid
        .add_assertion_salted("firstName", "John", true)
        .add_assertion_salted("lastName", "Smith", true)
        .add_assertion_salted("address", "123 Main St.", true)
        .add_assertion_salted("birthDate", Date::new_from_string("1970-01-01").unwrap(), true)
        .add_assertion_salted("photo", "This is John Smith's photo.", true)
        .add_assertion_salted("dlNumber", "123-456-789", true)
        .add_assertion_salted("nonCommercialVehicleEndorsement", true, true)
        .add_assertion_salted("motorocycleEndorsement", true, true)
        .add_assertion(known_values::ISSUER, "State of Example")
        .add_assertion(known_values::CONTROLLER, "State of Example")
        .wrap_envelope()
        .sign_with(&alice_private_keys)
        .add_assertion(known_values::NOTE, "Signed by the State of Example");

    let credential_root = credential.clone().elide_revealing_set(&HashSet::new());

    // In this case the holder of a credential wants to prove a single assertion from it, the address.
    let address_assertion = Envelope::new_assertion("address", "123 Main St.");
    let address_proof = credential.proof_contains_target(&address_assertion).unwrap().check_encoding().unwrap();
    // The proof includes digests from all the elided assertions.
    assert_eq!(address_proof.format(), indoc! {r#"
    {
        ELIDED [
            ELIDED [
                ELIDED
            ]
            ELIDED (9)
        ]
    } [
        ELIDED (2)
    ]
    "#}.trim());

    // The proof confirms the address, as intended.
    assert!(credential_root.clone().confirm_contains_target(&address_assertion, &address_proof));

    // Assertions without salt can also be confirmed.
    let issuer_assertion = Envelope::new_assertion(known_values::ISSUER, "State of Example");
    assert!(credential_root.clone().confirm_contains_target(&issuer_assertion, &address_proof));

    // The proof cannot be used to confirm salted assertions.
    let first_name_assertion = Envelope::new_assertion("firstName", "John");
    assert!(!credential_root.confirm_contains_target(&first_name_assertion, &address_proof));
}
