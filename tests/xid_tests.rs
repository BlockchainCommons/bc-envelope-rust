#[cfg(feature = "xid")]
use bc_envelope::prelude::*;
use bc_rand::make_fake_random_number_generator;
use indoc::indoc;
use bc_components::{tags, with_tags, PrivateKeyBase, XID};

#[test]
fn test_xid_document() {
    // Create a XID document.
    let mut rng = make_fake_random_number_generator();
    let private_key_base = PrivateKeyBase::new_using(&mut rng);
    let xid_document = XIDDocument::from(&private_key_base);

    // Extract the XID from the XID document.
    let xid = xid_document.xid();

    // Convert the XID document to an Envelope.
    let envelope = xid_document.clone().into_envelope();
    let expected_format = indoc! {r#"
    XID(71274df1) [
        'allow': 'all'
        'key': PublicKeyBase
    ]"#}.trim();
    assert_eq!(envelope.format(), expected_format);

    // Convert the Envelope back to a XIDDocument.
    let xid_document2 = XIDDocument::try_from(envelope).unwrap();
    assert_eq!(xid_document, xid_document2);

    // Print the document's XID in debug format, which shows the full
    // identifier.
    let xid_debug = format!("{:?}", xid);
    assert_eq!(xid_debug, "XID(71274df133169a0e2d2ffb11cbc7917732acafa31989f685cca6cb69d473b93c)");

    // Print the document's XID in display format, which shows the short
    // identifier (first 4 bytes).
    let xid_display = format!("{}", xid);
    assert_eq!(xid_display, "XID(71274df1)");

    // Print the CBOR diagnostic notation for the XID.
    let xid_cbor_diagnostic = xid.to_cbor().diagnostic();
    assert_eq!(xid_cbor_diagnostic, indoc! {r#"
    40024(
       h'71274df133169a0e2d2ffb11cbc7917732acafa31989f685cca6cb69d473b93c'
    )
    "#}.trim());

    // Print the hex encoding of the XID.
    with_tags!(|tags: &dyn dcbor::TagsStoreTrait| {
        assert_eq!(tags.name_for_tag(&tags::XID), "xid");
        let xid_cbor_dump = xid.to_cbor().hex_opt(true, Some(tags));
        assert_eq!(xid_cbor_dump, indoc! {r#"
        d9 9c58                                  # tag(40024) xid
           5820                                  # bytes(32)
              71274df133169a0e2d2ffb11cbc7917732acafa31989f685cca6cb69d473b93c
        "#}.trim());
    });

    // Print the XID's Bytewords and Bytemoji identifiers.
    let bytewords_identifier = xid.bytewords_identifier(true);
    assert_eq!(bytewords_identifier, "🅧 JUGS DELI GIFT WHEN");
    let bytemoji_identifier = xid.bytemoji_identifier(true);
    assert_eq!(bytemoji_identifier, "🅧 🌊 😹 🌽 🐞");

    // Print the XID's LifeHash fingerprint.
    assert_eq!(format!("{}", xid.lifehash_fingerprint()), "Digest(fc7b562825afa07ee9d49fe99991f767ad4bdad495724c08a918e13ee0eabd5e)");

    // Print the XID's UR.
    let xid_ur = xid.ur_string();
    println!("{}", xid_ur);
}

#[test]
fn test_minimal_xid_document() {
    // Create a XID.
    let mut rng = make_fake_random_number_generator();
    let private_key_base = PrivateKeyBase::new_using(&mut rng);
    let xid = XID::from(&private_key_base);

    // Create a XIDDocument directly from the XID.
    let xid_document = XIDDocument::from(&xid);

    // Convert the XIDDocument to an Envelope.
    let envelope = xid_document.clone().into_envelope();

    // The envelope is just the XID as its subject, with no assertions.
    let expected_format = indoc! {r#"
    XID(71274df1)
    "#}.trim();
    assert_eq!(envelope.format(), expected_format);

    // Convert the Envelope back to a XIDDocument.
    let xid_document2 = XIDDocument::try_from(envelope).unwrap();
    assert_eq!(xid_document, xid_document2);

    // The CBOR encoding of the XID and the XIDDocument should be the same.
    let xid_cbor = xid.to_cbor();
    let xid_document_cbor = xid_document.to_cbor();
    assert_eq!(xid_cbor, xid_document_cbor);

    // Either a XID or a XIDDocument can be created from the CBOR encoding.
    let xid2 = XID::try_from(xid_cbor).unwrap();
    assert_eq!(xid, xid2);
    let xid_document2 = XIDDocument::try_from(xid_document_cbor).unwrap();
    assert_eq!(xid_document, xid_document2);

    // The UR of the XID and the XIDDocument should be the same.
    let xid_ur = xid.ur_string();
    let expected_ur = "ur:xid/hdcxjsdigtwneocmnybadpdlzobysbstmekteypspeotcfldynlpsfolsbintyjkrhfnvsbyrdfw";
    assert_eq!(xid_ur, expected_ur);
    let xid_document_ur = xid_document.ur_string();
    assert_eq!(xid_document_ur, expected_ur);
}

#[test]
fn test_signed_xid_document() {
    // Generate the genesis key.
    let mut rng = make_fake_random_number_generator();
    let private_genesis_key = PrivateKeyBase::new_using(&mut rng);

    // Create a XIDDocument for the genesis key.
    let xid_document = XIDDocument::from(&private_genesis_key);

    let envelope = xid_document.clone().into_envelope();
    let expected_format = indoc! {r#"
    XID(71274df1) [
        'allow': 'all'
        'key': PublicKeyBase
    ]
    "#}.trim();
    assert_eq!(envelope.format(), expected_format);

    let signed_envelope = xid_document.to_signed_envelope(&private_genesis_key);
    // println!("{}", signed_envelope.format());
    let expected_format = indoc! {r#"
    {
        XID(71274df1) [
            'allow': 'all'
            'key': PublicKeyBase
        ]
    } [
        'verifiedBy': Signature
    ]
    "#}.trim();
    assert_eq!(signed_envelope.format(), expected_format);

    let self_certified_xid_document = XIDDocument::try_from_signed_envelope(&signed_envelope).unwrap();
    assert_eq!(xid_document, self_certified_xid_document);
}
