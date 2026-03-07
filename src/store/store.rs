use crate::StoreError;
use crate::definition::ObjectDefinition;
use crate::shareable_string::{ShareableString, SharedStringStore};
use crate::store::data::{Basic, Container, ContainerDefinition, ContainerItem, Table};
use crate::store::traits::CommonStoreTraitInternal;
use crate::store::{BasicProxy, ContainerProxy, ObjectProxy, Segment, StorePath, TableProxy};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use std::sync::Arc;

/// Represents the internal state of the store for serialization.
#[derive(Debug, Serialize, Deserialize)]
struct StoreState {
    objects: HashMap<ShareableString, (ObjectDefinition, Container)>,
    #[serde(skip, default = "SharedStringStore::new")]
    string_store: SharedStringStore,
}

/// The internal implementation of the data store.
#[derive(Debug)]
pub(crate) struct StoreInternal {
    objects: RwLock<HashMap<ShareableString, Container>>,
    pub(crate) string_store: SharedStringStore,
    blake3_hash: RwLock<[u8; 32]>,
}

impl StoreInternal {
    /// Creates a new `StoreInternal`.
    fn new(string_store: SharedStringStore) -> Self {
        let store = StoreInternal {
            objects: HashMap::new().into(),
            string_store,
            blake3_hash: [0u8; 32].into(),
        };
        store.update_blake3_hash_locked();
        store
    }

    /// Updates the BLAKE3 hash while holding the lock.
    fn update_blake3_hash_locked(&self) {
        let objects = self.objects.read();
        self.update_blake3_hash(&objects);
    }

    /// Updates the BLAKE3 hash based on the provided objects.
    fn update_blake3_hash(&self, objects: &HashMap<ShareableString, Container>) {
        let mut h = blake3::Hasher::new();

        // Domain separation for this node/type.
        h.update(&[0x01]);
        h.update(b"StoreInternal");

        h.update(&(objects.len() as u64).to_le_bytes());

        // Sort keys for deterministic hashing
        let mut keys: Vec<&ShareableString> = objects.keys().collect();
        keys.sort_by(|a, b| a.as_ref().cmp(b.as_ref()));

        for key in keys {
            h.update(&key.current_blake3_hash());
            h.update(&objects.get(key).unwrap().current_blake3_hash());
        }

        let digest = h.finalize();
        let mut writer = self.blake3_hash.write();
        *writer = *digest.as_bytes();
    }

    /// Creates a new object in the store.
    pub(crate) fn create_object(
        &self,
        object_key: &ShareableString,
        definition: &ObjectDefinition,
    ) -> Result<(), StoreError> {
        let mut writer = self.objects.write();

        if writer.contains_key(object_key) {
            return Err(StoreError::ObjectKeyAlreadyExists);
        }

        let launder_definition = definition.launder(&self.string_store);
        writer.insert(
            object_key.clone(),
            Container::new_object(&launder_definition),
        );

        self.update_blake3_hash(&writer);

        Ok(())
    }

    /// Deletes an object from the store.
    pub(crate) fn delete_object(&self, object_key: &ShareableString) -> Result<(), StoreError> {
        let mut writer = self.objects.write();

        let mut object = writer
            .remove(object_key)
            .ok_or(StoreError::ObjectNotFound)?;

        object.clear_hash_all();

        self.update_blake3_hash(&writer);

        Ok(())
    }

    /// Adds an existing container as an object to the store.
    pub(crate) fn add_object(
        &self,
        object_key: &ShareableString,
        container: &Container,
    ) -> Result<(), StoreError> {
        let laundered_container = container.launder(&self.string_store);
        let mut writer = self.objects.write();

        if writer.contains_key(object_key) {
            return Err(StoreError::ObjectKeyAlreadyExists);
        }

        writer.insert(self.string_store.launder(object_key), laundered_container);

        self.update_blake3_hash(&writer);

        Ok(())
    }

    /// Returns a copy of the container for the specified object key.
    pub(crate) fn get_object(&self, object_key: &ShareableString) -> Result<Container, StoreError> {
        let reader = self.objects.read();

        reader
            .get(object_key)
            .cloned()
            .ok_or(StoreError::ObjectNotFound)
    }
}

/// The main data store.
#[derive(Debug, Clone)]
pub struct Store {
    internal: Arc<StoreInternal>,
}

impl Store {
    /// Creates a new, empty `Store`.
    pub fn new(string_store: SharedStringStore) -> Self {
        Self {
            internal: Arc::new(StoreInternal::new(string_store)),
        }
    }

