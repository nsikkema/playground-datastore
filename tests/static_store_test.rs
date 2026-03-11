use datastore::definition::{BasicDefinition, ObjectDefinition, PropertyDefinition};
use datastore::shareable_string::SharedStringStore;
use datastore::store::{ProxyStoreTrait, Store};
use datastore::store_key;

#[test]
fn test_store_to_static() {
    let store = Store::new(SharedStringStore::new());
    let obj_key = store_key!("my_object");
    let def = ObjectDefinition::builder("My Test Object")
        .with_inserted(
            store_key!("prop1"),
            PropertyDefinition::new("Property 1", BasicDefinition::new_string("A string")),
        )
        .finish();

    let _proxy = store.create_object(obj_key.clone(), &def).unwrap();
    let prop_path = datastore::StorePath::builder(obj_key)
        .property("prop1")
        .build();

    {
        let mut basic = store.basic(&prop_path).unwrap();
        basic.set_value("Static data");
        basic.push().unwrap();
    }

    let static_store = store.to_static();

    // Verify hash consistency
    assert_eq!(store.get_blake3_hash(), static_store.get_blake3_hash());

    // Verify data access
    let obj = static_store
        .get("my_object")
        .expect("Object not found in static store");
    let prop = obj
        .get("prop1")
        .expect("Property not found in static store");

    if let Some(basic) = prop.get_basic() {
        assert_eq!(basic.value().as_str(), "Static data");
    } else {
        panic!("Property is not a StaticBasic");
    }

    // Verify tree print doesn't crash
    static_store.tree_print();
}

#[test]
fn test_static_to_store_roundtrip() {
    let store = Store::new(SharedStringStore::new());
    let obj_key = store_key!("my_object");
    let def = ObjectDefinition::builder("My Test Object")
        .with_inserted(
            store_key!("prop1"),
            PropertyDefinition::new("Property 1", BasicDefinition::new_string("A string")),
        )
        .finish();

    let _proxy = store.create_object(obj_key.clone(), &def).unwrap();
    let prop_path = datastore::StorePath::builder(obj_key)
        .property("prop1")
        .build();

    {
        let mut basic = store.basic(&prop_path).unwrap();
        basic.set_value("Roundtrip data");
        basic.push().unwrap();
    }

    let static_store = store.to_static();
    let restored_store = Store::new_from_static(&static_store);

    // Verify hash consistency
    assert_eq!(store.get_blake3_hash(), restored_store.get_blake3_hash());
    assert_eq!(
        static_store.get_blake3_hash(),
        restored_store.get_blake3_hash()
    );

    // Verify data access in restored store
    let basic_proxy = restored_store.basic(&prop_path).unwrap();
    assert_eq!(basic_proxy.value().as_str(), "Roundtrip data");
}

#[test]
fn test_update_from_static() {
    let string_store = SharedStringStore::new();
    let store = Store::new(string_store.clone());
    let obj_key = store_key!("my_object");
    let def = ObjectDefinition::builder("My Test Object")
        .with_inserted(
            store_key!("prop1"),
            PropertyDefinition::new("Property 1", BasicDefinition::new_string("Initial")),
        )
        .finish();

    let mut proxy = store.create_object(obj_key.clone(), &def).unwrap();
    {
        let mut basic = proxy.basic("prop1").unwrap();
        basic.set_value("Initial");
        basic.push().unwrap();
    }
    proxy.sync().unwrap();

    // Create static store from initial state
    let static_store = store.to_static();

    // Modify the static store (in a real scenario this might come from another source)
    // Here we'll just modify the store and create a new static one to simulate an updated version.
    {
        let mut basic = store
            .basic(
                &datastore::StorePath::builder(obj_key.clone())
                    .property("prop1")
                    .build(),
            )
            .unwrap();
        basic.set_value("Updated");
        basic.push().unwrap();
    }
    let updated_static_store = store.to_static();

    // Reset store to initial state (using the first static store)
    store.update_from_static(&static_store);
    proxy.sync().unwrap();
    assert_eq!(proxy.basic("prop1").unwrap().value().as_str(), "Initial");

    // Now update from the "updated" static store
    store.update_from_static(&updated_static_store);
    proxy.sync().unwrap();

    // Verify that the proxy still works and reflects the update
    assert_eq!(proxy.basic("prop1").unwrap().value().as_str(), "Updated");
    assert_eq!(
        store.get_blake3_hash(),
        updated_static_store.get_blake3_hash()
    );
}

#[test]
fn test_update_from_static_definition_mismatch() {
    let store = Store::new(SharedStringStore::new());
    let obj_key = store_key!("my_object");

    let def1 = ObjectDefinition::builder("Def 1")
        .with_inserted(
            store_key!("prop1"),
            PropertyDefinition::new("Property 1", BasicDefinition::new_string("Initial")),
        )
        .finish();

    let _proxy1 = store.create_object(obj_key.clone(), &def1).unwrap();

    let def2 = ObjectDefinition::builder("Def 2")
        .with_inserted(
            store_key!("prop2"),
            PropertyDefinition::new("Property 2", BasicDefinition::new_number("0")),
        )
        .finish();

    let other_store = Store::new(SharedStringStore::new());
    other_store.create_object(obj_key.clone(), &def2).unwrap();
    let static_store2 = other_store.to_static();

    // Update store with a static store that has a different definition for the same key
    store.update_from_static(&static_store2);

    // Verify it was replaced
    let obj_keys = store.object_keys().unwrap();
    assert_eq!(obj_keys.len(), 1);
    assert_eq!(obj_keys[0].as_str(), obj_key.as_str());

    let mut proxy = store
        .object(&datastore::StorePath::builder(obj_key.as_str()).build())
        .unwrap();
    assert!(proxy.basic("prop2").is_ok());
    assert!(proxy.basic("prop1").is_err());
}

