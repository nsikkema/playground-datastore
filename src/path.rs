use crate::{ShareableString, StoreError, StoreKey};
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
    Property(StoreKey),
    /// A map key segment.
    MapKey(StoreKey),
    /// A struct item segment.
    StructItem(StoreKey),
}

/// A path to a piece of data within the store.
#[derive(Debug, Clone)]
pub struct StorePath {
    object_key: StoreKey,
    segments: Vec<Segment>,
    kind: PathKind, // derived/validated, not hand-maintained
}

impl From<&str> for StorePath {
    fn from(s: &str) -> Self {
        StorePath::parse(s).unwrap_or_else(|_| StorePath::new_unsafe(""))
    }
}

impl From<String> for StorePath {
    fn from(s: String) -> Self {
        StorePath::from(s.as_str())
    }
}

impl From<ShareableString> for StorePath {
    fn from(s: ShareableString) -> Self {
        StorePath::builder(StoreKey::new_unsafe(s)).build()
    }
}

impl<S1, S2> From<(S1, S2)> for StorePath
where
    S1: Into<StoreKey>,
    S2: Into<StoreKey>,
{
    fn from((s1, s2): (S1, S2)) -> Self {
        StorePath::new(s1).property(s2)
    }
}

impl<S1, S2, S3> From<(S1, S2, S3)> for StorePath
where
    S1: Into<StoreKey>,
    S2: Into<StoreKey>,
    S3: Into<StoreKey>,
{
    fn from((s1, s2, s3): (S1, S2, S3)) -> Self {
        StorePath::new(s1).property(s2).property(s3)
    }
}

impl<S1, S2, S3, S4> From<(S1, S2, S3, S4)> for StorePath
where
    S1: Into<StoreKey>,
    S2: Into<StoreKey>,
    S3: Into<StoreKey>,
    S4: Into<StoreKey>,
{
    fn from((s1, s2, s3, s4): (S1, S2, S3, S4)) -> Self {
        StorePath::new(s1).property(s2).map_key(s3).struct_item(s4)
    }
}

impl<S1, S2, S3, S4, S5> From<(S1, S2, S3, S4, S5)> for StorePath
where
    S1: Into<StoreKey>,
    S2: Into<StoreKey>,
    S3: Into<StoreKey>,
    S4: Into<StoreKey>,
    S5: Into<StoreKey>,
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
    pub fn new(object_key: impl Into<StoreKey>) -> Self {
        Self::builder(object_key).build()
    }

    pub(crate) fn new_unsafe(object_key: impl Into<ShareableString>) -> Self {
        Self::builder(StoreKey::new_unsafe(object_key.into())).build()
    }

    /// Returns a builder for creating a `StorePath`.
    pub fn builder(object_key: impl Into<StoreKey>) -> StorePathBuilder<ObjectState> {
        StorePathBuilder::<ObjectState>::new(object_key.into())
    }

    /// Converts the `StorePath` back into a builder.
    pub fn to_builder(self) -> StorePathBuilder<AnyState> {
        StorePathBuilder::from(self)
    }

    /// Returns the object key part of the path.
    pub fn object_key(&self) -> &StoreKey {
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
                PathKind::Object => (
                    Segment::Property(StoreKey::new_unsafe(part.into())),
                    PathKind::Property,
                ),
                PathKind::Property => {
                    // Ambiguous. Let's assume MapKey for now as it's common.
                    // Or we could try to look ahead? No, let's just pick one that is valid.
                    (
                        Segment::MapKey(StoreKey::new_unsafe(part.into())),
                        PathKind::MapEntry,
                    )
                }
                PathKind::MapEntry => (
                    Segment::StructItem(StoreKey::new_unsafe(part.into())),
                    PathKind::StructItem,
                ),
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
            object_key: StoreKey::new(object_key.into())?,
            segments,
            kind,
        })
    }
    pub fn property(self, property_key: impl Into<StoreKey>) -> Self {
        self.push_property(property_key)
    }

    /// Adds a map key segment to the path and returns the new path.
    pub fn map_key(self, map_key: impl Into<StoreKey>) -> Self {
        self.push_map_key(map_key)
    }

    /// Adds a struct item segment to the path and returns the new path.
    pub fn struct_item(self, struct_key: impl Into<StoreKey>) -> Self {
        self.push_struct_item(struct_key)
    }

    /// Pushes a property segment onto the path and returns the new path.
    pub fn push_property(mut self, property_key: impl Into<StoreKey>) -> Self {
        self.segments.push(Segment::Property(property_key.into()));
        self.kind = PathKind::Property;
        self
    }

    /// Pushes a map key segment onto the path and returns the new path.
    pub fn push_map_key(mut self, map_key: impl Into<StoreKey>) -> Self {
        self.segments.push(Segment::MapKey(map_key.into()));
        self.kind = PathKind::MapEntry;
        self
    }

    /// Pushes a struct item segment onto the path and returns the new path.
    pub fn push_struct_item(mut self, struct_key: impl Into<StoreKey>) -> Self {
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
        let mut p = $crate::StorePath::new($crate::store_key!($obj));
        $(
            p = p.property($crate::store_key!($seg));
        )+
        p
    }};
    ($obj:tt) => {
        $crate::StorePath::new($crate::store_key!($obj))
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
    object_key: StoreKey,
    segments: Vec<Segment>,
    _state: PhantomData<S>,
}

