use crate::{StoreError, StoreKey};
use std::fmt::{Display, Formatter};
use std::marker::PhantomData;

/// A path to a piece of data within the store.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StorePath {
    object_key: StoreKey,
    segments: Vec<StoreKey>,
}

impl<S1, S2> From<(S1, S2)> for StorePath
where
    S1: Into<StoreKey>,
    S2: Into<StoreKey>,
{
    fn from((s1, s2): (S1, S2)) -> Self {
        StorePath::new(s1).segment(s2)
    }
}

impl<S1, S2, S3> From<(S1, S2, S3)> for StorePath
where
    S1: Into<StoreKey>,
    S2: Into<StoreKey>,
    S3: Into<StoreKey>,
{
    fn from((s1, s2, s3): (S1, S2, S3)) -> Self {
        StorePath::new(s1).segment(s2).segment(s3)
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
        StorePath::new(s1).segment(s2).segment(s3).segment(s4)
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
            .segment(s2)
            .segment(s3)
            .segment(s4)
            .segment(s5)
    }
}

impl StorePath {
    /// Creates a new `StorePath` pointing to an object.
    pub fn new(object_key: impl Into<StoreKey>) -> Self {
        Self::builder(object_key).build()
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

    /// Returns the segments of the path after the object key.
    pub fn segments(&self) -> &Vec<StoreKey> {
        &self.segments
    }

    /// Parses a string into a `StorePath`.
    ///
    /// The string should be in the format `object/segment1/segment2/...`.
    pub fn parse(s: &str) -> Result<Self, StoreError> {
        if s.is_empty() {
            return Err(StoreError::KeyEmpty);
        }
        let mut parts = s.split('/');
        let object_key = parts.next().ok_or(StoreError::KeyEmpty)?;
        let mut segments = Vec::new();

        for part in parts {
            if part.is_empty() {
                return Err(StoreError::InvalidPathSegment(part.to_string()));
            }
            segments.push(StoreKey::new(part.into())?);
        }

        Ok(StorePath {
            object_key: StoreKey::new(object_key.into())?,
            segments,
        })
    }

    /// Adds a segment to the path and returns the new path.
    pub fn segment(self, key: impl Into<StoreKey>) -> Self {
        self.add_segment(key)
    }

    /// Pushes a segment key onto the path and returns the new path.
    pub fn add_segment(mut self, key: impl Into<StoreKey>) -> Self {
        self.segments.push(key.into());
        self
    }

    /// Returns a path that points only to the object.
    pub fn get_object(&self) -> Self {
        Self {
            object_key: self.object_key.clone(),
            segments: vec![],
        }
    }

    /// Returns the last key in the path (either the object key or the last segment's key).
    pub fn get_last_key(&self) -> StoreKey {
        self.segments
            .last()
            .cloned()
            .unwrap_or_else(|| self.object_key.clone())
    }
}

impl PartialEq<&StorePath> for StorePath {
    fn eq(&self, other: &&StorePath) -> bool {
        self == *other
    }
}

impl PartialEq<StorePath> for &StorePath {
    fn eq(&self, other: &StorePath) -> bool {
        *self == other
    }
}

/// A macro to create a [`StorePath`] ergonomically.
///
/// The first argument is the object key. Each additional `/`-separated argument
/// appends a segment to the path.
///
/// # Examples
///
/// ```
/// use datastore::path;
/// let p = path!("obj" / "prop" / "nested");
/// assert_eq!(p.to_string(), "obj/prop/nested");
///
/// let p2 = path!("my_obj");
/// assert_eq!(p2.to_string(), "my_obj");
/// ```
#[macro_export]
macro_rules! path {
    ($obj:tt $(/ $seg:tt)+) => {{
        let mut p = $crate::StorePath::new($crate::store_key!($obj));
        $(
            p = p.segment($crate::store_key!($seg));
        )+
        p
    }};
    ($obj:tt) => {
        $crate::StorePath::new($crate::store_key!($obj))
    };
}

/// State for a `StorePathBuilder` pointing to an object.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObjectState;
/// State for a `StorePathBuilder` pointing to a property.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PropertyState;
/// State for a `StorePathBuilder` pointing to a map entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MapEntryState;
/// State for a `StorePathBuilder` pointing to a struct item.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructItemState;
/// State for a `StorePathBuilder` that can be in any state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnyState;

/// A builder for creating `StorePath` instances.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StorePathBuilder<S> {
    object_key: StoreKey,
    segments: Vec<StoreKey>,
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
        self.segments.push(property_key.into());
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
        }
    }
}

