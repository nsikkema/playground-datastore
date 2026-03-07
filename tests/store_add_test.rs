use datastore::definition::{BasicDefinition, ObjectDefinition, PropertyDefinition};
use datastore::shareable_string::{ShareableString, SharedStringStore};
use datastore::store::traits::ProxyStoreTrait;
use datastore::store::{Store, StorePath};

#[test]
fn test_add_object_from_another_store() {
    let store1 = Store::new(SharedStringStore::new());
    let store2 = Store::new(SharedStringStore::new());

    let obj_key1 = ShareableString::from("object1");
    let def = ObjectDefinition::builder("Test Object")
        .with(
            "prop1",
            PropertyDefinition::new("Property 1", BasicDefinition::new_string("String property")),
        )
        .unwrap()
        .finish();

    let _proxy1 = store1.create_object(&obj_key1, &def).unwrap();
    let prop_path = StorePath::builder(obj_key1.clone())
        .property(ShareableString::from("prop1"))
        .build();
    let mut basic1 = store1.basic(&prop_path).unwrap();
    basic1.set_value("Hello from Store 1");
    basic1.push().unwrap();

    // Add object from store1 to store2
    let obj_key2 = ShareableString::from("object2");
    let proxy2 = store2.copy_object(&obj_key2, &store1, &obj_key1).unwrap();

    assert_eq!(proxy2.description().as_str(), "Test Object");

    let prop_path2 = StorePath::builder(obj_key2.clone())
        .property(ShareableString::from("prop1"))
        .build();
    let basic2 = store2.basic(&prop_path2).unwrap();
    assert_eq!(basic2.value().unwrap().as_str(), "Hello from Store 1");

    // Verify they are independent
    let mut basic2_mut = store2.basic(&prop_path2).unwrap();
    basic2_mut.set_value("Changed in Store 2");

    let basic1_after = store1.basic(&prop_path).unwrap();
    assert_eq!(
        basic1_after.value().unwrap().as_str(),
        "Hello from Store 1"
    );
    assert_eq!(
        basic2_mut.value().unwrap().as_str(),
        "Changed in Store 2"
    );
}
