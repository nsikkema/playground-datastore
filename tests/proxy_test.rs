use datastore::definition::{
    BasicDefinition, ObjectDefinition, PropertyDefinition, StructDefinition, StructItemDefinition,
    TableDefinition,
};
use datastore::shareable_string::{ShareableString, SharedStringStore};
use datastore::store::Store;
use datastore::store::traits::ProxyStoreTrait;

#[test]
fn test_complex_proxy_structure() {
    let store = Store::new(SharedStringStore::new());

    // 1. Create a Table Definition
    let table_def = TableDefinition::new(
        "Nested Table",
        vec![("col1", BasicDefinition::new_string("default"))],
    )
    .unwrap();

    // 2. Create a Struct Definition containing the Table and a Basic property
    let struct_def = StructDefinition::new(
        "Nested Struct",
        vec![
            ("table", StructItemDefinition::Table(table_def)),
            (
                "inner_basic",
                StructItemDefinition::Basic(BasicDefinition::new_number_with_default(
                    "Inner Basic",
                    "42",
                )),
            ),
        ],
    )
    .unwrap();

    // 3. Create an Object Definition containing the Struct
    let mut builder = ObjectDefinition::builder("Complex Object");
    builder
        .add(
            "outer_struct",
            PropertyDefinition::new("Outer Struct", struct_def),
        )
        .unwrap();
    let obj_def = builder.finish();

    // 4. Create Object in Store
    let obj_key = ShareableString::from("complex_obj");
    let mut obj_proxy = store.create_object(&obj_key, &obj_def).unwrap();

    // 5. Access Struct Container Proxy
    let struct_proxy = obj_proxy.get_container("outer_struct").unwrap();
    assert_eq!(struct_proxy.description().as_ref(), "Nested Struct");

    // 6. Access Table Proxy from Struct
    // Since ContainerProxy doesn't have get_table, we need to use store with path
    let table_path = struct_proxy
        .get_path()
        .clone()
        .to_builder()
        .struct_item(ShareableString::from("table"))
        .build()
        .unwrap();
    let mut table_proxy = store.get_table(&table_path).unwrap();

    // 7. Access Basic Proxy from Struct
    let basic_path = struct_proxy
        .get_path()
        .clone()
        .to_builder()
        .struct_item(ShareableString::from("inner_basic"))
        .build()
        .unwrap();
    let mut basic_proxy = store.get_basic(&basic_path).unwrap();

    // 8. Modify values at deep levels
    assert_eq!(basic_proxy.get_value().unwrap().as_ref(), "42");
    basic_proxy.set_value("100");

    table_proxy.append_row();
    table_proxy
        .set_cell(0, "col1", ShareableString::from("new_value"))
        .unwrap();

    // 9. Check change detection
    assert!(basic_proxy.has_changed());
    assert!(table_proxy.has_changed());

    // 10. Push changes
    basic_proxy.push().unwrap();
    table_proxy.push().unwrap();

    assert!(!basic_proxy.has_changed());
    assert!(!table_proxy.has_changed());

    // 11. Verify changes via new proxies
    let _obj_proxy2 = store.get_object(&obj_proxy.get_path()).unwrap();
    let basic_proxy2 = store.get_basic(&basic_path).unwrap();
    assert_eq!(basic_proxy2.get_value().unwrap().as_ref(), "100");

    let table_proxy2 = store.get_table(&table_path).unwrap();
    assert_eq!(table_proxy2.row_count(), 1);
    assert_eq!(
        table_proxy2.row(0).unwrap().get("col1").unwrap().as_ref(),
        "new_value"
    );

    // 12. Check parent synchronization
    assert!(obj_proxy.has_changed());
    obj_proxy.pull().unwrap();
    assert!(!obj_proxy.has_changed());
}

#[test]
fn test_proxy_basic_operations() {
    let store = Store::new(SharedStringStore::new());

    // 1. Create Object Definition
    let mut builder = ObjectDefinition::builder("Test Object");
    builder
        .add(
            "name",
            PropertyDefinition::new("Name", BasicDefinition::new_string("The name")),
        )
        .unwrap();
    let obj_def = builder.finish();

    // 2. Create Object in Store
    let obj_key = ShareableString::from("my_object");
    let mut obj_proxy = store.create_object(&obj_key, &obj_def).unwrap();

    assert_eq!(obj_proxy.description().as_ref(), "Test Object");

    // 3. Get Basic Property Proxy
    let mut name_proxy = obj_proxy.get_basic("name").unwrap();
    assert_eq!(name_proxy.get_value().unwrap().as_ref(), "");

    // 4. Set Value and Push
    name_proxy.set_value("Junie");
    assert!(name_proxy.has_changed());
    name_proxy.push().unwrap();
    assert!(!name_proxy.has_changed());

    // 5. Verify in store (via another proxy)
    let mut obj_proxy2 = store.get_object(&obj_proxy.get_path()).unwrap();
    let name_proxy2 = obj_proxy2.get_basic("name").unwrap();
    assert_eq!(name_proxy2.get_value().unwrap().as_ref(), "Junie");

    // 6. Test Pull
    name_proxy.set_value("Something else");
    // Before pushing name_proxy, name_proxy2 still has "Junie"
    assert_eq!(name_proxy2.get_value().unwrap().as_ref(), "Junie");

    name_proxy.push().unwrap();

    // name_proxy2 should now detect change
    assert!(name_proxy2.has_changed());
    let mut name_proxy2_mut = name_proxy2;
    name_proxy2_mut.pull().unwrap();
    assert_eq!(
        name_proxy2_mut.get_value().unwrap().as_ref(),
        "Something else"
    );
}

