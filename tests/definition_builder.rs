use datastore::StoreError;
use datastore::definition::{BasicDefinition, ObjectDefinition, PropertyDefinition};

#[test]
fn test_object_builder_pattern() {
    let obj_def = ObjectDefinition::builder("Test Object")
        .with(
            "prop1",
            PropertyDefinition::new("Property 1", BasicDefinition::new_string("String prop")),
        )
        .expect("Valid key")
        .with(
            "prop2",
            PropertyDefinition::new(
                "Property 2",
                BasicDefinition::new_number_with_default("Number prop", "0"),
            ),
        )
        .expect("Valid key")
        .finish();

    assert_eq!(obj_def.count(), 2);
}

#[test]
fn test_object_inheritance() {
    let parent_def = ObjectDefinition::builder("Parent")
        .with(
            "prop1",
            PropertyDefinition::new("P1", BasicDefinition::new_string_with_default("D1", "V1")),
        )
        .expect("Valid key")
        .finish();

    let builder = parent_def.new_inherit("Child");
    assert_eq!(builder.finish().count(), 1);

    let mut builder = parent_def.new_inherit("Child");
    builder
        .add(
            "prop2",
            PropertyDefinition::new("P2", BasicDefinition::new_string_with_default("D2", "V2")),
        )
        .expect("Valid key");

    let child_def = builder.finish();
    assert_eq!(child_def.count(), 2);
    assert!(child_def.contains_key("prop1"));
    assert!(child_def.contains_key("prop2"));

    let mut builder = child_def.new_inherit("Grandchild");
    builder.remove("prop1");
    let grandchild_def = builder.finish();
    assert_eq!(grandchild_def.count(), 1);
    assert!(!grandchild_def.contains_key("prop1"));
}

#[test]
fn test_invalid_keys() {
    let res = ObjectDefinition::builder("Obj").with(
        "",
        PropertyDefinition::new("P", BasicDefinition::new_string("D")),
    );
    assert!(matches!(res, Err(StoreError::KeyEmpty)));

    let res = ObjectDefinition::builder("Obj").with(
        "Invalid Key!",
        PropertyDefinition::new("P", BasicDefinition::new_string("D")),
    );
    assert!(matches!(res, Err(StoreError::KeyInvalidCharacter(_))));
    if let Err(StoreError::KeyInvalidCharacter(s)) = res {
        assert_eq!(s, "Invalid Key!");
    }
}

#[test]
fn test_object_definition_immutability() {
    let obj_def = ObjectDefinition::builder("Test Object")
        .with(
            "prop1",
            PropertyDefinition::new("Property 1", BasicDefinition::new_string("String prop")),
        )
        .expect("Valid key")
        .finish();

    // The point of this test is that obj_def does NOT have .add() or .remove()
    // It is immutable by design.
    assert_eq!(obj_def.count(), 1);
    assert!(obj_def.contains_key("prop1"));
}
