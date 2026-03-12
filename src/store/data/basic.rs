use crate::definition::BasicDefinition;
use crate::shareable_string::{ShareableString, SharedStringStore};
use crate::static_store::data::StaticBasic;
use crate::store::{CommonStoreTraitInternal, StoreHashContainer, TreePrint};
use serde::{Deserialize, Serialize};

/// Represents a basic data value in the store (String, Number, etc.).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Basic {
    #[serde(skip)]
    definition: BasicDefinition,
    value: ShareableString,
    #[serde(skip)]
    blake3_hash: StoreHashContainer,
}

impl Basic {
    /// Creates a new `Basic` instance with the given definition.
    pub(crate) fn new(definition: BasicDefinition) -> Self {
        let mut s = Self {
            value: definition.default_value().clone(),
            definition,
            blake3_hash: StoreHashContainer::new(),
        };
        CommonStoreTraitInternal::update_blake3_hash(&mut s);
        s
    }

    /// Returns a new `Basic` instance with strings laundered through the provided store.
    pub(crate) fn launder(&self, store: &SharedStringStore) -> Self {
        let mut s = Self {
            definition: self.definition.launder(store),
            value: store.launder(&self.value),
            blake3_hash: StoreHashContainer::new(),
        };
        s.update_blake3_hash();
        s
    }

    /// Returns a reference to the basic definition.
    pub fn definition(&self) -> &BasicDefinition {
        &self.definition
    }

    /// Returns the current value.
    pub fn get(&self) -> ShareableString {
        self.value.clone()
    }

    /// Sets a new value and updates the hash.
    pub fn set<S: Into<ShareableString>>(&mut self, value: S) {
        self.value = value.into();
        self.update_blake3_hash();
    }

    /// Restores the definition after deserialization.
    pub(crate) fn restore_definition(&mut self, definition: BasicDefinition) {
        self.definition = definition;
    }

    pub(crate) fn update_from_static(&mut self, static_basic: &StaticBasic) {
        self.value = static_basic.value().clone();
        self.blake3_hash.set(static_basic.hash());
    }
}

impl From<&StaticBasic> for Basic {
    fn from(static_basic: &StaticBasic) -> Self {
        let s = Self {
            definition: static_basic.definition().clone(),
            value: static_basic.value().clone(),
            blake3_hash: StoreHashContainer::new(),
        };
        s.blake3_hash.set(static_basic.hash());
        s
    }
}

impl CommonStoreTraitInternal for Basic {
    fn current_blake3_hash(&self) -> [u8; 32] {
        self.blake3_hash.get()
    }

    fn update_blake3_hash(&mut self) {
        let mut h = blake3::Hasher::new();

        // Domain separation for this node/type.
        h.update(&[0x01]);
        h.update(b"Basic");

        h.update(&self.value.current_blake3_hash());

        let digest = h.finalize();
        self.blake3_hash.set(*digest.as_bytes());
    }

    fn clear_hash(&mut self) {
        self.blake3_hash.clear();
    }
}

impl TreePrint for Basic {
    fn tree_print(&self, label: &str, prefix: &str, last: bool) {
        println!(
            "{}{}{}: {} ({})",
            prefix,
            Self::branch_char(last),
            label,
            self.value,
            self.definition.description()
        );
    }
}
