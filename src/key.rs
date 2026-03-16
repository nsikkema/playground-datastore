use crate::StoreError;
use crate::shareable_string::{ShareableString, SharedStringStore};
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::hash::Hash;

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

impl PartialEq<&str> for ConstStoreKey {
    fn eq(&self, other: &&str) -> bool {
        self.0 == *other
    }
}

impl PartialEq<ConstStoreKey> for &str {
    fn eq(&self, other: &ConstStoreKey) -> bool {
        *self == other.0
    }
}

impl PartialEq<String> for ConstStoreKey {
    fn eq(&self, other: &String) -> bool {
        self.0 == other.as_str()
    }
}

impl PartialEq<ConstStoreKey> for String {
    fn eq(&self, other: &ConstStoreKey) -> bool {
        self.as_str() == other.0
    }
}

impl PartialEq<ShareableString> for ConstStoreKey {
    fn eq(&self, other: &ShareableString) -> bool {
        self.0 == other.as_str()
    }
}

impl PartialEq<ConstStoreKey> for ShareableString {
    fn eq(&self, other: &ConstStoreKey) -> bool {
        self.as_str() == other.0
    }
}

impl PartialEq<StoreKey> for ConstStoreKey {
    fn eq(&self, other: &StoreKey) -> bool {
        self.0 == other.as_str()
    }
}

impl PartialEq<ConstStoreKey> for StoreKey {
    fn eq(&self, other: &ConstStoreKey) -> bool {
        self.as_str() == other.0
    }
}

impl PartialOrd<&str> for ConstStoreKey {
    fn partial_cmp(&self, other: &&str) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(*other)
    }
}

impl PartialOrd<ConstStoreKey> for &str {
    fn partial_cmp(&self, other: &ConstStoreKey) -> Option<std::cmp::Ordering> {
        (*self).partial_cmp(other.0)
    }
}

impl PartialOrd<String> for ConstStoreKey {
    fn partial_cmp(&self, other: &String) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(other.as_str())
    }
}

impl PartialOrd<ConstStoreKey> for String {
    fn partial_cmp(&self, other: &ConstStoreKey) -> Option<std::cmp::Ordering> {
        self.as_str().partial_cmp(other.0)
    }
}

impl PartialOrd<ShareableString> for ConstStoreKey {
    fn partial_cmp(&self, other: &ShareableString) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(other.as_str())
    }
}

impl PartialOrd<ConstStoreKey> for ShareableString {
    fn partial_cmp(&self, other: &ConstStoreKey) -> Option<std::cmp::Ordering> {
        self.as_str().partial_cmp(other.0)
    }
}

impl PartialOrd<StoreKey> for ConstStoreKey {
    fn partial_cmp(&self, other: &StoreKey) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(other.as_str())
    }
}

impl PartialOrd<ConstStoreKey> for StoreKey {
    fn partial_cmp(&self, other: &ConstStoreKey) -> Option<std::cmp::Ordering> {
        self.as_str().partial_cmp(other.0)
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

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StoreKey {
    pub(crate) key: ShareableString,
}

impl Serialize for StoreKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for StoreKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        StoreKey::new(ShareableString::from(s)).map_err(serde::de::Error::custom)
    }
}

impl StoreKey {
    pub fn new(key: ShareableString) -> Result<Self, StoreError> {
        validate_key(&key)?;
        Ok(StoreKey { key })
    }

    pub(crate) fn new_unsafe(key: ShareableString) -> Self {
        StoreKey { key }
    }

    /// Returns the string slice.
    pub fn as_str(&self) -> &str {
        self.key.as_str()
    }

    /// Returns the underlying `ShareableString`.
    pub fn as_shareable_string(&self) -> &ShareableString {
        &self.key
    }

    pub fn launder(&self, store: &SharedStringStore) -> StoreKey {
        let laundered_key = store.launder(self.key.clone());

        StoreKey::new_unsafe(laundered_key)
    }

    pub fn current_blake3_hash(&self) -> [u8; 32] {
        self.key.current_blake3_hash()
    }
}

impl PartialEq<ShareableString> for StoreKey {
    fn eq(&self, other: &ShareableString) -> bool {
        self.key.as_ref() == other.as_ref()
    }
}