    /// Creates a new object in the store and returns a proxy to it.
    pub fn create_object(
        &self,
        object_key: &ShareableString,
        definition: &ObjectDefinition,
    ) -> Result<ObjectProxy, StoreError> {
        self.internal.create_object(object_key, definition)?;
        let path = StorePath::builder(object_key.clone()).build();
        self.get_object(&path)
    }

    /// Returns an `ObjectProxy` for the specified path.
    pub fn get_object(&self, store_path: &StorePath) -> Result<ObjectProxy, StoreError> {
        let object_key = store_path.get_object_key();
        let container = self.internal.get_object(object_key)?;
        if let ContainerDefinition::Object(definition) = container.definition() {
            let keys = container.keys();
            let object_hash = container.hash_container().clone();
            let last_sync_hash = container.current_blake3_hash();
            Ok(ObjectProxy::new(
                store_path.clone(),
                self.clone(),
                definition.clone(),
                keys,
                object_hash,
                last_sync_hash,
            ))
        } else {
            Err(StoreError::ObjectNotFound)
        }
    }

    /// Internal method to get a container at the specified path.
    pub(crate) fn get_container_internal(
        &self,
        store_path: &StorePath,
    ) -> Result<Container, StoreError> {
        let object_key = store_path.get_object_key();
        let mut current_container = self.internal.get_object(object_key)?;

        for segment in store_path.get_segments() {
            match segment {
                Segment::Property(key) | Segment::MapKey(key) | Segment::StructItem(key) => {
                    match current_container.get_item(key)? {
                        ContainerItem::Container(c) => current_container = c,
                        _ => return Err(StoreError::PropertyNotFound),
                    }
                }
            }
        }
        Ok(current_container)
    }

    /// Returns a `ContainerProxy` for the specified path.
    pub fn get_container(&self, store_path: &StorePath) -> Result<ContainerProxy, StoreError> {
        let container = self.get_container_internal(store_path)?;
        let keys = container.keys();
        let object_hash = container.hash_container().clone();
        let last_sync_hash = container.current_blake3_hash();

        Ok(ContainerProxy::new(
            store_path.clone(),
            self.clone(),
            container.definition().clone(),
            keys,
            object_hash,
            last_sync_hash,
        ))
    }

    /// Returns a `TableProxy` for the specified path.
    pub fn get_table(&self, store_path: &StorePath) -> Result<TableProxy, StoreError> {
        let segments = store_path.get_segments();
        if segments.is_empty() {
            return Err(StoreError::InvalidPath);
        }

        let (parent_path, last_segment) = split_path(store_path)?;
        let container = self.get_container_internal(&parent_path)?;

        if let ContainerItem::Table(table) = container.get_item(last_segment.key())? {
            Ok(TableProxy::new(store_path.clone(), self.clone(), table))
        } else {
            Err(StoreError::PropertyNotFound)
        }
    }

    /// Sets the table data at the specified path.
    pub(crate) fn set_table(&self, store_path: &StorePath, data: &Table) -> Result<(), StoreError> {
        let (parent_path, last_segment) = split_path(store_path)?;
        let mut container = self.get_container_internal(&parent_path)?;
        container.set_item(last_segment.key(), ContainerItem::Table(data.clone()))?;

        self.update_container_at_path(&parent_path, container)
    }

    /// Returns a `BasicProxy` for the specified path.
    pub fn get_basic(&self, store_path: &StorePath) -> Result<BasicProxy, StoreError> {
        let segments = store_path.get_segments();
        if segments.is_empty() {
            return Err(StoreError::InvalidPath);
        }

        let (parent_path, last_segment) = split_path(store_path)?;
        let container = self.get_container_internal(&parent_path)?;

        if let ContainerItem::Basic(basic) = container.get_item(last_segment.key())? {
            Ok(BasicProxy::new(store_path.clone(), self.clone(), basic))
        } else {
            Err(StoreError::PropertyNotFound)
        }
    }

    /// Sets the basic data at the specified path.
    pub(crate) fn set_basic(&self, store_path: &StorePath, data: &Basic) -> Result<(), StoreError> {
        let (parent_path, last_segment) = split_path(store_path)?;
        let mut container = self.get_container_internal(&parent_path)?;
        container.set_item(last_segment.key(), ContainerItem::Basic(data.clone()))?;

        self.update_container_at_path(&parent_path, container)
    }

