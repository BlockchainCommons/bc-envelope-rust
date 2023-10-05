#![cfg(feature = "attachment")]

use bc_envelope::prelude::*;
use indoc::indoc;

mod common;
use common::test_seed::Seed;

#[test]
fn test_attachment() -> anyhow::Result<()> {
    let seed = Seed::new_opt(
        hex::decode("82f32c855d3d542256180810797e0073").unwrap(),
        "Alice's Seed",
        "This is the note.",
        None
    );
    let seed_envelope = seed
        .into_envelope()
        .add_attachment("Attachment Data V1", "com.example", Some("https://example.com/seed-attachment/v1"))
        .add_attachment("Attachment Data V2", "com.example", Some("https://example.com/seed-attachment/v2"));

    with_format_context!(|context| {
        assert_eq!(seed_envelope.format_opt(Some(context)),
        indoc! {r#"
        Bytes(16) [
            'isA': 'Seed'
            'attachment': {
                "Attachment Data V1"
            } [
                'conformsTo': "https://example.com/seed-attachment/v1"
                'vendor': "com.example"
            ]
            'attachment': {
                "Attachment Data V2"
            } [
                'conformsTo': "https://example.com/seed-attachment/v2"
                'vendor': "com.example"
            ]
            'hasName': "Alice's Seed"
            'note': "This is the note."
        ]
        "#}.trim()
        );
    });

    assert_eq!(seed_envelope.clone().attachments()?.len(), 2);
    assert_eq!(seed_envelope.clone().attachments_with_vendor_and_conforms_to(Some("com.example"), None)?.len(), 2);
    let v1_attachment = seed_envelope.clone().attachment_with_vendor_and_conforms_to(None, Some("https://example.com/seed-attachment/v1"))?;
    let payload = v1_attachment.clone().attachment_payload()?;
    with_format_context!(|context| {
        assert_eq!(payload.format_opt(Some(context)),
        indoc! {r#"
        "Attachment Data V1"
        "#}.trim()
        );
    });
    assert_eq!(v1_attachment.clone().attachment_vendor()?, "com.example");
    assert_eq!(v1_attachment.attachment_conforms_to()?, Some("https://example.com/seed-attachment/v1".to_string()));

    let seed_envelope2 = seed.into_envelope();
    let attachments = seed_envelope.clone().attachments()?;
    let seed_envelope2 = seed_envelope2.add_assertions(&attachments);
    assert!(seed_envelope2.is_equivalent_to(seed_envelope));

    Ok(())
}
