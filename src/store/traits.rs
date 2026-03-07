use crate::StoreError;
use crate::shareable_string::ShareableString;
use crate::store::path::StorePath;

use crate::store::ObjectProxy;

/// Internal trait for common store operations related to hashing.
pub(in crate::store) trait CommonStoreTraitInternal {
    /// Returns the current BLAKE3 hash.
    fn current_blake3_hash(&self) -> [u8; 32];
    /// Updates the BLAKE3 hash.
    fn update_blake3_hash(&mut self);
    /// Clears the current hash.
    fn clear_hash(&mut self);
}

/// Trait for proxy objects that provide access to store data.
pub trait ProxyStoreTrait {
    /// Returns the path to the data this proxy represents.
    fn get_path(&self) -> &StorePath;
    /// Returns a description of the data.
    fn description(&self) -> ShareableString;
    /// Checks if the proxy is still valid.
    fn is_valid(&self) -> bool;
    /// Returns true if the data has changed compared to the store.
    fn has_changed(&self) -> bool;
    /// Pulls the latest data from the store.
    fn pull(&mut self) -> Result<(), StoreError>;
    /// Pushes the local changes to the store.
    fn push(&mut self) -> Result<(), StoreError>;
    /// Returns an `ObjectProxy` for the object containing this data.
    fn get_object(&self) -> Result<ObjectProxy, StoreError>;
}
