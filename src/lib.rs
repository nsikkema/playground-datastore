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
    /// The requested index was not found.
    IndexNotFound,
    /// Undo operation is not available.
    UndoNotAvailable,
    /// Redo operation is not available.
    RedoNotAvailable,
    /// An IO error occurred.
    IOError,
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
            StoreError::IndexNotFound => write!(f, "Index not found"),
            StoreError::UndoNotAvailable => write!(f, "Undo not available"),
            StoreError::RedoNotAvailable => write!(f, "Redo not available"),
            StoreError::IOError => write!(f, "IO error"),
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
