use crate::definition::BasicDefinition;
use crate::shareable_string::ShareableString;
use crate::store::traits::TreePrint;
use crate::store::{Basic, CommonStoreTraitInternal, ObjectProxy, ProxyStoreTrait, Store};
use crate::{StoreError, StorePath};

/// A proxy for a basic data value in the store.
pub struct BasicProxy {
    path: StorePath,
    store: Store,
    data: Basic,
    last_sync_hash: [u8; 32],
}

impl BasicProxy {
    /// Creates a new `BasicProxy`.
    pub(crate) fn new(path: StorePath, store: Store, data: Basic) -> Self {
        let last_sync_hash = data.current_blake3_hash();
        Self {
            path,
            store,
            data,
            last_sync_hash,
        }
    }

    /// Returns a reference to the basic definition.
    pub fn definition(&self) -> &BasicDefinition {
        self.data.definition()
    }

    /// Returns the current value from the proxy.
    pub fn value(&self) -> ShareableString {
        self.data.get()
    }

    /// Sets a new value in the proxy.
    pub fn set_value<S: Into<ShareableString>>(&mut self, value: S) {
        self.data.set(value.into());
    }
}

impl ProxyStoreTrait for BasicProxy {
    fn path(&self) -> &StorePath {
        &self.path
    }

    fn description(&self) -> ShareableString {
        self.definition().description()
    }

    fn is_valid(&self) -> bool {
        self.data.current_blake3_hash() != [0u8; 32]
    }

    fn has_changed(&self) -> bool {
        self.last_sync_hash != self.data.current_blake3_hash()
    }

    fn pull(&mut self) -> Result<(), StoreError> {
        if !self.is_valid() {
            return Err(StoreError::ExpiredProxy);
        }

        if !self.has_changed() {
            return Ok(());
        }

        let proxy = self.store.basic(&self.path)?;

        self.data = proxy.data;
        self.last_sync_hash = proxy.last_sync_hash;

        Ok(())
    }

    fn push(&mut self) -> Result<(), StoreError> {
        if !self.is_valid() {
            return Err(StoreError::ExpiredProxy);
        }

        self.store.set_basic(&self.path, &self.data)?;
        self.last_sync_hash = self.data.current_blake3_hash();
        Ok(())
    }

    fn object(&self) -> Result<ObjectProxy, StoreError> {
        let key = self.path.object_key();
        self.store.object(key)
    }
}

impl TreePrint for BasicProxy {
    fn tree_print(
        &self,
        f: &mut std::fmt::Formatter<'_>,
        label: &str,
        prefix: &str,
        last: bool,
    ) -> std::fmt::Result {
        self.data.tree_print(f, label, prefix, last)
    }
}

impl std::fmt::Display for BasicProxy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let label = self.path.get_last_key();
        self.tree_display(label.as_ref()).fmt(f)
    }
}
