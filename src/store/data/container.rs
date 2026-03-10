use crate::StoreError;
use crate::definition::{
    MapDefinition, ObjectDefinition, PropertyDefinitionType, StructDefinition, StructItemDefinition,
};
use crate::shareable_string::{ShareableString, SharedStringStore};
use crate::store::{Basic, CommonStoreTraitInternal, StoreHashContainer, Table, TreePrint};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// An item stored within a `Container`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) enum ContainerItem {
    /// A basic data value.
    Basic(Basic),
    /// A table of data.
    Table(Table),
    /// A nested container.
    Container(Container),
}

/// The definition for a `Container`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContainerDefinition {
    /// A struct definition.
    Struct(StructDefinition),
    /// A map definition.
    Map(MapDefinition),
    /// An object definition.
    Object(ObjectDefinition),
}

impl Default for ContainerDefinition {
    fn default() -> Self {
        Self::Object(ObjectDefinition::default())
    }
}

/// A container that holds multiple `ContainerItem`s.
/// It can represent a struct, a map, or a top-level object.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Container {
    #[serde(skip)]
    definition: ContainerDefinition,
    items: HashMap<ShareableString, ContainerItem>,
    #[serde(skip)]
    blake3_hash: StoreHashContainer,
    locked: bool,
}

impl Container {
    /// Creates a new `Container` representing an object.
    pub(crate) fn new_object(definition: &ObjectDefinition) -> Self {
        let mut items = HashMap::new();
        for (key, item_definition) in definition.iter() {
            match item_definition.item_type() {
                PropertyDefinitionType::Basic(basic) => {
                    items.insert(key.clone(), ContainerItem::Basic(Basic::new(basic.clone())));
                }
                PropertyDefinitionType::Struct(_struct) => {
                    items.insert(
                        key.clone(),
                        ContainerItem::Container(Self::new_struct(_struct.clone())),
                    );
                }
                PropertyDefinitionType::Table(table) => {
                    items.insert(key.clone(), ContainerItem::Table(Table::new(table.clone())));
                }
                PropertyDefinitionType::Map(map) => {
                    items.insert(
                        key.clone(),
                        ContainerItem::Container(Self::new_map(map.clone())),
                    );
                }
            }
        }
        let mut container = Container {
            definition: ContainerDefinition::Object(definition.clone()),
            items,
            blake3_hash: StoreHashContainer::default(),
            locked: true,
        };

        container.update_blake3_hash();

        container
    }

    /// Returns a new `Container` with strings laundered through the provided store.
    pub(crate) fn launder(&self, store: &SharedStringStore) -> Self {
        let mut items = HashMap::new();
        for (key, item) in &self.items {
            let laundered_item = match item {
                ContainerItem::Basic(b) => ContainerItem::Basic(b.launder(store)),
                ContainerItem::Table(t) => ContainerItem::Table(t.launder(store)),
                ContainerItem::Container(c) => ContainerItem::Container(c.launder(store)),
            };
            items.insert(store.launder(key), laundered_item);
        }

        let laundered_definition = match &self.definition {
            ContainerDefinition::Struct(s) => ContainerDefinition::Struct(s.launder(store)),
            ContainerDefinition::Map(m) => ContainerDefinition::Map(m.launder(store)),
            ContainerDefinition::Object(o) => ContainerDefinition::Object(o.launder(store)),
        };

        let mut laundered = Self {
            definition: laundered_definition,
            items,
            blake3_hash: StoreHashContainer::new(),
            locked: self.locked,
        };
        laundered.update_blake3_hash();
        laundered
    }

    /// Creates a new `Container` representing a struct.
    pub(crate) fn new_struct(definition: StructDefinition) -> Self {
        let mut items = HashMap::new();
        for (key, item_definition) in definition.iter() {
            match item_definition {
                StructItemDefinition::Basic(basic) => {
                    items.insert(key.clone(), ContainerItem::Basic(Basic::new(basic.clone())));
                }
                StructItemDefinition::Table(table) => {
                    items.insert(key.clone(), ContainerItem::Table(Table::new(table.clone())));
                }
            }
        }

        let mut container = Container {
            definition: ContainerDefinition::Struct(definition),
            items,
            blake3_hash: StoreHashContainer::default(),
            locked: true,
        };

        container.update_blake3_hash();
        container
    }

    /// Creates a new `Container` representing a map.
    pub(crate) fn new_map(definition: MapDefinition) -> Self {
        let mut container = Container {
            definition: ContainerDefinition::Map(definition),
            items: HashMap::new(),
            blake3_hash: StoreHashContainer::default(),
            locked: false,
        };
        container.update_blake3_hash();

        container
    }

    /// Returns the keys of all items in the container.
    pub(crate) fn keys(&self) -> Vec<ShareableString> {
        self.items.keys().cloned().collect()
    }

    /// Returns a reference to the hash container.
    pub(crate) fn hash_container(&self) -> &StoreHashContainer {
        &self.blake3_hash
    }

    /// Returns a reference to the container's definition.
    pub(crate) fn definition(&self) -> &ContainerDefinition {
        &self.definition
    }

    /// Returns the item associated with the given key.
    pub(crate) fn get_item(&self, key: &ShareableString) -> Result<ContainerItem, StoreError> {
        self.items
            .get(key)
            .cloned()
            .ok_or(StoreError::PropertyNotFound)
    }

