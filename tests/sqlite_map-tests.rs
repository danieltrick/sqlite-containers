// SQLiteMap Tests
// This file is part of the 'SQLite-based containers for Rust' project (sqlite-containers)
// SPDX-License-Identifier: Unlicense

mod utilities;

use crate::utilities::{HexEncoder, mix64};
use sqlite_containers::{Blob, SQLiteMap, SizeExt};
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// SQLiteMap<i64, String> Tests
// ---------------------------------------------------------------------------

#[test]
fn test_sqlite_map_i64_string() {
    let mut map = SQLiteMap::<i64, String>::new().unwrap();

    assert_eq!(map.len().unwrap(), 0);
    assert!(map.is_empty().unwrap());
    assert!(!map.contains(&0i64).unwrap());
    assert!(!map.contains(&42i64).unwrap());
    assert!(!map.contains(&i64::MAX).unwrap());
    assert!(!map.contains(&i64::MIN).unwrap());
    assert!(map.get(&0i64).unwrap().is_none());
    assert!(map.get(&42i64).unwrap().is_none());
    assert!(map.get(&i64::MAX).unwrap().is_none());
    assert!(map.get(&i64::MIN).unwrap().is_none());

    assert!(map.insert(&0i64, "12345678").unwrap());
    assert!(map.insert(&42i64, "87654321").unwrap());
    assert!(!map.insert(&0i64, "abcdefgh").unwrap());
    assert!(!map.insert(&42i64, "hgfedcba").unwrap());
    assert!(map.insert(&i64::MAX, "deadbeef").unwrap());
    assert!(!map.insert(&i64::MAX, "stuvwxyz").unwrap());

    assert_eq!(map.len().unwrap(), 3);
    assert!(!map.is_empty().unwrap());
    assert!(map.contains(&0i64).unwrap());
    assert!(map.contains(&42i64).unwrap());
    assert!(map.contains(&i64::MAX).unwrap());
    assert!(!map.contains(&i64::MIN).unwrap());
    assert_eq!(map.get(&0i64).unwrap().unwrap(), "12345678");
    assert_eq!(map.get(&42i64).unwrap().unwrap(), "87654321");
    assert_eq!(map.get(&i64::MAX).unwrap().unwrap(), "deadbeef");
    assert!(map.get(&i64::MIN).unwrap().is_none());

    assert!(map.remove(&0i64).unwrap());
    assert!(!map.remove(&0i64).unwrap());
    assert!(map.remove(&42i64).unwrap());
    assert!(!map.remove(&42i64).unwrap());
    assert!(!map.remove(&i64::MIN).unwrap());

    assert_eq!(map.len().unwrap(), 1);
    assert!(!map.is_empty().unwrap());
    assert!(!map.contains(&0i64).unwrap());
    assert!(!map.contains(&42i64).unwrap());
    assert!(map.contains(&i64::MAX).unwrap());
    assert!(!map.contains(&i64::MIN).unwrap());
    assert!(map.get(&0i64).unwrap().is_none());
    assert!(map.get(&42i64).unwrap().is_none());
    assert_eq!(map.get(&i64::MAX).unwrap().unwrap(), "deadbeef");
    assert!(map.get(&i64::MIN).unwrap().is_none());

    assert!(map.insert(&i64::MIN, "cafebabe").unwrap());
    assert!(!map.insert(&i64::MIN, "cafebabe").unwrap());
    assert!(map.insert(&0i64, "12345678").unwrap());
    assert!(!map.insert(&0i64, "12345678").unwrap());

    assert_eq!(map.len().unwrap(), 3);
    assert!(!map.is_empty().unwrap());
    assert!(map.contains(&0i64).unwrap());
    assert!(!map.contains(&42i64).unwrap());
    assert!(map.contains(&i64::MAX).unwrap());
    assert!(map.contains(&i64::MIN).unwrap());
    assert_eq!(map.get(&0i64).unwrap().unwrap(), "12345678");
    assert!(map.get(&42i64).unwrap().is_none());
    assert_eq!(map.get(&i64::MAX).unwrap().unwrap(), "deadbeef");
    assert_eq!(map.get(&i64::MIN).unwrap().unwrap(), "cafebabe");

    {
        let mut hash = HashMap::new();
        map.for_each(|key, value| assert!(hash.insert(key.to_owned(), value.to_owned()).is_none())).unwrap();
        assert_eq!(hash.len(), 3);
        assert_eq!(hash.get(&0i64).unwrap(), "12345678");
        assert_eq!(hash.get(&i64::MAX).unwrap(), "deadbeef");
        assert_eq!(hash.get(&i64::MIN).unwrap(), "cafebabe");
    }

    assert_eq!(map.find(|key, _| key.eq(&0i64)).unwrap().unwrap().1, "12345678");
    assert!(map.find(|key, _| key.eq(&42i64)).unwrap().is_none());
    assert_eq!(map.find(|key, _| key.eq(&i64::MAX)).unwrap().unwrap().1, "deadbeef");
    assert_eq!(map.find(|key, _| key.eq(&i64::MIN)).unwrap().unwrap().1, "cafebabe");

    map.update(&0i64, "abcdefgh").unwrap();
    map.update(&42i64, "hgfedcba").unwrap();
    map.update(&i64::MAX, "stuvwxyz").unwrap();

    assert_eq!(map.get(&0i64).unwrap().unwrap(), "abcdefgh");
    assert_eq!(map.get(&42i64).unwrap().unwrap(), "hgfedcba");
    assert_eq!(map.get(&i64::MAX).unwrap().unwrap(), "stuvwxyz");
    assert_eq!(map.get(&i64::MIN).unwrap().unwrap(), "cafebabe");

    {
        let mut hash = HashMap::new();
        map.for_each(|key, value| assert!(hash.insert(key.to_owned(), value.to_owned()).is_none())).unwrap();
        assert_eq!(hash.len(), 4);
        assert_eq!(hash.get(&0i64).unwrap(), "abcdefgh");
        assert_eq!(hash.get(&42i64).unwrap(), "hgfedcba");
        assert_eq!(hash.get(&i64::MAX).unwrap(), "stuvwxyz");
        assert_eq!(hash.get(&i64::MIN).unwrap(), "cafebabe");
    }

    assert_eq!(map.find(|key, _| key.eq(&0i64)).unwrap().unwrap().1, "abcdefgh");
    assert_eq!(map.find(|key, _| key.eq(&42i64)).unwrap().unwrap().1, "hgfedcba");
    assert_eq!(map.find(|key, _| key.eq(&i64::MAX)).unwrap().unwrap().1, "stuvwxyz");
    assert_eq!(map.find(|key, _| key.eq(&i64::MIN)).unwrap().unwrap().1, "cafebabe");

    map.clear().unwrap();

    assert_eq!(map.len().unwrap(), 0);
    assert!(map.is_empty().unwrap());
    assert!(!map.contains(&0i64).unwrap());
    assert!(!map.contains(&42i64).unwrap());
    assert!(!map.contains(&i64::MAX).unwrap());
    assert!(!map.contains(&i64::MIN).unwrap());
}

#[test]
fn test_sqlite_map_i64_string_transacted() {
    let mut map = SQLiteMap::<i64, String>::new().unwrap();
    let mut tx = map.transaction().unwrap();

    assert_eq!(tx.len().unwrap(), 0);
    assert!(tx.is_empty().unwrap());
    assert!(!tx.contains(&0i64).unwrap());
    assert!(!tx.contains(&42i64).unwrap());
    assert!(!tx.contains(&i64::MAX).unwrap());
    assert!(!tx.contains(&i64::MIN).unwrap());
    assert!(tx.get(&0i64).unwrap().is_none());
    assert!(tx.get(&42i64).unwrap().is_none());
    assert!(tx.get(&i64::MAX).unwrap().is_none());
    assert!(tx.get(&i64::MIN).unwrap().is_none());

    assert!(tx.insert(&0i64, "12345678").unwrap());
    assert!(tx.insert(&42i64, "87654321").unwrap());
    assert!(!tx.insert(&0i64, "abcdefgh").unwrap());
    assert!(!tx.insert(&42i64, "hgfedcba").unwrap());
    assert!(tx.insert(&i64::MAX, "deadbeef").unwrap());
    assert!(!tx.insert(&i64::MAX, "stuvwxyz").unwrap());

    assert_eq!(tx.len().unwrap(), 3);
    assert!(!tx.is_empty().unwrap());
    assert!(tx.contains(&0i64).unwrap());
    assert!(tx.contains(&42i64).unwrap());
    assert!(tx.contains(&i64::MAX).unwrap());
    assert!(!tx.contains(&i64::MIN).unwrap());
    assert_eq!(tx.get(&0i64).unwrap().unwrap(), "12345678");
    assert_eq!(tx.get(&42i64).unwrap().unwrap(), "87654321");
    assert_eq!(tx.get(&i64::MAX).unwrap().unwrap(), "deadbeef");
    assert!(tx.get(&i64::MIN).unwrap().is_none());

    assert!(tx.remove(&0i64).unwrap());
    assert!(!tx.remove(&0i64).unwrap());
    assert!(tx.remove(&42i64).unwrap());
    assert!(!tx.remove(&42i64).unwrap());
    assert!(!tx.remove(&i64::MIN).unwrap());

    assert_eq!(tx.len().unwrap(), 1);
    assert!(!tx.is_empty().unwrap());
    assert!(!tx.contains(&0i64).unwrap());
    assert!(!tx.contains(&42i64).unwrap());
    assert!(tx.contains(&i64::MAX).unwrap());
    assert!(!tx.contains(&i64::MIN).unwrap());
    assert!(tx.get(&0i64).unwrap().is_none());
    assert!(tx.get(&42i64).unwrap().is_none());
    assert_eq!(tx.get(&i64::MAX).unwrap().unwrap(), "deadbeef");
    assert!(tx.get(&i64::MIN).unwrap().is_none());

    assert!(tx.insert(&i64::MIN, "cafebabe").unwrap());
    assert!(!tx.insert(&i64::MIN, "cafebabe").unwrap());
    assert!(tx.insert(&0i64, "12345678").unwrap());
    assert!(!tx.insert(&0i64, "12345678").unwrap());

    assert_eq!(tx.len().unwrap(), 3);
    assert!(!tx.is_empty().unwrap());
    assert!(tx.contains(&0i64).unwrap());
    assert!(!tx.contains(&42i64).unwrap());
    assert!(tx.contains(&i64::MAX).unwrap());
    assert!(tx.contains(&i64::MIN).unwrap());
    assert_eq!(tx.get(&0i64).unwrap().unwrap(), "12345678");
    assert!(tx.get(&42i64).unwrap().is_none());
    assert_eq!(tx.get(&i64::MAX).unwrap().unwrap(), "deadbeef");
    assert_eq!(tx.get(&i64::MIN).unwrap().unwrap(), "cafebabe");

    {
        let mut hash = HashMap::new();
        tx.for_each(|key, value| assert!(hash.insert(key.to_owned(), value.to_owned()).is_none())).unwrap();
        assert_eq!(hash.len(), 3);
        assert_eq!(hash.get(&0i64).unwrap(), "12345678");
        assert_eq!(hash.get(&i64::MAX).unwrap(), "deadbeef");
        assert_eq!(hash.get(&i64::MIN).unwrap(), "cafebabe");
    }

    assert_eq!(tx.find(|key, _| key.eq(&0i64)).unwrap().unwrap().1, "12345678");
    assert!(tx.find(|key, _| key.eq(&42i64)).unwrap().is_none());
    assert_eq!(tx.find(|key, _| key.eq(&i64::MAX)).unwrap().unwrap().1, "deadbeef");
    assert_eq!(tx.find(|key, _| key.eq(&i64::MIN)).unwrap().unwrap().1, "cafebabe");

    tx.update(&0i64, "abcdefgh").unwrap();
    tx.update(&42i64, "hgfedcba").unwrap();
    tx.update(&i64::MAX, "stuvwxyz").unwrap();

    assert_eq!(tx.get(&0i64).unwrap().unwrap(), "abcdefgh");
    assert_eq!(tx.get(&42i64).unwrap().unwrap(), "hgfedcba");
    assert_eq!(tx.get(&i64::MAX).unwrap().unwrap(), "stuvwxyz");
    assert_eq!(tx.get(&i64::MIN).unwrap().unwrap(), "cafebabe");

    {
        let mut hash = HashMap::new();
        tx.for_each(|key, value| assert!(hash.insert(key.to_owned(), value.to_owned()).is_none())).unwrap();
        assert_eq!(hash.len(), 4);
        assert_eq!(hash.get(&0i64).unwrap(), "abcdefgh");
        assert_eq!(hash.get(&42i64).unwrap(), "hgfedcba");
        assert_eq!(hash.get(&i64::MAX).unwrap(), "stuvwxyz");
        assert_eq!(hash.get(&i64::MIN).unwrap(), "cafebabe");
    }

    assert_eq!(tx.find(|key, _| key.eq(&0i64)).unwrap().unwrap().1, "abcdefgh");
    assert_eq!(tx.find(|key, _| key.eq(&42i64)).unwrap().unwrap().1, "hgfedcba");
    assert_eq!(tx.find(|key, _| key.eq(&i64::MAX)).unwrap().unwrap().1, "stuvwxyz");
    assert_eq!(tx.find(|key, _| key.eq(&i64::MIN)).unwrap().unwrap().1, "cafebabe");

    tx.clear().unwrap();

    assert_eq!(tx.len().unwrap(), 0);
    assert!(tx.is_empty().unwrap());
    assert!(!tx.contains(&0i64).unwrap());
    assert!(!tx.contains(&42i64).unwrap());
    assert!(!tx.contains(&i64::MAX).unwrap());
    assert!(!tx.contains(&i64::MIN).unwrap());

    assert!(tx.insert(&0i64, "ijklmnop").unwrap());
    assert!(tx.insert(&42i64, "ponmlkji").unwrap());

    tx.commit();

    assert_eq!(map.len().unwrap(), 2);
    assert!(!map.is_empty().unwrap());
    assert!(map.contains(&0i64).unwrap());
    assert!(map.contains(&42i64).unwrap());
    assert!(!map.contains(&i64::MAX).unwrap());
    assert!(!map.contains(&i64::MIN).unwrap());
    assert_eq!(map.get(&0i64).unwrap().unwrap(), "ijklmnop");
    assert_eq!(map.get(&42i64).unwrap().unwrap(), "ponmlkji");
    assert!(map.get(&i64::MAX).unwrap().is_none());
    assert!(map.get(&i64::MIN).unwrap().is_none());
}

