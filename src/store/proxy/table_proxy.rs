use crate::StoreError;
use crate::definition::TableDefinition;
use crate::shareable_string::ShareableString;
use crate::store::{
    CommonStoreTraitInternal, ObjectProxy, ProxyStoreTrait, Store, StorePath, Table,
};
use std::collections::BTreeMap;

/// A proxy for a table in the store.
pub struct TableProxy {
    path: StorePath,
    store: Store,
    data: Table,
    last_sync_hash: [u8; 32],
}

impl TableProxy {
    /// Creates a new `TableProxy`.
    pub(crate) fn new(path: StorePath, store: Store, data: Table) -> Self {
        let last_sync_hash = data.current_blake3_hash();
        Self {
            path,
            store,
            data,
            last_sync_hash,
        }
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
    pub fn row(&self, index: usize) -> Option<&BTreeMap<ShareableString, ShareableString>> {
        self.data.row(index)
    }

    /// Sets the value of a cell in the table.
    pub fn set_cell<S1: Into<ShareableString>, S2: Into<ShareableString>>(
        &mut self,
        row_index: usize,
        column_key: S1,
        value: S2,
    ) -> Result<(), StoreError> {
        let column_key = column_key.into();
        let new_value = self.store.launder(value.into());
        self.data
            .set_cell(row_index, column_key.as_str(), new_value)
    }

    /// Sets the values of a row in the table.
    pub fn set_row<S: Into<ShareableString>>(
        &mut self,
        row_index: usize,
        values: Vec<S>,
    ) -> Result<(), StoreError> {
        let values: Vec<ShareableString> = values
            .into_iter()
            .map(|v| self.store.launder(v.into()))
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
        self.data.current_blake3_hash() != [0u8; 32]
    }

    fn has_changed(&self) -> bool {
        self.last_sync_hash != self.data.current_blake3_hash()
    }

    fn pull(&mut self) -> Result<(), StoreError> {
        if !self.is_valid() {
            return Err(StoreError::ExpiredProxy);
        }

        if !self.has_changed() {
            return Ok(());
        }

        let proxy = self.store.table(&self.path)?;

        self.data = proxy.data;
        self.last_sync_hash = proxy.last_sync_hash;

        Ok(())
    }

    fn push(&mut self) -> Result<(), StoreError> {
        if !self.is_valid() {
            return Err(StoreError::ExpiredProxy);
        }

        self.store.set_table(&self.path, &self.data)?;
        self.last_sync_hash = self.data.current_blake3_hash();
        Ok(())
    }

    fn object(&self) -> Result<ObjectProxy, StoreError> {
        let path = self.path.clone().get_object();
        self.store.object(&path)
    }
}
