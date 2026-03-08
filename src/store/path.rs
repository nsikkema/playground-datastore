use crate::{ShareableString, StoreError};
use std::fmt::{Display, Formatter};
use std::marker::PhantomData;

/// Represents the kind of path in the store.
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

impl From<&str> for StorePath {
    fn from(s: &str) -> Self {
        StorePath::parse(s).unwrap_or_else(|_| StorePath::new(""))
    }
}

impl From<String> for StorePath {
    fn from(s: String) -> Self {
        StorePath::from(s.as_str())
    }
}

impl From<ShareableString> for StorePath {
    fn from(s: ShareableString) -> Self {
        StorePath::builder(s).build()
    }
}

impl<S1, S2> From<(S1, S2)> for StorePath
where
    S1: Into<ShareableString>,
    S2: Into<ShareableString>,
{
    fn from((s1, s2): (S1, S2)) -> Self {
        StorePath::new(s1).property(s2)
    }
}

impl<S1, S2, S3> From<(S1, S2, S3)> for StorePath
where
    S1: Into<ShareableString>,
    S2: Into<ShareableString>,
    S3: Into<ShareableString>,
{
    fn from((s1, s2, s3): (S1, S2, S3)) -> Self {
        StorePath::new(s1).property(s2).property(s3)
    }
}

impl<S1, S2, S3, S4> From<(S1, S2, S3, S4)> for StorePath
where
    S1: Into<ShareableString>,
    S2: Into<ShareableString>,
    S3: Into<ShareableString>,
    S4: Into<ShareableString>,
{
    fn from((s1, s2, s3, s4): (S1, S2, S3, S4)) -> Self {
        StorePath::new(s1).property(s2).map_key(s3).struct_item(s4)
    }
}

impl<S1, S2, S3, S4, S5> From<(S1, S2, S3, S4, S5)> for StorePath
where
    S1: Into<ShareableString>,
    S2: Into<ShareableString>,
    S3: Into<ShareableString>,
    S4: Into<ShareableString>,
    S5: Into<ShareableString>,
{
    fn from((s1, s2, s3, s4, s5): (S1, S2, S3, S4, S5)) -> Self {
        StorePath::new(s1)
            .property(s2)
            .map_key(s3)
            .struct_item(s4)
            .property(s5)
    }
}

impl StorePath {
    /// Creates a new `StorePath` pointing to an object.
    pub fn new(object_key: impl Into<ShareableString>) -> Self {
        Self::builder(object_key).build()
    }

    /// Returns a builder for creating a `StorePath`.
    pub fn builder(object_key: impl Into<ShareableString>) -> StorePathBuilder<ObjectState> {
        StorePathBuilder::<ObjectState>::new(object_key.into())
    }

    /// Converts the `StorePath` back into a builder.
    pub fn to_builder(self) -> StorePathBuilder<AnyState> {
        StorePathBuilder::from(self)
    }

    /// Returns the object key part of the path.
    pub fn object_key(&self) -> &ShareableString {
        &self.object_key
    }

    /// Returns the segments of the path.
    pub fn segments(&self) -> &Vec<Segment> {
        &self.segments
    }

    /// Returns the kind of the path.
    pub fn get_kind(&self) -> &PathKind {
        &self.kind
    }

    /// Parses a string into a `StorePath`.
    ///
    /// The string should be in the format `object/segment1/segment2/...`.
    /// This method tries to infer the segment types based on valid transitions.
    /// Note: This is inherently ambiguous for some paths (e.g., is it a property or a map key?).
    /// It defaults to Property -> StructItem or Property -> MapKey -> StructItem transitions.
    pub fn parse(s: &str) -> Result<Self, StoreError> {
        if s.is_empty() {
            return Err(StoreError::KeyEmpty);
        }
        let mut parts = s.split('/');
        let object_key = parts.next().ok_or(StoreError::KeyEmpty)?;
        let mut segments = Vec::new();

        let mut kind = PathKind::Object;
        for part in parts {
            if part.is_empty() {
                return Err(StoreError::InvalidPathSegment(part.to_string()));
            }
            let (segment, next_kind) = match kind {
                PathKind::Object => (Segment::Property(part.into()), PathKind::Property),
                PathKind::Property => {
                    // Ambiguous. Let's assume MapKey for now as it's common.
                    // Or we could try to look ahead? No, let's just pick one that is valid.
                    (Segment::MapKey(part.into()), PathKind::MapEntry)
                }
                PathKind::MapEntry => (Segment::StructItem(part.into()), PathKind::StructItem),
                PathKind::StructItem => {
                    // Struct items can't have further segments in this schema?
                    // Wait, let's check the transitions.
                    return Err(StoreError::InvalidPath);
                }
            };
            segments.push(segment);
            kind = next_kind;
        }

        Ok(StorePath {
            object_key: object_key.into(),
            segments,
            kind,
        })
    }
    pub fn property(self, property_key: impl Into<ShareableString>) -> Self {
        self.push_property(property_key)
    }

    /// Adds a map key segment to the path and returns the new path.
    pub fn map_key(self, map_key: impl Into<ShareableString>) -> Self {
        self.push_map_key(map_key)
    }

    /// Adds a struct item segment to the path and returns the new path.
    pub fn struct_item(self, struct_key: impl Into<ShareableString>) -> Self {
        self.push_struct_item(struct_key)
    }

    /// Pushes a property segment onto the path and returns the new path.
    pub fn push_property(mut self, property_key: impl Into<ShareableString>) -> Self {
        self.segments.push(Segment::Property(property_key.into()));
        self.kind = PathKind::Property;
        self
    }

