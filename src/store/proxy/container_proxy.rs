use crate::StoreError;
use crate::shareable_string::ShareableString;
use crate::store::{
    Container, ContainerDefinition, ObjectProxy, ProxyStoreTrait, Store, StoreHashContainer,
    StorePath, TreePrint,
};

/// A proxy for a container in the store.
pub struct ContainerProxy {
    path: StorePath,
    store: Store,
    definition: ContainerDefinition,
    keys: Vec<ShareableString>,
    object_hash: StoreHashContainer,
    last_sync_hash: [u8; 32],
}

impl ContainerProxy {
    /// Creates a new `ContainerProxy`.
    pub(crate) fn new(
        path: StorePath,
        store: Store,
        definition: ContainerDefinition,
        keys: Vec<ShareableString>,
        object_hash: StoreHashContainer,
        last_sync_hash: [u8; 32],
    ) -> Self {
        Self {
            path,
            store,
            definition,
            keys,
            object_hash,
            last_sync_hash,
        }
    }
}

impl ProxyStoreTrait for ContainerProxy {
    fn path(&self) -> &StorePath {
        &self.path
    }

    fn description(&self) -> ShareableString {
        match &self.definition {
            ContainerDefinition::Struct(_struct) => _struct.description(),
            ContainerDefinition::Map(map) => map.description(),
            ContainerDefinition::Object(object) => object.description(),
        }
    }

    fn is_valid(&self) -> bool {
        self.object_hash.get() != [0u8; 32]
    }

    fn has_changed(&self) -> bool {
        self.last_sync_hash != self.object_hash.get()
    }

    fn pull(&mut self) -> Result<(), StoreError> {
        if !self.is_valid() {
            return Err(StoreError::ExpiredProxy);
        }
        if !self.has_changed() {
            return Ok(());
        }

        let container = self.store.container(&self.path)?;

        self.keys = container.keys;
        self.last_sync_hash = container.last_sync_hash;

        Ok(())
    }

    fn push(&mut self) -> Result<(), StoreError> {
        if !self.is_valid() {
            return Err(StoreError::ExpiredProxy);
        }

        Ok(())
    }

    fn object(&self) -> Result<ObjectProxy, StoreError> {
        let path = self.path.clone().get_object();
        self.store.object(&path)
    }
}

impl ContainerProxy {
    /// Inserts a new entry into a map container and returns a proxy to it.
    pub fn insert_map_entry<S: Into<ShareableString>>(
        &self,
        key: S,
    ) -> Result<ContainerProxy, StoreError> {
        let key = key.into();
        match &self.definition {
            ContainerDefinition::Map(map_def) => {
                let entry_container = Container::new_struct(map_def.item_type().clone());
                let entry_path = self.path.clone().to_builder().map_key(key).build()?;
                self.store
                    .update_container_at_path(&entry_path, entry_container)?;
                self.store.container(&entry_path)
            }
            _ => Err(StoreError::PropertyNotFound),
        }
    }

    /// Prints the container as a tree for debugging.
    pub fn tree_print(&self) {
        if let Ok(container) = self.store.get_container_internal(&self.path) {
            let label = self
                .path
                .segments()
                .last()
                .map(|s| s.key().as_str())
                .unwrap_or_else(|| self.path.object_key().as_str());
            container.tree_print(label, "", true);
        }
    }
}
