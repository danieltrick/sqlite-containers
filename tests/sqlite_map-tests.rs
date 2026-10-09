// SQLiteMap Tests
// This file is part of the 'SQLite-based containers for Rust' project (sqlite-containers)
// SPDX-License-Identifier: Unlicense

mod utilities;

use crate::utilities::{hex_enc, mix64};
use sqlite_containers::SQLiteMap;
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// SQLiteSet Tests
// ---------------------------------------------------------------------------

#[test]
fn test_sqlite_map() {
    let mut map = SQLiteMap::default();

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
fn test_sqlite_map_transacted() {
    let mut map = SQLiteMap::new().unwrap();
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
fn test_sqlite_map_ignorecase() {
    let mut map_1 = SQLiteMap::new().unwrap();
    let mut map_2 = SQLiteMap::with_nocase().unwrap();

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
fn test_sqlite_map_stresstest() {
    let mut set = SQLiteMap::new().unwrap();
    let mut hexstr_1 = [0; 16usize];
    let mut hexstr_2 = [0; 16usize];
    const MAX_ITEMS: u64 = 1_000_000u64;

    {
        let mut tx = set.transaction().unwrap();
        for i in 0u64..MAX_ITEMS {
            let key = hex_enc(mix64(i), &mut hexstr_1);
            let value = hex_enc(mix64(u64::MAX - i), &mut hexstr_2);
            assert!(tx.insert(key, value).unwrap());
        }
        for i in 0u64..MAX_ITEMS {
            let key = hex_enc(mix64(i), &mut hexstr_1);
            let value = hex_enc(mix64(!(u64::MAX - i)), &mut hexstr_2);
            tx.update(key, value).unwrap();
        }
    }
    {
        let tx = set.transaction().unwrap();
        for i in 0u64..MAX_ITEMS {
            let key = hex_enc(mix64(i), &mut hexstr_1);
            assert!(tx.contains(key).unwrap());
        }
        for i in 0u64..MAX_ITEMS {
            let key = hex_enc(mix64(MAX_ITEMS.checked_add(i).unwrap()), &mut hexstr_1);
            assert!(!tx.contains(key).unwrap());
        }
        for i in 0u64..MAX_ITEMS {
            let key = hex_enc(mix64(i), &mut hexstr_1);
            let value = hex_enc(mix64(!(u64::MAX - i)), &mut hexstr_2);
            assert_eq!(tx.get(key).unwrap().unwrap(), value);
        }
    }
    {
        let mut tx = set.transaction().unwrap();
        for i in 0u64..MAX_ITEMS {
            let key = hex_enc(mix64(i), &mut hexstr_1);
            assert!(tx.remove(key).unwrap());
        }
        for i in 0u64..MAX_ITEMS {
            let key = hex_enc(mix64(i), &mut hexstr_1);
            assert!(!tx.remove(key).unwrap());
        }
    }
}
