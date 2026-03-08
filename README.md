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

## Example
Examples can be found in the [examples/](examples/) folder:
- [basic_usage.rs](examples/basic_usage.rs): A quick introduction to creating a store, defining objects, and using proxies.
- [advanced_types.rs](examples/advanced_types.rs): Demonstrates how to use Structs, Maps, and Tables.
- [persistence_usage.rs](examples/persistence_usage.rs): Shows how to save and load the store state to/from JSON.


## Current limitations
- **JSON Only**: Persistence is currently limited to JSON format.
- **In-Memory**: The store is entirely in-memory; persistence is manual (save/load).
- **Undo/Redo**: Some basic structures exist, but full undo/redo functionality is not yet fully implemented across all proxy types.
- **Basic Types**: Currently focuses on String and Number for basic values.