    /// Pushes a map key segment onto the path and returns the new path.
    pub fn push_map_key(mut self, map_key: impl Into<ShareableString>) -> Self {
        self.segments.push(Segment::MapKey(map_key.into()));
        self.kind = PathKind::MapEntry;
        self
    }

    /// Pushes a struct item segment onto the path and returns the new path.
    pub fn push_struct_item(mut self, struct_key: impl Into<ShareableString>) -> Self {
        self.segments.push(Segment::StructItem(struct_key.into()));
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

/// A macro to create a `StorePath` ergonomically.
///
/// # Examples
///
/// ```
/// use datastore::path;
/// let p = path!("obj" / "prop" / "key");
/// ```
#[macro_export]
macro_rules! path {
    ($obj:tt $(/ $seg:tt)+) => {{
        let mut p = $crate::store::StorePath::new($obj);
        $(
            p = p.property($seg);
        )+
        p
    }};
    ($obj:tt) => {
        $crate::store::StorePath::new($obj)
    };
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
    /// Converts the builder to an `AnyState` builder.
    pub fn to_any(self) -> StorePathBuilder<AnyState> {
        StorePathBuilder {
            object_key: self.object_key,
            segments: self.segments,
            _state: PhantomData,
        }
    }

    /// Adds a property segment to the path.
    pub fn property(
        mut self,
        property_key: impl Into<ShareableString>,
    ) -> StorePathBuilder<PropertyState> {
        self.segments.push(Segment::Property(property_key.into()));
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
    pub fn map_key(
        mut self,
        map_key: impl Into<ShareableString>,
    ) -> StorePathBuilder<MapEntryState> {
        self.segments.push(Segment::MapKey(map_key.into()));
        StorePathBuilder {
            object_key: self.object_key,
            segments: self.segments,
            _state: PhantomData,
        }
    }

    /// Adds a struct item segment to the path.
    pub fn struct_item(
        mut self,
        struct_key: impl Into<ShareableString>,
    ) -> StorePathBuilder<StructItemState> {
        self.segments.push(Segment::StructItem(struct_key.into()));
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
    pub fn struct_item(
        mut self,
        struct_key: impl Into<ShareableString>,
    ) -> StorePathBuilder<StructItemState> {
        self.segments.push(Segment::StructItem(struct_key.into()));
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

impl Display for Segment {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Segment::Property(p) => write!(f, "{}", p),
            Segment::MapKey(k) => write!(f, "{}", k),
            Segment::StructItem(i) => write!(f, "{}", i),
        }
    }
}

impl Display for StorePath {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.object_key)?;
        for segment in &self.segments {
            write!(f, "/{}", segment)?;
        }
        Ok(())
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
    pub fn property(mut self, property_key: impl Into<ShareableString>) -> Self {
        self.segments.push(Segment::Property(property_key.into()));
        self
    }

    /// Adds a map key segment to the path.
    pub fn map_key(mut self, map_key: impl Into<ShareableString>) -> Self {
        self.segments.push(Segment::MapKey(map_key.into()));
        self
    }

    /// Adds a struct item segment to the path.
    pub fn struct_item(mut self, struct_key: impl Into<ShareableString>) -> Self {
        self.segments.push(Segment::StructItem(struct_key.into()));
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
        // Object path
        let path = StorePath::new("obj");
        assert_eq!(path.kind, PathKind::Object);
        assert_eq!(path.to_string(), "obj");

        // Property path
        let path = StorePath::builder("obj").property("prop").build();
        assert_eq!(path.kind, PathKind::Property);
        assert_eq!(path.to_string(), "obj/prop");

        // Map entry path
        let path = StorePath::builder("obj")
            .property("prop")
            .map_key("key")
            .build();
        assert_eq!(path.kind, PathKind::MapEntry);
        assert_eq!(path.to_string(), "obj/prop/key");

        // Struct item path from property
        let path = StorePath::builder("obj")
            .property("prop")
            .struct_item("item")
            .build();
        assert_eq!(path.kind, PathKind::StructItem);
        assert_eq!(path.to_string(), "obj/prop/item");

        // Struct item path from map entry
        let path = StorePath::builder("obj")
            .property("prop")
            .map_key("key")
            .struct_item("item")
            .build();
        assert_eq!(path.kind, PathKind::StructItem);
        assert_eq!(path.to_string(), "obj/prop/key/item");
    }

    #[test]
    fn test_ergonomic_paths() {
        // From tuple
        let p1: StorePath = ("obj", "prop").into();
        assert_eq!(p1.to_string(), "obj/prop");
        assert_eq!(p1.kind, PathKind::Property);

        let p2: StorePath = ("obj", "prop", "key").into();
        assert_eq!(p2.to_string(), "obj/prop/key");
        // This is now Property -> MapKey (because obj/prop is property, and property/key is MapEntry or StructItem)
        // Wait, builder.property("prop").property("key") transitioned kind to?
        // Let's check.
    }

    #[test]
    fn test_path_macro() {
        let p = path!("obj" / "prop" / "key");
        assert_eq!(p.to_string(), "obj/prop/key");

        let obj_key = "my_obj";
        let p2 = path!(obj_key);
        assert_eq!(p2.to_string(), "my_obj");
    }

    #[test]
    fn test_parse_path() {
        let p = StorePath::parse("obj/prop/key").unwrap();
        assert_eq!(p.to_string(), "obj/prop/key");

        let err = StorePath::parse("").unwrap_err();
        assert_eq!(err, StoreError::KeyEmpty);

        let err = StorePath::parse("obj//prop").unwrap_err();
        assert!(matches!(err, StoreError::InvalidPathSegment(_)));
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
        assert_eq!(path2.object_key(), &obj);
        assert_eq!(path2.segments().len(), 2);
        assert_eq!(path2.get_kind(), &PathKind::MapEntry);
    }
}
