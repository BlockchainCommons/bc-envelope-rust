// Example demonstrating null and tag patterns through the high-level Pattern
// API, including the new name-based tag matching functionality

use bc_envelope::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!(
        "Demonstrating NullPattern and TagPattern through the high-level Pattern API\n"
    );

    // Register all available tags for name-based matching
    dcbor::register_tags();
    bc_components::register_tags();

    // Example 1: Null Pattern
    println!("=== Null Pattern ===");

    // Create a null envelope
    let null_envelope = Envelope::null();
    println!("Null envelope: {}", null_envelope.format());

    // Test null pattern matching
    assert!(Pattern::null().matches(&null_envelope));
    println!("✓ Pattern::null() matches null envelope");

    // Non-null envelope should not match
    let text_envelope = Envelope::new("hello");
    assert!(!Pattern::null().matches(&text_envelope));
    println!("✓ Pattern::null() does not match text envelope");

    // Null envelope with assertion
    let null_with_assertion = null_envelope.add_assertion("type", "nothing");
    assert!(Pattern::null().matches(&null_with_assertion));
    println!("✓ Pattern::null() matches null envelope with assertion");

    println!();

    // Example 2: Tag Pattern (Value-based)
    println!("=== Tag Pattern (Value-based) ===");

    // Create a tagged value
    let tagged_cbor = CBOR::to_tagged_value(42, "meaning of life");
    let tagged_envelope = Envelope::new(tagged_cbor);
    println!("Tagged envelope: {}", tagged_envelope.format());

    // Test any tag pattern
    assert!(Pattern::any_tag().matches(&tagged_envelope));
    println!("✓ Pattern::any_tag() matches tagged envelope");

    // Test specific tag value pattern
    assert!(Pattern::tag_value(42).matches(&tagged_envelope));
    println!("✓ Pattern::tag_value(42) matches tagged envelope");

    // Test different tag value should not match
    assert!(!Pattern::tag_value(99).matches(&tagged_envelope));
    println!("✓ Pattern::tag_value(99) does not match tagged envelope");

    // Test specific tag object pattern
    let tag = Tag::with_value(42);
    assert!(Pattern::tag(tag).matches(&tagged_envelope));
    println!("✓ Pattern::tag(Tag::with_value(42)) matches tagged envelope");

    // Non-tagged envelope should not match
    assert!(!Pattern::any_tag().matches(&text_envelope));
    println!("✓ Pattern::any_tag() does not match non-tagged envelope");

    println!();

    // Example 3: Tag Pattern (Name-based) - NEW FUNCTIONALITY
    println!("=== Tag Pattern (Name-based) ===");

    // Create envelope with a registered tag (date tag = 1)
    let date_tagged_cbor = CBOR::to_tagged_value(1, "2023-12-25");
    let date_envelope = Envelope::new(date_tagged_cbor);
    println!("Date tagged envelope: {}", date_envelope.format());

    // Test name-based tag matching
    assert!(Pattern::tag_named("date").matches(&date_envelope));
    println!("✓ Pattern::tag_named(\"date\") matches date-tagged envelope");

    // Test with wrong name
    assert!(!Pattern::tag_named("unknown_tag").matches(&date_envelope));
    println!(
        "✓ Pattern::tag_named(\"unknown_tag\") does not match date-tagged envelope"
    );

    // Test unregistered tag with name pattern
    assert!(!Pattern::tag_named("date").matches(&tagged_envelope));
    println!(
        "✓ Pattern::tag_named(\"date\") does not match unregistered tag 42"
    );

    println!();

    // Example 4: Tag Pattern (Regex-based) - NEW FUNCTIONALITY
    println!("=== Tag Pattern (Regex-based) ===");

    // Test regex patterns on registered tags
    let regex_pattern = regex::Regex::new(r"^da.*")?; // Matches "date"
    assert!(Pattern::tag_regex(regex_pattern).matches(&date_envelope));
    println!("✓ Pattern::tag_regex(r\"^da.*\") matches date-tagged envelope");

    let regex_pattern = regex::Regex::new(r".*te$")?; // Matches names ending in "te"
    assert!(Pattern::tag_regex(regex_pattern).matches(&date_envelope));
    println!("✓ Pattern::tag_regex(r\".*te$\") matches date-tagged envelope");

    let regex_pattern = regex::Regex::new(r"^time.*")?; // Should not match "date"
    assert!(!Pattern::tag_regex(regex_pattern).matches(&date_envelope));
    println!(
        "✓ Pattern::tag_regex(r\"^time.*\") does not match date-tagged envelope"
    );

    // Regex patterns don't match unregistered tags (no names in registry)
    let regex_pattern = regex::Regex::new(r".*")?; // Match everything
    assert!(!Pattern::tag_regex(regex_pattern).matches(&tagged_envelope));
    println!(
        "✓ Pattern::tag_regex(r\".*\") does not match unregistered tag 42"
    );

    println!();

    // Example 5: Demonstrating different tag registries
    println!("=== Different Tag Sources ===");

    // Standard CBOR tags (from dcbor)
    println!(
        "Standard CBOR date tag (1): {}",
        Pattern::tag_named("date").matches(&date_envelope)
    );

    // Try some bc-components tags if available
    // Note: The actual availability depends on what's registered
    let digest_pattern = Pattern::tag_named("digest");
    let arid_pattern = Pattern::tag_named("arid");
    println!(
        "BC-Components digest tag available: {}",
        with_tags!(|tags: &TagsStore| tags.tag_for_name("digest").is_some())
    );
    println!(
        "BC-Components arid tag available: {}",
        with_tags!(|tags: &TagsStore| tags.tag_for_name("arid").is_some())
    );

    println!();

    // Example 6: Using patterns in sequences
    println!("=== Sequence Patterns ===");

    // Tagged envelope with assertion
    let tagged_with_assertion =
        tagged_envelope.add_assertion("context", "example");
    assert!(Pattern::any_tag().matches(&tagged_with_assertion));
    assert!(Pattern::tag_value(42).matches(&tagged_with_assertion));
    println!("✓ Tag patterns work with assertions");

    // Find null values and extract their subjects
    let sequence_pattern =
        Pattern::sequence(vec![Pattern::null(), Pattern::subject()]);

    let paths = sequence_pattern.paths(&null_with_assertion);
    println!("Null sequence pattern found {} paths", paths.len());

    // Find tagged values and extract their subjects
    let tag_sequence_pattern =
        Pattern::sequence(vec![Pattern::any_tag(), Pattern::subject()]);

    let paths = tag_sequence_pattern.paths(&tagged_with_assertion);
    println!("Tag sequence pattern found {} paths", paths.len());

    // Find date-tagged values specifically
    let date_sequence_pattern =
        Pattern::sequence(vec![Pattern::tag_named("date"), Pattern::subject()]);

    let date_with_assertion = date_envelope.add_assertion("format", "ISO-8601");
    let paths = date_sequence_pattern.paths(&date_with_assertion);
    println!("Date tag sequence pattern found {} paths", paths.len());

    println!("\n✅ All null and tag pattern examples completed successfully!");
    println!("New functionality demonstrated:");
    println!("  • Pattern::tag_named() - Match tags by registered name");
    println!("  • Pattern::tag_regex() - Match tags by regex on name");
    println!("  • Integration with global tags registry");
    println!("  • Support for both dcbor and bc-components tags");

    Ok(())
}
