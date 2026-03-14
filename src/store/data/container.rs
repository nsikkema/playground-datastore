use crate::StoreError;
use crate::definition::{
    MapDefinition, ObjectDefinition, PropertyDefinitionType, StructDefinition, StructItemDefinition,
};
use crate::shareable_string::{ShareableString, SharedStringStore};
use crate::static_store::data::{
    StaticMap, StaticObject, StaticProperty, StaticStruct, StaticStructItem,
};
use crate::store::{Basic, CommonStoreTraitInternal, StoreHashContainer, Table, TreePrint};
use std::collections::HashMap;

/// An item stored within a `Container`.
#[derive(Debug, Clone)]
pub(crate) enum ContainerItem {
    /// A basic data value.
    Basic(Basic),
    /// A table of data.
    Table(Table),
    /// A nested container.
    Container(Container),
}

/// The definition for a `Container`.
#[derive(Debug, Clone)]
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
#[derive(Debug, Clone)]
pub(crate) struct Container {
    definition: ContainerDefinition,
    items: HashMap<ShareableString, ContainerItem>,
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

    pub(crate) fn update_from_static(
        &mut self,
        items: &std::collections::BTreeMap<ShareableString, StaticProperty>,
    ) {
        for (key, static_property) in items {
            if let Some(item) = self.items.get_mut(key)
                && item.matches_static(static_property)
            {
                item.update_from_static(static_property);
                continue;
            }

            // If doesn't exist or type mismatch, replace it.
            self.items
                .insert(key.clone(), ContainerItem::from(static_property));
        }
        self.update_blake3_hash();
    }

    pub(crate) fn update_from_static_struct(
        &mut self,
        items: &std::collections::BTreeMap<ShareableString, StaticStructItem>,
    ) {
        for (key, static_item) in items {
            if let Some(item) = self.items.get_mut(key) {
                match (item, static_item) {
                    (ContainerItem::Basic(b), StaticStructItem::Basic(sb)) => {
                        b.update_from_static(sb);
                    }
                    (ContainerItem::Table(t), StaticStructItem::Table(st)) => {
                        t.update_from_static(st);
                    }
                    _ => {
                        self.items
                            .insert(key.clone(), ContainerItem::from(static_item));
                    }
                }
            } else {
                self.items
                    .insert(key.clone(), ContainerItem::from(static_item));
            }
        }
        self.update_blake3_hash();
    }

    pub(crate) fn update_from_static_map(
        &mut self,
        items: &std::collections::BTreeMap<ShareableString, StaticStruct>,
    ) {
        for (key, static_struct) in items {
            if let Some(ContainerItem::Container(c)) = self.items.get_mut(key) {
                c.update_from_static_struct(static_struct.items());
            } else {
                self.items
                    .insert(key.clone(), ContainerItem::from(static_struct));
            }
        }
        self.update_blake3_hash();
    }
}

impl From<&StaticObject> for Container {
    fn from(static_object: &StaticObject) -> Self {
        let items = static_object
            .items()
            .iter()
            .map(|(k, v)| (k.clone(), ContainerItem::from(v)))
            .collect();
        let c = Self {
            definition: ContainerDefinition::Object(static_object.definition().clone()),
            items,
            blake3_hash: StoreHashContainer::new(),
            locked: true,
        };
        c.blake3_hash.set(static_object.hash());
        c
    }
}

impl From<&StaticStruct> for Container {
    fn from(static_struct: &StaticStruct) -> Self {
        let items = static_struct
            .items()
            .iter()
            .map(|(k, v)| (k.clone(), ContainerItem::from(v)))
            .collect();
        let c = Self {
            definition: ContainerDefinition::Struct(static_struct.definition().clone()),
            items,
            blake3_hash: StoreHashContainer::new(),
            locked: true,
        };
        c.blake3_hash.set(static_struct.hash());
        c
    }
}

impl From<&StaticMap> for Container {
    fn from(static_map: &StaticMap) -> Self {
        let items = static_map
            .items()
            .iter()
            .map(|(k, v)| (k.clone(), ContainerItem::from(v)))
            .collect();
        let c = Self {
            definition: ContainerDefinition::Map(static_map.definition().clone()),
            items,
            blake3_hash: StoreHashContainer::new(),
            locked: false,
        };
        c.blake3_hash.set(static_map.hash());
        c
    }
}

impl From<&StaticStructItem> for ContainerItem {
    fn from(static_item: &StaticStructItem) -> Self {
        match static_item {
            StaticStructItem::Basic(b) => ContainerItem::Basic(Basic::from(b)),
            StaticStructItem::Table(t) => ContainerItem::Table(Table::from(t)),
        }
    }
}

impl From<&StaticStruct> for ContainerItem {
    fn from(static_struct: &StaticStruct) -> Self {
        ContainerItem::Container(Container::from(static_struct))
    }
}

impl From<&StaticProperty> for ContainerItem {
    fn from(static_property: &StaticProperty) -> Self {
        match static_property {
            StaticProperty::Basic(b) => ContainerItem::Basic(Basic::from(b)),
            StaticProperty::Table(t) => ContainerItem::Table(Table::from(t)),
            StaticProperty::Struct(s) => ContainerItem::Container(Container::from(s)),
            StaticProperty::Map(m) => ContainerItem::Container(Container::from(m)),
        }
    }
}

impl ContainerItem {
    pub(crate) fn matches_static(&self, static_property: &StaticProperty) -> bool {
        match (self, static_property) {
            (ContainerItem::Basic(b), StaticProperty::Basic(sb)) => {
                b.definition() == sb.definition()
            }
            (ContainerItem::Table(t), StaticProperty::Table(st)) => {
                t.definition() == st.definition()
            }
            (ContainerItem::Container(c), StaticProperty::Struct(ss)) => {
                matches!(c.definition(), ContainerDefinition::Struct(def) if def == ss.definition())
            }
            (ContainerItem::Container(c), StaticProperty::Map(sm)) => {
                matches!(c.definition(), ContainerDefinition::Map(def) if def == sm.definition())
            }
            _ => false,
        }
    }

    pub(crate) fn update_from_static(&mut self, static_property: &StaticProperty) {
        match (self, static_property) {
            (ContainerItem::Basic(b), StaticProperty::Basic(sb)) => {
                b.update_from_static(sb);
            }
            (ContainerItem::Table(t), StaticProperty::Table(st)) => {
                t.update_from_static(st);
            }
            (ContainerItem::Container(c), StaticProperty::Struct(ss)) => {
                c.update_from_static_struct(ss.items());
            }
            (ContainerItem::Container(c), StaticProperty::Map(sm)) => {
                c.update_from_static_map(sm.items());
            }
            _ => panic!(
                "Type mismatch in update_from_static - should have been checked by matches_static"
            ),
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