#[test]
fn test_sqlite_map_i64_string_stresstest() {
    let mut set = SQLiteMap::<i64, String>::new().unwrap();
    let mut hex = HexEncoder::default();
    const MAX_ITEMS: u64 = 1_000_000u64;

    {
        let mut tx = set.transaction().unwrap();
        for i in 0u64..MAX_ITEMS {
            let (key, value) = (mix64(i) as i64, hex.encode(mix64(u64::MAX - i)));
            assert!(tx.insert(&key, value).unwrap());
        }
        for i in 0u64..MAX_ITEMS {
            let (key, value) = (mix64(i) as i64, hex.encode(!mix64(u64::MAX - i)));
            tx.update(&key, value).unwrap();
        }
    }

    assert!(!set.is_empty().unwrap());
    assert_eq!(set.len().unwrap().to_u64(), MAX_ITEMS);

    {
        let tx = set.transaction().unwrap();
        for i in 0u64..MAX_ITEMS {
            assert!(tx.contains(&(mix64(i) as i64)).unwrap());
            assert!(!tx.contains(&(mix64(MAX_ITEMS.checked_add(i).unwrap()) as i64)).unwrap());
        }
        for i in 0u64..MAX_ITEMS {
            let (key, value) = (mix64(i) as i64, hex.encode(!mix64(u64::MAX - i)));
            assert_eq!(tx.get(&key).unwrap().unwrap(), value);
        }
    }

    assert!(!set.is_empty().unwrap());
    assert_eq!(set.len().unwrap().to_u64(), MAX_ITEMS);

    {
        let mut tx = set.transaction().unwrap();
        for i in 0u64..MAX_ITEMS {
            assert!(tx.remove(&(mix64(i) as i64)).unwrap());
        }
        for i in 0u64..MAX_ITEMS {
            assert!(!tx.remove(&(mix64(i) as i64)).unwrap());
        }
    }

    assert!(set.is_empty().unwrap());
}

// ---------------------------------------------------------------------------
// SQLiteMap<Blob, String> Tests
// ---------------------------------------------------------------------------

#[test]
fn test_sqlite_map_blob_string() {
    let mut map = SQLiteMap::<Blob, String>::new().unwrap();

    assert_eq!(map.len().unwrap(), 0);
    assert!(map.is_empty().unwrap());
    assert!(!map.contains(&b"foo"[..]).unwrap());
    assert!(!map.contains(&b"bar"[..]).unwrap());
    assert!(!map.contains(&b"baz"[..]).unwrap());
    assert!(!map.contains(&b"qux"[..]).unwrap());
    assert!(map.get(&b"foo"[..]).unwrap().is_none());
    assert!(map.get(&b"bar"[..]).unwrap().is_none());
    assert!(map.get(&b"baz"[..]).unwrap().is_none());
    assert!(map.get(&b"qux"[..]).unwrap().is_none());

    assert!(map.insert(&b"foo"[..], "12345678").unwrap());
    assert!(map.insert(&b"bar"[..], "87654321").unwrap());
    assert!(!map.insert(&b"foo"[..], "abcdefgh").unwrap());
    assert!(!map.insert(&b"bar"[..], "hgfedcba").unwrap());
    assert!(map.insert(&b"baz"[..], "deadbeef").unwrap());
    assert!(!map.insert(&b"baz"[..], "stuvwxyz").unwrap());

    assert_eq!(map.len().unwrap(), 3);
    assert!(!map.is_empty().unwrap());
    assert!(map.contains(&b"foo"[..]).unwrap());
    assert!(!map.contains(&b"FOO"[..]).unwrap());
    assert!(map.contains(&b"bar"[..]).unwrap());
    assert!(!map.contains(&b"BAR"[..]).unwrap());
    assert!(map.contains(&b"baz"[..]).unwrap());
    assert!(!map.contains(&b"BAZ"[..]).unwrap());
    assert!(!map.contains(&b"qux"[..]).unwrap());
    assert_eq!(map.get(&b"foo"[..]).unwrap().unwrap(), "12345678");
    assert!(map.get(&b"FOO"[..]).unwrap().is_none());
    assert_eq!(map.get(&b"bar"[..]).unwrap().unwrap(), "87654321");
    assert!(map.get(&b"BAR"[..]).unwrap().is_none());
    assert_eq!(map.get(&b"baz"[..]).unwrap().unwrap(), "deadbeef");
    assert!(map.get(&b"BAZ"[..]).unwrap().is_none());
    assert!(map.get(&b"qux"[..]).unwrap().is_none());

    assert!(map.remove(&b"foo"[..]).unwrap());
    assert!(!map.remove(&b"foo"[..]).unwrap());
    assert!(!map.remove(&b"BAR"[..]).unwrap());
    assert!(map.remove(&b"bar"[..]).unwrap());
    assert!(!map.remove(&b"bar"[..]).unwrap());
    assert!(!map.remove(&b"qux"[..]).unwrap());

    assert_eq!(map.len().unwrap(), 1);
    assert!(!map.is_empty().unwrap());
    assert!(!map.contains(&b"foo"[..]).unwrap());
    assert!(!map.contains(&b"bar"[..]).unwrap());
    assert!(map.contains(&b"baz"[..]).unwrap());
    assert!(!map.contains(&b"BAZ"[..]).unwrap());
    assert!(!map.contains(&b"qux"[..]).unwrap());
    assert!(map.get(&b"foo"[..]).unwrap().is_none());
    assert!(map.get(&b"bar"[..]).unwrap().is_none());
    assert_eq!(map.get(&b"baz"[..]).unwrap().unwrap(), "deadbeef");
    assert!(map.get(&b"BAZ"[..]).unwrap().is_none());
    assert!(map.get(&b"qux"[..]).unwrap().is_none());

    assert!(map.insert(&b"qux"[..], "cafebabe").unwrap());
    assert!(!map.insert(&b"qux"[..], "cafebabe").unwrap());
    assert!(map.insert(&b"foo"[..], "12345678").unwrap());
    assert!(!map.insert(&b"foo"[..], "12345678").unwrap());

    assert_eq!(map.len().unwrap(), 3);
    assert!(!map.is_empty().unwrap());
    assert!(map.contains(&b"foo"[..]).unwrap());
    assert!(!map.contains(&b"FOO"[..]).unwrap());
    assert!(!map.contains(&b"bar"[..]).unwrap());
    assert!(map.contains(&b"baz"[..]).unwrap());
    assert!(!map.contains(&b"BAZ"[..]).unwrap());
    assert!(map.contains(&b"qux"[..]).unwrap());
    assert!(!map.contains(&b"QUX"[..]).unwrap());
    assert_eq!(map.get(&b"foo"[..]).unwrap().unwrap(), "12345678");
    assert!(map.get(&b"FOO"[..]).unwrap().is_none());
    assert!(map.get(&b"bar"[..]).unwrap().is_none());
    assert_eq!(map.get(&b"baz"[..]).unwrap().unwrap(), "deadbeef");
    assert!(map.get(&b"BAZ"[..]).unwrap().is_none());
    assert_eq!(map.get(&b"qux"[..]).unwrap().unwrap(), "cafebabe");
    assert!(map.get(&b"QUX"[..]).unwrap().is_none());

    {
        let mut hash = HashMap::new();
        map.for_each(|key, value| assert!(hash.insert(key.to_owned(), value.to_owned()).is_none())).unwrap();
        assert_eq!(hash.len(), 3);
        assert_eq!(hash.get(&b"foo"[..]).unwrap(), "12345678");
        assert_eq!(hash.get(&b"baz"[..]).unwrap(), "deadbeef");
        assert_eq!(hash.get(&b"qux"[..]).unwrap(), "cafebabe");
    }

    assert_eq!(map.find(|key, _| key.eq(b"foo")).unwrap().unwrap().1, "12345678");
    assert_eq!(map.find(|key, _| key.eq_ignore_ascii_case(b"FOO")).unwrap().unwrap().1, "12345678");
    assert!(map.find(|key, _| key.eq(b"bar")).unwrap().is_none());

    map.update(&b"foo"[..], "abcdefgh").unwrap();
    map.update(&b"bar"[..], "hgfedcba").unwrap();
    map.update(&b"baz"[..], "stuvwxyz").unwrap();

    assert_eq!(map.get(&b"foo"[..]).unwrap().unwrap(), "abcdefgh");
    assert_eq!(map.get(&b"bar"[..]).unwrap().unwrap(), "hgfedcba");
    assert_eq!(map.get(&b"baz"[..]).unwrap().unwrap(), "stuvwxyz");
    assert_eq!(map.get(&b"qux"[..]).unwrap().unwrap(), "cafebabe");

    {
        let mut hash = HashMap::new();
        map.for_each(|key, value| assert!(hash.insert(key.to_owned(), value.to_owned()).is_none())).unwrap();
        assert_eq!(hash.len(), 4);
        assert_eq!(hash.get(&b"foo"[..]).unwrap(), "abcdefgh");
        assert_eq!(hash.get(&b"bar"[..]).unwrap(), "hgfedcba");
        assert_eq!(hash.get(&b"baz"[..]).unwrap(), "stuvwxyz");
        assert_eq!(hash.get(&b"qux"[..]).unwrap(), "cafebabe");
    }

    assert_eq!(map.find(|key, _| key.eq(b"foo")).unwrap().unwrap().1, "abcdefgh");
    assert_eq!(map.find(|key, _| key.eq_ignore_ascii_case(b"FOO")).unwrap().unwrap().1, "abcdefgh");
    assert_eq!(map.find(|key, _| key.eq(b"bar")).unwrap().unwrap().1, "hgfedcba");
    assert_eq!(map.find(|key, _| key.eq_ignore_ascii_case(b"BAR")).unwrap().unwrap().1, "hgfedcba");
    assert!(map.find(|key, _| key.eq(b"xyz")).unwrap().is_none());

    map.clear().unwrap();

    assert_eq!(map.len().unwrap(), 0);
    assert!(map.is_empty().unwrap());
    assert!(!map.contains(&b"foo"[..]).unwrap());
    assert!(!map.contains(&b"bar"[..]).unwrap());
    assert!(!map.contains(&b"baz"[..]).unwrap());
    assert!(!map.contains(&b"qux"[..]).unwrap());
}

