use crate::StoreError;
use crate::definition::BasicDefinition;
use crate::shareable_string::ShareableString;
use crate::store::traits::{CommonStoreTraitInternal, ProxyStoreTrait};
use crate::store::{Basic, ObjectProxy, Store, StorePath};

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
    pub fn get_definition(&self) -> &BasicDefinition {
        &self.data.definition()
    }

    /// Returns the current value from the proxy.
    pub fn get_value(&self) -> Result<ShareableString, StoreError> {
        Ok(self.data.get())
    }

    /// Sets a new value in the proxy.
    pub fn set_value<S: Into<ShareableString> + AsRef<str>>(&mut self, value: S) {
        let new_value = self.store.launder(value.into());
        self.data.set(new_value);
    }
}

impl ProxyStoreTrait for BasicProxy {
    fn get_path(&self) -> &StorePath {
        &self.path
    }

    fn description(&self) -> ShareableString {
        self.get_definition().description()
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

        let proxy = self.store.get_basic(&self.path)?;

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

    fn get_object(&self) -> Result<ObjectProxy, StoreError> {
        let path = self.path.clone().get_object();
        self.store.get_object(&path)
    }
}
