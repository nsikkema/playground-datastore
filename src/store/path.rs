use crate::{ShareableString, StoreError};
use std::marker::PhantomData;

/// Represents the kind of a path in the store.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PathKind {
    /// The path refers to an object.
    Object,
    /// The path refers to a property.
    Property,
    /// The path refers to a map entry.
    MapEntry,
    /// The path refers to a struct item.
    StructItem,
}

/// Represents a segment in a store path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Segment {
    /// A property segment.
    Property(ShareableString),
    /// A map key segment.
    MapKey(ShareableString),
    /// A struct item segment.
    StructItem(ShareableString),
}

/// A path to a piece of data within the store.
#[derive(Debug, Clone)]
pub struct StorePath {
    object_key: ShareableString,
    segments: Vec<Segment>,
    kind: PathKind, // derived/validated, not hand-maintained
}

impl StorePath {
    /// Returns a builder for creating a `StorePath`.
    pub fn builder(object_key: ShareableString) -> StorePathBuilder<ObjectState> {
        StorePathBuilder::<ObjectState>::new(object_key)
    }

    /// Converts the `StorePath` back into a builder.
    pub fn to_builder(self) -> StorePathBuilder<AnyState> {
        StorePathBuilder::from(self)
    }

    /// Returns the object key part of the path.
    pub fn get_object_key(&self) -> &ShareableString {
        &self.object_key
    }

    /// Returns the segments of the path.
    pub fn get_segments(&self) -> &Vec<Segment> {
        &self.segments
    }

    /// Returns the kind of the path.
    pub fn get_kind(&self) -> &PathKind {
        &self.kind
    }

    /// Pushes a property segment onto the path and returns the new path.
    pub fn push_property(mut self, property_key: ShareableString) -> Self {
        self.segments.push(Segment::Property(property_key));
        self.kind = PathKind::Property;
        self
    }

    /// Pushes a map key segment onto the path and returns the new path.
    pub fn push_map_key(mut self, map_key: ShareableString) -> Self {
        self.segments.push(Segment::MapKey(map_key));
        self.kind = PathKind::MapEntry;
        self
    }

    /// Pushes a struct item segment onto the path and returns the new path.
    pub fn push_struct_item(mut self, struct_key: ShareableString) -> Self {
        self.segments.push(Segment::StructItem(struct_key));
        self.kind = PathKind::StructItem;
        self
    }

    /// Returns a path that points only to the object.
    pub fn get_object(&self) -> Self {
        Self {
            object_key: self.object_key.clone(),
            segments: vec![],
            kind: PathKind::Object,
        }
    }
}

/// State for a `StorePathBuilder` pointing to an object.
pub struct ObjectState;
/// State for a `StorePathBuilder` pointing to a property.
pub struct PropertyState;
/// State for a `StorePathBuilder` pointing to a map entry.
pub struct MapEntryState;
/// State for a `StorePathBuilder` pointing to a struct item.
pub struct StructItemState;
/// State for a `StorePathBuilder` that can be in any state.
pub struct AnyState;

/// A builder for creating `StorePath` instances.
pub struct StorePathBuilder<S> {
    object_key: ShareableString,
    segments: Vec<Segment>,
    _state: PhantomData<S>,
}

impl StorePathBuilder<ObjectState> {
    fn new(object_key: ShareableString) -> Self {
        StorePathBuilder {
            object_key,
            segments: Vec::new(),
            _state: PhantomData,
        }
    }
}

impl From<StorePath> for StorePathBuilder<AnyState> {
    fn from(path: StorePath) -> Self {
        Self {
            object_key: path.object_key,
            segments: path.segments,
            _state: PhantomData,
        }
    }
}

impl StorePathBuilder<ObjectState> {
    /// Adds a property segment to the path.
    pub fn property(mut self, property_key: ShareableString) -> StorePathBuilder<PropertyState> {
        self.segments.push(Segment::Property(property_key));
        StorePathBuilder {
            object_key: self.object_key,
            segments: self.segments,
            _state: PhantomData,
        }
    }

    /// Builds the `StorePath`.
    pub fn build(self) -> StorePath {
        StorePath {
            object_key: self.object_key,
            segments: self.segments,
            kind: PathKind::Object,
        }
    }
}

impl StorePathBuilder<PropertyState> {
    /// Adds a map key segment to the path.
    pub fn map_key(mut self, map_key: ShareableString) -> StorePathBuilder<MapEntryState> {
        self.segments.push(Segment::MapKey(map_key));
        StorePathBuilder {
            object_key: self.object_key,
            segments: self.segments,
            _state: PhantomData,
        }
    }