#[test]
fn test_sqlite_map_blob_string_transacted() {
    let mut map = SQLiteMap::<Blob, String>::new().unwrap();
    let mut tx = map.transaction().unwrap();

    assert_eq!(tx.len().unwrap(), 0);
    assert!(tx.is_empty().unwrap());
    assert!(!tx.contains(&b"foo"[..]).unwrap());
    assert!(!tx.contains(&b"bar"[..]).unwrap());
    assert!(!tx.contains(&b"baz"[..]).unwrap());
    assert!(!tx.contains(&b"qux"[..]).unwrap());
    assert!(tx.get(&b"foo"[..]).unwrap().is_none());
    assert!(tx.get(&b"bar"[..]).unwrap().is_none());
    assert!(tx.get(&b"baz"[..]).unwrap().is_none());
    assert!(tx.get(&b"qux"[..]).unwrap().is_none());

    assert!(tx.insert(&b"foo"[..], "12345678").unwrap());
    assert!(tx.insert(&b"bar"[..], "87654321").unwrap());
    assert!(!tx.insert(&b"foo"[..], "abcdefgh").unwrap());
    assert!(!tx.insert(&b"bar"[..], "hgfedcba").unwrap());
    assert!(tx.insert(&b"baz"[..], "deadbeef").unwrap());
    assert!(!tx.insert(&b"baz"[..], "stuvwxyz").unwrap());

    assert_eq!(tx.len().unwrap(), 3);
    assert!(!tx.is_empty().unwrap());
    assert!(tx.contains(&b"foo"[..]).unwrap());
    assert!(!tx.contains(&b"FOO"[..]).unwrap());
    assert!(tx.contains(&b"bar"[..]).unwrap());
    assert!(!tx.contains(&b"BAR"[..]).unwrap());
    assert!(tx.contains(&b"baz"[..]).unwrap());
    assert!(!tx.contains(&b"BAZ"[..]).unwrap());
    assert!(!tx.contains(&b"qux"[..]).unwrap());
    assert_eq!(tx.get(&b"foo"[..]).unwrap().unwrap(), "12345678");
    assert!(tx.get(&b"FOO"[..]).unwrap().is_none());
    assert_eq!(tx.get(&b"bar"[..]).unwrap().unwrap(), "87654321");
    assert!(tx.get(&b"BAR"[..]).unwrap().is_none());
    assert_eq!(tx.get(&b"baz"[..]).unwrap().unwrap(), "deadbeef");
    assert!(tx.get(&b"BAZ"[..]).unwrap().is_none());
    assert!(tx.get(&b"qux"[..]).unwrap().is_none());

    assert!(tx.remove(&b"foo"[..]).unwrap());
    assert!(!tx.remove(&b"foo"[..]).unwrap());
    assert!(!tx.remove(&b"BAR"[..]).unwrap());
    assert!(tx.remove(&b"bar"[..]).unwrap());
    assert!(!tx.remove(&b"bar"[..]).unwrap());
    assert!(!tx.remove(&b"qux"[..]).unwrap());

    assert_eq!(tx.len().unwrap(), 1);
    assert!(!tx.is_empty().unwrap());
    assert!(!tx.contains(&b"foo"[..]).unwrap());
    assert!(!tx.contains(&b"bar"[..]).unwrap());
    assert!(tx.contains(&b"baz"[..]).unwrap());
    assert!(!tx.contains(&b"BAZ"[..]).unwrap());
    assert!(!tx.contains(&b"qux"[..]).unwrap());
    assert!(tx.get(&b"foo"[..]).unwrap().is_none());
    assert!(tx.get(&b"bar"[..]).unwrap().is_none());
    assert_eq!(tx.get(&b"baz"[..]).unwrap().unwrap(), "deadbeef");
    assert!(tx.get(&b"BAZ"[..]).unwrap().is_none());
    assert!(tx.get(&b"qux"[..]).unwrap().is_none());

    assert!(tx.insert(&b"qux"[..], "cafebabe").unwrap());
    assert!(!tx.insert(&b"qux"[..], "cafebabe").unwrap());
    assert!(tx.insert(&b"foo"[..], "12345678").unwrap());
    assert!(!tx.insert(&b"foo"[..], "12345678").unwrap());

    assert_eq!(tx.len().unwrap(), 3);
    assert!(!tx.is_empty().unwrap());
    assert!(tx.contains(&b"foo"[..]).unwrap());
    assert!(!tx.contains(&b"FOO"[..]).unwrap());
    assert!(!tx.contains(&b"bar"[..]).unwrap());
    assert!(tx.contains(&b"baz"[..]).unwrap());
    assert!(!tx.contains(&b"BAZ"[..]).unwrap());
    assert!(tx.contains(&b"qux"[..]).unwrap());
    assert!(!tx.contains(&b"QUX"[..]).unwrap());
    assert_eq!(tx.get(&b"foo"[..]).unwrap().unwrap(), "12345678");
    assert!(tx.get(&b"FOO"[..]).unwrap().is_none());
    assert!(tx.get(&b"bar"[..]).unwrap().is_none());
    assert_eq!(tx.get(&b"baz"[..]).unwrap().unwrap(), "deadbeef");
    assert!(tx.get(&b"BAZ"[..]).unwrap().is_none());
    assert_eq!(tx.get(&b"qux"[..]).unwrap().unwrap(), "cafebabe");
    assert!(tx.get(&b"QUX"[..]).unwrap().is_none());

    {
        let mut hash = HashMap::new();
        tx.for_each(|key, value| assert!(hash.insert(key.to_owned(), value.to_owned()).is_none())).unwrap();
        assert_eq!(hash.len(), 3);
        assert_eq!(hash.get(&b"foo"[..]).unwrap(), "12345678");
        assert_eq!(hash.get(&b"baz"[..]).unwrap(), "deadbeef");
        assert_eq!(hash.get(&b"qux"[..]).unwrap(), "cafebabe");
    }

    assert_eq!(tx.find(|key, _| key.eq(b"foo")).unwrap().unwrap().1, "12345678");
    assert_eq!(tx.find(|key, _| key.eq_ignore_ascii_case(b"FOO")).unwrap().unwrap().1, "12345678");
    assert!(tx.find(|key, _| key.eq(b"bar")).unwrap().is_none());

    tx.update(&b"foo"[..], "abcdefgh").unwrap();
    tx.update(&b"bar"[..], "hgfedcba").unwrap();
    tx.update(&b"baz"[..], "stuvwxyz").unwrap();

    assert_eq!(tx.get(&b"foo"[..]).unwrap().unwrap(), "abcdefgh");
    assert_eq!(tx.get(&b"bar"[..]).unwrap().unwrap(), "hgfedcba");
    assert_eq!(tx.get(&b"baz"[..]).unwrap().unwrap(), "stuvwxyz");
    assert_eq!(tx.get(&b"qux"[..]).unwrap().unwrap(), "cafebabe");

    {
        let mut hash = HashMap::new();
        tx.for_each(|key, value| assert!(hash.insert(key.to_owned(), value.to_owned()).is_none())).unwrap();
        assert_eq!(hash.len(), 4);
        assert_eq!(hash.get(&b"foo"[..]).unwrap(), "abcdefgh");
        assert_eq!(hash.get(&b"bar"[..]).unwrap(), "hgfedcba");
        assert_eq!(hash.get(&b"baz"[..]).unwrap(), "stuvwxyz");
        assert_eq!(hash.get(&b"qux"[..]).unwrap(), "cafebabe");
    }

    assert_eq!(tx.find(|key, _| key.eq(b"foo")).unwrap().unwrap().1, "abcdefgh");
    assert_eq!(tx.find(|key, _| key.eq_ignore_ascii_case(b"FOO")).unwrap().unwrap().1, "abcdefgh");
    assert_eq!(tx.find(|key, _| key.eq(b"bar")).unwrap().unwrap().1, "hgfedcba");
    assert_eq!(tx.find(|key, _| key.eq_ignore_ascii_case(b"BAR")).unwrap().unwrap().1, "hgfedcba");
    assert!(tx.find(|key, _| key.eq(b"xyz")).unwrap().is_none());

    tx.clear().unwrap();

    assert_eq!(tx.len().unwrap(), 0);
    assert!(tx.is_empty().unwrap());
    assert!(!tx.contains(&b"foo"[..]).unwrap());
    assert!(!tx.contains(&b"bar"[..]).unwrap());
    assert!(!tx.contains(&b"baz"[..]).unwrap());
    assert!(!tx.contains(&b"qux"[..]).unwrap());

    assert!(tx.insert(&b"foo"[..], "ijklmnop").unwrap());
    assert!(tx.insert(&b"bar"[..], "ponmlkji").unwrap());

    tx.commit();

    assert_eq!(map.len().unwrap(), 2);
    assert!(!map.is_empty().unwrap());
    assert!(map.contains(&b"foo"[..]).unwrap());
    assert!(map.contains(&b"bar"[..]).unwrap());
    assert!(!map.contains(&b"baz"[..]).unwrap());
    assert!(!map.contains(&b"qux"[..]).unwrap());
    assert_eq!(map.get(&b"foo"[..]).unwrap().unwrap(), "ijklmnop");
    assert_eq!(map.get(&b"bar"[..]).unwrap().unwrap(), "ponmlkji");
    assert!(map.get(&b"baz"[..]).unwrap().is_none());
    assert!(map.get(&b"qux"[..]).unwrap().is_none());
}

#[test]
fn test_sqlite_map_blob_string_stresstest() {
    let mut set = SQLiteMap::<Blob, String>::new().unwrap();
    let (mut hex_1, mut hex_2) = (HexEncoder::default(), HexEncoder::default());
    const MAX_ITEMS: u64 = 1_000_000u64;

    {
        let mut tx = set.transaction().unwrap();
        for i in 0u64..MAX_ITEMS {
            let (key, value) = (hex_1.encode_raw(mix64(i)), hex_2.encode(mix64(u64::MAX - i)));
            assert!(tx.insert(key, value).unwrap());
        }
        for i in 0u64..MAX_ITEMS {
            let (key, value) = (hex_1.encode_raw(mix64(i)), hex_2.encode(!mix64(u64::MAX - i)));
            tx.update(key, value).unwrap();
        }
    }

    assert!(!set.is_empty().unwrap());
    assert_eq!(set.len().unwrap().to_u64(), MAX_ITEMS);

    {
        let tx = set.transaction().unwrap();
        for i in 0u64..MAX_ITEMS {
            assert!(tx.contains(hex_1.encode_raw(mix64(i))).unwrap());
            assert!(!tx.contains(hex_1.encode_raw(mix64(MAX_ITEMS.checked_add(i).unwrap()))).unwrap());
        }
        for i in 0u64..MAX_ITEMS {
            let (key, value) = (hex_1.encode_raw(mix64(i)), hex_2.encode(!mix64(u64::MAX - i)));
            assert_eq!(tx.get(key).unwrap().unwrap(), value);
        }
    }

    assert!(!set.is_empty().unwrap());
    assert_eq!(set.len().unwrap().to_u64(), MAX_ITEMS);

    {
        let mut tx = set.transaction().unwrap();
        for i in 0u64..MAX_ITEMS {
            assert!(tx.remove(hex_1.encode_raw(mix64(i))).unwrap());
        }
        for i in 0u64..MAX_ITEMS {
            assert!(!tx.remove(hex_1.encode_raw(mix64(i))).unwrap());
        }
    }

    assert!(set.is_empty().unwrap());
}

// ---------------------------------------------------------------------------
// SQLiteMap<String, String> Tests
// ---------------------------------------------------------------------------