    /// Sets the item for the given key and updates the hash.
    pub(crate) fn set_item(
        &mut self,
        key: &ShareableString,
        item: ContainerItem,
    ) -> Result<(), StoreError> {
        if self.locked && !self.items.contains_key(key) {
            return Err(StoreError::PropertyNotFound);
        }
        self.items.insert(key.clone(), item);
        self.update_blake3_hash();
        Ok(())
    }

    /// Clears the hash of this container and all nested items.
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

    /// Updates the hash of this container and all nested items.
    pub(crate) fn update_blake3_hash_all(&mut self) {
        for item in self.items.values_mut() {
            match item {
                ContainerItem::Basic(item) => item.update_blake3_hash(),
                ContainerItem::Table(item) => item.update_blake3_hash(),
                ContainerItem::Container(item) => item.update_blake3_hash_all(),
            }
        }
        self.update_blake3_hash();
    }

    /// Restores the definition after deserialization.
    pub(crate) fn restore_definition(&mut self, definition: ContainerDefinition) {
        self.definition = definition.clone();
        match definition {
            ContainerDefinition::Struct(s) => {
                for (key, item) in self.items.iter_mut() {
                    if let Some(item_def) = s.get(key) {
                        match (item, item_def) {
                            (ContainerItem::Basic(b), StructItemDefinition::Basic(bd)) => {
                                b.restore_definition(bd.clone());
                            }
                            (ContainerItem::Table(t), StructItemDefinition::Table(td)) => {
                                t.restore_definition(td.clone());
                            }
                            _ => {}
                        }
                    }
                }
            }
            ContainerDefinition::Map(m) => {
                let item_def = m.item_type();
                for item in self.items.values_mut() {
                    if let ContainerItem::Container(c) = item {
                        c.restore_definition(ContainerDefinition::Struct(item_def.clone()));
                    }
                }
            }
            ContainerDefinition::Object(o) => {
                for (key, item) in self.items.iter_mut() {
                    if let Some(prop_def) = o.get(key) {
                        match item {
                            ContainerItem::Basic(b) => {
                                if let PropertyDefinitionType::Basic(bd) = prop_def.item_type() {
                                    b.restore_definition(bd.clone());
                                } else {
                                    panic!(
                                        "Definition mismatch for key {}: expected Basic",
                                        key.as_str()
                                    );
                                }
                            }
                            ContainerItem::Table(t) => {
                                if let PropertyDefinitionType::Table(td) = prop_def.item_type() {
                                    t.restore_definition(td.clone());
                                } else {
                                    panic!(
                                        "Definition mismatch for key {}: expected Table",
                                        key.as_str()
                                    );
                                }
                            }
                            ContainerItem::Container(c) => match prop_def.item_type() {
                                PropertyDefinitionType::Struct(sd) => {
                                    c.restore_definition(ContainerDefinition::Struct(sd.clone()));
                                }
                                PropertyDefinitionType::Map(md) => {
                                    c.restore_definition(ContainerDefinition::Map(md.clone()));
                                }
                                _ => {
                                    panic!(
                                        "Definition mismatch for key {}: expected Struct or Map",
                                        key.as_str()
                                    );
                                }
                            },
                        }
                    } else {
                        panic!("Key {} not found in object definition", key.as_str());
                    }
                }
            }
        }
    }
}

impl CommonStoreTraitInternal for Container {
    fn current_blake3_hash(&self) -> [u8; 32] {
        self.blake3_hash.get()
    }

    fn update_blake3_hash(&mut self) {
        let mut h = blake3::Hasher::new();

        h.update(&[0x01]);

        match &self.definition {
            ContainerDefinition::Struct(_) => {
                h.update(b"Struct");
            }
            ContainerDefinition::Map(_) => {
                h.update(b"Map");
            }
            ContainerDefinition::Object(_) => {
                h.update(b"Object");
            }
        }

        h.update(&(self.items.len() as u64).to_le_bytes());

        // Sort keys for deterministic hashing
        let mut keys: Vec<&ShareableString> = self.items.keys().collect();
        keys.sort_by(|a, b| a.as_ref().cmp(b.as_ref()));

        for key in keys {
            let value = self.items.get(key).unwrap();
            h.update(&key.current_blake3_hash());
            match value {
                ContainerItem::Basic(item) => h.update(&item.current_blake3_hash()),
                ContainerItem::Table(item) => h.update(&item.current_blake3_hash()),
                ContainerItem::Container(item) => h.update(&item.current_blake3_hash()),
            };
        }

        let digest = h.finalize();
        self.blake3_hash.set(*digest.as_bytes());
    }

    fn clear_hash(&mut self) {
        self.blake3_hash.clear();
    }
}

impl TreePrint for ContainerItem {
    fn tree_print(&self, label: &str, prefix: &str, last: bool) {
        match self {
            ContainerItem::Basic(b) => b.tree_print(label, prefix, last),
            ContainerItem::Table(t) => t.tree_print(label, prefix, last),
            ContainerItem::Container(c) => c.tree_print(label, prefix, last),
        }
    }
}

impl TreePrint for Container {
    fn tree_print(&self, label: &str, prefix: &str, last: bool) {
        let type_str = match &self.definition {
            ContainerDefinition::Struct(_) => "Struct",
            ContainerDefinition::Map(_) => "Map",
            ContainerDefinition::Object(_) => "Object",
        };
        let description = match &self.definition {
            ContainerDefinition::Struct(s) => s.description(),
            ContainerDefinition::Map(m) => m.description(),
            ContainerDefinition::Object(o) => o.description(),
        };

        println!(
            "{}{}{}: [{}] ({})",
            prefix,
            Self::branch_char(last),
            label,
            type_str,
            description
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