    /// Updates the container at the specified path and propagates changes up to the object root.
    pub(crate) fn update_container_at_path(
        &self,
        path: &StorePath,
        mut container: Container,
    ) -> Result<(), StoreError> {
        container.update_blake3_hash();
        let segments = path.get_segments();

        if segments.is_empty() {
            // Updating a top-level object
            {
                let mut writer = self.internal.objects.write();
                writer.insert(path.get_object_key().clone(), container);
                self.internal.update_blake3_hash(&writer);
            }
            return Ok(());
        }

        // We need to update all the way up to the root object
        let (parent_path, last_segment) = split_path(path)?;
        let mut parent_container = self.get_container_internal(&parent_path)?;
        parent_container.set_item(last_segment.key(), ContainerItem::Container(container))?;

        self.update_container_at_path(&parent_path, parent_container)
    }

    /// Deletes the object with the specified key.
    pub fn delete_object(&self, object_key: &ShareableString) -> Result<(), StoreError> {
        self.internal.delete_object(object_key)
    }

    /// Returns a list of all object keys in the store.
    pub fn get_object_keys(&self) -> Result<Vec<ShareableString>, StoreError> {
        let reader = self.internal.objects.read();
        Ok(reader.keys().cloned().collect())
    }

    /// Returns the overall BLAKE3 hash of the store.
    pub fn get_blake3_hash(&self) -> [u8; 32] {
        *self.internal.blake3_hash.read()
    }

    /// Saves the store state to a file.
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), StoreError> {
        let objects = self.internal.objects.read();
        let mut objects_state = HashMap::new();
        for (key, container) in objects.iter() {
            if let ContainerDefinition::Object(def) = container.definition() {
                objects_state.insert(key.clone(), (def.clone(), container.clone()));
            }
        }

        let state = StoreState {
            objects: objects_state,
            string_store: self.internal.string_store.clone(),
        };

        let json = serde_json::to_string(&state).map_err(|_| StoreError::IOError)?;
        let mut file = File::create(path).map_err(|_| StoreError::IOError)?;
        file.write_all(json.as_bytes())
            .map_err(|_| StoreError::IOError)?;
        Ok(())
    }

    /// Loads a store from a file.
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, StoreError> {
        let mut file = File::open(path).map_err(|_| StoreError::IOError)?;
        let mut json = String::new();
        file.read_to_string(&mut json)
            .map_err(|_| StoreError::IOError)?;
        let state: StoreState = serde_json::from_str(&json).map_err(|_| StoreError::IOError)?;

        let mut objects = HashMap::new();
        let string_store = state.string_store;
        for (key, (def, container)) in state.objects {
            let key = string_store.launder(key);
            let mut container = container.launder(&string_store);
            container.restore_definition(ContainerDefinition::Object(def));
            container.update_blake3_hash_all();
            objects.insert(key, container);
        }

        let internal = StoreInternal {
            objects: RwLock::new(objects),
            string_store,
            blake3_hash: RwLock::new([0u8; 32]),
        };
        internal.update_blake3_hash_locked();

        Ok(Store {
            internal: Arc::new(internal),
        })
    }

    /// Launders a `ShareableString` through the store's string store.
    pub fn launder(&self, string: ShareableString) -> ShareableString {
        self.internal.string_store.launder(string)
    }

    /// Copies an object from another store.
    pub fn copy_object(
        &self,
        object_key: &ShareableString,
        source_store: &Store,
        source_object_key: &ShareableString,
    ) -> Result<ObjectProxy, StoreError> {
        let container = source_store.internal.get_object(source_object_key)?;
        self.internal.add_object(object_key, &container)?;
        let path = StorePath::builder(object_key.clone()).build();
        self.get_object(&path)
    }
}

/// Splits a path into its parent path and the last segment.
fn split_path(path: &StorePath) -> Result<(StorePath, Segment), StoreError> {
    let mut segments = path.get_segments().clone();
    let last_segment = segments.pop().ok_or(StoreError::InvalidPath)?;
    let mut parent_path = StorePath::builder(path.get_object_key().clone()).build();
    for segment in segments {
        match segment {
            Segment::Property(key) => {
                parent_path = parent_path.push_property(key);
            }
            Segment::MapKey(key) => {
                parent_path = parent_path.push_map_key(key);
            }
            Segment::StructItem(key) => {
                parent_path = parent_path.push_struct_item(key);
            }
        }
    }
    Ok((parent_path, last_segment))
}

impl Segment {
    /// Returns the key associated with the segment.
    fn key(&self) -> &ShareableString {
        match self {
            Segment::Property(key) => key,
            Segment::MapKey(key) => key,
            Segment::StructItem(key) => key,
        }
    }
}
