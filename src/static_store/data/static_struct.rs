use crate::StoreKey;
use crate::definition::{StructDefinition, StructItemDefinition};
use crate::shareable_string::ShareableString;
use crate::static_store::data::{StaticBasic, StaticTable};
use crate::store::TreePrint;
use crate::store::data::{Container, ContainerDefinition, ContainerItem};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StaticStructItem {
    Basic(StaticBasic),
    Table(StaticTable),
}

impl StaticStructItem {
    pub fn get_basic(&self) -> Option<&StaticBasic> {
        match self {
            StaticStructItem::Basic(basic) => Some(basic),
            _ => None,
        }
    }

    pub fn get_table(&self) -> Option<&StaticTable> {
        match self {
            StaticStructItem::Table(table) => Some(table),
            _ => None,
        }
    }

    pub fn definition(&self) -> StructItemDefinition {
        match self {
            StaticStructItem::Basic(basic) => {
                StructItemDefinition::Basic(basic.definition().clone())
            }
            StaticStructItem::Table(table) => {
                StructItemDefinition::Table(table.definition().clone())
            }
        }
    }

    pub fn hash(&self) -> [u8; 32] {
        match self {
            StaticStructItem::Basic(basic) => basic.hash(),
            StaticStructItem::Table(table) => table.hash(),
        }
    }
}

impl TreePrint for StaticStructItem {
    fn tree_print(&self, label: &str, prefix: &str, last: bool) {
        match self {
            StaticStructItem::Basic(basic) => basic.tree_print(label, prefix, last),
            StaticStructItem::Table(table) => table.tree_print(label, prefix, last),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StaticStruct {
    definition: StructDefinition,
    items: BTreeMap<ShareableString, StaticStructItem>,
    hash: [u8; 32],
}

impl StaticStruct {
    pub fn new<S: Into<ShareableString>>(
        description: S,
        items: BTreeMap<StoreKey, StaticStructItem>,
    ) -> Self {
        let items_vec: Vec<(StoreKey, StructItemDefinition)> = items
            .iter()
            .map(|(k, v)| (k.clone(), v.definition()))
            .collect();
        let definition = StructDefinition::new(description, items_vec);
        let items = items.into_iter().map(|(k, v)| (k.key, v)).collect();
        let mut s = Self {
            definition,
            items,
            hash: [0u8; 32],
        };
        s.update_hash();
        s
    }

    fn update_hash(&mut self) {
        let mut h = blake3::Hasher::new();

        h.update(&[0x01]);
        h.update(b"Struct");

        h.update(&(self.items.len() as u64).to_le_bytes());

        // Sort keys for deterministic hashing
        let mut keys: Vec<&ShareableString> = self.items.keys().collect();
        keys.sort_by(|a, b| a.as_ref().cmp(b.as_ref()));

        for key in keys {
            let value = self.items.get(key).unwrap();
            h.update(&key.current_blake3_hash());
            h.update(&value.hash());
        }

        let digest = h.finalize();
        self.hash = *digest.as_bytes();
    }

    pub fn hash(&self) -> [u8; 32] {
        self.hash
    }

    pub(crate) fn items(&self) -> &BTreeMap<ShareableString, StaticStructItem> {
        &self.items
    }

    pub fn get<S: AsRef<str>>(&self, key: S) -> Option<&StaticStructItem> {
        self.items.get(key.as_ref())
    }

    pub fn iter(&self) -> impl Iterator<Item = (&ShareableString, &StaticStructItem)> {
        self.items.iter()
    }

    pub fn definition(&self) -> &StructDefinition {
        &self.definition
    }
}

impl From<ContainerItem> for StaticStructItem {
    fn from(item: ContainerItem) -> Self {
        match item {
            ContainerItem::Basic(basic) => StaticStructItem::Basic(StaticBasic::from(&basic)),
            ContainerItem::Table(table) => StaticStructItem::Table(StaticTable::from(&table)),
            ContainerItem::Container(_) => {
                panic!("Nested containers not supported in StaticStructItem")
            }
        }
    }
}

impl From<ContainerItem> for StaticStruct {
    fn from(item: ContainerItem) -> Self {
        match item {
            ContainerItem::Container(c) => match c.definition() {
                ContainerDefinition::Struct(_) => StaticStruct::from(&c),
                _ => panic!("Expected Struct container"),
            },
            _ => panic!("Expected ContainerItem::Container for StaticStruct conversion"),
        }
    }
}

impl From<&Container> for StaticStruct {
    fn from(container: &Container) -> Self {
        let mut items = BTreeMap::new();
        for key in container.keys() {
            if let Ok(item) = container.get_item(&key) {
                let store_key = StoreKey::new(key.clone()).expect("Valid key from container");
                items.insert(store_key, StaticStructItem::from(item));
            }
        }
        let description = match container.definition() {
            ContainerDefinition::Struct(def) => def.description(),
            _ => panic!("Expected StructDefinition"),
        };
        Self::new(description, items)
    }
}

impl TreePrint for StaticStruct {
    fn tree_print(&self, label: &str, prefix: &str, last: bool) {
        let type_str = "Struct";
        println!(
            "{}{}{}: {} - {}",
            prefix,
            Self::branch_char(last),
            label,
            type_str,
            &self.definition.description()
        );
        let next_prefix = Self::next_prefix(prefix, last);
        let keys: Vec<_> = self.items.keys().collect();
        for (i, key) in keys.iter().enumerate() {
            let is_last = i == keys.len() - 1;
            self.items[*key].tree_print(key.as_str(), &next_prefix, is_last);
        }
    }
}
