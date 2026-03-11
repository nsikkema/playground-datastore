use super::{ObjectProxy, StorePath};
use crate::StoreError;
use crate::shareable_string::ShareableString;

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
    fn path(&self) -> &StorePath;
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
    fn object(&self) -> Result<ObjectProxy, StoreError>;
}

/// Trait for types that can be printed as a tree for debugging.
pub trait TreePrint {
    /// Prints the object as a tree with the given label and prefix.
    fn tree_print(&self, label: &str, prefix: &str, last: bool);

    /// Helper to get the correct prefix for the next level.
    fn next_prefix(prefix: &str, last: bool) -> String {
        format!("{}{}", prefix, if last { "    " } else { "│   " })
    }

    /// Helper to get the branch character.
    fn branch_char(last: bool) -> &'static str {
        if last { "└── " } else { "├── " }
    }
}