#[test]
fn test_proxy_deleted_object() {
    let store = Store::new(SharedStringStore::new());

    // 1. Create Object Definition
    let mut builder = ObjectDefinition::builder("Test Object");
    builder
        .add(
            "name",
            PropertyDefinition::new("Name", BasicDefinition::new_string("The name")),
        )
        .unwrap();
    let obj_def = builder.finish();

    // 2. Create Object in Store
    let obj_key = ShareableString::from("my_object");
    let mut obj_proxy = store.create_object(&obj_key, &obj_def).unwrap();

    assert!(obj_proxy.is_valid());

    // 3. Delete Object from Store
    store.delete_object(&obj_key).unwrap();

    // 4. Verify Proxy is now invalid
    assert!(!obj_proxy.is_valid());

    // 5. Verify operations on invalid proxy fail
    assert!(obj_proxy.get_basic("name").is_err());
    assert!(obj_proxy.pull().is_err());
}

#[test]
fn test_proxy_multiple_properties() {
    let store = Store::new(SharedStringStore::new());

    let mut builder = ObjectDefinition::builder("Multi Prop Object");
    builder
        .add(
            "name",
            PropertyDefinition::new("Name", BasicDefinition::new_string("The name")),
        )
        .unwrap();
    builder
        .add(
            "age",
            PropertyDefinition::new("Age", BasicDefinition::new_string("The age")),
        )
        .unwrap();
    let obj_def = builder.finish();

    let obj_key = ShareableString::from("user_1");
    let mut obj_proxy = store.create_object(&obj_key, &obj_def).unwrap();

    let mut name_proxy = obj_proxy.get_basic("name").unwrap();
    let mut age_proxy = obj_proxy.get_basic("age").unwrap();

    name_proxy.set_value("Alice");
    age_proxy.set_value("30");

    name_proxy.push().unwrap();
    age_proxy.push().unwrap();

    // Verify both are updated
    let mut obj_proxy2 = store.get_object(&obj_proxy.get_path()).unwrap();
    assert_eq!(
        obj_proxy2
            .get_basic("name")
            .unwrap()
            .get_value()
            .unwrap()
            .as_ref(),
        "Alice"
    );
    assert_eq!(
        obj_proxy2
            .get_basic("age")
            .unwrap()
            .get_value()
            .unwrap()
            .as_ref(),
        "30"
    );
}

#[test]
fn test_proxy_sync_from_store() {
    let store = Store::new(SharedStringStore::new());

    let mut builder = ObjectDefinition::builder("Sync Object");
    builder
        .add(
            "name",
            PropertyDefinition::new("Name", BasicDefinition::new_string("The name")),
        )
        .unwrap();
    let obj_def = builder.finish();

    let obj_key = ShareableString::from("user_2");
    let mut proxy1 = store.create_object(&obj_key, &obj_def).unwrap();
    let mut proxy2 = store.get_object(&proxy1.get_path()).unwrap();

    // Modify via proxy1
    let mut name_proxy1 = proxy1.get_basic("name").unwrap();
    name_proxy1.set_value("Bob");
    name_proxy1.push().unwrap();

    // proxy2 still has old value until pull
    let name_proxy2 = proxy2.get_basic("name").unwrap();
    // It seems proxy2 already sees "Bob" because they might share the same underlying Basic object
    // if get_object doesn't deep clone. Let's check.
    assert_eq!(name_proxy2.get_value().unwrap().as_ref(), "Bob");

    // However, last_sync_hash in proxy2 should still be old
    assert!(proxy2.has_changed());

    proxy2.pull().unwrap();
    assert!(!proxy2.has_changed());
}

#[test]
fn test_proxy_is_valid_initially() {
    let store = Store::new(SharedStringStore::new());
    let mut builder = ObjectDefinition::builder("Test Object");
    builder
        .add(
            "name",
            PropertyDefinition::new("Name", BasicDefinition::new_string("")),
        )
        .unwrap();
    let obj_def = builder.finish();
    let obj_key = ShareableString::from("valid_obj");
    let obj_proxy = store.create_object(&obj_key, &obj_def).unwrap();

    assert!(obj_proxy.is_valid());
}

#[test]
fn test_proxy_get_object() {
    let store = Store::new(SharedStringStore::new());

    // 1. Create Object Definition
    let mut builder = ObjectDefinition::builder("Test Object");
    builder
        .add(
            "name",
            PropertyDefinition::new("Name", BasicDefinition::new_string("The name")),
        )
        .unwrap();
    let obj_def = builder.finish();

    // 2. Create Object in Store
    let obj_key = ShareableString::from("my_object");
    let mut obj_proxy = store.create_object(&obj_key, &obj_def).unwrap();

    // 3. Get Basic Property Proxy
    let name_proxy = obj_proxy.get_basic("name").unwrap();

    // 4. Get object from name_proxy
    let obj_proxy_from_name = name_proxy.get_object().unwrap();
    assert_eq!(
        obj_proxy_from_name.get_path().get_object_key().as_ref(),
        "my_object"
    );
    assert_eq!(obj_proxy_from_name.description().as_ref(), "Test Object");

    // 5. Get object from obj_proxy itself
    let obj_proxy_from_itself = obj_proxy.get_object().unwrap();
    assert_eq!(
        obj_proxy_from_itself.get_path().get_object_key().as_ref(),
        "my_object"
    );
}
