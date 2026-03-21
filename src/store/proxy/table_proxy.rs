use crate::definition::TableDefinition;
use crate::shareable_string::ShareableString;
use crate::store::traits::TreePrint;
use crate::store::{CommonStoreTraitInternal, ObjectProxy, ProxyStoreTrait, Store, Table};
use crate::{StoreError, StoreKey, StorePath};
use std::collections::BTreeMap;

/// A proxy for a table in the store.
pub struct TableProxy {
    path: StorePath,
    store: Store,
    data: Table,
}

impl TableProxy {
    /// Creates a new `TableProxy`.
    pub(crate) fn new(path: StorePath, store: Store, data: Table) -> Self {
        Self { path, store, data }
    }

    /// Returns a reference to the table definition.
    pub fn definition(&self) -> &TableDefinition {
        self.data.definition()
    }

    /// Returns the number of rows in the table.
    pub fn row_count(&self) -> usize {
        self.data.row_count()
    }

    /// Returns the number of columns in the table.
    pub fn column_count(&self) -> usize {
        self.data.column_count()
    }

    /// Appends a new row to the table.
    pub fn append_row(&mut self) {
        self.data.append_row()
    }

    /// Inserts a new row at the specified index.
    pub fn insert_row(&mut self, index: usize) {
        self.data.insert_row(index)
    }

    /// Removes the row at the specified index.
    pub fn remove_row(&mut self, index: usize) -> Result<(), StoreError> {
        self.data.remove_row(index)
    }

    /// Returns a reference to the row at the specified index.
    pub fn row(&self, index: usize) -> Option<&BTreeMap<StoreKey, ShareableString>> {
        self.data.row(index)
    }

    /// Sets the value of a cell in the table.
    pub fn set_cell<K: AsRef<str>, S: Into<ShareableString>>(
        &mut self,
        row_index: usize,
        column_key: K,
        value: S,
    ) -> Result<(), StoreError> {
        let column_key = column_key.as_ref();
        let new_value = self.store.launder_string(value.into());
        self.data.set_cell(row_index, column_key, new_value)
    }

    /// Sets the values of a row in the table.
    pub fn set_row<S: Into<ShareableString>>(
        &mut self,
        row_index: usize,
        values: Vec<S>,
    ) -> Result<(), StoreError> {
        let values: Vec<ShareableString> = values
            .into_iter()
            .map(|v| self.store.launder_string(v.into()))
            .collect();
        self.data.set_row(row_index, values)
    }
}

impl ProxyStoreTrait for TableProxy {
    fn path(&self) -> &StorePath {
        &self.path
    }

    fn description(&self) -> ShareableString {
        self.definition().description()
    }

    fn is_valid(&self) -> bool {
        self.data.is_valid()
    }

    fn has_changed(&self) -> bool {
        self.data.has_changed()
    }

    fn pull(&mut self) -> Result<(), StoreError> {
        if !self.is_valid() {
            let proxy = match self.store.table(&self.path) {
                Ok(p) => p,
                Err(_) => return Err(StoreError::ExpiredProxy),
            };
            return if proxy.definition() == self.definition() {
                self.data = proxy.data;
                Ok(())
            } else {
                Err(StoreError::ExpiredProxy)
            };
        }

        if !self.has_changed() {
            return Ok(());
        }

        let proxy = self.store.table(&self.path)?;

        self.data = proxy.data;

        Ok(())
    }

    fn push(&mut self) -> Result<(), StoreError> {
        if !self.is_valid() {
            let proxy = match self.store.table(&self.path) {
                Ok(p) => p,
                Err(_) => return Err(StoreError::ExpiredProxy),
            };
            if proxy.definition() == self.definition() {
                self.data = proxy.data;
            } else {
                return Err(StoreError::ExpiredProxy);
            }
        }

        self.store.set_table(&self.path, &self.data)?;
        self.data.update_shared_hash(); // Sync shared hash after successful push
        Ok(())
    }

    fn object(&self) -> Result<ObjectProxy, StoreError> {
        let key = self.path.object_key();
        self.store.object(key)
    }
}

impl TreePrint for TableProxy {
    fn tree_print(
        &self,
        f: &mut std::fmt::Formatter<'_>,
        label: &str,
        prefix: &str,
        last: bool,
    ) -> std::fmt::Result {
        self.data.tree_print(f, label, prefix, last)
    }
}

impl std::fmt::Display for TableProxy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let label = self.path.get_last_key();
        self.tree_display(label.as_ref()).fmt(f)
    }
}
