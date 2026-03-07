# Datastore

A hierarchical, thread-safe, and observable data store with proxy-based access for Rust.

## What is this?
Datastore is a library for managing complex, structured data in memory. It provides a way to define data schemas, store objects based on those schemas, and interact with the data through lightweight "proxies." It's designed for scenarios where multiple parts of an application need to access and modify a shared data structure in a thread-safe manner, with the ability to detect and react to changes.

## Why use it?
- **Schema-Driven**: Define your data structure once using `Definitions`.
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
- `MapDefinition` and `TableDefinition`: For collections of items.

See `examples/advanced_types.rs` for a detailed example of using Structs, Maps, and Tables.

### Store
The `Store` is the central repository for all your data. It holds a collection of top-level `Objects`.

### Proxies
Proxies are handles to specific parts of the data within the store (e.g., `ObjectProxy`, `BasicProxy`). They allow you to:
- `get_value()` / `set_value()`: Read or update data.
- `push()`: Commit changes from the proxy back to the store.
- `pull()`: Sync the proxy with the latest data from the store.
- `has_changed()`: Check if the underlying data in the store has been updated.

### Paths
Every piece of data in the store is uniquely identified by a `StorePath` (e.g., `user_1/profile/age`).

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
    builder.add("name", PropertyDefinition::new("Name", BasicDefinition::new_string(""))).unwrap();
    let def = builder.finish();
    
    // Add it to the store
    store.create_object(&"user_1".into(), &def).unwrap();
    
    // Get a proxy and update data
    let mut name_proxy = store.get_basic(&"user_1/name".into()).unwrap();
    name_proxy.set_value("Alice");
    name_proxy.push().unwrap();
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
- **No Undo/Redo**: Although some structures for undo/redo exist, the full functionality is not yet exposed or implemented across all proxy types.
- **Basic Types**: Currently focuses on String and Number for basic values.
