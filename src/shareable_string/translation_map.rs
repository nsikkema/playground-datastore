use crate::shareable_string::{ShareableString, SharedStringStore};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct SharedStringTranslationMap {
    store: SharedStringStore,
    data: Arc<RwLock<HashMap<ShareableString, HashMap<ShareableString, ShareableString>>>>,
}

impl SharedStringTranslationMap {
    pub fn new(store: SharedStringStore) -> Self {
        SharedStringTranslationMap {
            store,
            data: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn get_translation(
        &self,
        key: &ShareableString,
        language: &ShareableString,
    ) -> Option<ShareableString> {
        let read_lock = self.data.read();
        read_lock
            .get(key)
            .and_then(|translations| translations.get(language).cloned())
    }

    pub fn set_translation(
        &self,
        key: &ShareableString,
        language: &ShareableString,
        translation: &ShareableString,
    ) {
        let interned_key = self.store.launder(key);
        let interned_lang = self.store.launder(language);
        let interned_translation = self.store.launder(translation);
        let mut write_lock = self.data.write();
        write_lock
            .entry(interned_key)
            .or_default()
            .insert(interned_lang, interned_translation);
    }

    pub fn set_translation_key(
        &self,
        key: &ShareableString,
        data: &HashMap<ShareableString, ShareableString>,
    ) {
        let interned_key = self.store.launder(key);
        let mut interned_data = HashMap::with_capacity(data.len());
        for (lang, translation) in data {
            let interned_lang = self.store.launder(lang);
            let interned_translation = self.store.launder(translation);
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
        let key = ShareableString::from("key");
        let lang = ShareableString::from("en");
        let translation = ShareableString::from("hello");

        // Assert they are NOT in the store initially
        assert!(!map.store.contains("key"));
        assert!(!map.store.contains("en"));
        assert!(!map.store.contains("hello"));

        map.set_translation(&key, &lang, &translation);

        // Now they should be in the store because set_translation launders
        assert!(map.store.contains("key"));
        assert!(map.store.contains("en"));
        assert!(map.store.contains("hello"));

        // And the data in the map should be using the interned instances
        let interned_key = map.store.get("key");
        let interned_lang = map.store.get("en");
        let interned_translation = map.store.get("hello");

        let retrieved_translation = map.get_translation(&interned_key, &interned_lang).unwrap();

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

        let key = ShareableString::from("key2");
        let mut data = HashMap::new();
        data.insert(
            ShareableString::from("fr"),
            ShareableString::from("bonjour"),
        );

        map.set_translation_key(&key, &data);

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
