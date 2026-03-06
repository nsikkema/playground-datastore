use parking_lot::RwLock;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::sync::Arc;

/// A thread-safe container for a BLAKE3 hash.
#[derive(Debug, Clone)]
pub struct StoreHashContainer {
    hash: Arc<RwLock<[u8; 32]>>,
}

impl Serialize for StoreHashContainer {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let hash = self.hash.read();
        hash.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for StoreHashContainer {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let hash: [u8; 32] = Deserialize::deserialize(deserializer)?;
        Ok(Self {
            hash: Arc::new(RwLock::new(hash)),
        })
    }
}

impl StoreHashContainer {
    /// Creates a new `StoreHashContainer` initialized with a zero hash.
    pub(in crate::store) fn new() -> Self {
        Self {
            hash: Arc::new(RwLock::new([0u8; 32])),
        }
    }

    /// Sets the hash value.
    pub(in crate::store) fn set(&self, new_hash: [u8; 32]) {
        *self.hash.write() = new_hash;
    }

    /// Returns the current hash value.
    pub fn get(&self) -> [u8; 32] {
        self.hash.read().clone()
    }

    /// Clears the hash value (sets it to zero).
    pub(in crate::store) fn clear(&self) {
        self.set([0u8; 32]);
    }
}

impl Default for StoreHashContainer {
    fn default() -> Self {
        Self::new()
    }
}
