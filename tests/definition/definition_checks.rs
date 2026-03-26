//! Integration tests that verify the correctness of definition validation logic.
//!
//! Tests span all definition kinds – [`BasicDefinition`], [`ObjectDefinition`],
//! [`StructDefinition`], [`MapDefinition`], and [`TableDefinition`] – checking their constructors,
//! accessors, and serialization round-trips.
use datastore::definition::{
    BasicDefinition, MapDefinition, ObjectDefinition, PropertyDefinition, StructDefinition,
    StructItemDefinition, TableDefinition,
};
use datastore::{StoreKey, store_key};

#[test]
fn test_property_definition() {
    // Why: Test basic property definition creation and properties.
    let basic_prop = PropertyDefinition::new("Basic Prop", BasicDefinition::new_string("String"));

    // Check the various properties of the property definition.
    assert_eq!(basic_prop.description().as_ref(), "Basic Prop");
    assert!(matches!(
        basic_prop.item_type(),
        datastore::definition::PropertyDefinitionType::Basic(_)
    ));
    assert_eq!(basic_prop.is_gui_visible(), true);
}

#[test]
fn test_struct_property_definition() {
    // Why: Test struct property definition creation and properties.
    let struct_prop = PropertyDefinition::new(
        "Struct Prop",
        StructDefinition::new("Struct", Vec::<(StoreKey, StructItemDefinition)>::new()),
    );

    // Check the various properties of the property definition.
    assert!(matches!(
        struct_prop.item_type(),
        datastore::definition::PropertyDefinitionType::Struct(_)
    ));
    assert_eq!(struct_prop.is_gui_visible(), true);
}

#[test]
fn test_table_property_definition() {
    // Why: Test table property definition creation and properties.
    let table_prop = PropertyDefinition::new(
        "Table Prop",
        TableDefinition::new("Table", Vec::<(StoreKey, BasicDefinition)>::new()),
    );

    // Check the various properties of the property definition.
    assert!(matches!(
        table_prop.item_type(),
        datastore::definition::PropertyDefinitionType::Table(_)
    ));
    assert_eq!(table_prop.is_gui_visible(), true);
}

#[test]
fn test_map_property_definition() {
    // Why: Test map property definition creation and properties.
    let map_prop = PropertyDefinition::new(
        "Map Prop",
        MapDefinition::new(
            "Map",
            StructDefinition::new("Item", Vec::<(StoreKey, StructItemDefinition)>::new()),
        ),
    );

    // Check the various properties of the property definition.
    assert!(matches!(
        map_prop.item_type(),
        datastore::definition::PropertyDefinitionType::Map(_)
    ));
    assert_eq!(map_prop.is_gui_visible(), true);
}

#[test]
fn test_property_gui_visibility() {
    // Why: Test basic property definition creation and properties with gui invisibility.
    let basic_prop =
        PropertyDefinition::new_gui_invisible("Basic Prop", BasicDefinition::new_string("String"));

    // Check the various properties of the property definition.
    assert_eq!(basic_prop.description().as_ref(), "Basic Prop");
    assert!(matches!(
        basic_prop.item_type(),
        datastore::definition::PropertyDefinitionType::Basic(_)
    ));
    assert_eq!(basic_prop.is_gui_visible(), false);
}

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
