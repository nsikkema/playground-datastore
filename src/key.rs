use crate::StoreError;
use crate::shareable_string::ShareableString;
use std::fmt::Display;

use serde::{Deserialize, Serialize};

/// Returns true if the key is not empty and only contains valid characters.
/// The first character must be lowercase a-z.
/// Remaining characters may be lowercase a-z, digits 0-9, and underscores.
pub const fn is_valid_key(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }

    let bytes = s.as_bytes();
    let first = bytes[0];
    if !first.is_ascii_lowercase() {
        return false;
    }

    let mut i = 1;
    while i < bytes.len() {
        let c = bytes[i];
        if !c.is_ascii_lowercase() && !c.is_ascii_digit() && c != b'_' {
            return false;
        }
        i += 1;
    }
    true
}

/// Validates that a key is not empty and only contains valid characters.
/// The first character must be lowercase a-z.
/// Remaining characters may be lowercase a-z, digits 0-9, and underscores.
fn validate_key(key: &ShareableString) -> Result<(), StoreError> {
    let s = key.as_str();
    if is_valid_key(s) {
        Ok(())
    } else if s.is_empty() {
        Err(StoreError::KeyEmpty)
    } else {
        Err(StoreError::KeyInvalidCharacter(s.to_string()))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ConstStoreKey(pub(crate) &'static str);

impl ConstStoreKey {
    /// Creates a new `ConstStoreKey` from a validated literal.
    /// Panics at compile-time if the key is invalid.
    pub const fn new(key: &'static str) -> Self {
        if !is_valid_key(key) {
            panic!("Invalid StoreKey literal");
        }
        Self(key)
    }

    /// Returns the string slice.
    pub const fn as_str(&self) -> &'static str {
        self.0
    }
}

impl Display for ConstStoreKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<ConstStoreKey> for StoreKey {
    fn from(value: ConstStoreKey) -> Self {
        StoreKey {
            key: ShareableString::from(value.0),
        }
    }
}

impl From<&ConstStoreKey> for StoreKey {
    fn from(value: &ConstStoreKey) -> Self {
        StoreKey {
            key: ShareableString::from(value.0),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct StoreKey {
    pub(crate) key: ShareableString,
}

impl StoreKey {
    pub fn new(key: ShareableString) -> Result<Self, StoreError> {
        validate_key(&key)?;
        Ok(StoreKey { key })
    }

    /// Returns the string slice.
    pub fn as_str(&self) -> &str {
        self.key.as_str()
    }

    /// Returns the underlying `ShareableString`.
    pub fn as_shareable_string(&self) -> &ShareableString {
        &self.key
    }
}

impl From<ConstStoreKey> for ShareableString {
    fn from(value: ConstStoreKey) -> Self {
        ShareableString::from(value.0)
    }
}

impl From<&ConstStoreKey> for ShareableString {
    fn from(value: &ConstStoreKey) -> Self {
        ShareableString::from(value.0)
    }
}

impl From<StoreKey> for ShareableString {
    fn from(value: StoreKey) -> Self {
        value.key
    }
}

impl From<&StoreKey> for ShareableString {
    fn from(value: &StoreKey) -> Self {
        value.key.clone()
    }
}

/// A macro to create a `ConstStoreKey` from a string literal.
/// Validates the key at compile-time.
#[macro_export]
macro_rules! store_key {
    ($key:expr) => {
        $crate::key::ConstStoreKey::new($key)
    };
}
