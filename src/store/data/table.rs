use crate::StoreError;
use crate::definition::TableDefinition;
use crate::shareable_string::{ShareableString, SharedStringStore};
use crate::static_store::data::StaticTable;
use crate::store::{CommonStoreTraitInternal, StoreHashContainer, TreePrint};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Represents a table of data in the store.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Table {
    #[serde(skip)]
    definition: TableDefinition,
    rows: Vec<BTreeMap<ShareableString, ShareableString>>,
    #[serde(skip)]
    blake3_hash: StoreHashContainer,
}

impl Table {
    /// Creates a new `Table` instance with the given definition.
    pub(crate) fn new(definition: TableDefinition) -> Self {
        let mut s = Self {
            definition,
            rows: Vec::new(),
            blake3_hash: StoreHashContainer::new(),
        };
        s.update_blake3_hash();
        s
    }

    /// Returns a new `Table` instance with strings laundered through the provided store.
    pub(crate) fn launder(&self, store: &SharedStringStore) -> Self {
        let mut s = Self {
            definition: self.definition.launder(store),
            rows: self
                .rows
                .iter()
                .map(|row| {
                    row.iter()
                        .map(|(k, v)| (store.launder(k), store.launder(v)))
                        .collect()
                })
                .collect(),
            blake3_hash: StoreHashContainer::new(),
        };
        s.update_blake3_hash();
        s
    }

    /// Returns a reference to the table definition.
    pub fn definition(&self) -> &TableDefinition {
        &self.definition
    }

    /// Returns the number of rows in the table.
    pub fn row_count(&self) -> usize {
        self.rows.len()
    }

    /// Returns the number of columns in the table.
    pub fn column_count(&self) -> usize {
        self.definition.count()
    }

    /// Appends a new row with default values to the table.
    pub(crate) fn append_row(&mut self) {
        let default_row = self.default_row();
        self.rows.push(default_row);
        self.update_blake3_hash();
    }

    /// Inserts a new row with default values at the specified index.
    pub(crate) fn insert_row(&mut self, index: usize) {
        if index >= self.rows.len() {
            self.append_row();
            return;
        }

        let default_row = self.default_row();
        self.rows.insert(index, default_row);
        self.update_blake3_hash();
    }

    /// Removes the row at the specified index.
    pub(crate) fn remove_row(&mut self, index: usize) -> Result<(), StoreError> {
        if index >= self.rows.len() {
            return Err(StoreError::IndexNotFound);
        }

        self.rows.remove(index);
        self.update_blake3_hash();
        Ok(())
    }

    /// Returns a reference to the row at the specified index.
    pub fn row(&self, index: usize) -> Option<&BTreeMap<ShareableString, ShareableString>> {
        self.rows.get(index)
    }

    /// Sets the value of a cell in the table.
    pub(crate) fn set_cell(
        &mut self,
        row_index: usize,
        column_key: &str,
        value: ShareableString,
    ) -> Result<(), StoreError> {
        if let Some(row) = self.rows.get_mut(row_index) {
            if let Some(cell) = row.get_mut(column_key) {
                *cell = value;
                self.update_blake3_hash();
                Ok(())
            } else {
                Err(StoreError::KeyNotFound)
            }
        } else {
            Err(StoreError::IndexNotFound)
        }
    }

    /// Sets the values of a row in the table.
    pub(crate) fn set_row(
        &mut self,
        row_index: usize,
        values: Vec<ShareableString>,
    ) -> Result<(), StoreError> {
        if let Some(row) = self.rows.get_mut(row_index) {
            for (value, key) in values.into_iter().zip(self.definition.keys()) {
                if let Some(cell) = row.get_mut(key) {
                    *cell = value;
                }
            }
            self.update_blake3_hash();
            Ok(())
        } else {
            Err(StoreError::IndexNotFound)
        }
    }

    /// Returns a new row populated with default values according to the table definition.
    fn default_row(&self) -> BTreeMap<ShareableString, ShareableString> {
        self.definition
            .iter()
            .map(|column| (column.0.clone(), column.1.default_value().clone()))
            .collect()
    }

    /// Restores the definition after deserialization.
    pub(crate) fn restore_definition(&mut self, definition: TableDefinition) {
        self.definition = definition;
    }

    pub(crate) fn update_from_static(&mut self, static_table: &StaticTable) {
        self.rows = static_table.rows().clone();
        self.blake3_hash.set(static_table.hash());
    }
}

impl From<&StaticTable> for Table {
    fn from(static_table: &StaticTable) -> Self {
        let s = Self {
            definition: static_table.definition().clone(),
            rows: static_table.rows().clone(),
            blake3_hash: StoreHashContainer::new(),
        };
        s.blake3_hash.set(static_table.hash());
        s
    }
}

impl CommonStoreTraitInternal for Table {
    fn current_blake3_hash(&self) -> [u8; 32] {
        self.blake3_hash.get()
    }

    fn update_blake3_hash(&mut self) {
        let mut h = blake3::Hasher::new();

        // Domain separation for this node/type.
        h.update(&[0x01]);
        h.update(b"Table");

        h.update(&(self.rows.len() as u64).to_le_bytes());
        for row in &self.rows {
            h.update(&(row.len() as u64).to_le_bytes());
            for (key, value) in row {
                h.update(&key.current_blake3_hash());
                h.update(&value.current_blake3_hash());
            }
        }

        let digest = h.finalize();
        self.blake3_hash.set(*digest.as_bytes());
    }

    fn clear_hash(&mut self) {
        self.blake3_hash.clear()
    }
}

impl TreePrint for Table {
    fn tree_print(&self, label: &str, prefix: &str, last: bool) {
        println!(
            "{}{}{}: [Table, {} rows] ({})",
            prefix,
            Self::branch_char(last),
            label,
            self.rows.len(),
            self.definition.description()
        );
        let next_prefix = Self::next_prefix(prefix, last);
        for (i, row) in self.rows.iter().enumerate() {
            let row_last = i == self.rows.len() - 1;
            println!("{}{}Row {}", next_prefix, Self::branch_char(row_last), i);
            let row_prefix = Self::next_prefix(&next_prefix, row_last);
            let keys: Vec<_> = row.keys().collect();
            for (j, key) in keys.iter().enumerate() {
                let cell_last = j == keys.len() - 1;
                println!(
                    "{}{}{}: {}",
                    row_prefix,
                    Self::branch_char(cell_last),
                    key,
                    row.get(*key).unwrap()
                );
            }
        }
    }
}