#[test]
fn test_update_from_static_does_not_remove_missing_properties() {
    let store = Store::new(SharedStringStore::new());

    // Object: Has two properties
    let obj_key = store_key!("my_object");
    let def = ObjectDefinition::builder("My Object")
        .with_inserted(
            store_key!("prop1"),
            PropertyDefinition::new("Property 1", BasicDefinition::new_string("Initial")),
        )
        .with_inserted(
            store_key!("prop2"),
            PropertyDefinition::new("Property 2", BasicDefinition::new_string("Stay")),
        )
        .finish();
    store.create_object(obj_key.clone(), &def).unwrap();
    store.tree_print();

    // Create a static store with the same object but ONLY prop1 (updated)
    let other_store = Store::new(SharedStringStore::new());
    let def_updated = ObjectDefinition::builder("My Object")
        .with_inserted(
            store_key!("prop1"),
            PropertyDefinition::new("Property 1", BasicDefinition::new_string("Updated")),
        )
        .finish();
    let mut other_proxy = other_store
        .create_object(obj_key.clone(), &def_updated)
        .unwrap();
    {
        let mut prop1_proxy = other_proxy.basic("prop1").unwrap();
        prop1_proxy.set_value("Updated");
        prop1_proxy.push().unwrap();
    }
    let static_store = other_store.to_static();

    // Update original store from static store
    store.update_from_static(&static_store);
    store.tree_print();

    // Verify prop1 was updated
    let mut proxy = store
        .object(&datastore::StorePath::builder(obj_key.as_str()).build())
        .unwrap();
    let prop1_proxy = proxy.basic("prop1").unwrap();
    assert_eq!(prop1_proxy.value().as_str(), "Updated");

    // Verify prop2 still exists and was removed
    assert!(proxy.basic("prop2").is_err());
}

#[test]
fn test_update_from_static_does_not_remove_missing_objects() {
    let store = Store::new(SharedStringStore::new());

    // Object 1: Will be in both
    let obj_key1 = store_key!("object1");
    let def1 = ObjectDefinition::builder("Object 1")
        .with_inserted(
            store_key!("prop1"),
            PropertyDefinition::new("Property 1", BasicDefinition::new_string("Initial")),
        )
        .finish();
    store.create_object(obj_key1.clone(), &def1).unwrap();

    // Object 2: Only in original store, NOT in static store
    let obj_key2 = store_key!("object2");
    let def2 = ObjectDefinition::builder("Object 2")
        .with_inserted(
            store_key!("prop2"),
            PropertyDefinition::new("Property 2", BasicDefinition::new_string("Stay")),
        )
        .finish();
    store.create_object(obj_key2.clone(), &def2).unwrap();

    store.tree_print();

    // Create a static store that only has object1 (updated)
    let other_store = Store::new(SharedStringStore::new());
    let def1_updated = ObjectDefinition::builder("Object 1")
        .with_inserted(
            store_key!("prop1"),
            PropertyDefinition::new("Property 1", BasicDefinition::new_string("Updated")),
        )
        .finish();
    let mut other_proxy = other_store
        .create_object(obj_key1.clone(), &def1_updated)
        .unwrap();
    let mut prop1_proxy = other_proxy.basic("prop1").unwrap();
    prop1_proxy.set_value("Updated");
    prop1_proxy.push().unwrap();
    let static_store = other_store.to_static();

    // Update original store from static store
    store.update_from_static(&static_store);
    store.tree_print();

    // Verify object1 was updated
    let mut proxy1 = store
        .object(&datastore::StorePath::builder(obj_key1.as_str()).build())
        .unwrap();
    // No need to pull because we created a NEW proxy from the store
    let prop1_proxy = proxy1.basic("prop1").unwrap();
    assert_eq!(prop1_proxy.value().as_str(), "Updated");

    // Verify object2 still exists
    let mut proxy2 = store
        .object(&datastore::StorePath::builder(obj_key2.as_str()).build())
        .unwrap();
    assert!(proxy2.basic("prop2").is_ok());

    // Verify both keys are present
    let obj_keys = store.object_keys().unwrap();
    assert_eq!(obj_keys.len(), 2);
    assert!(obj_keys.iter().any(|k| k.as_str() == "object1"));
    assert!(obj_keys.iter().any(|k| k.as_str() == "object2"));
}

#[test]
fn test_update_from_static_add_object() {
    let store = Store::new(SharedStringStore::new());

    let other_store = Store::new(SharedStringStore::new());
    let obj_key = store_key!("new_object");
    let def = ObjectDefinition::builder("New Object").finish();
    other_store.create_object(obj_key.clone(), &def).unwrap();
    let static_store = other_store.to_static();

    // Update store (which is empty) from static store
    store.update_from_static(&static_store);

    // Verify it was added
    let obj_keys = store.object_keys().unwrap();
    assert_eq!(obj_keys.len(), 1);
    assert_eq!(obj_keys[0].as_str(), obj_key.as_str());
    assert!(
        store
            .object(&datastore::StorePath::builder(obj_key.as_str()).build())
            .is_ok()
    );
}
