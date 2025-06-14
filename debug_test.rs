// Quick test to understand the envelope structure
use bc_envelope::prelude::*;
use dcbor::prelude::*;

fn main() {
    // Ensure tags are registered for testing
    dcbor::register_tags();
    bc_components::register_tags();

    // Test with registered tag (date tag = 1)
    let tagged_cbor = CBOR::to_tagged_value(1, "2023-12-25");
    let envelope = Envelope::new(tagged_cbor);
    
    println!("Envelope: {}", envelope.format());
    println!("Is node: {}", envelope.is_node());
    println!("Subject paths: {:?}", Pattern::subject().paths(&envelope));
    
    // Test the individual patterns
    println!("Tag pattern paths: {:?}", Pattern::tagged_with_name("date").paths(&envelope));
    
    // Test sequence
    let seq_paths = Pattern::sequence(vec![
        Pattern::tagged_with_name("date"),
        Pattern::subject(),
    ]).paths(&envelope);
    println!("Sequence paths: {:?}", seq_paths);
}