#[test]
fn test_sqlite_map_string_string() {
    let mut map = SQLiteMap::<String, String>::new().unwrap();

    assert_eq!(map.len().unwrap(), 0);
    assert!(map.is_empty().unwrap());
    assert!(!map.contains("foo").unwrap());
    assert!(!map.contains("bar").unwrap());
    assert!(!map.contains("baz").unwrap());
    assert!(!map.contains("qux").unwrap());
    assert!(map.get("foo").unwrap().is_none());
    assert!(map.get("bar").unwrap().is_none());
    assert!(map.get("baz").unwrap().is_none());
    assert!(map.get("qux").unwrap().is_none());

    assert!(map.insert("foo", "12345678").unwrap());
    assert!(map.insert("bar", "87654321").unwrap());
    assert!(!map.insert("foo", "abcdefgh").unwrap());
    assert!(!map.insert("bar", "hgfedcba").unwrap());
    assert!(map.insert("baz", "deadbeef").unwrap());
    assert!(!map.insert("baz", "stuvwxyz").unwrap());

    assert_eq!(map.len().unwrap(), 3);
    assert!(!map.is_empty().unwrap());
    assert!(map.contains("foo").unwrap());
    assert!(!map.contains("FOO").unwrap());
    assert!(map.contains_nocase("FOO").unwrap());
    assert!(map.contains("bar").unwrap());
    assert!(!map.contains("BAR").unwrap());
    assert!(map.contains_nocase("BAR").unwrap());
    assert!(map.contains("baz").unwrap());
    assert!(!map.contains("BAZ").unwrap());
    assert!(map.contains_nocase("BAZ").unwrap());
    assert!(!map.contains("qux").unwrap());
    assert_eq!(map.get("foo").unwrap().unwrap(), "12345678");
    assert!(map.get("FOO").unwrap().is_none());
    assert_eq!(map.get_nocase("FOO").unwrap().unwrap(), "12345678");
    assert_eq!(map.get("bar").unwrap().unwrap(), "87654321");
    assert!(map.get("BAR").unwrap().is_none());
    assert_eq!(map.get_nocase("BAR").unwrap().unwrap(), "87654321");
    assert_eq!(map.get("baz").unwrap().unwrap(), "deadbeef");
    assert!(map.get("BAZ").unwrap().is_none());
    assert_eq!(map.get_nocase("BAZ").unwrap().unwrap(), "deadbeef");
    assert!(map.get("qux").unwrap().is_none());

    assert!(map.remove("foo").unwrap());
    assert!(!map.remove("foo").unwrap());
    assert!(!map.remove("BAR").unwrap());
    assert!(map.remove_nocase("BAR").unwrap());
    assert!(!map.remove_nocase("BAR").unwrap());
    assert!(!map.remove("qux").unwrap());

    assert_eq!(map.len().unwrap(), 1);
    assert!(!map.is_empty().unwrap());
    assert!(!map.contains("foo").unwrap());
    assert!(!map.contains("bar").unwrap());
    assert!(map.contains("baz").unwrap());
    assert!(!map.contains("BAZ").unwrap());
    assert!(map.contains_nocase("BAZ").unwrap());
    assert!(!map.contains("qux").unwrap());
    assert!(map.get("foo").unwrap().is_none());
    assert!(map.get("bar").unwrap().is_none());
    assert_eq!(map.get("baz").unwrap().unwrap(), "deadbeef");
    assert!(map.get("BAZ").unwrap().is_none());
    assert_eq!(map.get_nocase("BAZ").unwrap().unwrap(), "deadbeef");
    assert!(map.get("qux").unwrap().is_none());

    assert!(map.insert("qux", "cafebabe").unwrap());
    assert!(!map.insert("qux", "cafebabe").unwrap());
    assert!(map.insert("foo", "12345678").unwrap());
    assert!(!map.insert("foo", "12345678").unwrap());

    assert_eq!(map.len().unwrap(), 3);
    assert!(!map.is_empty().unwrap());
    assert!(map.contains("foo").unwrap());
    assert!(!map.contains("FOO").unwrap());
    assert!(map.contains_nocase("FOO").unwrap());
    assert!(!map.contains("bar").unwrap());
    assert!(map.contains("baz").unwrap());
    assert!(!map.contains("BAZ").unwrap());
    assert!(map.contains_nocase("BAZ").unwrap());
    assert!(map.contains("qux").unwrap());
    assert!(!map.contains("QUX").unwrap());
    assert!(map.contains_nocase("QUX").unwrap());
    assert_eq!(map.get("foo").unwrap().unwrap(), "12345678");
    assert!(map.get("FOO").unwrap().is_none());
    assert_eq!(map.get_nocase("FOO").unwrap().unwrap(), "12345678");
    assert!(map.get("bar").unwrap().is_none());
    assert_eq!(map.get("baz").unwrap().unwrap(), "deadbeef");
    assert!(map.get("BAZ").unwrap().is_none());
    assert_eq!(map.get_nocase("BAZ").unwrap().unwrap(), "deadbeef");
    assert_eq!(map.get("qux").unwrap().unwrap(), "cafebabe");
    assert!(map.get("QUX").unwrap().is_none());
    assert_eq!(map.get_nocase("QUX").unwrap().unwrap(), "cafebabe");

    {
        let mut hash = HashMap::new();
        map.for_each(|key, value| assert!(hash.insert(key.to_owned(), value.to_owned()).is_none())).unwrap();
        assert_eq!(hash.len(), 3);
        assert_eq!(hash.get("foo").unwrap(), "12345678");
        assert_eq!(hash.get("baz").unwrap(), "deadbeef");
        assert_eq!(hash.get("qux").unwrap(), "cafebabe");
    }

    assert_eq!(map.find(|key, _| key.eq("foo")).unwrap().unwrap().1, "12345678");
    assert_eq!(map.find(|key, _| key.eq_ignore_ascii_case("FOO")).unwrap().unwrap().1, "12345678");
    assert!(map.find(|key, _| key.eq("bar")).unwrap().is_none());

    map.update("foo", "abcdefgh").unwrap();
    map.update("bar", "hgfedcba").unwrap();
    map.update("baz", "stuvwxyz").unwrap();

    assert_eq!(map.get("foo").unwrap().unwrap(), "abcdefgh");
    assert_eq!(map.get("bar").unwrap().unwrap(), "hgfedcba");
    assert_eq!(map.get("baz").unwrap().unwrap(), "stuvwxyz");
    assert_eq!(map.get("qux").unwrap().unwrap(), "cafebabe");

    {
        let mut hash = HashMap::new();
        map.for_each(|key, value| assert!(hash.insert(key.to_owned(), value.to_owned()).is_none())).unwrap();
        assert_eq!(hash.len(), 4);
        assert_eq!(hash.get("foo").unwrap(), "abcdefgh");
        assert_eq!(hash.get("bar").unwrap(), "hgfedcba");
        assert_eq!(hash.get("baz").unwrap(), "stuvwxyz");
        assert_eq!(hash.get("qux").unwrap(), "cafebabe");
    }

    assert_eq!(map.find(|key, _| key.eq("foo")).unwrap().unwrap().1, "abcdefgh");
    assert_eq!(map.find(|key, _| key.eq_ignore_ascii_case("FOO")).unwrap().unwrap().1, "abcdefgh");
    assert_eq!(map.find(|key, _| key.eq("bar")).unwrap().unwrap().1, "hgfedcba");
    assert_eq!(map.find(|key, _| key.eq_ignore_ascii_case("BAR")).unwrap().unwrap().1, "hgfedcba");
    assert!(map.find(|key, _| key.eq("xyz")).unwrap().is_none());

    map.clear().unwrap();

    assert_eq!(map.len().unwrap(), 0);
    assert!(map.is_empty().unwrap());
    assert!(!map.contains("foo").unwrap());
    assert!(!map.contains("bar").unwrap());
    assert!(!map.contains("baz").unwrap());
    assert!(!map.contains("qux").unwrap());
}

#[test]
fn test_sqlite_map_string_string_transacted() {
    let mut map = SQLiteMap::<String, String>::new().unwrap();
    let mut tx = map.transaction().unwrap();

    assert_eq!(tx.len().unwrap(), 0);
    assert!(tx.is_empty().unwrap());
    assert!(!tx.contains("foo").unwrap());
    assert!(!tx.contains("bar").unwrap());
    assert!(!tx.contains("baz").unwrap());
    assert!(!tx.contains("qux").unwrap());
    assert!(tx.get("foo").unwrap().is_none());
    assert!(tx.get("bar").unwrap().is_none());
    assert!(tx.get("baz").unwrap().is_none());
    assert!(tx.get("qux").unwrap().is_none());

    assert!(tx.insert("foo", "12345678").unwrap());
    assert!(tx.insert("bar", "87654321").unwrap());
    assert!(!tx.insert("foo", "abcdefgh").unwrap());
    assert!(!tx.insert("bar", "hgfedcba").unwrap());
    assert!(tx.insert("baz", "deadbeef").unwrap());
    assert!(!tx.insert("baz", "stuvwxyz").unwrap());

    assert_eq!(tx.len().unwrap(), 3);
    assert!(!tx.is_empty().unwrap());
    assert!(tx.contains("foo").unwrap());
    assert!(!tx.contains("FOO").unwrap());
    assert!(tx.contains_nocase("FOO").unwrap());
    assert!(tx.contains("bar").unwrap());
    assert!(!tx.contains("BAR").unwrap());
    assert!(tx.contains_nocase("BAR").unwrap());
    assert!(tx.contains("baz").unwrap());
    assert!(!tx.contains("BAZ").unwrap());
    assert!(tx.contains_nocase("BAZ").unwrap());
    assert!(!tx.contains("qux").unwrap());
    assert_eq!(tx.get("foo").unwrap().unwrap(), "12345678");
    assert!(tx.get("FOO").unwrap().is_none());
    assert_eq!(tx.get_nocase("FOO").unwrap().unwrap(), "12345678");
    assert_eq!(tx.get("bar").unwrap().unwrap(), "87654321");
    assert!(tx.get("BAR").unwrap().is_none());
    assert_eq!(tx.get_nocase("BAR").unwrap().unwrap(), "87654321");
    assert_eq!(tx.get("baz").unwrap().unwrap(), "deadbeef");
    assert!(tx.get("BAZ").unwrap().is_none());
    assert_eq!(tx.get_nocase("BAZ").unwrap().unwrap(), "deadbeef");
    assert!(tx.get("qux").unwrap().is_none());

    assert!(tx.remove("foo").unwrap());
    assert!(!tx.remove("foo").unwrap());
    assert!(!tx.remove("BAR").unwrap());
    assert!(tx.remove_nocase("BAR").unwrap());
    assert!(!tx.remove_nocase("BAR").unwrap());
    assert!(!tx.remove("qux").unwrap());

    assert_eq!(tx.len().unwrap(), 1);
    assert!(!tx.is_empty().unwrap());
    assert!(!tx.contains("foo").unwrap());
    assert!(!tx.contains("bar").unwrap());
    assert!(tx.contains("baz").unwrap());
    assert!(!tx.contains("BAZ").unwrap());
    assert!(tx.contains_nocase("BAZ").unwrap());
    assert!(!tx.contains("qux").unwrap());
    assert!(tx.get("foo").unwrap().is_none());
    assert!(tx.get("bar").unwrap().is_none());
    assert_eq!(tx.get("baz").unwrap().unwrap(), "deadbeef");
    assert!(tx.get("BAZ").unwrap().is_none());
    assert_eq!(tx.get_nocase("BAZ").unwrap().unwrap(), "deadbeef");
    assert!(tx.get("qux").unwrap().is_none());

    assert!(tx.insert("qux", "cafebabe").unwrap());
    assert!(!tx.insert("qux", "cafebabe").unwrap());
    assert!(tx.insert("foo", "12345678").unwrap());
    assert!(!tx.insert("foo", "12345678").unwrap());

    assert_eq!(tx.len().unwrap(), 3);
    assert!(!tx.is_empty().unwrap());
    assert!(tx.contains("foo").unwrap());
    assert!(!tx.contains("FOO").unwrap());
    assert!(tx.contains_nocase("FOO").unwrap());
    assert!(!tx.contains("bar").unwrap());
    assert!(tx.contains("baz").unwrap());
    assert!(!tx.contains("BAZ").unwrap());
    assert!(tx.contains_nocase("BAZ").unwrap());
    assert!(tx.contains("qux").unwrap());
    assert!(!tx.contains("QUX").unwrap());
    assert!(tx.contains_nocase("QUX").unwrap());
    assert_eq!(tx.get("foo").unwrap().unwrap(), "12345678");
    assert!(tx.get("FOO").unwrap().is_none());
    assert_eq!(tx.get_nocase("FOO").unwrap().unwrap(), "12345678");
    assert!(tx.get("bar").unwrap().is_none());
    assert_eq!(tx.get("baz").unwrap().unwrap(), "deadbeef");
    assert!(tx.get("BAZ").unwrap().is_none());
    assert_eq!(tx.get_nocase("BAZ").unwrap().unwrap(), "deadbeef");
    assert_eq!(tx.get("qux").unwrap().unwrap(), "cafebabe");
    assert!(tx.get("QUX").unwrap().is_none());
    assert_eq!(tx.get_nocase("QUX").unwrap().unwrap(), "cafebabe");

    {
        let mut hash = HashMap::new();
        tx.for_each(|key, value| assert!(hash.insert(key.to_owned(), value.to_owned()).is_none())).unwrap();
        assert_eq!(hash.len(), 3);
        assert_eq!(hash.get("foo").unwrap(), "12345678");
        assert_eq!(hash.get("baz").unwrap(), "deadbeef");
        assert_eq!(hash.get("qux").unwrap(), "cafebabe");
    }

    assert_eq!(tx.find(|key, _| key.eq("foo")).unwrap().unwrap().1, "12345678");
    assert_eq!(tx.find(|key, _| key.eq_ignore_ascii_case("FOO")).unwrap().unwrap().1, "12345678");
    assert!(tx.find(|key, _| key.eq("bar")).unwrap().is_none());

    tx.update("foo", "abcdefgh").unwrap();
    tx.update("bar", "hgfedcba").unwrap();
    tx.update("baz", "stuvwxyz").unwrap();

    assert_eq!(tx.get("foo").unwrap().unwrap(), "abcdefgh");
    assert_eq!(tx.get("bar").unwrap().unwrap(), "hgfedcba");
    assert_eq!(tx.get("baz").unwrap().unwrap(), "stuvwxyz");
    assert_eq!(tx.get("qux").unwrap().unwrap(), "cafebabe");

    {
        let mut hash = HashMap::new();
        tx.for_each(|key, value| assert!(hash.insert(key.to_owned(), value.to_owned()).is_none())).unwrap();
        assert_eq!(hash.len(), 4);
        assert_eq!(hash.get("foo").unwrap(), "abcdefgh");
        assert_eq!(hash.get("bar").unwrap(), "hgfedcba");
        assert_eq!(hash.get("baz").unwrap(), "stuvwxyz");
        assert_eq!(hash.get("qux").unwrap(), "cafebabe");
    }

    assert_eq!(tx.find(|key, _| key.eq("foo")).unwrap().unwrap().1, "abcdefgh");
    assert_eq!(tx.find(|key, _| key.eq_ignore_ascii_case("FOO")).unwrap().unwrap().1, "abcdefgh");
    assert_eq!(tx.find(|key, _| key.eq("bar")).unwrap().unwrap().1, "hgfedcba");
    assert_eq!(tx.find(|key, _| key.eq_ignore_ascii_case("BAR")).unwrap().unwrap().1, "hgfedcba");
    assert!(tx.find(|key, _| key.eq("xyz")).unwrap().is_none());

    tx.clear().unwrap();

    assert_eq!(tx.len().unwrap(), 0);
    assert!(tx.is_empty().unwrap());
    assert!(!tx.contains("foo").unwrap());
    assert!(!tx.contains("bar").unwrap());
    assert!(!tx.contains("baz").unwrap());
    assert!(!tx.contains("qux").unwrap());

    assert!(tx.insert("foo", "ijklmnop").unwrap());
    assert!(tx.insert("bar", "ponmlkji").unwrap());

    tx.commit();

    assert_eq!(map.len().unwrap(), 2);
    assert!(!map.is_empty().unwrap());
    assert!(map.contains("foo").unwrap());
    assert!(map.contains("bar").unwrap());
    assert!(!map.contains("baz").unwrap());
    assert!(!map.contains("qux").unwrap());
    assert_eq!(map.get("foo").unwrap().unwrap(), "ijklmnop");
    assert_eq!(map.get("bar").unwrap().unwrap(), "ponmlkji");
    assert!(map.get("baz").unwrap().is_none());
    assert!(map.get("qux").unwrap().is_none());
}

