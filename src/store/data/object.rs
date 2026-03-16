use crate::definition::{ObjectDefinition, PropertyDefinitionType};
use crate::shareable_string::SharedStringStore;
use crate::static_store::data::StaticObject;
use crate::store::{
    Basic, CommonStoreTraitInternal, Container, ContainerItem, StoreHashContainer, Table, TreePrint,
};
use crate::{StoreError, StoreKey};
use std::collections::HashMap;

/// A top-level object in the store.
#[derive(Debug, Clone)]
pub struct Object {
    definition: ObjectDefinition,
    items: HashMap<StoreKey, ContainerItem>,
    blake3_hash: StoreHashContainer,
}

impl Object {
    /// Creates a new `Object` from a definition.
    pub(crate) fn new(definition: &ObjectDefinition) -> Self {
        let mut items = HashMap::new();
        for (key, item_definition) in definition.iter() {
            match item_definition.item_type() {
                PropertyDefinitionType::Basic(basic) => {
                    items.insert(key.clone(), ContainerItem::Basic(Basic::new(basic.clone())));
                }
                PropertyDefinitionType::Struct(_struct) => {
                    items.insert(
                        key.clone(),
                        ContainerItem::Container(Container::new_struct(_struct.clone())),
                    );
                }
                PropertyDefinitionType::Table(table) => {
                    items.insert(key.clone(), ContainerItem::Table(Table::new(table.clone())));
                }
                PropertyDefinitionType::Map(map) => {
                    items.insert(
                        key.clone(),
                        ContainerItem::Container(Container::new_map(map.clone())),
                    );
                }
            }
        }
        let mut object = Object {
            definition: definition.clone(),
            items,
            blake3_hash: StoreHashContainer::default(),
        };

        object.update_blake3_hash();

        object
    }

    /// Returns a new `Object` with strings laundered through the provided store.
    pub(crate) fn launder(&self, store: &SharedStringStore) -> Self {
        let mut items = HashMap::new();
        for (key, item) in &self.items {
            let laundered_item = match item {
                ContainerItem::Basic(b) => ContainerItem::Basic(b.launder(store)),
                ContainerItem::Table(t) => ContainerItem::Table(t.launder(store)),
                ContainerItem::Container(c) => ContainerItem::Container(c.launder(store)),
            };
            items.insert(key.launder(store), laundered_item);
        }

        let laundered_definition = self.definition.launder(store);

        let mut laundered = Self {
            definition: laundered_definition,
            items,
            blake3_hash: StoreHashContainer::new(),
        };
        laundered.update_blake3_hash();
        laundered
    }

    /// Returns the keys of all items in the object.
    pub(crate) fn keys(&self) -> Vec<StoreKey> {
        self.items.keys().cloned().collect()
    }

    /// Returns a reference to the hash container.
    pub(crate) fn hash_container(&self) -> &StoreHashContainer {
        &self.blake3_hash
    }

    /// Returns a reference to the object's definition.
    pub(crate) fn definition(&self) -> &ObjectDefinition {
        &self.definition
    }

    /// Returns the item associated with the given key.
    pub(crate) fn get_item<K: AsRef<str>>(&self, key: K) -> Result<ContainerItem, StoreError> {
        self.items
            .get(key.as_ref())
            .cloned()
            .ok_or(StoreError::PropertyNotFound)
    }

    /// Sets the item for the given key and updates the hash.
    pub(crate) fn set_item(
        &mut self,
        key: &StoreKey,
        item: ContainerItem,
    ) -> Result<(), StoreError> {
        if !self.items.contains_key(key) {
            return Err(StoreError::PropertyNotFound);
        }
        self.items.insert(key.clone(), item);
        self.update_blake3_hash();
        Ok(())
    }

    pub(crate) fn update_from_static(
        &mut self,
        items: &std::collections::BTreeMap<StoreKey, crate::static_store::data::StaticProperty>,
    ) -> Result<(), crate::StoreError> {
        for (key, static_property) in items {
            if let Some(item) = self.items.get_mut(key)
                && item.matches_static(static_property)
            {
                item.update_from_static(static_property)?;
                continue;
            }

            // If doesn't exist or type mismatch, replace it.
            self.items
                .insert(key.clone(), ContainerItem::from(static_property));
        }
        self.update_blake3_hash();
        Ok(())
    }

    /// Clears the hash of this object and all nested items.
    pub(crate) fn clear_hash_all(&mut self) {
        self.clear_hash();
        for item in self.items.values_mut() {
            match item {
                ContainerItem::Basic(item) => item.clear_hash(),
                ContainerItem::Table(item) => item.clear_hash(),
                ContainerItem::Container(item) => item.clear_hash_all(),
            }
        }
    }
}

impl From<&StaticObject> for Object {
    fn from(static_object: &StaticObject) -> Self {
        let items = static_object
            .items()
            .iter()
            .map(|(k, v)| (k.clone(), ContainerItem::from(v)))
            .collect();
        let o = Self {
            definition: static_object.definition().clone(),
            items,
            blake3_hash: StoreHashContainer::new(),
        };
        o.blake3_hash.set(static_object.hash());
        o
    }
}

impl CommonStoreTraitInternal for Object {
    fn current_blake3_hash(&self) -> [u8; 32] {
        self.blake3_hash.get()
    }

    fn update_blake3_hash(&mut self) {
        let mut h = blake3::Hasher::new();

        h.update(&[0x01]);
        h.update(b"Object");

        h.update(&(self.items.len() as u64).to_le_bytes());

        // Sort keys for deterministic hashing
        let mut keys: Vec<&StoreKey> = self.items.keys().collect();
        keys.sort_by(|a, b| a.as_str().cmp(b.as_str()));

        for key in keys {
            h.update(&key.current_blake3_hash());
            if let Some(value) = self.items.get(key) {
                h.update(&value.current_blake3_hash());
            }
        }

        let digest = h.finalize();
        self.blake3_hash.set(*digest.as_bytes());
    }

    fn clear_hash(&mut self) {
        self.blake3_hash.clear();
    }
}

impl TreePrint for Object {
    fn tree_print(&self, label: &str, prefix: &str, last: bool) {
        println!(
            "{}{}{}: [Object] ({})",
            prefix,
            Self::branch_char(last),
            label,
            self.definition.description()
        );

        let next_prefix = Self::next_prefix(prefix, last);
        let mut keys: Vec<_> = self.items.keys().collect();
        keys.sort();

        for (i, key) in keys.iter().enumerate() {
            let item_last = i == keys.len() - 1;
            if let Some(item) = self.items.get(*key) {
                item.tree_print(key.as_str(), &next_prefix, item_last);
            }
        }
    }
}
