// SQLiteSet Tests
// This file is part of the 'SQLite-based containers for Rust' project (sqlite-containers)
// SPDX-License-Identifier: Unlicense

mod utilities;

use crate::utilities::{HexEncoder, mix64};
use sqlite_containers::{Blob, SQLiteSet, SizeExt};
use std::{
    array,
    collections::HashSet,
    sync::{Arc, Barrier, mpsc::sync_channel},
    thread,
};

// ---------------------------------------------------------------------------
// SQLiteSet<i64> Tests
// ---------------------------------------------------------------------------

#[test]
fn test_sqlite_set_i64() {
    let mut set = SQLiteSet::<i64>::new().unwrap();

    assert_eq!(set.len().unwrap(), 0);
    assert!(set.is_empty().unwrap());
    assert!(!set.contains(&0i64).unwrap());
    assert!(!set.contains(&42i64).unwrap());
    assert!(!set.contains(&i64::MAX).unwrap());
    assert!(!set.contains(&i64::MIN).unwrap());

    assert!(set.insert(&0i64).unwrap());
    assert!(set.insert(&42i64).unwrap());
    assert!(!set.insert(&0i64).unwrap());
    assert!(!set.insert(&42i64).unwrap());
    assert!(set.insert(&i64::MAX).unwrap());
    assert!(!set.insert(&i64::MAX).unwrap());

    assert_eq!(set.len().unwrap(), 3);
    assert!(!set.is_empty().unwrap());
    assert!(set.contains(&0i64).unwrap());
    assert!(set.contains(&42i64).unwrap());
    assert!(set.contains(&i64::MAX).unwrap());
    assert!(!set.contains(&i64::MIN).unwrap());

    assert!(set.remove(&0i64).unwrap());
    assert!(!set.remove(&0i64).unwrap());
    assert!(set.remove(&42i64).unwrap());
    assert!(!set.remove(&42i64).unwrap());
    assert!(!set.remove(&i64::MIN).unwrap());

    assert_eq!(set.len().unwrap(), 1);
    assert!(!set.is_empty().unwrap());
    assert!(!set.contains(&0i64).unwrap());
    assert!(!set.contains(&42i64).unwrap());
    assert!(set.contains(&i64::MAX).unwrap());
    assert!(!set.contains(&i64::MIN).unwrap());

    assert!(set.insert(&i64::MIN).unwrap());
    assert!(!set.insert(&i64::MIN).unwrap());
    assert!(set.insert(&0i64).unwrap());
    assert!(!set.insert(&0i64).unwrap());

    assert_eq!(set.len().unwrap(), 3);
    assert!(!set.is_empty().unwrap());
    assert!(set.contains(&0i64).unwrap());
    assert!(!set.contains(&42i64).unwrap());
    assert!(set.contains(&i64::MAX).unwrap());
    assert!(set.contains(&i64::MIN).unwrap());

    {
        let mut hash = HashSet::new();
        set.for_each(|key| assert!(hash.insert(key.to_owned()))).unwrap();
        assert_eq!(hash.len(), 3);
        assert!(hash.contains(&0i64));
        assert!(hash.contains(&i64::MAX));
        assert!(hash.contains(&i64::MIN));
    }

    assert!(set.find(|item| item.eq(&0i64)).unwrap().is_some());
    assert!(set.find(|item| item.eq(&42i64)).unwrap().is_none());
    assert!(set.find(|item| item.eq(&i64::MAX)).unwrap().is_some());
    assert!(set.find(|item| item.eq(&i64::MIN)).unwrap().is_some());

    set.clear().unwrap();

    assert_eq!(set.len().unwrap(), 0);
    assert!(set.is_empty().unwrap());
    assert!(!set.contains(&0i64).unwrap());
    assert!(!set.contains(&42i64).unwrap());
    assert!(!set.contains(&i64::MAX).unwrap());
    assert!(!set.contains(&i64::MIN).unwrap());
}

