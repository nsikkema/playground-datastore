use datastore::definition::{
    BasicDefinition, BasicDefinitionType, ChoiceDefinition, FileDefinition, MapDefinition,
    ObjectDefinition, PropertyDefinition, StructDefinition, StructItemDefinition, TableDefinition,
};
use datastore::{StoreKey, store_key};

#[test]
fn test_basic_definition_string() {
    // Why: Test basic string definition creation and properties.
    let def = BasicDefinition::new_string("A string property");

    // Check the various properties of the string definition.
    assert_eq!(def.description().as_ref(), "A string property");
    assert!(matches!(def.type_definition(), BasicDefinitionType::String));
    assert_eq!(def.default_value().as_ref(), "");
}

#[test]
fn test_basic_definition_string_with_default() {
    // Why: Test basic string definition creation with a default value.
    let def = BasicDefinition::new_string_with_default("A string property", "default value");

    // Check the various properties of the string definition.
    assert_eq!(def.description().as_ref(), "A string property");
    assert!(matches!(def.type_definition(), BasicDefinitionType::String));
    assert_eq!(def.default_value().as_ref(), "default value");
}

#[test]
fn test_basic_definition_number() {
    // Why: Test basic number definition creation and properties.
    let def = BasicDefinition::new_number("A number property");

    // Check the various properties of the number definition.
    assert_eq!(def.description().as_ref(), "A number property");
    assert!(matches!(def.type_definition(), BasicDefinitionType::Number));
    assert_eq!(def.default_value().as_ref(), "");
}

#[test]
fn test_basic_definition_number_with_default() {
    // Why: Test basic number definition creation with a default value.
    let def = BasicDefinition::new_number_with_default("A number property", "5.0");

    // Check the various properties of the number definition.
    assert_eq!(def.description().as_ref(), "A number property");
    assert!(matches!(def.type_definition(), BasicDefinitionType::Number));
    assert_eq!(def.default_value().as_ref(), "5.0");
}

#[test]
fn test_basic_definition_file() {
    // Why: Test basic file definition creation and properties.
    let file_def = FileDefinition::new("txt");
    let def = BasicDefinition::new_file("A file property", file_def.clone());

    // Check the various properties of the file definition.
    assert_eq!(def.description().as_ref(), "A file property");
    if let BasicDefinitionType::File(f) = def.type_definition() {
        assert_eq!(f.extension_filter().as_ref(), "txt");
    } else {
        panic!("Expected File type");
    }
    assert_eq!(def.default_value().as_ref(), "");
}

#[test]
fn test_basic_definition_file_with_default() {
    // Why: Test basic file definition creation and properties with a default value.
    let file_def = FileDefinition::new("txt");
    let def =
        BasicDefinition::new_file_with_default("A file property", file_def.clone(), "test.txt");

    // Check the various properties of the file definition.
    assert_eq!(def.description().as_ref(), "A file property");
    if let BasicDefinitionType::File(f) = def.type_definition() {
        assert_eq!(f.extension_filter().as_ref(), "txt");
    } else {
        panic!("Expected File type");
    }
    assert_eq!(def.default_value().as_ref(), "test.txt");
}

#[test]
fn test_basic_definition_choice() {
    // Why: Test basic choice definition creation and properties.
    let choice_def = ChoiceDefinition::new(vec!["A".into(), "B".into()]);
    let def = BasicDefinition::new_choice("A choice property", choice_def.clone());

    // Check the various properties of the choice definition.
    assert_eq!(def.description().as_ref(), "A choice property");
    if let BasicDefinitionType::Choice(c) = def.type_definition() {
        assert_eq!(c.choices().len(), 2);
        assert_eq!(c.choices()[0].as_ref(), "A");
        assert_eq!(c.choices()[1].as_ref(), "B");
    } else {
        panic!("Expected Choice type");
    }
    assert_eq!(def.default_value().as_ref(), "");
}

#[test]
fn test_basic_definition_choice_with_default() {
    // Why: Test basic choice definition creation and properties with a default value.
    let choice_def = ChoiceDefinition::new(vec!["A".into(), "B".into()]);
    let def =
        BasicDefinition::new_choice_with_default("A choice property", choice_def.clone(), "A");

    // Check the various properties of the choice definition.
    assert_eq!(def.description().as_ref(), "A choice property");
    if let BasicDefinitionType::Choice(c) = def.type_definition() {
        assert_eq!(c.choices().len(), 2);
        assert_eq!(c.choices()[0].as_ref(), "A");
        assert_eq!(c.choices()[1].as_ref(), "B");
    } else {
        panic!("Expected Choice type");
    }
    assert_eq!(def.default_value().as_ref(), "A");
}

#[test]
fn test_table_definition() {
    // Why: Test table definition creation and properties.
    let table_def = TableDefinition::new(
        "A table",
        vec![
            (store_key!("col1"), BasicDefinition::new_string("Column 1")),
            (
                store_key!("col2"),
                BasicDefinition::new_number_with_default("Column 2", "test"),
            ),
        ],
    );

    // Check the various properties of the table definition.
    assert_eq!(table_def.description().as_ref(), "A table");
    assert_eq!(table_def.count(), 2);
    assert!(table_def.contains_key(store_key!("col1")));
    assert!(table_def.contains_key(store_key!("col2")));
    assert!(!table_def.contains_key(store_key!("col3")));

    let col1 = table_def.get(store_key!("col1")).unwrap();
    assert_eq!(col1.description().as_ref(), "Column 1");
    assert_eq!(col1.default_value().as_ref(), "");

    let col2 = table_def.get(store_key!("col2")).unwrap();
    assert_eq!(col2.description().as_ref(), "Column 2");
    assert_eq!(col2.default_value().as_ref(), "test");
}