    /// Adds a struct item segment to the path.
    pub fn struct_item(mut self, struct_key: ShareableString) -> StorePathBuilder<StructItemState> {
        self.segments.push(Segment::StructItem(struct_key));
        StorePathBuilder {
            object_key: self.object_key,
            segments: self.segments,
            _state: PhantomData,
        }
    }

    /// Builds the `StorePath`.
    pub fn build(self) -> StorePath {
        StorePath {
            object_key: self.object_key,
            segments: self.segments,
            kind: PathKind::Property,
        }
    }
}

impl StorePathBuilder<MapEntryState> {
    /// Adds a struct item segment to the path.
    pub fn struct_item(mut self, struct_key: ShareableString) -> StorePathBuilder<StructItemState> {
        self.segments.push(Segment::StructItem(struct_key));
        StorePathBuilder {
            object_key: self.object_key,
            segments: self.segments,
            _state: PhantomData,
        }
    }

    /// Builds the `StorePath`.
    pub fn build(self) -> StorePath {
        StorePath {
            object_key: self.object_key,
            segments: self.segments,
            kind: PathKind::MapEntry,
        }
    }
}

impl StorePathBuilder<StructItemState> {
    /// Builds the `StorePath`.
    pub fn build(self) -> StorePath {
        StorePath {
            object_key: self.object_key,
            segments: self.segments,
            kind: PathKind::StructItem,
        }
    }
}

impl StorePathBuilder<AnyState> {
    /// Adds a property segment to the path.
    pub fn property(mut self, property_key: ShareableString) -> Self {
        self.segments.push(Segment::Property(property_key));
        self
    }

    /// Adds a map key segment to the path.
    pub fn map_key(mut self, map_key: ShareableString) -> Self {
        self.segments.push(Segment::MapKey(map_key));
        self
    }

    /// Adds a struct item segment to the path.
    pub fn struct_item(mut self, struct_key: ShareableString) -> Self {
        self.segments.push(Segment::StructItem(struct_key));
        self
    }

    /// Builds the `StorePath`, validating its structure.
    pub fn build(self) -> Result<StorePath, StoreError> {
        let mut kind = PathKind::Object;

        for segment in &self.segments {
            kind = match (kind, segment) {
                (PathKind::Object, Segment::Property(_)) => PathKind::Property,
                (PathKind::Property, Segment::MapKey(_)) => PathKind::MapEntry,
                (PathKind::Property, Segment::StructItem(_)) => PathKind::StructItem,
                (PathKind::MapEntry, Segment::StructItem(_)) => PathKind::StructItem,
                _ => return Err(StoreError::InvalidPath),
            };
        }

        Ok(StorePath {
            object_key: self.object_key,
            segments: self.segments,
            kind,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_paths() {
        let obj = ShareableString::from("obj");
        let prop = ShareableString::from("prop");
        let key = ShareableString::from("key");
        let item = ShareableString::from("item");

        // Object path
        let path = StorePath::builder(obj.clone()).build();
        assert_eq!(path.kind, PathKind::Object);

        // Property path
        let path = StorePath::builder(obj.clone())
            .property(prop.clone())
            .build();
        assert_eq!(path.kind, PathKind::Property);

        // Map entry path
        let path = StorePath::builder(obj.clone())
            .property(prop.clone())
            .map_key(key.clone())
            .build();
        assert_eq!(path.kind, PathKind::MapEntry);

        // Struct item path from property
        let path = StorePath::builder(obj.clone())
            .property(prop.clone())
            .struct_item(item.clone())
            .build();
        assert_eq!(path.kind, PathKind::StructItem);

        // Struct item path from map entry
        let path = StorePath::builder(obj.clone())
            .property(prop.clone())
            .map_key(key.clone())
            .struct_item(item.clone())
            .build();
        assert_eq!(path.kind, PathKind::StructItem);
    }

    #[test]
    fn test_to_builder() {
        let obj = ShareableString::from("obj");
        let prop = ShareableString::from("prop");
        let key = ShareableString::from("key");

        let path = StorePath::builder(obj.clone())
            .property(prop.clone())
            .build();

        let path2 = path.to_builder().map_key(key.clone()).build();

        assert!(path2.is_ok());
        let path2 = path2.unwrap();
        assert_eq!(path2.get_object_key(), &obj);
        assert_eq!(path2.get_segments().len(), 2);
        assert_eq!(path2.get_kind(), &PathKind::MapEntry);
    }
}
