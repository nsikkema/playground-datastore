use crate::StoreKey;
use crate::definition::{MapDefinition, StructDefinition, StructItemDefinition};
use crate::shareable_string::ShareableString;
use crate::static_store::data::StaticStruct;
use crate::store::TreePrint;
use crate::store::data::{Container, ContainerDefinition};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StaticMap {
    definition: MapDefinition,
    items: BTreeMap<ShareableString, StaticStruct>,
    hash: [u8; 32],
}

impl StaticMap {
    pub fn new<S: Into<ShareableString>>(
        description: S,
        items: BTreeMap<StoreKey, StaticStruct>,
    ) -> Self {
        let item_type = if let Some(first_item) = items.values().next() {
            first_item.definition().clone()
        } else {
            // If the map is empty, we need more information to infer the item type.
            // In a real scenario, we might want to ask for more information or have a default.
            // For now, we'll create an empty StructDefinition as a placeholder.
            StructDefinition::new("", Vec::<(StoreKey, StructItemDefinition)>::new())
        };

        let definition = MapDefinition::new(description, item_type);
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
        h.update(b"Map");

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

    pub(crate) fn items(&self) -> &BTreeMap<ShareableString, StaticStruct> {
        &self.items
    }

    pub fn get<S: AsRef<str>>(&self, key: S) -> Option<&StaticStruct> {
        self.items.get(key.as_ref())
    }

    pub fn iter(&self) -> impl Iterator<Item = (&ShareableString, &StaticStruct)> {
        self.items.iter()
    }

    pub fn definition(&self) -> &MapDefinition {
        &self.definition
    }
}

impl From<&Container> for StaticMap {
    fn from(container: &Container) -> Self {
        let mut items = BTreeMap::new();
        for key in container.keys() {
            if let Ok(item) = container.get_item(&key) {
                let store_key = StoreKey::new(key.clone()).expect("Valid key from container");
                items.insert(store_key, StaticStruct::from(item));
            }
        }
        let description = match container.definition() {
            ContainerDefinition::Map(def) => def.description(),
            _ => panic!("Expected MapDefinition"),
        };
        Self::new(description, items)
    }
}

impl TreePrint for StaticMap {
    fn tree_print(&self, label: &str, prefix: &str, last: bool) {
        let type_str = "Map";
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
