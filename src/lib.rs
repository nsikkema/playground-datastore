pub mod definition;
pub mod shareable_string;

use crate::shareable_string::ShareableString;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, PartialEq)]
pub enum StoreError {
    KeyEmpty,
    KeyInvalidCharacter(String),
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
        }
    }
}

impl std::error::Error for StoreError {}

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