#[test]
fn test_sqlite_map_string_string_ignorecase() {
    let mut map_1 = SQLiteMap::<String, String>::new().unwrap();
    let mut map_2 = SQLiteMap::<String, String>::with_nocase().unwrap();

    assert!(map_1.insert("foo", "12345678").unwrap());
    assert!(map_1.insert("FOO", "87654321").unwrap());
    assert!(map_1.insert("BAR", "abcdefgh").unwrap());
    assert!(map_1.insert("bar", "hgfedcba").unwrap());

    assert!(map_2.insert("foo", "12345678").unwrap());
    assert!(!map_2.insert("FOO", "87654321").unwrap());
    assert!(map_2.insert("BAR", "abcdefgh").unwrap());
    assert!(!map_2.insert("bar", "hgfedcba").unwrap());

    assert!(map_1.remove("FOO").unwrap());
    assert!(map_1.remove("bar").unwrap());

    assert!(map_1.contains("foo").unwrap());
    assert!(!map_1.contains("FOO").unwrap());
    assert!(!map_1.contains("bar").unwrap());
    assert!(map_1.contains("BAR").unwrap());
    assert_eq!(map_1.get("foo").unwrap().unwrap(), "12345678");
    assert!(map_1.get("FOO").unwrap().is_none());
    assert!(map_1.get("bar").unwrap().is_none());
    assert_eq!(map_1.get("BAR").unwrap().unwrap(), "abcdefgh");

    assert!(map_2.contains("foo").unwrap());
    assert!(map_2.contains("FOO").unwrap());
    assert!(map_2.contains("bar").unwrap());
    assert!(map_2.contains("BAR").unwrap());
    assert_eq!(map_2.get("foo").unwrap().unwrap(), "12345678");
    assert_eq!(map_2.get("FOO").unwrap().unwrap(), "12345678");
    assert_eq!(map_2.get("bar").unwrap().unwrap(), "abcdefgh");
    assert_eq!(map_2.get("BAR").unwrap().unwrap(), "abcdefgh");

    assert!(map_2.remove("FOO").unwrap());
    assert!(map_2.remove("bar").unwrap());

    assert!(!map_2.contains("foo").unwrap());
    assert!(!map_2.contains("FOO").unwrap());
    assert!(!map_2.contains("bar").unwrap());
    assert!(!map_2.contains("BAR").unwrap());
    assert!(map_2.get("foo").unwrap().is_none());
    assert!(map_2.get("FOO").unwrap().is_none());
    assert!(map_2.get("bar").unwrap().is_none());
    assert!(map_2.get("BAR").unwrap().is_none());
}

#[test]
fn test_sqlite_map_string_string_stresstest() {
    let mut set = SQLiteMap::<String, String>::new().unwrap();
    let (mut hex_1, mut hex_2) = (HexEncoder::default(), HexEncoder::default());
    const MAX_ITEMS: u64 = 1_000_000u64;

    {
        let mut tx = set.transaction().unwrap();
        for i in 0u64..MAX_ITEMS {
            let (key, value) = (hex_1.encode(mix64(i)), hex_2.encode(mix64(u64::MAX - i)));
            assert!(tx.insert(key, value).unwrap());
        }
        for i in 0u64..MAX_ITEMS {
            let (key, value) = (hex_1.encode(mix64(i)), hex_2.encode(!mix64(u64::MAX - i)));
            tx.update(key, value).unwrap();
        }
    }

    assert!(!set.is_empty().unwrap());
    assert_eq!(set.len().unwrap().to_u64(), MAX_ITEMS);

    {
        let tx = set.transaction().unwrap();
        for i in 0u64..MAX_ITEMS {
            assert!(tx.contains(hex_1.encode(mix64(i))).unwrap());
            assert!(!tx.contains(hex_1.encode(mix64(MAX_ITEMS.checked_add(i).unwrap()))).unwrap());
        }
        for i in 0u64..MAX_ITEMS {
            let (key, value) = (hex_1.encode(mix64(i)), hex_2.encode(!mix64(u64::MAX - i)));
            assert_eq!(tx.get(key).unwrap().unwrap(), value);
        }
    }

    assert!(!set.is_empty().unwrap());
    assert_eq!(set.len().unwrap().to_u64(), MAX_ITEMS);

    {
        let mut tx = set.transaction().unwrap();
        for i in 0u64..MAX_ITEMS {
            assert!(tx.remove(hex_1.encode(mix64(i))).unwrap());
        }
        for i in 0u64..MAX_ITEMS {
            assert!(!tx.remove(hex_1.encode(mix64(i))).unwrap());
        }
    }

    assert!(set.is_empty().unwrap());
}

// ---------------------------------------------------------------------------
// SQLiteMap<String, Blob> Tests
// ---------------------------------------------------------------------------

#[test]
fn test_sqlite_map_string_blob() {
    let mut map = SQLiteMap::<String, Blob>::new().unwrap();

    assert_eq!(map.len().unwrap(), 0);
    assert!(map.is_empty().unwrap());
    assert!(!map.contains("foo").unwrap());
    assert!(!map.contains("bar").unwrap());
    assert!(!map.contains("baz").unwrap());
    assert!(!map.contains("qux").unwrap());
    assert!(map.get("foo").unwrap().is_none());
    assert!(map.get("bar").unwrap().is_none());
    assert!(map.get("baz").unwrap().is_none());
    assert!(map.get("qux").unwrap().is_none());

    assert!(map.insert("foo", &b"12345678"[..]).unwrap());
    assert!(map.insert("bar", &b"87654321"[..]).unwrap());
    assert!(!map.insert("foo", &b"abcdefgh"[..]).unwrap());
    assert!(!map.insert("bar", &b"hgfedcba"[..]).unwrap());
    assert!(map.insert("baz", &b"deadbeef"[..]).unwrap());
    assert!(!map.insert("baz", &b"stuvwxyz"[..]).unwrap());

    assert_eq!(map.len().unwrap(), 3);
    assert!(!map.is_empty().unwrap());
    assert!(map.contains("foo").unwrap());
    assert!(!map.contains("FOO").unwrap());
    assert!(map.contains_nocase("FOO").unwrap());
    assert!(map.contains("bar").unwrap());
    assert!(!map.contains("BAR").unwrap());
    assert!(map.contains_nocase("BAR").unwrap());
    assert!(map.contains("baz").unwrap());
    assert!(!map.contains("BAZ").unwrap());
    assert!(map.contains_nocase("BAZ").unwrap());
    assert!(!map.contains("qux").unwrap());
    assert_eq!(map.get("foo").unwrap().unwrap(), &b"12345678"[..]);
    assert!(map.get("FOO").unwrap().is_none());
    assert_eq!(map.get_nocase("FOO").unwrap().unwrap(), &b"12345678"[..]);
    assert_eq!(map.get("bar").unwrap().unwrap(), &b"87654321"[..]);
    assert!(map.get("BAR").unwrap().is_none());
    assert_eq!(map.get_nocase("BAR").unwrap().unwrap(), &b"87654321"[..]);
    assert_eq!(map.get("baz").unwrap().unwrap(), &b"deadbeef"[..]);
    assert!(map.get("BAZ").unwrap().is_none());
    assert_eq!(map.get_nocase("BAZ").unwrap().unwrap(), &b"deadbeef"[..]);
    assert!(map.get("qux").unwrap().is_none());

    assert!(map.remove("foo").unwrap());
    assert!(!map.remove("foo").unwrap());
    assert!(!map.remove("BAR").unwrap());
    assert!(map.remove_nocase("BAR").unwrap());
    assert!(!map.remove_nocase("BAR").unwrap());
    assert!(!map.remove("qux").unwrap());

    assert_eq!(map.len().unwrap(), 1);
    assert!(!map.is_empty().unwrap());
    assert!(!map.contains("foo").unwrap());
    assert!(!map.contains("bar").unwrap());
    assert!(map.contains("baz").unwrap());
    assert!(!map.contains("BAZ").unwrap());
    assert!(map.contains_nocase("BAZ").unwrap());
    assert!(!map.contains("qux").unwrap());
    assert!(map.get("foo").unwrap().is_none());
    assert!(map.get("bar").unwrap().is_none());
    assert_eq!(map.get("baz").unwrap().unwrap(), &b"deadbeef"[..]);
    assert!(map.get("BAZ").unwrap().is_none());
    assert_eq!(map.get_nocase("BAZ").unwrap().unwrap(), &b"deadbeef"[..]);
    assert!(map.get("qux").unwrap().is_none());

    assert!(map.insert("qux", &b"cafebabe"[..]).unwrap());
    assert!(!map.insert("qux", &b"cafebabe"[..]).unwrap());
    assert!(map.insert("foo", &b"12345678"[..]).unwrap());
    assert!(!map.insert("foo", &b"12345678"[..]).unwrap());

    assert_eq!(map.len().unwrap(), 3);
    assert!(!map.is_empty().unwrap());
    assert!(map.contains("foo").unwrap());
    assert!(!map.contains("FOO").unwrap());
    assert!(map.contains_nocase("FOO").unwrap());
    assert!(!map.contains("bar").unwrap());
    assert!(map.contains("baz").unwrap());
    assert!(!map.contains("BAZ").unwrap());
    assert!(map.contains_nocase("BAZ").unwrap());
    assert!(map.contains("qux").unwrap());
    assert!(!map.contains("QUX").unwrap());
    assert!(map.contains_nocase("QUX").unwrap());
    assert_eq!(map.get("foo").unwrap().unwrap(), &b"12345678"[..]);
    assert!(map.get("FOO").unwrap().is_none());
    assert_eq!(map.get_nocase("FOO").unwrap().unwrap(), &b"12345678"[..]);
    assert!(map.get("bar").unwrap().is_none());
    assert_eq!(map.get("baz").unwrap().unwrap(), &b"deadbeef"[..]);
    assert!(map.get("BAZ").unwrap().is_none());
    assert_eq!(map.get_nocase("BAZ").unwrap().unwrap(), &b"deadbeef"[..]);
    assert_eq!(map.get("qux").unwrap().unwrap(), &b"cafebabe"[..]);
    assert!(map.get("QUX").unwrap().is_none());
    assert_eq!(map.get_nocase("QUX").unwrap().unwrap(), &b"cafebabe"[..]);

    {
        let mut hash = HashMap::new();
        map.for_each(|key, value| assert!(hash.insert(key.to_owned(), value.to_owned()).is_none())).unwrap();
        assert_eq!(hash.len(), 3);
        assert_eq!(hash.get("foo").unwrap(), &b"12345678"[..]);
        assert_eq!(hash.get("baz").unwrap(), &b"deadbeef"[..]);
        assert_eq!(hash.get("qux").unwrap(), &b"cafebabe"[..]);
    }

    assert_eq!(map.find(|key, _| key.eq("foo")).unwrap().unwrap().1, &b"12345678"[..]);
    assert_eq!(map.find(|key, _| key.eq_ignore_ascii_case("FOO")).unwrap().unwrap().1, &b"12345678"[..]);
    assert!(map.find(|key, _| key.eq("bar")).unwrap().is_none());

    map.update("foo", &b"abcdefgh"[..]).unwrap();
    map.update("bar", &b"hgfedcba"[..]).unwrap();
    map.update("baz", &b"stuvwxyz"[..]).unwrap();

    assert_eq!(map.get("foo").unwrap().unwrap(), &b"abcdefgh"[..]);
    assert_eq!(map.get("bar").unwrap().unwrap(), &b"hgfedcba"[..]);
    assert_eq!(map.get("baz").unwrap().unwrap(), &b"stuvwxyz"[..]);
    assert_eq!(map.get("qux").unwrap().unwrap(), &b"cafebabe"[..]);

    {
        let mut hash = HashMap::new();
        map.for_each(|key, value| assert!(hash.insert(key.to_owned(), value.to_owned()).is_none())).unwrap();
        assert_eq!(hash.len(), 4);
        assert_eq!(hash.get("foo").unwrap(), &b"abcdefgh"[..]);
        assert_eq!(hash.get("bar").unwrap(), &b"hgfedcba"[..]);
        assert_eq!(hash.get("baz").unwrap(), &b"stuvwxyz"[..]);
        assert_eq!(hash.get("qux").unwrap(), &b"cafebabe"[..]);
    }

    assert_eq!(map.find(|key, _| key.eq("foo")).unwrap().unwrap().1, &b"abcdefgh"[..]);
    assert_eq!(map.find(|key, _| key.eq_ignore_ascii_case("FOO")).unwrap().unwrap().1, &b"abcdefgh"[..]);
    assert_eq!(map.find(|key, _| key.eq("bar")).unwrap().unwrap().1, &b"hgfedcba"[..]);
    assert_eq!(map.find(|key, _| key.eq_ignore_ascii_case("BAR")).unwrap().unwrap().1, &b"hgfedcba"[..]);
    assert!(map.find(|key, _| key.eq("xyz")).unwrap().is_none());

    map.clear().unwrap();

    assert_eq!(map.len().unwrap(), 0);
    assert!(map.is_empty().unwrap());
    assert!(!map.contains("foo").unwrap());
    assert!(!map.contains("bar").unwrap());
    assert!(!map.contains("baz").unwrap());
    assert!(!map.contains("qux").unwrap());
}

