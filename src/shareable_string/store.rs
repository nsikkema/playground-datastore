use crate::shareable_string::string::ShareableString;
use parking_lot::RwLock;
use std::collections::HashSet;
use std::sync::Arc;

/// A store for interning `ShareableString`s.
/// This store ensures that duplicate strings are only stored once in memory.
#[derive(Debug, Clone)]
pub struct SharedStringStore {
    string_store: Arc<RwLock<HashSet<ShareableString>>>,
}

impl Default for SharedStringStore {
    fn default() -> Self {
        Self::new()
    }
}

impl SharedStringStore {
    /// Creates a new, empty `SharedStringStore`.
    pub fn new() -> Self {
        Self {
            string_store: Arc::new(RwLock::new(HashSet::new())),
        }
    }

    /// Returns a `ShareableString` for the given key, interning it if it's not already in the store.
    pub fn get<S>(&self, key: S) -> ShareableString
    where
        S: Into<ShareableString> + AsRef<str>,
    {
        self.launder(key)
    }

    /// Returns the number of unique strings in the store.
    pub fn len(&self) -> usize {
        self.string_store.read().len()
    }

    /// Checks if the internal string store is empty.
    pub fn is_empty(&self) -> bool {
        self.string_store.read().is_empty()
    }

    /// Returns true if the store contains the specified string.
    pub fn contains(&self, key: &str) -> bool {
        self.string_store.read().contains(key)
    }

    /// Copies all strings from another store into this one.
    pub fn copy_from(&self, other: &SharedStringStore) {
        let other_store = other.string_store.read();
        let mut self_store = self.string_store.write();

        for value in other_store.iter() {
            // We only add the string if it doesn't already exist in the store.
            if !self_store.contains(value.as_str()) {
                self_store.insert(value.clone());
            }
        }
    }

    /// Adds a `ShareableString` to the store if it's not already present.
    pub fn add(&self, string: &ShareableString) {
        let mut store = self.string_store.write();
        // If the string is already in the store, we don't need to do anything.
        // If not, we add it to enable interning for this string in the future.
        if !store.contains(string.as_str()) {
            store.insert(string.clone());
        }
    }

