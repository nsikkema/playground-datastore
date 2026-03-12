use crate::definition::ObjectDefinition;
use crate::shareable_string::ShareableString;
use crate::static_store::data::StaticProperty;
use crate::store::data::{Container, ContainerDefinition};
use crate::store::{CommonStoreTraitInternal, TreePrint};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StaticObject {
    definition: ObjectDefinition,
    items: BTreeMap<ShareableString, StaticProperty>,
    hash: [u8; 32],
}

impl StaticObject {
    pub fn new(
        definition: ObjectDefinition,
        items: BTreeMap<ShareableString, StaticProperty>,
    ) -> Self {
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
        h.update(b"Object");

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

    pub(crate) fn items(&self) -> &BTreeMap<ShareableString, StaticProperty> {
        &self.items
    }

    pub fn get<S: AsRef<str>>(&self, key: S) -> Option<&StaticProperty> {
        self.items.get(key.as_ref())
    }

    pub fn iter(&self) -> impl Iterator<Item = (&ShareableString, &StaticProperty)> {
        self.items.iter()
    }

    pub fn definition(&self) -> &ObjectDefinition {
        &self.definition
    }
}

impl From<&Container> for StaticObject {
    fn from(container: &Container) -> Self {
        let mut items = BTreeMap::new();
        for key in container.keys() {
            if let Ok(item) = container.get_item(&key) {
                items.insert(key, StaticProperty::from(item));
            }
        }
        let definition = match container.definition() {
            ContainerDefinition::Object(def) => def.clone(),
            _ => panic!("Expected ObjectDefinition"),
        };
        Self {
            definition,
            items,
            hash: container.current_blake3_hash(),
        }
    }
}

impl TreePrint for StaticObject {
    fn tree_print(&self, label: &str, prefix: &str, last: bool) {
        let type_str = "Object";
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
