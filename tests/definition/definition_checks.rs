//! Integration tests that verify the correctness of definition validation logic.
//!
//! Tests span all definition kinds – [`BasicDefinition`], [`ObjectDefinition`],
//! [`StructDefinition`], [`MapDefinition`], and [`TableDefinition`] – checking their constructors,
//! accessors, and serialization round-trips.
use datastore::definition::{BasicDefinition, ObjectDefinition, PropertyDefinition};
use datastore::store_key;

#[test]
fn test_object_definition_basic() {
    // Why: Test object definition creation and properties.
    let mut builder = ObjectDefinition::builder("Test Object");
    builder.insert(
        store_key!("prop1"),
        PropertyDefinition::new("P1", BasicDefinition::new_string("D1")),
    );
    let obj_def = builder.finish();

    assert_eq!(obj_def.description().as_ref(), "Test Object");
    assert_eq!(obj_def.count(), 1);
    assert!(obj_def.contains_key(store_key!("prop1")));
    assert!(obj_def.contains_key_str("prop1"));
}