    /// Interns the given key and returns the shared instance.
    pub(crate) fn launder<S>(&self, key: S) -> ShareableString
    where
        S: Into<ShareableString> + AsRef<str>,
    {
        {
            let store = self.string_store.read();
            if let Some(existing) = store.get(key.as_ref()) {
                return existing.clone();
            }
        }

        let mut store = self.string_store.write();

        if let Some(existing) = store.get(key.as_ref()) {
            return existing.clone();
        }

        let key = key.into();
        store.insert(key.clone());
        key
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_string_interning() {
        let store = SharedStringStore::new();
        let s1 = store.get("hello");
        let s2 = store.get("hello");
        let s3 = store.get("world");

        // Check underline data.
        assert_eq!(s1.as_ref(), "hello");
        assert_eq!(s2.as_ref(), "hello");
        assert_eq!(s3.as_ref(), "world");

        // Check equality.
        assert_eq!(s1, s2);
        assert_ne!(s1, s3);

        // Check underlying Arc sharing.
        assert!(Arc::ptr_eq(s1.as_arc(), s2.as_arc()));
        assert!(!Arc::ptr_eq(s1.as_arc(), s3.as_arc()));

        // Check hash.
        assert_eq!(s1.current_blake3_hash(), s2.current_blake3_hash());
        assert_ne!(s1.current_blake3_hash(), s3.current_blake3_hash());

        // Check store count.
        assert_eq!(store.len(), 2);
    }

    #[test]
    fn test_various_input_types() {
        let store = SharedStringStore::new();

        let a = store.get("hi");
        let b = store.get(String::from("hi"));

        // Check underline data.
        assert_eq!(a.as_ref(), "hi");
        assert_eq!(b.as_ref(), "hi");

        // Check equality.
        assert_eq!(a, b);

        // Check underlying Arc sharing.
        assert!(Arc::ptr_eq(a.as_arc(), b.as_arc()));

        // Check hash.
        assert_eq!(a.current_blake3_hash(), b.current_blake3_hash());

        // Check store count.
        assert_eq!(store.len(), 1);
    }

    #[test]
    fn test_copy_from_preserves_memory_address() {
        let store_a = SharedStringStore::new();
        let a_hello = store_a.get("hello");
        let a_world = store_a.get("world");

        assert_eq!(store_a.len(), 2);

        let store_b = SharedStringStore::new();
        assert_eq!(store_b.len(), 0);

        store_b.copy_from(&store_a);
        assert_eq!(store_b.len(), 2);

        // After copying, store_b should hold clones of the same ShareableString values (same Arc)
        let b_hello = store_b.get("hello");
        let b_world = store_b.get("world");

        assert_eq!(a_hello, b_hello);
        assert_eq!(a_world, b_world);

        assert!(Arc::ptr_eq(a_hello.as_arc(), b_hello.as_arc()));
        assert!(Arc::ptr_eq(a_world.as_arc(), b_world.as_arc()));
    }

    #[test]
    fn test_store_add() {
        let store_a = SharedStringStore::new();
        let a_x = store_a.get("x");

        let store_b = SharedStringStore::new();
        let b_x = store_b.get("x");
        let b_y = store_b.get("y");

        // Check underling arc pointers.
        assert!(!Arc::ptr_eq(a_x.as_arc(), b_x.as_arc()));

        // If the key exists, add() inserts it.
        let len_before_x = store_a.len();
        store_a.add(&b_x);

        let a_x_after = store_a.get("x");
        assert_eq!(store_a.len(), len_before_x);
        assert!(Arc::ptr_eq(a_x_after.as_arc(), a_x.as_arc()));
        assert!(!Arc::ptr_eq(a_x_after.as_arc(), b_x.as_arc()));

        // If the key is missing, add() inserts it.
        let len_before_y = store_a.len();
        store_a.add(&b_y);
        assert_eq!(store_a.len(), len_before_y + 1);

        let a_y_after = store_a.get("y");
        assert!(Arc::ptr_eq(a_y_after.as_arc(), b_y.as_arc()));
        assert_eq!(store_a.len(), 2);
    }

    #[test]
    fn test_store_len() {
        let store = SharedStringStore::new();
        assert_eq!(store.len(), 0);

        let _a = store.get("a");
        assert_eq!(store.len(), 1);

        let _a = store.get("a");
        assert_eq!(store.len(), 1);

        let _b = store.get("b");
        assert_eq!(store.len(), 2);
    }

    #[test]
    fn test_shared_string_hashset() {
        use std::collections::HashSet;

        let store = SharedStringStore::new();
        let a1 = store.get("same");
        let a2 = store.get("same");
        let b = store.get("different");

        let mut set = HashSet::new();
        assert!(set.insert(a1.clone()));
        assert!(!set.insert(a2.clone())); // equal => should not insert
        assert!(set.insert(b));

        assert_eq!(set.len(), 2);
        assert!(set.contains(&a1));
        assert!(set.contains(&a2));
        assert!(!set.contains(&store.get("not-present")));
    }

    #[test]
    fn test_store_is_thread_safe() {
        use std::thread;

        let store = SharedStringStore::new();
        let store2 = store.clone();
        let store3 = store.clone();

        let t1 = thread::spawn(move || store.get("shared"));
        let t2 = thread::spawn(move || store2.get("shared"));
        let t3 = thread::spawn(move || store3.get("shared2"));

        let s1 = t1.join().unwrap();
        let s2 = t2.join().unwrap();
        let s3 = t3.join().unwrap();

        assert_eq!(s1, s2);
        assert_ne!(s1, s3);

        assert!(Arc::ptr_eq(s1.as_arc(), s2.as_arc()));
        assert!(!Arc::ptr_eq(s1.as_arc(), s3.as_arc()));
    }

    #[test]
    fn test_copy_from_does_not_override_existing() {
        let dst = SharedStringStore::new();
        let dst_v = dst.get("k");
        assert_eq!(dst.len(), 1);

        let src = SharedStringStore::new();
        let src_v = src.get("k");
        assert_eq!(src.len(), 1);

        // Sanity: different stores => different allocations initially.
        assert!(!Arc::ptr_eq(dst_v.as_arc(), src_v.as_arc()));

        dst.copy_from(&src);

        // After copy, dst should still point at its original value for the same key.
        let dst_after = dst.get("k");
        assert!(Arc::ptr_eq(dst_after.as_arc(), dst_v.as_arc()));
        assert!(!Arc::ptr_eq(dst_after.as_arc(), src_v.as_arc()));
    }

    #[test]
    fn test_store_launder() {
        let store = SharedStringStore::new();
        let s1 = store.launder("test");
        let s2 = store.launder(String::from("test"));
        let s3 = ShareableString::new("test");
        let s4 = store.launder(s3.clone());

        assert_eq!(s1, "test");
        assert_eq!(s2, "test");
        assert_eq!(s4, "test");

        assert!(Arc::ptr_eq(s1.as_arc(), s2.as_arc()));
        assert!(Arc::ptr_eq(s1.as_arc(), s4.as_arc()));
        // s3 was created outside, but launder should return the one in store.
        assert!(!Arc::ptr_eq(s1.as_arc(), s3.as_arc()));
    }

    #[test]
    fn test_store_contains() {
        let store = SharedStringStore::new();
        store.get("present");
        assert!(store.contains("present"));
        assert!(!store.contains("absent"));
    }

    #[test]
    fn test_store_default() {
        let store = SharedStringStore::default();
        assert_eq!(store.len(), 0);
    }
}
