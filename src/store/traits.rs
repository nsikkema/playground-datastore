use super::ObjectProxy;
use crate::shareable_string::ShareableString;
use crate::{StoreError, StorePath};
use std::fmt;

/// Internal trait for common store operations related to hashing.
pub(crate) trait CommonStoreTraitInternal {
    /// Returns the current shared BLAKE3 hash.
    fn current_shared_hash(&self) -> [u8; 32];
    /// Returns a new BLAKE3 Hash for the object.
    fn update_current_hash(&mut self);
    /// Force sync shared hash.
    fn update_shared_hash(&mut self);
    /// Clears the current shared hash.
    fn clear_shared_hash(&mut self);
    /// Check if de-synced from the original store.
    fn has_changed(&self) -> bool;
    /// Check if the original store still exists.
    fn is_valid(&self) -> bool;
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
    fn tree_print(
        &self,
        f: &mut fmt::Formatter<'_>,
        label: &str,
        prefix: &str,
        last: bool,
    ) -> fmt::Result;

    /// Helper to get the correct prefix for the next level.
    fn next_prefix(prefix: &str, last: bool) -> String {
        format!("{}{}", prefix, if last { "    " } else { "│   " })
    }

    /// Helper to get the branch character.
    fn branch_char(prefix: &str, last: bool) -> &'static str {
        if prefix.is_empty() {
            ""
        } else if last {
            "└── "
        } else {
            "├── "
        }
    }

    /// Returns a `TreeDisplay` for the given item.
    fn tree_display(&self, label: &str) -> TreeDisplay<'_, Self>
    where
        Self: Sized,
    {
        TreeDisplay {
            item: self,
            label: label.to_string(),
        }
    }
}

pub struct TreeDisplay<'a, T: TreePrint> {
    pub item: &'a T,
    pub label: String,
}

impl<'a, T: TreePrint> fmt::Display for TreeDisplay<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.item.tree_print(f, &self.label, "", true)
    }
}