#[test]
fn test_struct_all_basic_definition() {
    // Why: Test struct definition creation and properties.
    let struct_def = StructDefinition::new(
        "A struct",
        vec![
            (store_key!("field1"), BasicDefinition::new_string("Field 1")),
            (store_key!("field2"), BasicDefinition::new_string("Field 2")),
        ],
    );

    // Check the various properties of the struct definition.
    assert_eq!(struct_def.description().as_ref(), "A struct");
    assert_eq!(struct_def.count(), 2);

    let mut keys: Vec<String> = struct_def.keys().map(|k| k.as_ref().to_string()).collect();
    keys.sort();
    assert_eq!(keys, vec!["field1", "field2"]);

    let item1 = struct_def.get(&store_key!("field1")).unwrap();
    if let StructItemDefinition::Basic(def) = item1 {
        assert_eq!(def.description().as_ref(), "Field 1");
        assert!(matches!(def.type_definition(), BasicDefinitionType::String));
        assert_eq!(def.default_value().as_ref(), "");
    } else {
        panic!(
            "Expected item1 to be StructItemDefinition::Basic, but got {:?}",
            item1
        );
    }

    let item2 = struct_def.get(&store_key!("field2")).unwrap();
    if let StructItemDefinition::Basic(def) = item2 {
        assert_eq!(def.description().as_ref(), "Field 2");
        assert!(matches!(def.type_definition(), BasicDefinitionType::String));
        assert_eq!(def.default_value().as_ref(), "");
    } else {
        panic!(
            "Expected item1 to be StructItemDefinition::Table, but got {:?}",
            item1
        );
    }
}

#[test]
fn test_struct_all_table_definition() {
    // Why: Test struct definition creation and properties.
    let struct_def = StructDefinition::new(
        "A struct",
        vec![
            (
                store_key!("field1"),
                TableDefinition::new("Table field 1", Vec::<(StoreKey, BasicDefinition)>::new()),
            ),
            (
                store_key!("field2"),
                TableDefinition::new("Table field 2", Vec::<(StoreKey, BasicDefinition)>::new()),
            ),
        ],
    );

    // Check the various properties of the struct definition.
    assert_eq!(struct_def.description().as_ref(), "A struct");
    assert_eq!(struct_def.count(), 2);

    let mut keys: Vec<String> = struct_def.keys().map(|k| k.as_ref().to_string()).collect();
    keys.sort();
    assert_eq!(keys, vec!["field1", "field2"]);

    let item1 = struct_def.get(&store_key!("field1")).unwrap();
    if let StructItemDefinition::Table(def) = item1 {
        assert_eq!(def.description().as_ref(), "Table field 1");
        assert_eq!(def.count(), 0);
    } else {
        panic!(
            "Expected item1 to be StructItemDefinition::Basic, but got {:?}",
            item1
        );
    }

    let item2 = struct_def.get(&store_key!("field2")).unwrap();
    if let StructItemDefinition::Table(def) = item2 {
        assert_eq!(def.description().as_ref(), "Table field 2");
        assert_eq!(def.count(), 0);
    } else {
        panic!(
            "Expected item1 to be StructItemDefinition::Table, but got {:?}",
            item1
        );
    }
}

#[test]
fn test_struct_mixed_definition() {
    // Why: Test struct definition creation and properties.
    let struct_def = StructDefinition::new(
        "A struct",
        vec![
            (
                store_key!("field1"),
                StructItemDefinition::Basic(BasicDefinition::new_string("Field 1")),
            ),
            (
                store_key!("field2"),
                StructItemDefinition::Table(TableDefinition::new(
                    "Table field",
                    Vec::<(StoreKey, BasicDefinition)>::new(),
                )),
            ),
        ],
    );

    // Check the various properties of the struct definition.
    assert_eq!(struct_def.description().as_ref(), "A struct");
    assert_eq!(struct_def.count(), 2);

    let mut keys: Vec<String> = struct_def.keys().map(|k| k.as_ref().to_string()).collect();
    keys.sort();
    assert_eq!(keys, vec!["field1", "field2"]);

    let item1 = struct_def.get(&store_key!("field1")).unwrap();
    if let StructItemDefinition::Basic(def) = item1 {
        assert_eq!(def.description().as_ref(), "Field 1");
        assert!(matches!(def.type_definition(), BasicDefinitionType::String));
        assert_eq!(def.default_value().as_ref(), "");
    } else {
        panic!(
            "Expected item1 to be StructItemDefinition::Basic, but got {:?}",
            item1
        );
    }

    let item2 = struct_def.get(&store_key!("field2")).unwrap();
    if let StructItemDefinition::Table(def) = item2 {
        assert_eq!(def.description().as_ref(), "Table field");
        assert_eq!(def.count(), 0);
    } else {
        panic!(
            "Expected item1 to be StructItemDefinition::Table, but got {:?}",
            item1
        );
    }
}

#[test]
fn test_map_definition() {
    // Why: Test map definition creation and properties.
    let struct_def = StructDefinition::new(
        "Item struct",
        Vec::<(StoreKey, StructItemDefinition)>::new(),
    );
    let map_def = MapDefinition::new("A map", struct_def.clone());

    // Check the various properties of the map definition.
    assert_eq!(map_def.description().as_ref(), "A map");
    assert_eq!(map_def.item_type().description().as_ref(), "Item struct");

    let item_def = map_def.item_type();
    assert_eq!(item_def.description().as_ref(), "Item struct");
    assert_eq!(item_def.count(), 0);
}

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