impl StorePathBuilder<PropertyState> {
    /// Adds a map key segment to the path.
    pub fn map_key(mut self, map_key: impl Into<StoreKey>) -> StorePathBuilder<MapEntryState> {
        self.segments.push(map_key.into());
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
        self.segments.push(struct_key.into());
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
        }
    }
}

impl StorePathBuilder<MapEntryState> {
    /// Adds a struct item segment to the path.
    pub fn struct_item(
        mut self,
        struct_key: impl Into<StoreKey>,
    ) -> StorePathBuilder<StructItemState> {
        self.segments.push(struct_key.into());
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
        }
    }
}

impl StorePathBuilder<AnyState> {
    /// Adds a segment to the path.
    pub fn property(mut self, property_key: impl Into<StoreKey>) -> Self {
        self.segments.push(property_key.into());
        self
    }

    /// Adds a segment to the path.
    pub fn map_key(mut self, map_key: impl Into<StoreKey>) -> Self {
        self.segments.push(map_key.into());
        self
    }

    /// Adds a segment to the path.
    pub fn struct_item(mut self, struct_key: impl Into<StoreKey>) -> Self {
        self.segments.push(struct_key.into());
        self
    }

    /// Builds the `StorePath`.
    pub fn build(self) -> StorePath {
        StorePath {
            object_key: self.object_key,
            segments: self.segments,
        }
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
        assert_eq!(path.to_string(), "obj");

        // Property path
        let path = StorePath::builder(store_key!("obj"))
            .property(store_key!("prop"))
            .build();
        assert_eq!(path.to_string(), "obj/prop");

        // Map entry path
        let path = StorePath::builder(store_key!("obj"))
            .property(store_key!("prop"))
            .map_key(store_key!("key"))
            .build();
        assert_eq!(path.to_string(), "obj/prop/key");

        // Struct item path from property
        let path = StorePath::builder(store_key!("obj"))
            .property(store_key!("prop"))
            .struct_item(store_key!("item"))
            .build();
        assert_eq!(path.to_string(), "obj/prop/item");

        // Struct item path from map entry
        let path = StorePath::builder(store_key!("obj"))
            .property(store_key!("prop"))
            .map_key(store_key!("key"))
            .struct_item(store_key!("item"))
            .build();
        assert_eq!(path.to_string(), "obj/prop/key/item");
    }

    #[test]
    fn test_ergonomic_paths() {
        // From string
        let p0: StorePath = StorePath::parse("obj/prop/key").unwrap();
        assert_eq!(p0.to_string(), "obj/prop/key");

        // From tuple
        let p1: StorePath = (store_key!("obj"), store_key!("prop")).into();
        assert_eq!(p1.to_string(), "obj/prop");

        let p2: StorePath = (store_key!("obj"), store_key!("prop"), store_key!("key")).into();
        assert_eq!(p2.to_string(), "obj/prop/key");

        let p3: StorePath = (
            store_key!("obj"),
            store_key!("prop"),
            store_key!("key"),
            store_key!("item"),
        )
            .into();
        assert_eq!(p3.to_string(), "obj/prop/key/item");

        let p4: StorePath = (
            store_key!("obj"),
            store_key!("prop"),
            store_key!("key"),
            store_key!("item"),
            store_key!("nested"),
        )
            .into();
        assert_eq!(p4.to_string(), "obj/prop/key/item/nested");
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

        let p2 = StorePath::parse("obj/prop/key/item").unwrap();
        assert_eq!(p2.to_string(), "obj/prop/key/item");

        // Any number of segments is valid
        let p3 = StorePath::parse("obj/prop/key/item/extra").unwrap();
        assert_eq!(p3.to_string(), "obj/prop/key/item/extra");

        let err = StorePath::parse("").unwrap_err();
        assert_eq!(err, StoreError::KeyEmpty);

        let err = StorePath::parse("obj/").unwrap_err();
        assert!(matches!(err, StoreError::InvalidPathSegment(_)));

        let err = StorePath::parse("obj//prop").unwrap_err();
        assert!(matches!(err, StoreError::InvalidPathSegment(_)));
    }