#[test]
fn test_sqlite_map_string_blob_transacted() {
    let mut map = SQLiteMap::<String, Blob>::new().unwrap();
    let mut tx = map.transaction().unwrap();

    assert_eq!(tx.len().unwrap(), 0);
    assert!(tx.is_empty().unwrap());
    assert!(!tx.contains("foo").unwrap());
    assert!(!tx.contains("bar").unwrap());
    assert!(!tx.contains("baz").unwrap());
    assert!(!tx.contains("qux").unwrap());
    assert!(tx.get("foo").unwrap().is_none());
    assert!(tx.get("bar").unwrap().is_none());
    assert!(tx.get("baz").unwrap().is_none());
    assert!(tx.get("qux").unwrap().is_none());

    assert!(tx.insert("foo", &b"12345678"[..]).unwrap());
    assert!(tx.insert("bar", &b"87654321"[..]).unwrap());
    assert!(!tx.insert("foo", &b"abcdefgh"[..]).unwrap());
    assert!(!tx.insert("bar", &b"hgfedcba"[..]).unwrap());
    assert!(tx.insert("baz", &b"deadbeef"[..]).unwrap());
    assert!(!tx.insert("baz", &b"stuvwxyz"[..]).unwrap());

    assert_eq!(tx.len().unwrap(), 3);
    assert!(!tx.is_empty().unwrap());
    assert!(tx.contains("foo").unwrap());
    assert!(!tx.contains("FOO").unwrap());
    assert!(tx.contains_nocase("FOO").unwrap());
    assert!(tx.contains("bar").unwrap());
    assert!(!tx.contains("BAR").unwrap());
    assert!(tx.contains_nocase("BAR").unwrap());
    assert!(tx.contains("baz").unwrap());
    assert!(!tx.contains("BAZ").unwrap());
    assert!(tx.contains_nocase("BAZ").unwrap());
    assert!(!tx.contains("qux").unwrap());
    assert_eq!(tx.get("foo").unwrap().unwrap(), &b"12345678"[..]);
    assert!(tx.get("FOO").unwrap().is_none());
    assert_eq!(tx.get_nocase("FOO").unwrap().unwrap(), &b"12345678"[..]);
    assert_eq!(tx.get("bar").unwrap().unwrap(), &b"87654321"[..]);
    assert!(tx.get("BAR").unwrap().is_none());
    assert_eq!(tx.get_nocase("BAR").unwrap().unwrap(), &b"87654321"[..]);
    assert_eq!(tx.get("baz").unwrap().unwrap(), &b"deadbeef"[..]);
    assert!(tx.get("BAZ").unwrap().is_none());
    assert_eq!(tx.get_nocase("BAZ").unwrap().unwrap(), &b"deadbeef"[..]);
    assert!(tx.get("qux").unwrap().is_none());

    assert!(tx.remove("foo").unwrap());
    assert!(!tx.remove("foo").unwrap());
    assert!(!tx.remove("BAR").unwrap());
    assert!(tx.remove_nocase("BAR").unwrap());
    assert!(!tx.remove_nocase("BAR").unwrap());
    assert!(!tx.remove("qux").unwrap());

    assert_eq!(tx.len().unwrap(), 1);
    assert!(!tx.is_empty().unwrap());
    assert!(!tx.contains("foo").unwrap());
    assert!(!tx.contains("bar").unwrap());
    assert!(tx.contains("baz").unwrap());
    assert!(!tx.contains("BAZ").unwrap());
    assert!(tx.contains_nocase("BAZ").unwrap());
    assert!(!tx.contains("qux").unwrap());
    assert!(tx.get("foo").unwrap().is_none());
    assert!(tx.get("bar").unwrap().is_none());
    assert_eq!(tx.get("baz").unwrap().unwrap(), &b"deadbeef"[..]);
    assert!(tx.get("BAZ").unwrap().is_none());
    assert_eq!(tx.get_nocase("BAZ").unwrap().unwrap(), &b"deadbeef"[..]);
    assert!(tx.get("qux").unwrap().is_none());

    assert!(tx.insert("qux", &b"cafebabe"[..]).unwrap());
    assert!(!tx.insert("qux", &b"cafebabe"[..]).unwrap());
    assert!(tx.insert("foo", &b"12345678"[..]).unwrap());
    assert!(!tx.insert("foo", &b"12345678"[..]).unwrap());

    assert_eq!(tx.len().unwrap(), 3);
    assert!(!tx.is_empty().unwrap());
    assert!(tx.contains("foo").unwrap());
    assert!(!tx.contains("FOO").unwrap());
    assert!(tx.contains_nocase("FOO").unwrap());
    assert!(!tx.contains("bar").unwrap());
    assert!(tx.contains("baz").unwrap());
    assert!(!tx.contains("BAZ").unwrap());
    assert!(tx.contains_nocase("BAZ").unwrap());
    assert!(tx.contains("qux").unwrap());
    assert!(!tx.contains("QUX").unwrap());
    assert!(tx.contains_nocase("QUX").unwrap());
    assert_eq!(tx.get("foo").unwrap().unwrap(), &b"12345678"[..]);
    assert!(tx.get("FOO").unwrap().is_none());
    assert_eq!(tx.get_nocase("FOO").unwrap().unwrap(), &b"12345678"[..]);
    assert!(tx.get("bar").unwrap().is_none());
    assert_eq!(tx.get("baz").unwrap().unwrap(), &b"deadbeef"[..]);
    assert!(tx.get("BAZ").unwrap().is_none());
    assert_eq!(tx.get_nocase("BAZ").unwrap().unwrap(), &b"deadbeef"[..]);
    assert_eq!(tx.get("qux").unwrap().unwrap(), &b"cafebabe"[..]);
    assert!(tx.get("QUX").unwrap().is_none());
    assert_eq!(tx.get_nocase("QUX").unwrap().unwrap(), &b"cafebabe"[..]);

    {
        let mut hash = HashMap::new();
        tx.for_each(|key, value| assert!(hash.insert(key.to_owned(), value.to_owned()).is_none())).unwrap();
        assert_eq!(hash.len(), 3);
        assert_eq!(hash.get("foo").unwrap(), &b"12345678"[..]);
        assert_eq!(hash.get("baz").unwrap(), &b"deadbeef"[..]);
        assert_eq!(hash.get("qux").unwrap(), &b"cafebabe"[..]);
    }

    assert_eq!(tx.find(|key, _| key.eq("foo")).unwrap().unwrap().1, &b"12345678"[..]);
    assert_eq!(tx.find(|key, _| key.eq_ignore_ascii_case("FOO")).unwrap().unwrap().1, &b"12345678"[..]);
    assert!(tx.find(|key, _| key.eq("bar")).unwrap().is_none());

    tx.update("foo", &b"abcdefgh"[..]).unwrap();
    tx.update("bar", &b"hgfedcba"[..]).unwrap();
    tx.update("baz", &b"stuvwxyz"[..]).unwrap();

    assert_eq!(tx.get("foo").unwrap().unwrap(), &b"abcdefgh"[..]);
    assert_eq!(tx.get("bar").unwrap().unwrap(), &b"hgfedcba"[..]);
    assert_eq!(tx.get("baz").unwrap().unwrap(), &b"stuvwxyz"[..]);
    assert_eq!(tx.get("qux").unwrap().unwrap(), &b"cafebabe"[..]);

    {
        let mut hash = HashMap::new();
        tx.for_each(|key, value| assert!(hash.insert(key.to_owned(), value.to_owned()).is_none())).unwrap();
        assert_eq!(hash.len(), 4);
        assert_eq!(hash.get("foo").unwrap(), &b"abcdefgh"[..]);
        assert_eq!(hash.get("bar").unwrap(), &b"hgfedcba"[..]);
        assert_eq!(hash.get("baz").unwrap(), &b"stuvwxyz"[..]);
        assert_eq!(hash.get("qux").unwrap(), &b"cafebabe"[..]);
    }

    assert_eq!(tx.find(|key, _| key.eq("foo")).unwrap().unwrap().1, &b"abcdefgh"[..]);
    assert_eq!(tx.find(|key, _| key.eq_ignore_ascii_case("FOO")).unwrap().unwrap().1, &b"abcdefgh"[..]);
    assert_eq!(tx.find(|key, _| key.eq("bar")).unwrap().unwrap().1, &b"hgfedcba"[..]);
    assert_eq!(tx.find(|key, _| key.eq_ignore_ascii_case("BAR")).unwrap().unwrap().1, &b"hgfedcba"[..]);
    assert!(tx.find(|key, _| key.eq("xyz")).unwrap().is_none());

    tx.clear().unwrap();

    assert_eq!(tx.len().unwrap(), 0);
    assert!(tx.is_empty().unwrap());
    assert!(!tx.contains("foo").unwrap());
    assert!(!tx.contains("bar").unwrap());
    assert!(!tx.contains("baz").unwrap());
    assert!(!tx.contains("qux").unwrap());

    assert!(tx.insert("foo", &b"ijklmnop"[..]).unwrap());
    assert!(tx.insert("bar", &b"ponmlkji"[..]).unwrap());

    tx.commit();

    assert_eq!(map.len().unwrap(), 2);
    assert!(!map.is_empty().unwrap());
    assert!(map.contains("foo").unwrap());
    assert!(map.contains("bar").unwrap());
    assert!(!map.contains("baz").unwrap());
    assert!(!map.contains("qux").unwrap());
    assert_eq!(map.get("foo").unwrap().unwrap(), &b"ijklmnop"[..]);
    assert_eq!(map.get("bar").unwrap().unwrap(), &b"ponmlkji"[..]);
    assert!(map.get("baz").unwrap().is_none());
    assert!(map.get("qux").unwrap().is_none());
}