#[test]
fn test_sqlite_set_i64_transacted() {
    let mut set = SQLiteSet::<i64>::new().unwrap();
    let mut tx = set.transaction().unwrap();

    assert_eq!(tx.len().unwrap(), 0);
    assert!(tx.is_empty().unwrap());
    assert!(!tx.contains(&0i64).unwrap());
    assert!(!tx.contains(&42i64).unwrap());
    assert!(!tx.contains(&i64::MAX).unwrap());
    assert!(!tx.contains(&i64::MIN).unwrap());

    assert!(tx.insert(&0i64).unwrap());
    assert!(tx.insert(&42i64).unwrap());
    assert!(!tx.insert(&0i64).unwrap());
    assert!(!tx.insert(&42i64).unwrap());
    assert!(tx.insert(&i64::MAX).unwrap());
    assert!(!tx.insert(&i64::MAX).unwrap());

    assert_eq!(tx.len().unwrap(), 3);
    assert!(!tx.is_empty().unwrap());
    assert!(tx.contains(&0i64).unwrap());
    assert!(tx.contains(&42i64).unwrap());
    assert!(tx.contains(&i64::MAX).unwrap());
    assert!(!tx.contains(&i64::MIN).unwrap());

    assert!(tx.remove(&0i64).unwrap());
    assert!(!tx.remove(&0i64).unwrap());
    assert!(tx.remove(&42i64).unwrap());
    assert!(!tx.remove(&i64::MIN).unwrap());

    assert_eq!(tx.len().unwrap(), 1);
    assert!(!tx.is_empty().unwrap());
    assert!(!tx.contains(&0i64).unwrap());
    assert!(!tx.contains(&42i64).unwrap());
    assert!(tx.contains(&i64::MAX).unwrap());
    assert!(!tx.contains(&i64::MIN).unwrap());

    assert!(tx.insert(&i64::MIN).unwrap());
    assert!(!tx.insert(&i64::MIN).unwrap());
    assert!(tx.insert(&0i64).unwrap());
    assert!(!tx.insert(&0i64).unwrap());

    assert_eq!(tx.len().unwrap(), 3);
    assert!(!tx.is_empty().unwrap());
    assert!(tx.contains(&0i64).unwrap());
    assert!(!tx.contains(&42i64).unwrap());
    assert!(tx.contains(&i64::MAX).unwrap());
    assert!(tx.contains(&i64::MIN).unwrap());

    {
        let mut hash = HashSet::new();
        tx.for_each(|key| assert!(hash.insert(key.to_owned()))).unwrap();
        assert_eq!(hash.len(), 3);
        assert!(hash.contains(&0i64));
        assert!(hash.contains(&i64::MAX));
        assert!(hash.contains(&i64::MIN));
    }

    assert!(tx.find(|item| item.eq(&0i64)).unwrap().is_some());
    assert!(tx.find(|item| item.eq(&42i64)).unwrap().is_none());
    assert!(tx.find(|item| item.eq(&i64::MAX)).unwrap().is_some());
    assert!(tx.find(|item| item.eq(&i64::MIN)).unwrap().is_some());

    tx.clear().unwrap();

    assert_eq!(tx.len().unwrap(), 0);
    assert!(tx.is_empty().unwrap());
    assert!(!tx.contains(&0i64).unwrap());
    assert!(!tx.contains(&42i64).unwrap());
    assert!(!tx.contains(&i64::MAX).unwrap());
    assert!(!tx.contains(&i64::MIN).unwrap());

    assert!(tx.insert(&0i64).unwrap());
    assert!(tx.insert(&42i64).unwrap());

    tx.commit();

    assert_eq!(set.len().unwrap(), 2);
    assert!(!set.is_empty().unwrap());
    assert!(set.contains(&0i64).unwrap());
    assert!(set.contains(&42i64).unwrap());
    assert!(!set.contains(&i64::MAX).unwrap());
    assert!(!set.contains(&i64::MIN).unwrap());
}

