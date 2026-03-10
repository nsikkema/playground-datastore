use datastore::definition::{BasicDefinition, ObjectDefinition, PropertyDefinition};
use datastore::store::{ProxyStoreTrait, Store};

fn main() {
    // 1. Create a Store
    let store = Store::new(Default::default());

    // 2. Define an Object Structure
    // An Object is a collection of named properties.
    let mut user_def = ObjectDefinition::builder("User Profile");
    user_def.insert(
        "username".try_into().unwrap(),
        PropertyDefinition::new("The user's unique name", BasicDefinition::new_string("")),
    );
    user_def.insert(
        "age".try_into().unwrap(),
        PropertyDefinition::new("The user's age", BasicDefinition::new_number("0")),
    );

    let def = user_def.finish();

    // 3. Add an Object to the Store
    // Objects are added at the top level with a unique key.
    store
        .create_object("user_123".try_into().unwrap(), &def)
        .unwrap();

    // 4. Access Data via Proxies
    // The proxy provides a way to interact with data in the store.
    let mut user_proxy = store.object(&"user_123".into()).unwrap();
    let mut username_proxy = user_proxy.basic("username").unwrap();
    let mut age_proxy = user_proxy.basic("age").unwrap();

    // 5. Update Data
    // Changes are made to the proxy first, then pushed to the store.
    username_proxy.set_value("john doe");
    username_proxy.push().unwrap();

    age_proxy.set_value("30");
    age_proxy.push().unwrap();

    // 6. Observe Changes
    // If another handle to the same data exists, it can observe changes.
    let mut observer_proxy = store.basic(&"user_123/username".into()).unwrap();

    // We update the data via another proxy.
    username_proxy.set_value("john doe updated");
    username_proxy.push().unwrap();

    // The store has changed, so has_changed() will return true.
    assert!(observer_proxy.has_changed());

    // Pull the latest data from the store.
    observer_proxy.pull().unwrap();
    assert_eq!(observer_proxy.value().unwrap().as_str(), "john doe updated");

    println!("Username: {}", observer_proxy.value().unwrap());
    println!("Age: {}", age_proxy.value().unwrap());
}
