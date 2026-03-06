use datastore::definition::{
    BasicDefinition, BasicDefinitionType, ChoiceDefinition, FileDefinition, MapDefinition,
    ObjectDefinition, PropertyDefinition, StructDefinition, StructItemDefinition, TableDefinition,
};

#[test]
fn test_basic_definition_string() {
    let def = BasicDefinition::new_string("A string property");
    assert_eq!(def.description().as_ref(), "A string property");
    assert!(matches!(def.type_definition(), BasicDefinitionType::String));
    assert_eq!(def.default_value().as_ref(), "");
}

#[test]
fn test_basic_definition_string_with_default() {
    let def = BasicDefinition::new_string_with_default("A string property", "default value");
    assert_eq!(def.description().as_ref(), "A string property");
    assert!(matches!(def.type_definition(), BasicDefinitionType::String));
    assert_eq!(def.default_value().as_ref(), "default value");
}

#[test]
fn test_basic_definition_number() {
    let def = BasicDefinition::new_number("A number property");
    assert_eq!(def.description().as_ref(), "A number property");
    assert!(matches!(def.type_definition(), BasicDefinitionType::Number));
}

#[test]
fn test_basic_definition_file() {
    let file_def = FileDefinition::new("txt");
    let def = BasicDefinition::new_file("A file property", file_def.clone());
    assert_eq!(def.description().as_ref(), "A file property");
    if let BasicDefinitionType::File(f) = def.type_definition() {
        assert_eq!(f.extension_filter().as_ref(), "txt");
    } else {
        panic!("Expected File type");
    }
}

#[test]
fn test_basic_definition_choice() {
    let choice_def = ChoiceDefinition::new(vec!["A".into(), "B".into()]);
    let def = BasicDefinition::new_choice("A choice property", choice_def.clone());
    assert_eq!(def.description().as_ref(), "A choice property");
    if let BasicDefinitionType::Choice(c) = def.type_definition() {
        assert_eq!(c.choices().len(), 2);
        assert_eq!(c.choices()[0].as_ref(), "A");
        assert_eq!(c.choices()[1].as_ref(), "B");
    } else {
        panic!("Expected Choice type");
    }
}

#[test]
fn test_table_definition() {
    let table_def_res = TableDefinition::new(
        "A table",
        vec![
            ("col1", BasicDefinition::new_string("Column 1")),
            ("col2", BasicDefinition::new_number("Column 2")),
        ],
    );
    let table_def = table_def_res.expect("Valid keys");

    assert_eq!(table_def.description().as_ref(), "A table");
    assert_eq!(table_def.count(), 2);
    assert!(table_def.contains_key("col1"));
    assert!(table_def.contains_key("col2"));
    assert!(!table_def.contains_key("col3"));

    let col1 = table_def.get("col1").unwrap();
    assert_eq!(col1.description().as_ref(), "Column 1");
}

#[test]
fn test_struct_definition() {
    let struct_def_res = StructDefinition::new(
        "A struct",
        vec![
            (
                "field1",
                StructItemDefinition::Basic(BasicDefinition::new_string("Field 1")),
            ),
            (
                "field2",
                StructItemDefinition::Table(
                    TableDefinition::new("Table field", Vec::<(String, BasicDefinition)>::new())
                        .expect("Valid keys"),
                ),
            ),
        ],
    );
    let struct_def = struct_def_res.expect("Valid keys");

    assert_eq!(struct_def.description().as_ref(), "A struct");
    assert_eq!(struct_def.count(), 2);

    let mut keys: Vec<String> = struct_def
        .iter()
        .map(|(k, _)| k.as_ref().to_string())
        .collect();
    keys.sort();
    assert_eq!(keys, vec!["field1", "field2"]);
}

#[test]
fn test_map_definition() {
    let struct_def =
        StructDefinition::new("Item struct", Vec::<(String, StructItemDefinition)>::new())
            .expect("Valid keys");
    let map_def = MapDefinition::new("A map", struct_def.clone());

    assert_eq!(map_def.description().as_ref(), "A map");
    assert_eq!(map_def.item_type().description().as_ref(), "Item struct");
}

#[test]
fn test_property_definition() {
    let basic_prop = PropertyDefinition::new("Basic Prop", BasicDefinition::new_string("String"));
    assert_eq!(basic_prop.description().as_ref(), "Basic Prop");
    assert!(matches!(
        basic_prop.item_type(),
        datastore::definition::PropertyDefinitionType::Basic(_)
    ));

    let struct_def = StructDefinition::new("Struct", Vec::<(String, StructItemDefinition)>::new())
        .expect("Valid keys");
    let struct_prop = PropertyDefinition::new("Struct Prop", struct_def);
    assert!(matches!(
        struct_prop.item_type(),
        datastore::definition::PropertyDefinitionType::Struct(_)
    ));

    let table_def =
        TableDefinition::new("Table", Vec::<(String, BasicDefinition)>::new()).expect("Valid keys");
    let table_prop = PropertyDefinition::new("Table Prop", table_def);
    assert!(matches!(
        table_prop.item_type(),
        datastore::definition::PropertyDefinitionType::Table(_)
    ));

    let map_def = MapDefinition::new(
        "Map",
        StructDefinition::new("Item", Vec::<(String, StructItemDefinition)>::new())
            .expect("Valid keys"),
    );
    let map_prop = PropertyDefinition::new("Map Prop", map_def);
    assert!(matches!(
        map_prop.item_type(),
        datastore::definition::PropertyDefinitionType::Map(_)
    ));
}

#[test]
fn test_object_definition_basic() {
    let mut builder = ObjectDefinition::builder("Test Object");
    builder
        .add(
            "prop1",
            PropertyDefinition::new("P1", BasicDefinition::new_string("D1")),
        )
        .expect("Valid key");
    let obj_def = builder.finish();

    assert_eq!(obj_def.description().as_ref(), "Test Object");
    assert_eq!(obj_def.count(), 1);
    assert!(obj_def.contains_key("prop1"));
    assert!(obj_def.contains_key_str("prop1"));
}