#[test]
fn test_sqlite_set_i64_stresstest() {
    let mut set = SQLiteSet::<i64>::new().unwrap();
    const MAX_ITEMS: u64 = 1_000_000u64;

    {
        let mut tx = set.transaction().unwrap();
        for i in 0u64..MAX_ITEMS {
            assert!(tx.insert(&(mix64(i) as i64)).unwrap());
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
// SQLiteSet<String> Tests
// ---------------------------------------------------------------------------

#[test]
fn test_sqlite_set_string() {
    let mut set = SQLiteSet::<String>::new().unwrap();

    assert_eq!(set.len().unwrap(), 0);
    assert!(set.is_empty().unwrap());
    assert!(!set.contains("foo").unwrap());
    assert!(!set.contains("bar").unwrap());
    assert!(!set.contains("baz").unwrap());
    assert!(!set.contains("qux").unwrap());

    assert!(set.insert("foo").unwrap());
    assert!(set.insert("bar").unwrap());
    assert!(!set.insert("foo").unwrap());
    assert!(!set.insert("bar").unwrap());
    assert!(set.insert("baz").unwrap());
    assert!(!set.insert("baz").unwrap());

    assert_eq!(set.len().unwrap(), 3);
    assert!(!set.is_empty().unwrap());
    assert!(set.contains("foo").unwrap());
    assert!(!set.contains("FOO").unwrap());
    assert!(set.contains_nocase("FOO").unwrap());
    assert!(set.contains("bar").unwrap());
    assert!(!set.contains("BAR").unwrap());
    assert!(set.contains_nocase("BAR").unwrap());
    assert!(set.contains("baz").unwrap());
    assert!(!set.contains("BAZ").unwrap());
    assert!(set.contains_nocase("BAZ").unwrap());
    assert!(!set.contains("qux").unwrap());

    assert!(set.remove("foo").unwrap());
    assert!(!set.remove("foo").unwrap());
    assert!(!set.remove("BAR").unwrap());
    assert!(set.remove_nocase("BAR").unwrap());
    assert!(!set.remove_nocase("BAR").unwrap());
    assert!(!set.remove("qux").unwrap());

    assert_eq!(set.len().unwrap(), 1);
    assert!(!set.is_empty().unwrap());
    assert!(!set.contains("foo").unwrap());
    assert!(!set.contains("bar").unwrap());
    assert!(set.contains("baz").unwrap());
    assert!(!set.contains("BAZ").unwrap());
    assert!(set.contains_nocase("BAZ").unwrap());
    assert!(!set.contains("qux").unwrap());

    assert!(set.insert("qux").unwrap());
    assert!(!set.insert("qux").unwrap());
    assert!(set.insert("foo").unwrap());
    assert!(!set.insert("foo").unwrap());

    assert_eq!(set.len().unwrap(), 3);
    assert!(!set.is_empty().unwrap());
    assert!(set.contains("foo").unwrap());
    assert!(!set.contains("FOO").unwrap());
    assert!(set.contains_nocase("FOO").unwrap());
    assert!(!set.contains("bar").unwrap());
    assert!(set.contains("baz").unwrap());
    assert!(!set.contains("BAZ").unwrap());
    assert!(set.contains_nocase("BAZ").unwrap());
    assert!(set.contains("qux").unwrap());
    assert!(!set.contains("QUX").unwrap());
    assert!(set.contains_nocase("QUX").unwrap());

    {
        let mut hash = HashSet::new();
        set.for_each(|key| assert!(hash.insert(key))).unwrap();
        assert_eq!(hash.len(), 3);
        assert!(hash.contains("foo"));
        assert!(hash.contains("baz"));
        assert!(hash.contains("qux"));
    }

    assert!(set.find(|item: &str| item.eq("foo")).unwrap().is_some());
    assert!(set.find(|item: &str| item.eq_ignore_ascii_case("FOO")).unwrap().is_some());
    assert!(set.find(|item: &str| item.eq("bar")).unwrap().is_none());

    set.clear().unwrap();

    assert_eq!(set.len().unwrap(), 0);
    assert!(set.is_empty().unwrap());
    assert!(!set.contains("foo").unwrap());
    assert!(!set.contains("bar").unwrap());
    assert!(!set.contains("baz").unwrap());
    assert!(!set.contains("qux").unwrap());
}

#[test]
fn test_sqlite_set_string_transacted() {
    let mut set = SQLiteSet::<String>::new().unwrap();
    let mut tx = set.transaction().unwrap();

    assert_eq!(tx.len().unwrap(), 0);
    assert!(tx.is_empty().unwrap());
    assert!(!tx.contains("foo").unwrap());
    assert!(!tx.contains("bar").unwrap());
    assert!(!tx.contains("baz").unwrap());
    assert!(!tx.contains("qux").unwrap());

    assert!(tx.insert("foo").unwrap());
    assert!(tx.insert("bar").unwrap());
    assert!(!tx.insert("foo").unwrap());
    assert!(!tx.insert("bar").unwrap());
    assert!(tx.insert("baz").unwrap());
    assert!(!tx.insert("baz").unwrap());

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

    assert!(tx.insert("qux").unwrap());
    assert!(!tx.insert("qux").unwrap());
    assert!(tx.insert("foo").unwrap());
    assert!(!tx.insert("foo").unwrap());

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

    {
        let mut hash = HashSet::new();
        tx.for_each(|key| assert!(hash.insert(key))).unwrap();
        assert_eq!(hash.len(), 3);
        assert!(hash.contains("foo"));
        assert!(hash.contains("baz"));
        assert!(hash.contains("qux"));
    }

    assert!(tx.find(|item: &str| item.eq("foo")).unwrap().is_some());
    assert!(tx.find(|item: &str| item.eq_ignore_ascii_case("FOO")).unwrap().is_some());
    assert!(tx.find(|item: &str| item.eq("bar")).unwrap().is_none());

    tx.clear().unwrap();

    assert_eq!(tx.len().unwrap(), 0);
    assert!(tx.is_empty().unwrap());
    assert!(!tx.contains("foo").unwrap());
    assert!(!tx.contains("bar").unwrap());
    assert!(!tx.contains("baz").unwrap());
    assert!(!tx.contains("qux").unwrap());

    assert!(tx.insert("foo").unwrap());
    assert!(tx.insert("bar").unwrap());

    tx.commit();

    assert_eq!(set.len().unwrap(), 2);
    assert!(!set.is_empty().unwrap());
    assert!(set.contains("foo").unwrap());
    assert!(set.contains("bar").unwrap());
    assert!(!set.contains("baz").unwrap());
    assert!(!set.contains("qux").unwrap());
}

#[test]
fn test_sqlite_set_string_ignorecase() {
    let mut set_1 = SQLiteSet::<String>::new().unwrap();
    let mut set_2 = SQLiteSet::<String>::with_nocase().unwrap();

    assert!(set_1.insert("foo").unwrap());
    assert!(set_1.insert("FOO").unwrap());
    assert!(set_1.insert("BAR").unwrap());
    assert!(set_1.insert("bar").unwrap());

    assert!(set_2.insert("foo").unwrap());
    assert!(!set_2.insert("FOO").unwrap());
    assert!(set_2.insert("BAR").unwrap());
    assert!(!set_2.insert("bar").unwrap());

    assert!(set_1.remove("FOO").unwrap());
    assert!(set_1.remove("bar").unwrap());

    assert!(set_1.contains("foo").unwrap());
    assert!(!set_1.contains("FOO").unwrap());
    assert!(!set_1.contains("bar").unwrap());
    assert!(set_1.contains("BAR").unwrap());

    assert!(set_2.contains("foo").unwrap());
    assert!(set_2.contains("FOO").unwrap());
    assert!(set_2.contains("bar").unwrap());
    assert!(set_2.contains("BAR").unwrap());

    assert!(set_2.remove("FOO").unwrap());
    assert!(set_2.remove("bar").unwrap());

    assert!(!set_2.contains("foo").unwrap());
    assert!(!set_2.contains("FOO").unwrap());
    assert!(!set_2.contains("bar").unwrap());
    assert!(!set_2.contains("BAR").unwrap());
}

#[test]
fn test_sqlite_set_string_stresstest() {
    let mut set = SQLiteSet::<String>::new().unwrap();
    let mut hex = HexEncoder::default();
    const MAX_ITEMS: u64 = 1_000_000u64;

    {
        let mut tx = set.transaction().unwrap();
        for i in 0u64..MAX_ITEMS {
            assert!(tx.insert(hex.encode(mix64(i))).unwrap());
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

// ---------------------------------------------------------------------------
// SQLiteSet<Blob> Tests
// ---------------------------------------------------------------------------

#[test]
fn test_sqlite_set_blob() {
    let mut set = SQLiteSet::<Blob>::new().unwrap();

    assert_eq!(set.len().unwrap(), 0);
    assert!(set.is_empty().unwrap());
    assert!(!set.contains::<[u8]>(b"foo").unwrap());
    assert!(!set.contains::<[u8]>(b"bar").unwrap());
    assert!(!set.contains::<[u8]>(b"baz").unwrap());
    assert!(!set.contains::<[u8]>(b"qux").unwrap());

    assert!(set.insert::<[u8]>(b"foo").unwrap());
    assert!(set.insert::<[u8]>(b"bar").unwrap());
    assert!(!set.insert::<[u8]>(b"foo").unwrap());
    assert!(!set.insert::<[u8]>(b"bar").unwrap());
    assert!(set.insert::<[u8]>(b"baz").unwrap());
    assert!(!set.insert::<[u8]>(b"baz").unwrap());

    assert_eq!(set.len().unwrap(), 3);
    assert!(!set.is_empty().unwrap());
    assert!(set.contains::<[u8]>(b"foo").unwrap());
    assert!(!set.contains::<[u8]>(b"FOO").unwrap());
    assert!(set.contains::<[u8]>(b"bar").unwrap());
    assert!(!set.contains::<[u8]>(b"BAR").unwrap());
    assert!(set.contains::<[u8]>(b"baz").unwrap());
    assert!(!set.contains::<[u8]>(b"BAZ").unwrap());
    assert!(!set.contains::<[u8]>(b"qux").unwrap());

    assert!(set.remove::<[u8]>(b"foo").unwrap());
    assert!(!set.remove::<[u8]>(b"foo").unwrap());
    assert!(!set.remove::<[u8]>(b"BAR").unwrap());
    assert!(set.remove::<[u8]>(b"bar").unwrap());
    assert!(!set.remove::<[u8]>(b"qux").unwrap());

    assert_eq!(set.len().unwrap(), 1);
    assert!(!set.is_empty().unwrap());
    assert!(!set.contains::<[u8]>(b"foo").unwrap());
    assert!(!set.contains::<[u8]>(b"bar").unwrap());
    assert!(set.contains::<[u8]>(b"baz").unwrap());
    assert!(!set.contains::<[u8]>(b"BAZ").unwrap());
    assert!(!set.contains::<[u8]>(b"qux").unwrap());

    assert!(set.insert::<[u8]>(b"qux").unwrap());
    assert!(!set.insert::<[u8]>(b"qux").unwrap());
    assert!(set.insert::<[u8]>(b"foo").unwrap());
    assert!(!set.insert::<[u8]>(b"foo").unwrap());

    assert_eq!(set.len().unwrap(), 3);
    assert!(!set.is_empty().unwrap());
    assert!(set.contains::<[u8]>(b"foo").unwrap());
    assert!(!set.contains::<[u8]>(b"FOO").unwrap());
    assert!(!set.contains::<[u8]>(b"bar").unwrap());
    assert!(set.contains::<[u8]>(b"baz").unwrap());
    assert!(!set.contains::<[u8]>(b"BAZ").unwrap());
    assert!(set.contains::<[u8]>(b"qux").unwrap());
    assert!(!set.contains::<[u8]>(b"QUX").unwrap());

    {
        let mut hash = HashSet::new();
        set.for_each(|key| assert!(hash.insert(key))).unwrap();
        assert_eq!(hash.len(), 3);
        assert!(hash.contains::<[u8]>(b"foo"));
        assert!(hash.contains::<[u8]>(b"baz"));
        assert!(hash.contains::<[u8]>(b"qux"));
    }

    assert!(set.find(|item: &[u8]| item.eq(b"foo")).unwrap().is_some());
    assert!(set.find(|item: &[u8]| item.eq_ignore_ascii_case(b"FOO")).unwrap().is_some());
    assert!(set.find(|item: &[u8]| item.eq(b"bar")).unwrap().is_none());

    set.clear().unwrap();

    assert_eq!(set.len().unwrap(), 0);
    assert!(set.is_empty().unwrap());
    assert!(!set.contains::<[u8]>(b"foo").unwrap());
    assert!(!set.contains::<[u8]>(b"bar").unwrap());
    assert!(!set.contains::<[u8]>(b"baz").unwrap());
    assert!(!set.contains::<[u8]>(b"qux").unwrap());
}

#[test]
fn test_sqlite_set_blob_transacted() {
    let mut set = SQLiteSet::<Blob>::new().unwrap();
    let mut tx = set.transaction().unwrap();

    assert_eq!(tx.len().unwrap(), 0);
    assert!(tx.is_empty().unwrap());
    assert!(!tx.contains::<[u8]>(b"foo").unwrap());
    assert!(!tx.contains::<[u8]>(b"bar").unwrap());
    assert!(!tx.contains::<[u8]>(b"baz").unwrap());
    assert!(!tx.contains::<[u8]>(b"qux").unwrap());

    assert!(tx.insert::<[u8]>(b"foo").unwrap());
    assert!(tx.insert::<[u8]>(b"bar").unwrap());
    assert!(!tx.insert::<[u8]>(b"foo").unwrap());
    assert!(!tx.insert::<[u8]>(b"bar").unwrap());
    assert!(tx.insert::<[u8]>(b"baz").unwrap());
    assert!(!tx.insert::<[u8]>(b"baz").unwrap());

    assert_eq!(tx.len().unwrap(), 3);
    assert!(!tx.is_empty().unwrap());
    assert!(tx.contains::<[u8]>(b"foo").unwrap());
    assert!(!tx.contains::<[u8]>(b"FOO").unwrap());
    assert!(tx.contains::<[u8]>(b"bar").unwrap());
    assert!(!tx.contains::<[u8]>(b"BAR").unwrap());
    assert!(tx.contains::<[u8]>(b"baz").unwrap());
    assert!(!tx.contains::<[u8]>(b"BAZ").unwrap());
    assert!(!tx.contains::<[u8]>(b"qux").unwrap());

    assert!(tx.remove::<[u8]>(b"foo").unwrap());
    assert!(!tx.remove::<[u8]>(b"foo").unwrap());
    assert!(!tx.remove::<[u8]>(b"BAR").unwrap());
    assert!(tx.remove::<[u8]>(b"bar").unwrap());
    assert!(!tx.remove::<[u8]>(b"qux").unwrap());

    assert_eq!(tx.len().unwrap(), 1);
    assert!(!tx.is_empty().unwrap());
    assert!(!tx.contains::<[u8]>(b"foo").unwrap());
    assert!(!tx.contains::<[u8]>(b"bar").unwrap());
    assert!(tx.contains::<[u8]>(b"baz").unwrap());
    assert!(!tx.contains::<[u8]>(b"BAZ").unwrap());
    assert!(!tx.contains::<[u8]>(b"qux").unwrap());

    assert!(tx.insert::<[u8]>(b"qux").unwrap());
    assert!(!tx.insert::<[u8]>(b"qux").unwrap());
    assert!(tx.insert::<[u8]>(b"foo").unwrap());
    assert!(!tx.insert::<[u8]>(b"foo").unwrap());

    assert_eq!(tx.len().unwrap(), 3);
    assert!(!tx.is_empty().unwrap());
    assert!(tx.contains::<[u8]>(b"foo").unwrap());
    assert!(!tx.contains::<[u8]>(b"FOO").unwrap());
    assert!(!tx.contains::<[u8]>(b"bar").unwrap());
    assert!(tx.contains::<[u8]>(b"baz").unwrap());
    assert!(!tx.contains::<[u8]>(b"BAZ").unwrap());
    assert!(tx.contains::<[u8]>(b"qux").unwrap());
    assert!(!tx.contains::<[u8]>(b"QUX").unwrap());

    {
        let mut hash = HashSet::new();
        tx.for_each(|key| assert!(hash.insert(key))).unwrap();
        assert_eq!(hash.len(), 3);
        assert!(hash.contains::<[u8]>(b"foo"));
        assert!(hash.contains::<[u8]>(b"baz"));
        assert!(hash.contains::<[u8]>(b"qux"));
    }

    assert!(tx.find(|item: &[u8]| item.eq(b"foo")).unwrap().is_some());
    assert!(tx.find(|item: &[u8]| item.eq_ignore_ascii_case(b"FOO")).unwrap().is_some());
    assert!(tx.find(|item: &[u8]| item.eq(b"bar")).unwrap().is_none());

    tx.clear().unwrap();

    assert_eq!(tx.len().unwrap(), 0);
    assert!(tx.is_empty().unwrap());
    assert!(!tx.contains::<[u8]>(b"foo").unwrap());
    assert!(!tx.contains::<[u8]>(b"bar").unwrap());
    assert!(!tx.contains::<[u8]>(b"baz").unwrap());
    assert!(!tx.contains::<[u8]>(b"qux").unwrap());

    assert!(tx.insert::<[u8]>(b"foo").unwrap());
    assert!(tx.insert::<[u8]>(b"bar").unwrap());

    tx.commit();

    assert_eq!(set.len().unwrap(), 2);
    assert!(!set.is_empty().unwrap());
    assert!(set.contains::<[u8]>(b"foo").unwrap());
    assert!(set.contains::<[u8]>(b"bar").unwrap());
    assert!(!set.contains::<[u8]>(b"baz").unwrap());
    assert!(!set.contains::<[u8]>(b"qux").unwrap());
}

#[test]
fn test_sqlite_set_blob_stresstest() {
    let mut set = SQLiteSet::<Blob>::new().unwrap();
    let mut hex = HexEncoder::default();
    const MAX_ITEMS: u64 = 1_000_000u64;

    {
        let mut tx = set.transaction().unwrap();
        for i in 0u64..MAX_ITEMS {
            assert!(tx.insert(hex.encode_raw(mix64(i))).unwrap());
        }
    }

    assert!(!set.is_empty().unwrap());
    assert_eq!(set.len().unwrap().to_u64(), MAX_ITEMS);

    {
        let tx = set.transaction().unwrap();
        for i in 0u64..MAX_ITEMS {
            assert!(tx.contains(hex.encode_raw(mix64(i))).unwrap());
            assert!(!tx.contains(hex.encode_raw(mix64(MAX_ITEMS.checked_add(i).unwrap()))).unwrap());
        }
    }

    assert!(!set.is_empty().unwrap());
    assert_eq!(set.len().unwrap().to_u64(), MAX_ITEMS);

    {
        let mut tx = set.transaction().unwrap();
        for i in 0u64..MAX_ITEMS {
            assert!(tx.remove(hex.encode_raw(mix64(i))).unwrap());
        }
        for i in 0u64..MAX_ITEMS {
            assert!(!tx.remove(hex.encode_raw(mix64(i))).unwrap());
        }
    }

    assert!(set.is_empty().unwrap());
}

// ---------------------------------------------------------------------------
// SQLiteSet Multi-Thread Tests
// ---------------------------------------------------------------------------

fn sqlite_set_thread_worker<const MAX_ITEMS: u64>(mut thread_set: SQLiteSet<String>, barrier: &Barrier) -> SQLiteSet<String> {
    let mut hex = HexEncoder::default();

    barrier.wait();
    {
        let mut tx = thread_set.transaction().unwrap();
        for i in 0u64..MAX_ITEMS {
            assert!(tx.insert(hex.encode(mix64(i))).unwrap());
        }
    }

    thread_set
}

#[test]
fn test_sqlite_set_multithreading() {
    const MAX_ITEMS: u64 = 1_000_000u64;
    const THREADS: usize = 8usize;

    let maps: [SQLiteSet<String>; THREADS] = array::from_fn(|_| SQLiteSet::new().unwrap());
    let barrier = Arc::new(Barrier::new(THREADS));
    let mut handles: Vec<_> = maps
        .into_iter()
        .map(|map| {
            let barrier = Arc::clone(&barrier);
            thread::spawn(move || sqlite_set_thread_worker::<MAX_ITEMS>(map, &barrier))
        })
        .collect();

    for handle in handles.drain(..) {
        let thread_map = handle.join().unwrap();
        assert_eq!(thread_map.len().unwrap().to_u64(), MAX_ITEMS);
    }
}

// ---------------------------------------------------------------------------
// SQLiteSet Test to Destruction
// ---------------------------------------------------------------------------

#[ignore]
#[test]
fn test_sqlite_set_to_destruction() {
    let mut set = SQLiteSet::<String>::new().unwrap();
    let (channel_tx, channel_rx) = sync_channel::<String>(1024usize);

    let _handle = thread::spawn(move || {
        let mut hex = HexEncoder::default();
        for i in 0u64..=u64::MAX {
            if channel_tx.send(hex.encode(mix64(i)).to_owned()).is_err() {
                break;
            }
        }
    });

    let mut tx = set.transaction().unwrap();

    loop {
        let key = channel_rx.recv().expect("Out of items!");
        match tx.insert(&key) {
            Err(error) => match error.into_inner() {
                sqlite_containers::rusqlite::Error::SqliteFailure(sqlite_error, _) => match sqlite_error.code {
                    sqlite_containers::rusqlite::ErrorCode::OutOfMemory => break,
                    sqlite_containers::rusqlite::ErrorCode::DiskFull => break,
                    _ => panic!("Unexpected SQLite failure has been encountered! [{}]", sqlite_error),
                },
                _ => panic!("Unexpected SQLiteMap has been encountered!"),
            },
            Ok(success) => assert!(success),
        }
    }
}
