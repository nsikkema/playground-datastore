use crate::StoreError;
use crate::shareable_string::ShareableString;

use serde::{Deserialize, Serialize};

/// Validates that a key is not empty and only contains valid characters.
/// The first character must be lowercase a-z.
/// Remaining characters may be lowercase a-z, digits 0-9, and underscores.
fn validate_key(key: &ShareableString) -> Result<(), StoreError> {
    let s = key.as_str();
    if s.is_empty() {
        return Err(StoreError::KeyEmpty);
    }

    let mut chars = s.chars();
    let first = chars.next().expect("key was checked to be non-empty");
    if !first.is_ascii_lowercase() {
        return Err(StoreError::KeyInvalidCharacter(s.to_string()));
    }

    for c in chars {
        if !c.is_ascii_lowercase() && !c.is_ascii_digit() && c != '_' {
            return Err(StoreError::KeyInvalidCharacter(s.to_string()));
        }
    }
    Ok(())
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
}

impl From<&str> for StoreKey {
    fn from(value: &str) -> Self {
        Self::new(value.into()).expect("Invalid StoreKey in From<&str>")
    }
}

impl From<String> for StoreKey {
    fn from(value: String) -> Self {
        Self::new(value.into()).expect("Invalid StoreKey in From<String>")
    }
}

impl From<ShareableString> for StoreKey {
    fn from(value: ShareableString) -> Self {
        Self::new(value).expect("Invalid StoreKey in From<ShareableString>")
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