impl StorePathBuilder<ObjectState> {
    fn new(object_key: StoreKey) -> Self {
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
        property_key: impl Into<StoreKey>,
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
    pub fn map_key(mut self, map_key: impl Into<StoreKey>) -> StorePathBuilder<MapEntryState> {
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
        struct_key: impl Into<StoreKey>,
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
        struct_key: impl Into<StoreKey>,
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
    pub fn property(mut self, property_key: impl Into<StoreKey>) -> Self {
        self.segments.push(Segment::Property(property_key.into()));
        self
    }

    /// Adds a map key segment to the path.
    pub fn map_key(mut self, map_key: impl Into<StoreKey>) -> Self {
        self.segments.push(Segment::MapKey(map_key.into()));
        self
    }

    /// Adds a struct item segment to the path.
    pub fn struct_item(mut self, struct_key: impl Into<StoreKey>) -> Self {
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
    use crate::{StorePath, store_key};

    #[test]
    fn test_valid_paths() {
        // Object path
        let path = StorePath::new(store_key!("obj"));
        assert_eq!(path.kind, PathKind::Object);
        assert_eq!(path.to_string(), "obj");

        // Property path
        let path = StorePath::builder(store_key!("obj"))
            .property(store_key!("prop"))
            .build();
        assert_eq!(path.kind, PathKind::Property);
        assert_eq!(path.to_string(), "obj/prop");

        // Map entry path
        let path = StorePath::builder(store_key!("obj"))
            .property(store_key!("prop"))
            .map_key(store_key!("key"))
            .build();
        assert_eq!(path.kind, PathKind::MapEntry);
        assert_eq!(path.to_string(), "obj/prop/key");

        // Struct item path from property
        let path = StorePath::builder(store_key!("obj"))
            .property(store_key!("prop"))
            .struct_item(store_key!("item"))
            .build();
        assert_eq!(path.kind, PathKind::StructItem);
        assert_eq!(path.to_string(), "obj/prop/item");

        // Struct item path from map entry
        let path = StorePath::builder(store_key!("obj"))
            .property(store_key!("prop"))
            .map_key(store_key!("key"))
            .struct_item(store_key!("item"))
            .build();
        assert_eq!(path.kind, PathKind::StructItem);
        assert_eq!(path.to_string(), "obj/prop/key/item");
    }

    #[test]
    fn test_ergonomic_paths() {
        // From string
        let p0: StorePath = StorePath::parse("obj/prop/key").unwrap();
        assert_eq!(p0.to_string(), "obj/prop/key");
        assert_eq!(p0.get_kind(), &PathKind::MapEntry);

        // From String object
        let s = String::from("obj/prop");
        let ps: StorePath = s.into();
        assert_eq!(ps.to_string(), "obj/prop");
        assert_eq!(ps.get_kind(), &PathKind::Property);

        // From ShareableString
        let ss = ShareableString::from("obj");
        let pss: StorePath = ss.into();
        assert_eq!(pss.to_string(), "obj");
        assert_eq!(pss.get_kind(), &PathKind::Object);

        // From tuple
        let p1: StorePath = (store_key!("obj"), store_key!("prop")).into();
        assert_eq!(p1.to_string(), "obj/prop");
        assert_eq!(p1.get_kind(), &PathKind::Property);

        let p2: StorePath = (store_key!("obj"), store_key!("prop"), store_key!("key")).into();
        assert_eq!(p2.to_string(), "obj/prop/key");
        assert_eq!(p2.get_kind(), &PathKind::Property); // property(s2).property(s3)

        let p3: StorePath = (
            store_key!("obj"),
            store_key!("prop"),
            store_key!("key"),
            store_key!("item"),
        )
            .into();
        assert_eq!(p3.to_string(), "obj/prop/key/item");
        assert_eq!(p3.get_kind(), &PathKind::StructItem);

        let p4: StorePath = (
            store_key!("obj"),
            store_key!("prop"),
            store_key!("key"),
            store_key!("item"),
            store_key!("nested"),
        )
            .into();
        assert_eq!(p4.to_string(), "obj/prop/key/item/nested");
        assert_eq!(p4.get_kind(), &PathKind::Property);
    }

    #[test]
    fn test_path_macro() {
        let p = path!("obj" / "prop" / "key");
        assert_eq!(p.to_string(), "obj/prop/key");

        let p2 = path!("my_obj");
        assert_eq!(p2.to_string(), "my_obj");
    }

    #[test]
    fn test_parse_path() {
        let p = StorePath::parse("obj/prop/key").unwrap();
        assert_eq!(p.to_string(), "obj/prop/key");
        assert_eq!(p.get_kind(), &PathKind::MapEntry); // Object -> Prop -> MapKey

        let p2 = StorePath::parse("obj/prop/key/item").unwrap();
        assert_eq!(p2.to_string(), "obj/prop/key/item");
        assert_eq!(p2.get_kind(), &PathKind::StructItem); // Object -> Prop -> MapKey -> StructItem

        let err = StorePath::parse("").unwrap_err();
        assert_eq!(err, StoreError::KeyEmpty);

        let err = StorePath::parse("obj/").unwrap_err();
        assert!(matches!(err, StoreError::InvalidPathSegment(_)));

        let err = StorePath::parse("obj//prop").unwrap_err();
        assert!(matches!(err, StoreError::InvalidPathSegment(_)));

        let err = StorePath::parse("obj/prop/key/item/extra").unwrap_err();
        assert_eq!(err, StoreError::InvalidPath);
    }

    #[test]
    fn test_get_object() {
        let path = StorePath::parse("obj/prop/key").unwrap();
        let obj_path = path.get_object();
        assert_eq!(obj_path.to_string(), "obj");
        assert_eq!(obj_path.get_kind(), &PathKind::Object);
    }

    #[test]
    fn test_to_builder() {
        let path = StorePath::builder(store_key!("obj"))
            .property(store_key!("prop"))
            .build();

        let path2 = path.to_builder().map_key(store_key!("key")).build();

        assert!(path2.is_ok());
        let path2 = path2.unwrap();
        assert_eq!(path2.object_key().as_str(), "obj");
        assert_eq!(path2.segments().len(), 2);
        assert_eq!(path2.get_kind(), &PathKind::MapEntry);

        // Test AnyState builder with invalid transition
        let path3 = path2.to_builder().property(store_key!("invalid")).build();
        assert_eq!(path3.unwrap_err(), StoreError::InvalidPath);
    }

    #[test]
    fn test_builder_states() {
        // ObjectState -> build
        let p = StorePathBuilder::new(store_key!("obj").into()).build();
        assert_eq!(p.get_kind(), &PathKind::Object);

        // ObjectState -> property -> build
        let p = StorePathBuilder::new(store_key!("obj").into())
            .property(store_key!("prop"))
            .build();
        assert_eq!(p.get_kind(), &PathKind::Property);

        // PropertyState -> map_key -> build
        let p = StorePathBuilder::new(store_key!("obj").into())
            .property(store_key!("prop"))
            .map_key(store_key!("key"))
            .build();
        assert_eq!(p.get_kind(), &PathKind::MapEntry);

        // PropertyState -> struct_item -> build
        let p = StorePathBuilder::new(store_key!("obj").into())
            .property(store_key!("prop"))
            .struct_item(store_key!("item"))
            .build();
        assert_eq!(p.get_kind(), &PathKind::StructItem);

        // MapEntryState -> struct_item -> build
        let p = StorePathBuilder::new(store_key!("obj").into())
            .property(store_key!("prop"))
            .map_key(store_key!("key"))
            .struct_item(store_key!("item"))
            .build();
        assert_eq!(p.get_kind(), &PathKind::StructItem);

        // ObjectState -> to_any
        let p = StorePathBuilder::new(store_key!("obj").into())
            .to_any()
            .property(store_key!("prop"))
            .build()
            .unwrap();
        assert_eq!(p.get_kind(), &PathKind::Property);
    }

    #[test]
    fn test_display() {
        let p = StorePath::builder(store_key!("obj"))
            .property(store_key!("prop"))
            .map_key(store_key!("key"))
            .struct_item(store_key!("item"))
            .build();

        assert_eq!(p.to_string(), "obj/prop/key/item");

        assert_eq!(Segment::Property(store_key!("p").into()).to_string(), "p");
        assert_eq!(Segment::MapKey(store_key!("k").into()).to_string(), "k");
        assert_eq!(Segment::StructItem(store_key!("s").into()).to_string(), "s");
    }
}
