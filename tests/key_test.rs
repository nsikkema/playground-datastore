use datastore::key::{ConstStoreKey, StoreKey, is_valid_key};
use datastore::shareable_string::ShareableString;
use datastore::store_key;

#[test]
fn test_is_valid_key() {
    assert!(is_valid_key("a"));
    assert!(is_valid_key("abc"));
    assert!(is_valid_key("a123"));
    assert!(is_valid_key("a_b_c"));
    assert!(is_valid_key("a_1_b_2"));

    assert!(!is_valid_key(""));
    assert!(!is_valid_key("1abc"));
    assert!(!is_valid_key("_abc"));
    assert!(!is_valid_key("Abc"));
    assert!(!is_valid_key("a-b"));
    assert!(!is_valid_key("a b"));
}

#[test]
fn test_const_store_key() {
    const KEY: ConstStoreKey = ConstStoreKey::new("valid_key");
    assert_eq!(KEY.as_str(), "valid_key");
    assert_eq!(format!("{}", KEY), "valid_key");

    // From<ConstStoreKey>
    let store_key: StoreKey = KEY.into();
    assert_eq!(store_key.as_str(), "valid_key");

    // From<&ConstStoreKey>
    let store_key_ref: StoreKey = (&KEY).into();
    assert_eq!(store_key_ref.as_str(), "valid_key");
}

#[test]
fn test_store_key_macro() {
    const KEY: ConstStoreKey = store_key!("macro_key");
    assert_eq!(KEY.as_str(), "macro_key");
}

#[test]
#[should_panic(expected = "Invalid StoreKey literal")]
fn test_const_store_key_invalid() {
    let _ = ConstStoreKey::new("Invalid");
}

#[test]
fn test_store_key_from_runtime_string() {
    let s = String::from("runtime_key");
    let key = StoreKey::new(s.into()).unwrap();
    assert_eq!(key.as_str(), "runtime_key");

    let invalid_s = String::from("Invalid");
    let result = StoreKey::new(invalid_s.into());
    assert!(result.is_err());
}

#[test]
fn test_store_key_as_shareable_string() {
    let key = store_key!("my_key");
    let store_key: StoreKey = key.into();

    let shareable: &ShareableString = store_key.as_shareable_string();
    assert_eq!(shareable.as_str(), "my_key");

    // From<&StoreKey> for ShareableString
    let shareable_cloned: ShareableString = (&store_key).into();
    assert_eq!(shareable_cloned.as_str(), "my_key");
}
