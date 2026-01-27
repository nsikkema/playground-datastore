use crate::definition::PropertyDefinition;
use crate::shareable_string::{ShareableString, SharedStringStore};
use crate::{StoreError, validate_key};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::sync::Arc;

/// Builder for creating an `ObjectDefinition`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ObjectDefinitionBuilder {
    description: ShareableString,
    properties: BTreeMap<ShareableString, PropertyDefinition>,
}

impl ObjectDefinitionBuilder {
    /// Creates a new `ObjectDefinitionBuilder` with a description.
    pub fn new<S: Into<ShareableString>>(description: S) -> Self {
        Self {
            description: description.into(),
            properties: BTreeMap::new(),
        }
    }

    /// Adds a property to the builder and returns the builder.
    pub fn with<S: Into<ShareableString>>(
        mut self,
        key: S,
        property: PropertyDefinition,
    ) -> Result<Self, StoreError> {
        self.add(key, property)?;
        Ok(self)
    }

    /// Adds a property to the builder.
    pub fn add<S: Into<ShareableString>>(
        &mut self,
        key: S,
        property: PropertyDefinition,
    ) -> Result<(), StoreError> {
        let key = key.into();
        validate_key(&key)?;
        self.properties.insert(key, property);
        Ok(())
    }

    /// Removes a property from the builder.
    pub fn remove<S: Into<ShareableString>>(&mut self, key: S) {
        self.properties.remove(&key.into());
    }

    /// Builds the `ObjectDefinition`.
    pub fn finish(self) -> ObjectDefinition {
        ObjectDefinition {
            description: self.description,
            properties: Arc::new(self.properties),
        }
    }
}

/// Definition for an object, which is a collection of named properties.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct ObjectDefinition {
    description: ShareableString,
    properties: Arc<BTreeMap<ShareableString, PropertyDefinition>>,
}

impl ObjectDefinition {
    /// Returns a new `ObjectDefinitionBuilder` with the specified description.
    pub fn builder<S: Into<ShareableString>>(description: S) -> ObjectDefinitionBuilder {
        ObjectDefinitionBuilder::new(description)
    }

    /// Returns a new `ObjectDefinitionBuilder` initialized with the properties of this definition.
    pub fn new_inherit<S: Into<ShareableString>>(&self, description: S) -> ObjectDefinitionBuilder {
        ObjectDefinitionBuilder {
            description: description.into(),
            properties: BTreeMap::clone(&self.properties),
        }
    }

    /// Returns the description of the object.
    pub fn description(&self) -> ShareableString {
        self.description.clone()
    }

    /// Returns a reference to the description.
    pub fn description_ref(&self) -> &ShareableString {
        &self.description
    }

    /// Returns the number of properties in the object.
    pub fn count(&self) -> usize {
        self.properties.len()
    }

    /// Returns true if the object contains a property with the specified key.
    pub fn contains_key<S: Into<ShareableString>>(&self, key: S) -> bool {
        self.properties.contains_key(&key.into())
    }

    /// Returns a reference to the property definition for the specified key.
    pub fn get<S: Into<ShareableString>>(&self, key: S) -> Option<&PropertyDefinition> {
        self.properties.get(&key.into())
    }

    /// Returns a reference to the property definition for the specified key string.
    pub fn get_str(&self, key: &str) -> Option<&PropertyDefinition> {
        self.properties.get(key)
    }

    /// Returns an iterator over the keys of the properties.
    pub fn keys(&self) -> impl Iterator<Item = &ShareableString> {
        self.properties.keys()
    }

    /// Returns true if the object contains a property with the specified key string.
    pub fn contains_key_str(&self, key: &str) -> bool {
        self.properties.contains_key(key)
    }

    /// Returns an iterator over the property definitions.
    pub fn iter(&self) -> impl Iterator<Item = (&ShareableString, &PropertyDefinition)> {
        self.properties.iter()
    }

    /// Returns a new `ObjectDefinition` with strings laundered through the provided store.
    pub fn launder(&self, store: &SharedStringStore) -> Self {
        Self {
            description: store.launder(&self.description),
            properties: Arc::new(
                self.properties
                    .iter()
                    .map(|(k, v)| (store.launder(k), v.launder(store)))
                    .collect(),
            ),
        }
    }
}