    #[test]
    fn test_get_object() {
        let path = StorePath::parse("obj/prop/key").unwrap();
        let obj_path = path.get_object();
        assert_eq!(obj_path.to_string(), "obj");
        assert!(obj_path.segments().is_empty());
    }

    #[test]
    fn test_get_last_key_object() {
        let path = StorePath::parse("obj").unwrap();
        let obj_path = path.get_last_key();
        assert_eq!(obj_path.to_string(), "obj");
    }

    #[test]
    fn test_get_last_key_full_path() {
        let path = StorePath::parse("obj/prop/key").unwrap();
        let obj_path = path.get_last_key();
        assert_eq!(obj_path.to_string(), "key");
    }

    #[test]
    fn test_path_equality() {
        let p1 = path!("obj" / "prop" / "key");
        let p2 = path!("obj" / "prop" / "key");
        let p3 = path!("obj" / "prop" / "other");
        let p4 = path!("other" / "prop" / "key");

        assert_eq!(p1, p2);
        assert_ne!(p1, p3);
        assert_ne!(p1, p4);

        let p5 = StorePath::new(store_key!("obj"));
        let p6 = StorePath::new(store_key!("obj"));
        assert_eq!(p5, p6);

        // Reference comparisons
        assert_eq!(p1, &p2);
        assert_eq!(&p1, p2);
        assert_eq!(&p1, &p2);
    }

    #[test]
    fn test_to_builder() {
        let path = StorePath::builder(store_key!("obj"))
            .property(store_key!("prop"))
            .build();

        let path2 = path.to_builder().map_key(store_key!("key")).build();

        assert_eq!(path2.object_key().as_str(), "obj");
        assert_eq!(path2.segments().len(), 2);

        // AnyState builder extends the path
        let path3 = path2.to_builder().property(store_key!("more")).build();
        assert_eq!(path3.segments().len(), 3);
    }

    #[test]
    fn test_builder_states() {
        // ObjectState -> build
        let p = StorePathBuilder::new(store_key!("obj").into()).build();
        assert!(p.segments().is_empty());

        // ObjectState -> property -> build
        let p = StorePathBuilder::new(store_key!("obj").into())
            .property(store_key!("prop"))
            .build();
        assert_eq!(p.segments().len(), 1);

        // PropertyState -> map_key -> build
        let p = StorePathBuilder::new(store_key!("obj").into())
            .property(store_key!("prop"))
            .map_key(store_key!("key"))
            .build();
        assert_eq!(p.segments().len(), 2);

        // PropertyState -> struct_item -> build
        let p = StorePathBuilder::new(store_key!("obj").into())
            .property(store_key!("prop"))
            .struct_item(store_key!("item"))
            .build();
        assert_eq!(p.segments().len(), 2);

        // MapEntryState -> struct_item -> build
        let p = StorePathBuilder::new(store_key!("obj").into())
            .property(store_key!("prop"))
            .map_key(store_key!("key"))
            .struct_item(store_key!("item"))
            .build();
        assert_eq!(p.segments().len(), 3);

        // ObjectState -> to_any
        let p = StorePathBuilder::new(store_key!("obj").into())
            .to_any()
            .property(store_key!("prop"))
            .build();
        assert_eq!(p.segments().len(), 1);
    }

    #[test]
    fn test_display() {
        let p = StorePath::builder(store_key!("obj"))
            .property(store_key!("prop"))
            .map_key(store_key!("key"))
            .struct_item(store_key!("item"))
            .build();

        assert_eq!(p.to_string(), "obj/prop/key/item");
    }
}
