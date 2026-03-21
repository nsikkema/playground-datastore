use crate::shareable_string::{ShareableString, SharedStringStore};
use parking_lot::RwLock;
use rustc_hash::FxHashMap;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct SharedStringTranslationMap {
    store: SharedStringStore,
    data: Arc<RwLock<FxHashMap<ShareableString, FxHashMap<ShareableString, ShareableString>>>>,
}

impl SharedStringTranslationMap {
    pub fn new(store: SharedStringStore) -> Self {
        SharedStringTranslationMap {
            store,
            data: Arc::new(RwLock::new(FxHashMap::default())),
        }
    }

    pub fn get_translation<K, L>(&self, key: K, language: L) -> Option<ShareableString>
    where
        K: AsRef<str>,
        L: AsRef<str>,
    {
        let read_lock = self.data.read();
        read_lock
            .get(key.as_ref())
            .and_then(|translations| translations.get(language.as_ref()).cloned())
    }

    pub fn set_translation<K, L, T>(&self, key: K, language: L, translation: T)
    where
        K: Into<ShareableString> + AsRef<str>,
        L: Into<ShareableString> + AsRef<str>,
        T: Into<ShareableString> + AsRef<str>,
    {
        let interned_key = self.store.launder(key);
        let interned_lang = self.store.launder(language);
        let interned_translation = self.store.launder(translation);
        let mut write_lock = self.data.write();
        write_lock
            .entry(interned_key)
            .or_default()
            .insert(interned_lang, interned_translation);
    }

    pub fn set_translation_key<K, K2, V2>(&self, key: K, data: &FxHashMap<K2, V2>)
    where
        K: Into<ShareableString> + AsRef<str>,
        K2: Into<ShareableString> + AsRef<str> + Clone,
        V2: Into<ShareableString> + AsRef<str> + Clone,
    {
        let interned_key = self.store.launder(key);
        let mut interned_data = FxHashMap::with_capacity_and_hasher(data.len(), Default::default());
        for (lang, translation) in data {
            let interned_lang = self.store.launder(lang.clone());
            let interned_translation = self.store.launder(translation.clone());
            interned_data.insert(interned_lang, interned_translation);
        }
        let mut write_lock = self.data.write();
        write_lock.insert(interned_key, interned_data);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_translation_map_auto_launder() {
        let store = SharedStringStore::new();
        let map = SharedStringTranslationMap::new(store);

        // Create strings that are NOT in the map's store
        let key = "key";
        let lang = "en";
        let translation = "hello";

        // Assert they are NOT in the store initially
        assert!(!map.store.contains("key"));
        assert!(!map.store.contains("en"));
        assert!(!map.store.contains("hello"));

        map.set_translation(key, lang, translation);

        // Now they should be in the store because set_translation launders
        assert!(map.store.contains("key"));
        assert!(map.store.contains("en"));
        assert!(map.store.contains("hello"));

        // And the data in the map should be using the interned instances
        let interned_key = map.store.get("key");
        let interned_lang = map.store.get("en");
        let interned_translation = map.store.get("hello");

        let retrieved_translation = map.get_translation("key", "en").unwrap();

        assert!(Arc::ptr_eq(
            retrieved_translation.as_arc(),
            interned_translation.as_arc()
        ));

        // Verify the keys in the map are also interned
        let read_lock = map.data.read();
        let (k, v) = read_lock.get_key_value(&interned_key).unwrap();
        assert!(Arc::ptr_eq(k.as_arc(), interned_key.as_arc()));

        let (l, t) = v.get_key_value(&interned_lang).unwrap();
        assert!(Arc::ptr_eq(l.as_arc(), interned_lang.as_arc()));
        assert!(Arc::ptr_eq(t.as_arc(), interned_translation.as_arc()));
    }

    #[test]
    fn test_translation_map_key_auto_launder() {
        let store = SharedStringStore::new();
        let map = SharedStringTranslationMap::new(store);

        let key = "key2";
        let mut data = FxHashMap::default();
        data.insert("fr", "bonjour");

        map.set_translation_key(key, &data);

        assert!(map.store.contains("key2"));
        assert!(map.store.contains("fr"));
        assert!(map.store.contains("bonjour"));

        let interned_key = map.store.get("key2");
        let interned_lang = map.store.get("fr");
        let interned_translation = map.store.get("bonjour");

        let read_lock = map.data.read();
        let (k, v) = read_lock.get_key_value(&interned_key).unwrap();
        assert!(Arc::ptr_eq(k.as_arc(), interned_key.as_arc()));

        let (l, t) = v.get_key_value(&interned_lang).unwrap();
        assert!(Arc::ptr_eq(l.as_arc(), interned_lang.as_arc()));
        assert!(Arc::ptr_eq(t.as_arc(), interned_translation.as_arc()));
    }
}
