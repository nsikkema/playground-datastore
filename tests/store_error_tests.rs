use datastore::StoreError;
use datastore::shareable_string::ShareableString;

#[test]
fn test_store_error_display() {
    assert_eq!(
        format!("{}", StoreError::KeyEmpty),
        "Invalid key: Key cannot be empty"
    );
    assert_eq!(
        format!("{}", StoreError::KeyInvalidCharacter("abc!".to_string())),
        "Invalid key: 'abc!'. Keys must only contain a-z, 0-9 and _"
    );
    assert_eq!(
        format!("{}", StoreError::ObjectNotFound),
        "Object not found"
    );
    assert_eq!(
        format!("{}", StoreError::ObjectKeyAlreadyExists),
        "Object key already exists"
    );
    assert_eq!(
        format!("{}", StoreError::PropertyNotFound),
        "Property not found"
    );
    assert_eq!(format!("{}", StoreError::ExpiredProxy), "Proxy is invalid");
    assert_eq!(format!("{}", StoreError::KeyNotFound), "Key not found");
    assert_eq!(format!("{}", StoreError::InvalidPath), "Invalid path");
    assert_eq!(
        format!("{}", StoreError::InvalidPathSegment("segment".to_string())),
        "Invalid path segment: segment"
    );
    assert_eq!(format!("{}", StoreError::IndexNotFound), "Index not found");
    assert_eq!(
        format!("{}", StoreError::UndoNotAvailable),
        "Undo not available"
    );
    assert_eq!(
        format!("{}", StoreError::RedoNotAvailable),
        "Redo not available"
    );
    assert_eq!(
        format!("{}", StoreError::SerializationError("failed".to_string())),
        "Serialization error: failed"
    );
    assert_eq!(
        format!(
            "{}",
            StoreError::PropertyConflict(ShareableString::from("conflict"))
        ),
        "Property conflict: conflict"
    );
    assert_eq!(
        format!("{}", StoreError::MissingSchema("schema".to_string())),
        "Missing schema: schema"
    );
}

#[test]
fn test_store_error_partial_eq() {
    assert_eq!(StoreError::KeyEmpty, StoreError::KeyEmpty);
    assert_eq!(
        StoreError::KeyInvalidCharacter("a".to_string()),
        StoreError::KeyInvalidCharacter("a".to_string())
    );
    assert_ne!(
        StoreError::KeyInvalidCharacter("a".to_string()),
        StoreError::KeyInvalidCharacter("b".to_string())
    );
    assert_ne!(StoreError::KeyEmpty, StoreError::ObjectNotFound);
}

#[test]
fn test_store_error_clone() {
    let err = StoreError::KeyInvalidCharacter("abc".to_string());
    let cloned_err = err.clone();
    assert_eq!(err, cloned_err);
}

#[test]
fn test_store_error_debug() {
    let err = StoreError::KeyEmpty;
    assert_eq!(format!("{:?}", err), "KeyEmpty");
}