#[test]
fn test_sqlite_map_string_blob_ignorecase() {
    let mut map_1 = SQLiteMap::<String, Blob>::new().unwrap();
    let mut map_2 = SQLiteMap::<String, Blob>::with_nocase().unwrap();

    assert!(map_1.insert("foo", &b"12345678"[..]).unwrap());
    assert!(map_1.insert("FOO", &b"87654321"[..]).unwrap());
    assert!(map_1.insert("BAR", &b"abcdefgh"[..]).unwrap());
    assert!(map_1.insert("bar", &b"hgfedcba"[..]).unwrap());

    assert!(map_2.insert("foo", &b"12345678"[..]).unwrap());
    assert!(!map_2.insert("FOO", &b"87654321"[..]).unwrap());
    assert!(map_2.insert("BAR", &b"abcdefgh"[..]).unwrap());
    assert!(!map_2.insert("bar", &b"hgfedcba"[..]).unwrap());

    assert!(map_1.remove("FOO").unwrap());
    assert!(map_1.remove("bar").unwrap());

    assert!(map_1.contains("foo").unwrap());
    assert!(!map_1.contains("FOO").unwrap());
    assert!(!map_1.contains("bar").unwrap());
    assert!(map_1.contains("BAR").unwrap());
    assert_eq!(map_1.get("foo").unwrap().unwrap(), &b"12345678"[..]);
    assert!(map_1.get("FOO").unwrap().is_none());
    assert!(map_1.get("bar").unwrap().is_none());
    assert_eq!(map_1.get("BAR").unwrap().unwrap(), &b"abcdefgh"[..]);

    assert!(map_2.contains("foo").unwrap());
    assert!(map_2.contains("FOO").unwrap());
    assert!(map_2.contains("bar").unwrap());
    assert!(map_2.contains("BAR").unwrap());
    assert_eq!(map_2.get("foo").unwrap().unwrap(), &b"12345678"[..]);
    assert_eq!(map_2.get("FOO").unwrap().unwrap(), &b"12345678"[..]);
    assert_eq!(map_2.get("bar").unwrap().unwrap(), &b"abcdefgh"[..]);
    assert_eq!(map_2.get("BAR").unwrap().unwrap(), &b"abcdefgh"[..]);

    assert!(map_2.remove("FOO").unwrap());
    assert!(map_2.remove("bar").unwrap());

    assert!(!map_2.contains("foo").unwrap());
    assert!(!map_2.contains("FOO").unwrap());
    assert!(!map_2.contains("bar").unwrap());
    assert!(!map_2.contains("BAR").unwrap());
    assert!(map_2.get("foo").unwrap().is_none());
    assert!(map_2.get("FOO").unwrap().is_none());
    assert!(map_2.get("bar").unwrap().is_none());
    assert!(map_2.get("BAR").unwrap().is_none());
}

#[test]
fn test_sqlite_map_string_blob_stresstest() {
    let mut set = SQLiteMap::<String, Blob>::new().unwrap();
    let (mut hex_1, mut hex_2) = (HexEncoder::default(), HexEncoder::default());
    const MAX_ITEMS: u64 = 1_000_000u64;

    {
        let mut tx = set.transaction().unwrap();
        for i in 0u64..MAX_ITEMS {
            let (key, value) = (hex_1.encode(mix64(i)), hex_2.encode_raw(mix64(u64::MAX - i)));
            assert!(tx.insert(key, value).unwrap());
        }
        for i in 0u64..MAX_ITEMS {
            let (key, value) = (hex_1.encode(mix64(i)), hex_2.encode_raw(!mix64(u64::MAX - i)));
            tx.update(key, value).unwrap();
        }
    }

    assert!(!set.is_empty().unwrap());
    assert_eq!(set.len().unwrap().to_u64(), MAX_ITEMS);

    {
        let tx = set.transaction().unwrap();
        for i in 0u64..MAX_ITEMS {
            assert!(tx.contains(hex_1.encode(mix64(i))).unwrap());
            assert!(!tx.contains(hex_1.encode(mix64(MAX_ITEMS.checked_add(i).unwrap()))).unwrap());
        }
        for i in 0u64..MAX_ITEMS {
            let (key, value) = (hex_1.encode(mix64(i)), hex_2.encode_raw(!mix64(u64::MAX - i)));
            assert_eq!(tx.get(key).unwrap().unwrap(), value);
        }
    }

    assert!(!set.is_empty().unwrap());
    assert_eq!(set.len().unwrap().to_u64(), MAX_ITEMS);

    {
        let mut tx = set.transaction().unwrap();
        for i in 0u64..MAX_ITEMS {
            assert!(tx.remove(hex_1.encode(mix64(i))).unwrap());
        }
        for i in 0u64..MAX_ITEMS {
            assert!(!tx.remove(hex_1.encode(mix64(i))).unwrap());
        }
    }

    assert!(set.is_empty().unwrap());
}

// ---------------------------------------------------------------------------
// SQLiteMap<String, i64> Tests
// ---------------------------------------------------------------------------

#[test]
fn test_sqlite_map_string_i64() {
    let mut map = SQLiteMap::<String, i64>::new().unwrap();

    assert_eq!(map.len().unwrap(), 0);
    assert!(map.is_empty().unwrap());
    assert!(!map.contains("foo").unwrap());
    assert!(!map.contains("bar").unwrap());
    assert!(!map.contains("baz").unwrap());
    assert!(!map.contains("qux").unwrap());
    assert!(map.get("foo").unwrap().is_none());
    assert!(map.get("bar").unwrap().is_none());
    assert!(map.get("baz").unwrap().is_none());
    assert!(map.get("qux").unwrap().is_none());

    assert!(map.insert("foo", &0x12345678).unwrap());
    assert!(map.insert("bar", &0x87654321).unwrap());
    assert!(!map.insert("foo", &0x9ABCDEF0).unwrap());
    assert!(!map.insert("bar", &0x0FEDCBA9).unwrap());
    assert!(map.insert("baz", &0xDEADBEEF).unwrap());
    assert!(!map.insert("baz", &0x8BADF00D).unwrap());

    assert_eq!(map.len().unwrap(), 3);
    assert!(!map.is_empty().unwrap());
    assert!(map.contains("foo").unwrap());
    assert!(!map.contains("FOO").unwrap());
    assert!(map.contains_nocase("FOO").unwrap());
    assert!(map.contains("bar").unwrap());
    assert!(!map.contains("BAR").unwrap());
    assert!(map.contains_nocase("BAR").unwrap());
    assert!(map.contains("baz").unwrap());
    assert!(!map.contains("BAZ").unwrap());
    assert!(map.contains_nocase("BAZ").unwrap());
    assert!(!map.contains("qux").unwrap());
    assert_eq!(map.get("foo").unwrap().unwrap(), 0x12345678);
    assert!(map.get("FOO").unwrap().is_none());
    assert_eq!(map.get_nocase("FOO").unwrap().unwrap(), 0x12345678);
    assert_eq!(map.get("bar").unwrap().unwrap(), 0x87654321);
    assert!(map.get("BAR").unwrap().is_none());
    assert_eq!(map.get_nocase("BAR").unwrap().unwrap(), 0x87654321);
    assert_eq!(map.get("baz").unwrap().unwrap(), 0xDEADBEEF);
    assert!(map.get("BAZ").unwrap().is_none());
    assert_eq!(map.get_nocase("BAZ").unwrap().unwrap(), 0xDEADBEEF);
    assert!(map.get("qux").unwrap().is_none());

    assert!(map.remove("foo").unwrap());
    assert!(!map.remove("foo").unwrap());
    assert!(!map.remove("BAR").unwrap());
    assert!(map.remove_nocase("BAR").unwrap());
    assert!(!map.remove_nocase("BAR").unwrap());
    assert!(!map.remove("qux").unwrap());

    assert_eq!(map.len().unwrap(), 1);
    assert!(!map.is_empty().unwrap());
    assert!(!map.contains("foo").unwrap());
    assert!(!map.contains("bar").unwrap());
    assert!(map.contains("baz").unwrap());
    assert!(!map.contains("BAZ").unwrap());
    assert!(map.contains_nocase("BAZ").unwrap());
    assert!(!map.contains("qux").unwrap());
    assert!(map.get("foo").unwrap().is_none());
    assert!(map.get("bar").unwrap().is_none());
    assert_eq!(map.get("baz").unwrap().unwrap(), 0xDEADBEEF);
    assert!(map.get("BAZ").unwrap().is_none());
    assert_eq!(map.get_nocase("BAZ").unwrap().unwrap(), 0xDEADBEEF);
    assert!(map.get("qux").unwrap().is_none());

    assert!(map.insert("qux", &0xCAFEBABE).unwrap());
    assert!(!map.insert("qux", &0xCAFEBABE).unwrap());
    assert!(map.insert("foo", &0x12345678).unwrap());
    assert!(!map.insert("foo", &0x12345678).unwrap());

    assert_eq!(map.len().unwrap(), 3);
    assert!(!map.is_empty().unwrap());
    assert!(map.contains("foo").unwrap());
    assert!(!map.contains("FOO").unwrap());
    assert!(map.contains_nocase("FOO").unwrap());
    assert!(!map.contains("bar").unwrap());
    assert!(map.contains("baz").unwrap());
    assert!(!map.contains("BAZ").unwrap());
    assert!(map.contains_nocase("BAZ").unwrap());
    assert!(map.contains("qux").unwrap());
    assert!(!map.contains("QUX").unwrap());
    assert!(map.contains_nocase("QUX").unwrap());
    assert_eq!(map.get("foo").unwrap().unwrap(), 0x12345678);
    assert!(map.get("FOO").unwrap().is_none());
    assert_eq!(map.get_nocase("FOO").unwrap().unwrap(), 0x12345678);
    assert!(map.get("bar").unwrap().is_none());
    assert_eq!(map.get("baz").unwrap().unwrap(), 0xDEADBEEF);
    assert!(map.get("BAZ").unwrap().is_none());
    assert_eq!(map.get_nocase("BAZ").unwrap().unwrap(), 0xDEADBEEF);
    assert_eq!(map.get("qux").unwrap().unwrap(), 0xCAFEBABE);
    assert!(map.get("QUX").unwrap().is_none());
    assert_eq!(map.get_nocase("QUX").unwrap().unwrap(), 0xCAFEBABE);

    {
        let mut hash = HashMap::new();
        map.for_each(|key, value| assert!(hash.insert(key.to_owned(), value.to_owned()).is_none())).unwrap();
        assert_eq!(hash.len(), 3);
        assert_eq!(hash.get("foo").unwrap(), &0x12345678);
        assert_eq!(hash.get("baz").unwrap(), &0xDEADBEEF);
        assert_eq!(hash.get("qux").unwrap(), &0xCAFEBABE);
    }

    assert_eq!(map.find(|key, _| key.eq("foo")).unwrap().unwrap().1, 0x12345678);
    assert_eq!(map.find(|key, _| key.eq_ignore_ascii_case("FOO")).unwrap().unwrap().1, 0x12345678);
    assert!(map.find(|key, _| key.eq("bar")).unwrap().is_none());

    map.update("foo", &0x9ABCDEF0).unwrap();
    map.update("bar", &0x0FEDCBA9).unwrap();
    map.update("baz", &0x8BADF00D).unwrap();

    assert_eq!(map.get("foo").unwrap().unwrap(), 0x9ABCDEF0);
    assert_eq!(map.get("bar").unwrap().unwrap(), 0x0FEDCBA9);
    assert_eq!(map.get("baz").unwrap().unwrap(), 0x8BADF00D);
    assert_eq!(map.get("qux").unwrap().unwrap(), 0xCAFEBABE);

    {
        let mut hash = HashMap::new();
        map.for_each(|key, value| assert!(hash.insert(key.to_owned(), value.to_owned()).is_none())).unwrap();
        assert_eq!(hash.len(), 4);
        assert_eq!(hash.get("foo").unwrap(), &0x9ABCDEF0);
        assert_eq!(hash.get("bar").unwrap(), &0x0FEDCBA9);
        assert_eq!(hash.get("baz").unwrap(), &0x8BADF00D);
        assert_eq!(hash.get("qux").unwrap(), &0xCAFEBABE);
    }

    assert_eq!(map.find(|key, _| key.eq("foo")).unwrap().unwrap().1, 0x9ABCDEF0);
    assert_eq!(map.find(|key, _| key.eq_ignore_ascii_case("FOO")).unwrap().unwrap().1, 0x9ABCDEF0);
    assert_eq!(map.find(|key, _| key.eq("bar")).unwrap().unwrap().1, 0x0FEDCBA9);
    assert_eq!(map.find(|key, _| key.eq_ignore_ascii_case("BAR")).unwrap().unwrap().1, 0x0FEDCBA9);
    assert!(map.find(|key, _| key.eq("xyz")).unwrap().is_none());

    map.clear().unwrap();

    assert_eq!(map.len().unwrap(), 0);
    assert!(map.is_empty().unwrap());
    assert!(!map.contains("foo").unwrap());
    assert!(!map.contains("bar").unwrap());
    assert!(!map.contains("baz").unwrap());
    assert!(!map.contains("qux").unwrap());
}

