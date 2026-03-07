//! # Datastore
//!
//! A hierarchical, thread-safe, and observable data store with proxy-based access.
//!
//! ## Core Concepts
//!
//! - **Store**: The root container for all data objects. It manages thread safety and persistence.
//! - **Definitions**: Define the structure of your data (Objects, Structs, Maps, Tables, and Basic values).
//! - **Proxies**: Lightweight handles to data within the store. They provide a way to read and update data while maintaining sync with the store.
//! - **Paths**: Unique identifiers for every piece of data in the store.
//! - **Shareable Strings**: Interned, thread-safe strings used throughout the store to reduce memory overhead and enable fast comparisons.
//!
//! ## Thread Safety and Invariants
//!
//! - **Thread Safety**: The `Store` is thread-safe (`Send` + `Sync`) and uses internal locking (`parking_lot::RwLock`).
//! - **Proxy Validity**: A proxy becomes "invalid" (expired) if its underlying data is removed from the store. Use `proxy.is_valid()` to check.
//! - **Cloning**: Cloning a `Store` or a `Proxy` creates a new handle to the *same* underlying data (shallow copy).
//! - **Change Tracking**: Use `has_changed()` on a proxy to check if the store has been updated since the proxy was last synced.
//! - **Updates**: Updates via proxies are pushed to the store. Other proxies must `pull()` to see these changes.
//!
//! ## Example
//!
//! ```rust
//! use datastore::store::{Store, StorePath};
//! use datastore::definition::{ObjectDefinition, BasicDefinition, PropertyDefinition};
//! use datastore::store::traits::ProxyStoreTrait;
//!
//! // 1. Define your data structure
//! let mut builder = ObjectDefinition::builder("My Object");
//! builder.add("name", PropertyDefinition::new("User Name", BasicDefinition::new_string("Name"))).unwrap();
//! let def = builder.finish();
//!
//! // 2. Create a store and add an object
//! let store = Store::new(Default::default());
//! store.create_object("user_1", &def).unwrap();
//!
//! // 3. Access data via a proxy
//! let mut proxy = store.object(&"user_1".into()).unwrap();
//! let mut name_proxy = proxy.basic("name").unwrap();
//!
//! name_proxy.set_value("Alice");
//! name_proxy.push().unwrap();
//!
//! assert_eq!(name_proxy.value().unwrap().as_str(), "Alice");
//! ```

pub mod definition;
pub mod shareable_string;
pub mod store;

use crate::shareable_string::ShareableString;
use std::fmt::{Display, Formatter};

/// Error types for the store operations.
#[derive(Debug, Clone, PartialEq)]
pub enum StoreError {
    /// The provided key is empty.
    KeyEmpty,
    /// The key contains an invalid character.
    KeyInvalidCharacter(String),
    /// The requested object was not found.
    ObjectNotFound,
    /// An object with the specified key already exists.
    ObjectKeyAlreadyExists,
    /// The requested property was not found.
    PropertyNotFound,
    /// The proxy has expired or is no longer valid.
    ExpiredProxy,
    /// The key was not found in the map.
    KeyNotFound,
    /// The provided path is invalid.
    InvalidPath,
    /// The provided path segment is invalid.
    InvalidPathSegment(String),
    /// The requested index was not found.
    IndexNotFound,
    /// Undo operation is not available.
    UndoNotAvailable,
    /// Redo operation is not available.
    RedoNotAvailable,
    /// Failed to serialize or deserialize the store state.
    SerializationError(String),
}

impl Display for StoreError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            StoreError::KeyEmpty => write!(f, "Invalid key: Key cannot be empty"),
            StoreError::KeyInvalidCharacter(s) => write!(
                f,
                "Invalid key: '{}'. Keys must only contain a-z, 0-9 and _",
                s
            ),
            StoreError::ObjectNotFound => write!(f, "Object not found"),
            StoreError::ObjectKeyAlreadyExists => write!(f, "Object key already exists"),
            StoreError::PropertyNotFound => write!(f, "Property not found"),
            StoreError::ExpiredProxy => write!(f, "Proxy is invalid"),
            StoreError::KeyNotFound => write!(f, "Key not found"),
            StoreError::InvalidPath => write!(f, "Invalid path"),
            StoreError::InvalidPathSegment(s) => write!(f, "Invalid path segment: {}", s),
            StoreError::IndexNotFound => write!(f, "Index not found"),
            StoreError::UndoNotAvailable => write!(f, "Undo not available"),
            StoreError::RedoNotAvailable => write!(f, "Redo not available"),
            StoreError::SerializationError(s) => write!(f, "Serialization error: {}", s),
        }
    }
}

impl std::error::Error for StoreError {}

/// Validates that a key is not empty and only contains valid characters.
/// Valid characters are lowercase a-z, digits 0-9, and underscores.
pub(crate) fn validate_key(key: &ShareableString) -> Result<(), StoreError> {
    let s = key.as_str();
    if s.is_empty() {
        return Err(StoreError::KeyEmpty);
    }
    for c in s.chars() {
        if !c.is_ascii_lowercase() && !c.is_ascii_digit() && c != '_' {
            return Err(StoreError::KeyInvalidCharacter(s.to_string()));
        }
    }
    Ok(())
}
