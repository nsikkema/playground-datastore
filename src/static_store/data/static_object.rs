use crate::StoreError;
use crate::StoreKey;
use crate::definition::ObjectDefinition;
use crate::shareable_string::ShareableString;
use crate::static_store::data::StaticProperty;
use crate::store::TreePrint;
use crate::store::data::Object;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StaticObject {
    definition: ObjectDefinition,
    items: BTreeMap<ShareableString, StaticProperty>,
    hash: [u8; 32],
}

impl StaticObject {
    pub fn new<S: Into<ShareableString>>(
        description: S,
        items: BTreeMap<StoreKey, StaticProperty>,
    ) -> Result<Self, StoreError> {
        let mut builder = ObjectDefinition::builder(description);
        for (k, v) in &items {
            builder.insert(k.clone(), v.definition());
        }
        let definition = builder.finish();
        let items = items.into_iter().map(|(k, v)| (k.key, v)).collect();
        let mut s = Self {
            definition,
            items,
            hash: [0u8; 32],
        };
        s.update_hash();
        Ok(s)
    }

    fn update_hash(&mut self) {
        let mut h = blake3::Hasher::new();

        h.update(&[0x01]);
        h.update(b"Object");

        h.update(&(self.items.len() as u64).to_le_bytes());

        for (key, item) in &self.items {
            h.update(&key.current_blake3_hash());
            h.update(&item.hash());
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

impl TryFrom<&Object> for StaticObject {
    type Error = StoreError;

    fn try_from(object: &Object) -> Result<Self, Self::Error> {
        let mut items = BTreeMap::new();
        for key in object.keys() {
            if let Ok(item) = object.get_item(&key) {
                let store_key = StoreKey::new(key.clone())?;
                items.insert(store_key, StaticProperty::try_from(item)?);
            }
        }
        let description = object.definition().description();
        Self::new(description, items)
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
