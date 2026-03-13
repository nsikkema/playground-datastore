use crate::definition::StructDefinition;
use crate::shareable_string::{ShareableString, SharedStringStore};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Definition for a map property where keys are strings and values follow a `StructDefinition`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MapDefinition {
    description: ShareableString,
    item_type: Arc<StructDefinition>,
}

impl MapDefinition {
    /// Creates a new `MapDefinition` with a description and item type.
    pub fn new<S: Into<ShareableString>>(description: S, item_type: StructDefinition) -> Self {
        Self {
            description: description.into(),
            item_type: Arc::new(item_type),
        }
    }

    /// Returns the description of the map.
    pub fn description(&self) -> ShareableString {
        self.description.clone()
    }

    /// Returns a reference to the item type definition.
    pub fn item_type(&self) -> &StructDefinition {
        self.item_type.as_ref()
    }

    /// Returns a reference to the description.
    pub fn description_ref(&self) -> &ShareableString {
        &self.description
    }

    /// Returns a new `MapDefinition` with strings laundered through the provided store.
    pub fn launder(&self, store: &SharedStringStore) -> Self {
        Self {
            description: store.launder(&self.description),
            item_type: Arc::new(self.item_type.launder(store)),
        }
    }
}