impl PartialEq<StoreKey> for ShareableString {
    fn eq(&self, other: &StoreKey) -> bool {
        self.as_str() == other.as_str()
    }
}

impl PartialEq<&str> for StoreKey {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}

impl PartialEq<StoreKey> for &str {
    fn eq(&self, other: &StoreKey) -> bool {
        *self == other.as_str()
    }
}

impl PartialEq<String> for StoreKey {
    fn eq(&self, other: &String) -> bool {
        self.as_str() == other.as_str()
    }
}

impl PartialEq<StoreKey> for String {
    fn eq(&self, other: &StoreKey) -> bool {
        self.as_str() == other.as_str()
    }
}

impl AsRef<str> for StoreKey {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl PartialOrd<&str> for StoreKey {
    fn partial_cmp(&self, other: &&str) -> Option<std::cmp::Ordering> {
        self.as_str().partial_cmp(*other)
    }
}

impl PartialOrd<ShareableString> for StoreKey {
    fn partial_cmp(&self, other: &ShareableString) -> Option<std::cmp::Ordering> {
        self.key.partial_cmp(other)
    }
}

impl PartialOrd<StoreKey> for ShareableString {
    fn partial_cmp(&self, other: &StoreKey) -> Option<std::cmp::Ordering> {
        self.partial_cmp(other.as_str())
    }
}

impl PartialOrd<StoreKey> for &str {
    fn partial_cmp(&self, other: &StoreKey) -> Option<std::cmp::Ordering> {
        (*self).partial_cmp(other.as_str())
    }
}

impl PartialOrd<String> for StoreKey {
    fn partial_cmp(&self, other: &String) -> Option<std::cmp::Ordering> {
        self.as_str().partial_cmp(other.as_str())
    }
}

impl PartialOrd<StoreKey> for String {
    fn partial_cmp(&self, other: &StoreKey) -> Option<std::cmp::Ordering> {
        self.as_str().partial_cmp(other.as_str())
    }
}

impl Display for StoreKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.key)
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

impl std::borrow::Borrow<str> for StoreKey {
    fn borrow(&self) -> &str {
        self.key.as_str()
    }
}

impl std::borrow::Borrow<ShareableString> for StoreKey {
    fn borrow(&self) -> &ShareableString {
        &self.key
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_store_key_comparisons() {
        let sk = StoreKey::new(ShareableString::new("key")).unwrap();
        let ss = ShareableString::new("key");
        let s = "key";
        let string = String::from("key");

        // PartialEq
        assert_eq!(sk, ss);
        assert_eq!(ss, sk);
        assert_eq!(sk, s);
        assert_eq!(s, sk);
        assert_eq!(sk, s);
        assert_eq!(s, sk);
        assert_eq!(sk, string);
        assert_eq!(string, sk);

        // PartialOrd
        assert!(sk >= ss);
        assert!(ss <= sk);
        assert!(sk >= s);
        assert!(s <= sk);
        assert!(sk >= s);
        assert!(s <= sk);
        assert!(sk >= string);
        assert!(string <= sk);
    }

    #[test]
    fn test_const_store_key_comparisons() {
        let csk = ConstStoreKey::new("key");
        let sk = StoreKey::new(ShareableString::new("key")).unwrap();
        let ss = ShareableString::new("key");
        let s = "key";
        let string = String::from("key");

        // PartialEq
        assert_eq!(csk, s);
        assert_eq!(s, csk);
        assert_eq!(csk, s);
        assert_eq!(s, csk);
        assert_eq!(csk, string);
        assert_eq!(string, csk);
        assert_eq!(csk, ss);
        assert_eq!(ss, csk);
        assert_eq!(csk, sk);
        assert_eq!(sk, csk);

        // PartialOrd
        assert!(csk >= s);
        assert!(s <= csk);
        assert!(csk >= s);
        assert!(s <= csk);
        assert!(csk >= string);
        assert!(string <= csk);
        assert!(csk >= ss);
        assert!(ss <= csk);
        assert!(csk >= sk);
        assert!(sk <= csk);
    }
}
