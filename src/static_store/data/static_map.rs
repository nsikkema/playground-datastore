use crate::StoreError;
use crate::StoreKey;
use crate::definition::MapDefinition;
use crate::shareable_string::ShareableString;
use crate::static_store::data::StaticStruct;
use crate::store::TreePrint;
use crate::store::data::{Container, ContainerDefinition};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StaticMap {
    definition: MapDefinition,
    items: BTreeMap<StoreKey, StaticStruct>,
    hash: [u8; 32],
}

impl StaticMap {
    pub fn new<S: Into<ShareableString>>(
        description: S,
        items: BTreeMap<StoreKey, StaticStruct>,
    ) -> Result<Self, StoreError> {
        let item_type = if let Some(first_item) = items.values().next() {
            let first_def = first_item.definition().clone();
            for item in items.values().skip(1) {
                if item.definition() != first_def {
                    return Err(StoreError::SchemaMismatch(format!(
                        "StaticMap items must have the same struct definition. Expected: {:?}, Found: {:?}",
                        first_def,
                        item.definition()
                    )));
                }
            }
            first_def
        } else {
            return Err(StoreError::MissingSchema(
                "StaticMap cannot be empty as item type cannot be inferred".into(),
            ));
        };

        let definition = MapDefinition::new(description, item_type);
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
        h.update(b"Map");

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

    pub(crate) fn items(&self) -> &BTreeMap<StoreKey, StaticStruct> {
        &self.items
    }

    pub fn get<S: Into<ShareableString>>(&self, key: S) -> Option<&StaticStruct> {
        self.items.get(&key.into())
    }

    pub fn iter(&self) -> impl Iterator<Item = (&StoreKey, &StaticStruct)> {
        self.items.iter()
    }

    pub fn definition(&self) -> &MapDefinition {
        &self.definition
    }
}

impl TryFrom<&Container> for StaticMap {
    type Error = StoreError;

    fn try_from(container: &Container) -> Result<Self, Self::Error> {
        let mut items = BTreeMap::new();
        for key in container.keys() {
            if let Ok(item) = container.get_item(&key) {
                items.insert(key.clone(), StaticStruct::try_from(item)?);
            }
        }
        let description = match container.definition() {
            ContainerDefinition::Map(def) => def.description(),
            _ => return Err(StoreError::SchemaMismatch("Expected MapDefinition".into())),
        };
        Self::new(description, items)
    }
}

impl TreePrint for StaticMap {
    fn tree_print(
        &self,
        f: &mut std::fmt::Formatter<'_>,
        label: &str,
        prefix: &str,
        last: bool,
    ) -> std::fmt::Result {
        let type_str = "Map";
        writeln!(
            f,
            "{}{}{}: {} - {}",
            prefix,
            Self::branch_char(prefix, last),
            label,
            type_str,
            &self.definition.description()
        )?;
        let next_prefix = Self::next_prefix(prefix, last);
        let entries: Vec<_> = self.items.iter().collect();
        for (i, (key, item)) in entries.iter().enumerate() {
            let is_last = i == entries.len() - 1;
            item.tree_print(f, key.as_str(), &next_prefix, is_last)?;
        }
        Ok(())
    }
}
