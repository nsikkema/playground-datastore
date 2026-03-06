use datastore::definition::{
    BasicDefinition, ChoiceDefinition, FileDefinition, MapDefinition, ObjectDefinition,
    PropertyDefinition, StructDefinition, StructItemDefinition, TableDefinition,
};
use datastore::shareable_string::ShareableString;
use datastore::store::traits::ProxyStoreTrait;
use datastore::store::{Store, StorePath};
use std::fs;

#[test]
fn test_save_load_file() {
    let store = Store::new();
    let obj_key = ShareableString::from("my_object");
    let def = ObjectDefinition::builder("My Test Object")
        .with(
            "prop1",
            PropertyDefinition::new("Property 1", BasicDefinition::new_string("A string")),
        )
        .unwrap()
        .finish();

    let _proxy = store.create_object(&obj_key, &def).unwrap();
    let prop_path = StorePath::builder(obj_key.clone())
        .property(ShareableString::from("prop1"))
        .build();

    {
        let mut basic = store.get_basic(&prop_path).unwrap();
        basic.set_value("Saved data");
        basic.push().unwrap();
    }

    let original_hash = store.get_blake3_hash();

    let temp_file = "test_store_save_load.json";
    store
        .save_to_file(temp_file)
        .expect("Failed to save to file");

    // Verify JSON content doesn't have redundant definitions
    let json_content = fs::read_to_string(temp_file).unwrap();
    // In "items", we should not find "definition"
    // The items are under "my_object" (the 2nd element of the tuple in "objects")
    // The structure is now: "objects": { "my_object": [ { "description": ... }, { "items": { ... } } ] }
    assert!(
        !json_content.contains(r#""definition": {"#),
        "JSON still contains redundant definitions: {}",
        json_content
    );
    assert!(
        !json_content.contains(r#""string_store""#),
        "JSON still contains string_store: {}",
        json_content
    );

    let loaded_store = Store::load_from_file(temp_file).expect("Failed to load from file");

    // Check data consistency
    let loaded_basic = loaded_store
        .get_basic(&prop_path)
        .expect("Could not find property in loaded store");
    assert_eq!(loaded_basic.get_value().unwrap().as_str(), "Saved data");

    // Check hash consistency
    let loaded_hash = loaded_store.get_blake3_hash();
    assert_eq!(original_hash, loaded_hash);

    // Cleanup
    fs::remove_file(temp_file).ok();
}

#[test]
fn test_save_load_comprehensive() {
    let store = Store::new();

    // 1. Define nested structures
    let table_def = TableDefinition::new(
        "Table",
        vec![
            ("col_str", BasicDefinition::new_string("String col")),
            ("col_num", BasicDefinition::new_number("Number col")),
        ],
    )
    .unwrap();

    let struct_def = StructDefinition::new(
        "Struct",
        vec![
            (
                "s_basic",
                StructItemDefinition::Basic(BasicDefinition::new_string("Struct basic")),
            ),
            ("s_table", StructItemDefinition::Table(table_def.clone())),
        ],
    )
    .unwrap();

    let map_def = MapDefinition::new("Map", struct_def.clone());

    // 2. Define Object with all types
    let obj_def = ObjectDefinition::builder("Comprehensive Object")
        .with(
            "p_string",
            PropertyDefinition::new("String", BasicDefinition::new_string("S")),
        )
        .unwrap()
        .with(
            "p_number",
            PropertyDefinition::new("Number", BasicDefinition::new_number("N")),
        )
        .unwrap()
        .with(
            "p_file",
            PropertyDefinition::new(
                "File",
                BasicDefinition::new_file("F", FileDefinition::new("*.txt")),
            ),
        )
        .unwrap()
        .with(
            "p_choice",
            PropertyDefinition::new(
                "Choice",
                BasicDefinition::new_choice(
                    "C",
                    ChoiceDefinition::new(vec!["A".into(), "B".into()]),
                ),
            ),
        )
        .unwrap()
        .with("p_struct", PropertyDefinition::new("Struct", struct_def))
        .unwrap()
        .with("p_table", PropertyDefinition::new("Table", table_def))
        .unwrap()
        .with("p_map", PropertyDefinition::new("Map", map_def))
        .unwrap()
        .finish();

    let obj_key = ShareableString::from("comp_obj");
    let mut obj_proxy = store.create_object(&obj_key, &obj_def).unwrap();

    // 3. Populate data
    // p_string
    {
        let mut p = obj_proxy.get_basic("p_string").unwrap();
        p.set_value("Hello String");
        p.push().unwrap();
    }
    // p_number
    {
        let mut p = obj_proxy.get_basic("p_number").unwrap();
        p.set_value("12345");
        p.push().unwrap();
    }
    // p_file
    {
        let mut p = obj_proxy.get_basic("p_file").unwrap();
        p.set_value("test.txt");
        p.push().unwrap();
    }
    // p_choice
    {
        let mut p = obj_proxy.get_basic("p_choice").unwrap();
        p.set_value("B");
        p.push().unwrap();
    }
    // p_struct -> s_basic
    {
        let path = obj_proxy
            .get_container("p_struct")
            .unwrap()
            .get_path()
            .clone()
            .to_builder()
            .struct_item(ShareableString::from("s_basic"))
            .build()
            .unwrap();
        let mut p = store.get_basic(&path).unwrap();
        p.set_value("Struct Value");
        p.push().unwrap();
    }
    // p_struct -> s_table
    {
        let path = obj_proxy
            .get_container("p_struct")
            .unwrap()
            .get_path()
            .clone()
            .to_builder()
            .struct_item(ShareableString::from("s_table"))
            .build()
            .unwrap();
        let mut p = store.get_table(&path).unwrap();
        p.append_row();
        p.set_cell(0, "col_str", "Row 0".into()).unwrap();
        p.set_cell(0, "col_num", "10".into()).unwrap();
        p.push().unwrap();
    }
    // p_table
    {
        let mut p = obj_proxy.get_table("p_table").unwrap();
        p.append_row();
        p.set_cell(0, "col_str", "Table Row".into()).unwrap();
        p.push().unwrap();
    }
    // p_map
    {
        let map_container = obj_proxy.get_container("p_map").unwrap();
        let item_key = ShareableString::from("entry_1");
        let entry_proxy = map_container.insert_map_entry(item_key).unwrap();
        let path = entry_proxy.get_path();
        // Entry in map is a Struct. Let's set its s_basic.
        let basic_path = path
            .clone()
            .to_builder()
            .struct_item(ShareableString::from("s_basic"))
            .build()
            .unwrap();
        let mut p = store.get_basic(&basic_path).unwrap();
        p.set_value("Map Struct Value");
        p.push().unwrap();
    }

    let original_hash = store.get_blake3_hash();
    let temp_file = "test_comprehensive_save_load.json";
    store.save_to_file(temp_file).expect("Failed to save");

    // 4. Load and Verify
    let loaded_store = Store::load_from_file(temp_file).expect("Failed to load");

    let mut loaded_obj = loaded_store
        .get_object(&StorePath::builder(obj_key.clone()).build())
        .unwrap();

    assert_eq!(
        loaded_obj
            .get_basic("p_string")
            .unwrap()
            .get_value()
            .unwrap()
            .as_ref(),
        "Hello String"
    );
    assert_eq!(
        loaded_obj
            .get_basic("p_number")
            .unwrap()
            .get_value()
            .unwrap()
            .as_ref(),
        "12345"
    );
    assert_eq!(
        loaded_obj
            .get_basic("p_file")
            .unwrap()
            .get_value()
            .unwrap()
            .as_ref(),
        "test.txt"
    );
    assert_eq!(
        loaded_obj
            .get_basic("p_choice")
            .unwrap()
            .get_value()
            .unwrap()
            .as_ref(),
        "B"
    );

    // Verify Struct
    let s_basic_path = StorePath::builder(obj_key.clone())
        .property("p_struct".into())
        .struct_item("s_basic".into())
        .build();
    assert_eq!(
        loaded_store
            .get_basic(&s_basic_path)
            .unwrap()
            .get_value()
            .unwrap()
            .as_ref(),
        "Struct Value"
    );

    // Verify Map
    let m_basic_path = StorePath::builder(obj_key.clone())
        .property("p_map".into())
        .map_key("entry_1".into())
        .struct_item("s_basic".into())
        .build();
    assert_eq!(
        loaded_store
            .get_basic(&m_basic_path)
            .unwrap()
            .get_value()
            .unwrap()
            .as_ref(),
        "Map Struct Value"
    );

    // Verify Table
    let table_path = StorePath::builder(obj_key.clone())
        .property("p_table".into())
        .build();
    let loaded_table = loaded_store.get_table(&table_path).unwrap();
    assert_eq!(loaded_table.row_count(), 1);
    assert_eq!(
        loaded_table
            .row(0)
            .unwrap()
            .get("col_str")
            .unwrap()
            .as_ref(),
        "Table Row"
    );

    let loaded_hash = loaded_store.get_blake3_hash();
    assert_eq!(original_hash, loaded_hash, "Hashes should match after load");

    fs::remove_file(temp_file).ok();
}

#[test]
fn test_launder_consistency_after_load() {
    let store = Store::new();
    let obj_key = ShareableString::from("test_obj");
    let def = ObjectDefinition::builder("Test").finish();
    store.create_object(&obj_key, &def).unwrap();

    let temp_file = "test_launder.json";
    store.save_to_file(temp_file).unwrap();

    let loaded_store = Store::load_from_file(temp_file).unwrap();

    let s1 = ShareableString::from("hello");
    let s2 = ShareableString::from("hello");

    let laundered1 = loaded_store.launder(s1);
    let laundered2 = loaded_store.launder(s2);

    assert!(laundered1.ptr_eq(&laundered2));

    fs::remove_file(temp_file).ok();
}