#[test]
fn test_sqlite_map_string_i64_transacted() {
    let mut map = SQLiteMap::<String, i64>::new().unwrap();
    let mut tx = map.transaction().unwrap();

    assert_eq!(tx.len().unwrap(), 0);
    assert!(tx.is_empty().unwrap());
    assert!(!tx.contains("foo").unwrap());
    assert!(!tx.contains("bar").unwrap());
    assert!(!tx.contains("baz").unwrap());
    assert!(!tx.contains("qux").unwrap());
    assert!(tx.get("foo").unwrap().is_none());
    assert!(tx.get("bar").unwrap().is_none());
    assert!(tx.get("baz").unwrap().is_none());
    assert!(tx.get("qux").unwrap().is_none());

    assert!(tx.insert("foo", &0x12345678).unwrap());
    assert!(tx.insert("bar", &0x87654321).unwrap());
    assert!(!tx.insert("foo", &0x9ABCDEF0).unwrap());
    assert!(!tx.insert("bar", &0x0FEDCBA9).unwrap());
    assert!(tx.insert("baz", &0xDEADBEEF).unwrap());
    assert!(!tx.insert("baz", &0x8BADF00D).unwrap());

    assert_eq!(tx.len().unwrap(), 3);
    assert!(!tx.is_empty().unwrap());
    assert!(tx.contains("foo").unwrap());
    assert!(!tx.contains("FOO").unwrap());
    assert!(tx.contains_nocase("FOO").unwrap());
    assert!(tx.contains("bar").unwrap());
    assert!(!tx.contains("BAR").unwrap());
    assert!(tx.contains_nocase("BAR").unwrap());
    assert!(tx.contains("baz").unwrap());
    assert!(!tx.contains("BAZ").unwrap());
    assert!(tx.contains_nocase("BAZ").unwrap());
    assert!(!tx.contains("qux").unwrap());
    assert_eq!(tx.get("foo").unwrap().unwrap(), 0x12345678);
    assert!(tx.get("FOO").unwrap().is_none());
    assert_eq!(tx.get_nocase("FOO").unwrap().unwrap(), 0x12345678);
    assert_eq!(tx.get("bar").unwrap().unwrap(), 0x87654321);
    assert!(tx.get("BAR").unwrap().is_none());
    assert_eq!(tx.get_nocase("BAR").unwrap().unwrap(), 0x87654321);
    assert_eq!(tx.get("baz").unwrap().unwrap(), 0xDEADBEEF);
    assert!(tx.get("BAZ").unwrap().is_none());
    assert_eq!(tx.get_nocase("BAZ").unwrap().unwrap(), 0xDEADBEEF);
    assert!(tx.get("qux").unwrap().is_none());

    assert!(tx.remove("foo").unwrap());
    assert!(!tx.remove("foo").unwrap());
    assert!(!tx.remove("BAR").unwrap());
    assert!(tx.remove_nocase("BAR").unwrap());
    assert!(!tx.remove_nocase("BAR").unwrap());
    assert!(!tx.remove("qux").unwrap());

    assert_eq!(tx.len().unwrap(), 1);
    assert!(!tx.is_empty().unwrap());
    assert!(!tx.contains("foo").unwrap());
    assert!(!tx.contains("bar").unwrap());
    assert!(tx.contains("baz").unwrap());
    assert!(!tx.contains("BAZ").unwrap());
    assert!(tx.contains_nocase("BAZ").unwrap());
    assert!(!tx.contains("qux").unwrap());
    assert!(tx.get("foo").unwrap().is_none());
    assert!(tx.get("bar").unwrap().is_none());
    assert_eq!(tx.get("baz").unwrap().unwrap(), 0xDEADBEEF);
    assert!(tx.get("BAZ").unwrap().is_none());
    assert_eq!(tx.get_nocase("BAZ").unwrap().unwrap(), 0xDEADBEEF);
    assert!(tx.get("qux").unwrap().is_none());

    assert!(tx.insert("qux", &0xCAFEBABE).unwrap());
    assert!(!tx.insert("qux", &0xCAFEBABE).unwrap());
    assert!(tx.insert("foo", &0x12345678).unwrap());
    assert!(!tx.insert("foo", &0x12345678).unwrap());

    assert_eq!(tx.len().unwrap(), 3);
    assert!(!tx.is_empty().unwrap());
    assert!(tx.contains("foo").unwrap());
    assert!(!tx.contains("FOO").unwrap());
    assert!(tx.contains_nocase("FOO").unwrap());
    assert!(!tx.contains("bar").unwrap());
    assert!(tx.contains("baz").unwrap());
    assert!(!tx.contains("BAZ").unwrap());
    assert!(tx.contains_nocase("BAZ").unwrap());
    assert!(tx.contains("qux").unwrap());
    assert!(!tx.contains("QUX").unwrap());
    assert!(tx.contains_nocase("QUX").unwrap());
    assert_eq!(tx.get("foo").unwrap().unwrap(), 0x12345678);
    assert!(tx.get("FOO").unwrap().is_none());
    assert_eq!(tx.get_nocase("FOO").unwrap().unwrap(), 0x12345678);
    assert!(tx.get("bar").unwrap().is_none());
    assert_eq!(tx.get("baz").unwrap().unwrap(), 0xDEADBEEF);
    assert!(tx.get("BAZ").unwrap().is_none());
    assert_eq!(tx.get_nocase("BAZ").unwrap().unwrap(), 0xDEADBEEF);
    assert_eq!(tx.get("qux").unwrap().unwrap(), 0xCAFEBABE);
    assert!(tx.get("QUX").unwrap().is_none());
    assert_eq!(tx.get_nocase("QUX").unwrap().unwrap(), 0xCAFEBABE);

    {
        let mut hash = HashMap::new();
        tx.for_each(|key, value| assert!(hash.insert(key.to_owned(), value.to_owned()).is_none())).unwrap();
        assert_eq!(hash.len(), 3);
        assert_eq!(hash.get("foo").unwrap(), &0x12345678);
        assert_eq!(hash.get("baz").unwrap(), &0xDEADBEEF);
        assert_eq!(hash.get("qux").unwrap(), &0xCAFEBABE);
    }

    assert_eq!(tx.find(|key, _| key.eq("foo")).unwrap().unwrap().1, 0x12345678);
    assert_eq!(tx.find(|key, _| key.eq_ignore_ascii_case("FOO")).unwrap().unwrap().1, 0x12345678);
    assert!(tx.find(|key, _| key.eq("bar")).unwrap().is_none());

    tx.update("foo", &0x9ABCDEF0).unwrap();
    tx.update("bar", &0x0FEDCBA9).unwrap();
    tx.update("baz", &0x8BADF00D).unwrap();

    assert_eq!(tx.get("foo").unwrap().unwrap(), 0x9ABCDEF0);
    assert_eq!(tx.get("bar").unwrap().unwrap(), 0x0FEDCBA9);
    assert_eq!(tx.get("baz").unwrap().unwrap(), 0x8BADF00D);
    assert_eq!(tx.get("qux").unwrap().unwrap(), 0xCAFEBABE);

    {
        let mut hash = HashMap::new();
        tx.for_each(|key, value| assert!(hash.insert(key.to_owned(), value.to_owned()).is_none())).unwrap();
        assert_eq!(hash.len(), 4);
        assert_eq!(hash.get("foo").unwrap(), &0x9ABCDEF0);
        assert_eq!(hash.get("bar").unwrap(), &0x0FEDCBA9);
        assert_eq!(hash.get("baz").unwrap(), &0x8BADF00D);
        assert_eq!(hash.get("qux").unwrap(), &0xCAFEBABE);
    }

    assert_eq!(tx.find(|key, _| key.eq("foo")).unwrap().unwrap().1, 0x9ABCDEF0);
    assert_eq!(tx.find(|key, _| key.eq_ignore_ascii_case("FOO")).unwrap().unwrap().1, 0x9ABCDEF0);
    assert_eq!(tx.find(|key, _| key.eq("bar")).unwrap().unwrap().1, 0x0FEDCBA9);
    assert_eq!(tx.find(|key, _| key.eq_ignore_ascii_case("BAR")).unwrap().unwrap().1, 0x0FEDCBA9);
    assert!(tx.find(|key, _| key.eq("xyz")).unwrap().is_none());

    tx.clear().unwrap();

    assert_eq!(tx.len().unwrap(), 0);
    assert!(tx.is_empty().unwrap());
    assert!(!tx.contains("foo").unwrap());
    assert!(!tx.contains("bar").unwrap());
    assert!(!tx.contains("baz").unwrap());
    assert!(!tx.contains("qux").unwrap());

    assert!(tx.insert("foo", &i64::MAX).unwrap());
    assert!(tx.insert("bar", &i64::MIN).unwrap());

    tx.commit();

    assert_eq!(map.len().unwrap(), 2);
    assert!(!map.is_empty().unwrap());
    assert!(map.contains("foo").unwrap());
    assert!(map.contains("bar").unwrap());
    assert!(!map.contains("baz").unwrap());
    assert!(!map.contains("qux").unwrap());
    assert_eq!(map.get("foo").unwrap().unwrap(), i64::MAX);
    assert_eq!(map.get("bar").unwrap().unwrap(), i64::MIN);
    assert!(map.get("baz").unwrap().is_none());
    assert!(map.get("qux").unwrap().is_none());
}

#[test]
fn test_sqlite_map_string_i64_ignorecase() {
    let mut map_1 = SQLiteMap::<String, i64>::new().unwrap();
    let mut map_2 = SQLiteMap::<String, i64>::with_nocase().unwrap();

    assert!(map_1.insert("foo", &0x12345678).unwrap());
    assert!(map_1.insert("FOO", &0x87654321).unwrap());
    assert!(map_1.insert("BAR", &0x9ABCDEF0).unwrap());
    assert!(map_1.insert("bar", &0x0FEDCBA9).unwrap());

    assert!(map_2.insert("foo", &0x12345678).unwrap());
    assert!(!map_2.insert("FOO", &0x87654321).unwrap());
    assert!(map_2.insert("BAR", &0x9ABCDEF0).unwrap());
    assert!(!map_2.insert("bar", &0x0FEDCBA9).unwrap());

    assert!(map_1.remove("FOO").unwrap());
    assert!(map_1.remove("bar").unwrap());

    assert!(map_1.contains("foo").unwrap());
    assert!(!map_1.contains("FOO").unwrap());
    assert!(!map_1.contains("bar").unwrap());
    assert!(map_1.contains("BAR").unwrap());
    assert_eq!(map_1.get("foo").unwrap().unwrap(), 0x12345678);
    assert!(map_1.get("FOO").unwrap().is_none());
    assert!(map_1.get("bar").unwrap().is_none());
    assert_eq!(map_1.get("BAR").unwrap().unwrap(), 0x9ABCDEF0);

    assert!(map_2.contains("foo").unwrap());
    assert!(map_2.contains("FOO").unwrap());
    assert!(map_2.contains("bar").unwrap());
    assert!(map_2.contains("BAR").unwrap());
    assert_eq!(map_2.get("foo").unwrap().unwrap(), 0x12345678);
    assert_eq!(map_2.get("FOO").unwrap().unwrap(), 0x12345678);
    assert_eq!(map_2.get("bar").unwrap().unwrap(), 0x9ABCDEF0);
    assert_eq!(map_2.get("BAR").unwrap().unwrap(), 0x9ABCDEF0);

    assert!(map_2.remove("FOO").unwrap());
    assert!(map_2.remove("bar").unwrap());

    assert!(!map_2.contains("foo").unwrap());
    assert!(!map_2.contains("FOO").unwrap());
    assert!(!map_2.contains("bar").unwrap());
    assert!(!map_2.contains("BAR").unwrap());
    assert!(map_2.get("foo").unwrap().is_none());
    assert!(map_2.get("FOO").unwrap().is_none());
    assert!(map_2.get("bar").unwrap().is_none());
    assert!(map_2.get("BAR").unwrap().is_none());
}

#[test]
fn test_sqlite_map_string_i64_stresstest() {
    let mut set = SQLiteMap::<String, i64>::new().unwrap();
    let mut hex = HexEncoder::default();
    const MAX_ITEMS: u64 = 1_000_000u64;

    {
        let mut tx = set.transaction().unwrap();
        for i in 0u64..MAX_ITEMS {
            let (key, value) = (hex.encode(mix64(i)), mix64(u64::MAX - i) as i64);
            assert!(tx.insert(key, &value).unwrap());
        }
        for i in 0u64..MAX_ITEMS {
            let (key, value) = (hex.encode(mix64(i)), (!mix64(u64::MAX - i)) as i64);
            tx.update(key, &value).unwrap();
        }
    }

    assert!(!set.is_empty().unwrap());
    assert_eq!(set.len().unwrap().to_u64(), MAX_ITEMS);

    {
        let tx = set.transaction().unwrap();
        for i in 0u64..MAX_ITEMS {
            assert!(tx.contains(hex.encode(mix64(i))).unwrap());
            assert!(!tx.contains(hex.encode(mix64(MAX_ITEMS.checked_add(i).unwrap()))).unwrap());
        }
        for i in 0u64..MAX_ITEMS {
            let (key, value) = (hex.encode(mix64(i)), (!mix64(u64::MAX - i)) as i64);
            assert_eq!(tx.get(key).unwrap().unwrap(), value);
        }
    }

    assert!(!set.is_empty().unwrap());
    assert_eq!(set.len().unwrap().to_u64(), MAX_ITEMS);

    {
        let mut tx = set.transaction().unwrap();
        for i in 0u64..MAX_ITEMS {
            assert!(tx.remove(hex.encode(mix64(i))).unwrap());
        }
        for i in 0u64..MAX_ITEMS {
            assert!(!tx.remove(hex.encode(mix64(i))).unwrap());
        }
    }

    assert!(set.is_empty().unwrap());
}
