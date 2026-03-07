# Datastore

A hierarchical, thread-safe, and observable data store with proxy-based access for Rust.

## What is this?
Datastore is a library for managing complex, structured data in memory. It provides a way to define data schemas, store objects based on those schemas, and interact with the data through lightweight "proxies."

## Why use it?
- **Schema-Driven**: Define your data structure once using `Definitions` and ensure your data stays consistent.
- **Thread-Safe**: Built-in support for concurrent access using `RwLock`.
- **Observable**: Detect if data has changed since you last synced your proxy.
- **Efficient**: Uses "Shareable Strings" (interned strings) to reduce memory overhead and speed up comparisons.
- **Persistent**: Easily save and load the entire store state to/from JSON.

## Core Concepts

### Definitions
Definitions specify the structure and types of your data.
- `BasicDefinition`: For primitive types like Strings, Numbers, and Files.
- `ObjectDefinition`: A collection of named properties.
- `StructDefinition`: Similar to objects, used for nested structures.
- `MapDefinition`: For collections of items keyed by a string.
- `TableDefinition`: For tabular data with fixed columns.

See `examples/advanced_types.rs` for a detailed example of using Structs, Maps, and Tables.

### Store
The `Store` is the central repository for all your data. It holds a collection of top-level `Objects`.

### Proxies
Proxies are handles to specific parts of the data within the store. They allow you to:
- `ObjectProxy`: Access and manage a top-level object.
- `BasicProxy`: Read or update basic values like Strings and Numbers.
- `ContainerProxy`: Interact with nested Structs and Maps.
- `TableProxy`: Manipulate tabular data.

All proxies provide:
- `get_value()` / `set_value()`: Read or update data (where applicable).
- `push()`: Commit changes from the proxy back to the store.
- `pull()`: Sync the proxy with the latest data from the store.
- `has_changed()`: Check if the underlying data in the store has been updated.

### Paths
Every piece of data in the store is uniquely identified by a `StorePath`.
You can use the `path!` macro for ergonomic path construction: `path!("user_1" / "profile" / "age")`.
Alternatively, you can use tuples for simple paths: `let p: StorePath = ("user_1", "profile", "age").into();`.

### Shareable Strings
`ShareableString` is an interned string type used throughout the library to optimize memory and performance.

## Small example
```rust
use datastore::store::{Store, StorePath};
use datastore::definition::{ObjectDefinition, BasicDefinition, PropertyDefinition};
use datastore::store::traits::ProxyStoreTrait;

fn main() {
    let store = Store::new(Default::default());
    
    // Define an object
    let mut builder = ObjectDefinition::builder("User");
    builder.add("name".into(), PropertyDefinition::new("Name", BasicDefinition::new_string(""))).unwrap();
    let def = builder.finish();
    
    // Add it to the store
    store.create_object("user_1".into(), &def).unwrap();
    
    // Get a proxy and update data
    let mut name_proxy = store.get_basic(&"user_1/name".into()).unwrap();
    name_proxy.set_value("Alice");
    name_proxy.push().unwrap();
}
```

## Nested data example
```rust
use datastore::store::Store;
use datastore::definition::{ObjectDefinition, StructDefinition, MapDefinition, PropertyDefinition, BasicDefinition};
use datastore::store::traits::ProxyStoreTrait;
use datastore::path;

fn main() {
    let store = Store::new(Default::default());

    // Define a struct for Address
    let address_def = StructDefinition::new("Address", vec![
        ("city".into(), BasicDefinition::new_string("").into()),
    ]);

    // Define a map of addresses
    let contacts_def = MapDefinition::new("Contacts", address_def);

    let mut builder = ObjectDefinition::builder("User Profile");
    builder.add("addresses".into(), PropertyDefinition::new("Addresses", contacts_def));
    let user_def = builder.finish();

    store.create_object("user_1".into(), &user_def).unwrap();

    // Access the map and insert a new entry
    let mut user_proxy = store.object(&"user_1".into()).unwrap();
    let addresses_proxy = user_proxy.container("addresses").unwrap();
    addresses_proxy.insert_map_entry("home").unwrap();

    // Update the city in the new entry
    let mut city_proxy = store.basic(&path!("user_1" / "addresses" / "home" / "city")).unwrap();
    city_proxy.set_value("New York");
    city_proxy.push().unwrap();
}
```

## Persistence example
```rust
use datastore::store::Store;

fn main() {
    let store = Store::new(Default::default());
    // ... populate store ...
    
    // Serialize to JSON
    let json = store.to_json().expect("Failed to serialize");
    std::fs::write("my_data.json", json).expect("Failed to write to file");
    
    // Load from JSON
    let loaded_json = std::fs::read_to_string("my_data.json").expect("Failed to read");
    let loaded_store = Store::from_json(&loaded_json).expect("Failed to load");
}
```

## Current limitations
- **JSON Only**: Persistence is currently limited to JSON format.
- **In-Memory**: The store is entirely in-memory; persistence is manual (save/load).
- **Undo/Redo**: Some basic structures exist, but full undo/redo functionality is not yet fully implemented across all proxy types.
- **Basic Types**: Currently focuses on String and Number for basic values.
