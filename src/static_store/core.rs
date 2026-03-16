use crate::StoreKey;
use crate::shareable_string::ShareableString;
use crate::static_store::data::StaticObject;
use crate::store::Store;
use crate::store::TreePrint;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StaticStore {
    objects: BTreeMap<ShareableString, StaticObject>,
    hash: [u8; 32],
}

impl From<&Store> for StaticStore {
    fn from(store: &Store) -> Self {
        let mut objects = BTreeMap::new();
        if let Ok(keys) = store.object_keys() {
            for key in keys {
                if let Ok(object) = store.get_object_internal(&key) {
                    let store_key = StoreKey::new(key.clone()).expect("Valid key from store");
                    objects.insert(store_key, StaticObject::from(&object));
                }
            }
        }
        Self::new(objects)
    }
}

impl StaticStore {
    pub fn new(objects: BTreeMap<StoreKey, StaticObject>) -> Self {
        let objects = objects.into_iter().map(|(k, v)| (k.key, v)).collect();
        let mut s = Self {
            objects,
            hash: [0u8; 32],
        };
        s.update_hash();
        s
    }

    fn update_hash(&mut self) {
        let mut h = blake3::Hasher::new();

        // Domain separation for this node/type.
        h.update(&[0x01]);
        h.update(b"StoreInternal");

        h.update(&(self.objects.len() as u64).to_le_bytes());

        // Sort keys for deterministic hashing
        let mut keys: Vec<&ShareableString> = self.objects.keys().collect();
        keys.sort_by(|a, b| a.as_ref().cmp(b.as_ref()));

        for key in keys {
            h.update(&key.current_blake3_hash());
            h.update(&self.objects.get(key).unwrap().hash());
        }

        let digest = h.finalize();
        self.hash = *digest.as_bytes();
    }

    pub fn tree_print(&self) {
        println!("Static Store");
        let keys: Vec<_> = self.objects.keys().collect();
        for (i, key) in keys.iter().enumerate() {
            let is_last = i == keys.len() - 1;
            self.objects[*key].tree_print(key.as_str(), "", is_last);
        }
    }

    pub fn get_blake3_hash(&self) -> [u8; 32] {
        self.hash
    }

    pub(crate) fn objects(&self) -> &BTreeMap<ShareableString, StaticObject> {
        &self.objects
    }

    pub fn get<S: AsRef<str>>(&self, key: S) -> Option<&StaticObject> {
        self.objects.get(key.as_ref())
    }

    pub fn iter(&self) -> impl Iterator<Item = (&ShareableString, &StaticObject)> {
        self.objects.iter()
    }
}
