// SQLiteSet Tests
// This file is part of the 'SQLite-based containers for Rust' project (sqlite-containers)
// SPDX-License-Identifier: Unlicense

#[path = "utilities/consts.rs"]
mod consts;
#[path = "utilities/utilities.rs"]
mod utilities;

use crate::{
    consts::MAX_ITEMS,
    utilities::{HexEncoder, mix64},
};
use rand_xoshiro::{
    Xoshiro256PlusPlus,
    rand_core::{Rng, SeedableRng},
};
use sqlite_containers::{Blob, SQLiteSet, SizeExt};
use std::{
    array,
    collections::HashSet,
    fs,
    sync::{Arc, Barrier, mpsc::sync_channel},
    thread,
};
use tempfile::tempdir_in;

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

    assert!(set.find(|item| item.eq(&0i64)).unwrap().is_some_and(|item| item == 0i64));
    assert!(set.find(|item| item.eq(&42i64)).unwrap().is_none());
    assert!(set.find(|item| item.eq(&i64::MAX)).unwrap().is_some_and(|item| item == i64::MAX));
    assert!(set.find(|item| item.eq(&i64::MIN)).unwrap().is_some_and(|item| item == i64::MIN));

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

    assert!(tx.find(|item| item.eq(&0i64)).unwrap().is_some_and(|item| item == 0i64));
    assert!(tx.find(|item| item.eq(&42i64)).unwrap().is_none());
    assert!(tx.find(|item| item.eq(&i64::MAX)).unwrap().is_some_and(|item| item == i64::MAX));
    assert!(tx.find(|item| item.eq(&i64::MIN)).unwrap().is_some_and(|item| item == i64::MIN));

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

    {
        let mut tx = set.transaction().unwrap();
        for i in 0u64..MAX_ITEMS {
            assert!(tx.insert(&(mix64(i) as i64)).unwrap());
        }
    }

    assert!(!set.is_empty().unwrap());
    assert_eq!(set.len().unwrap().to_u64(), MAX_ITEMS);

    {
        let mut tx = set.transaction().unwrap();
        for i in 0u64..MAX_ITEMS {
            assert!(tx.remove(&(mix64(i) as i64)).unwrap());
        }
    }

    assert!(set.is_empty().unwrap());
    assert_eq!(set.len().unwrap().to_u64(), 0);
}

#[test]
fn test_sqlite_set_i64_randomized() {
    let mut set = SQLiteSet::<i64>::new().unwrap();
    const COUNT: u64 = 125_000u64;

    for seed in [0x9DDC5F3146219B97u64, 0x79B9E4DC002B32D6u64] {
        for b in [true, false] {
            let mut xoshiro = Xoshiro256PlusPlus::seed_from_u64(seed);
            let mut tx = set.transaction().unwrap();
            for _i in 0u64..COUNT {
                assert_eq!(tx.insert(&(xoshiro.next_u64() as i64)).unwrap(), b);
            }
        }
    }

    for (seed, b) in [(0x9DDC5F3146219B97u64, true), (0x79B9E4DC002B32D6u64, true), (0x2F381F8B2F0E517Au64, false)] {
        let mut xoshiro = Xoshiro256PlusPlus::seed_from_u64(seed);
        let tx = set.transaction().unwrap();
        for _i in 0u64..COUNT {
            assert_eq!(tx.contains(&(xoshiro.next_u64() as i64)).unwrap(), b);
        }
    }
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

    assert!(set.find(|item: &str| item.eq("foo")).unwrap().is_some_and(|item| item.eq("foo")));
    assert!(set.find(|item: &str| item.eq_ignore_ascii_case("FOO")).unwrap().is_some_and(|item| item.eq("foo")));
    assert!(set.find(|item: &str| item.eq("bar")).unwrap().is_none());

    {
        let mut hash = HashSet::new();
        set.glob(false, "f*", |key| assert!(hash.insert(key))).unwrap();
        assert_eq!(hash.len(), 1);
        assert!(hash.contains("foo"));
    }

    {
        let mut hash = HashSet::new();
        set.glob(false, "b?z", |key| assert!(hash.insert(key))).unwrap();
        assert_eq!(hash.len(), 1);
        assert!(hash.contains("baz"));
    }

    {
        let mut hash = HashSet::new();
        set.glob(true, "b?z", |key| assert!(hash.insert(key))).unwrap();
        assert_eq!(hash.len(), 2);
        assert!(hash.contains("foo"));
        assert!(hash.contains("qux"));
    }

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

    assert!(tx.find(|item: &str| item.eq("foo")).unwrap().is_some_and(|key| key.eq("foo")));
    assert!(tx.find(|item: &str| item.eq_ignore_ascii_case("FOO")).unwrap().is_some_and(|key| key.eq("foo")));
    assert!(tx.find(|item: &str| item.eq("bar")).unwrap().is_none());

    {
        let mut hash = HashSet::new();
        tx.glob(false, "f*", |key| assert!(hash.insert(key))).unwrap();
        assert_eq!(hash.len(), 1);
        assert!(hash.contains("foo"));
    }

    {
        let mut hash = HashSet::new();
        tx.glob(false, "b?z", |key| assert!(hash.insert(key))).unwrap();
    }

    {
        let mut hash = HashSet::new();
        tx.glob(true, "b?z", |key| assert!(hash.insert(key))).unwrap();
        assert_eq!(hash.len(), 2);
        assert!(hash.contains("foo"));
        assert!(hash.contains("qux"));
    }

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

    {
        let mut tx = set.transaction().unwrap();
        for i in 0u64..MAX_ITEMS {
            assert!(tx.insert(hex.encode(mix64(i))).unwrap());
        }
    }

    assert!(!set.is_empty().unwrap());
    assert_eq!(set.len().unwrap().to_u64(), MAX_ITEMS);

    {
        let mut tx = set.transaction().unwrap();
        for i in 0u64..MAX_ITEMS {
            assert!(tx.remove(hex.encode(mix64(i))).unwrap());
        }
    }

    assert!(set.is_empty().unwrap());
    assert_eq!(set.len().unwrap().to_u64(), 0);
}

#[test]
fn test_sqlite_set_string_randomized() {
    const COUNT: u64 = 125_000u64;
    let mut set = SQLiteSet::<String>::new().unwrap();
    let mut hex = HexEncoder::default();

    for seed in [0x9DDC5F3146219B97u64, 0x79B9E4DC002B32D6u64] {
        for b in [true, false] {
            let mut xoshiro = Xoshiro256PlusPlus::seed_from_u64(seed);
            let mut tx = set.transaction().unwrap();
            for _i in 0u64..COUNT {
                assert_eq!(tx.insert(hex.encode(xoshiro.next_u64())).unwrap(), b);
            }
        }
    }

    for (seed, b) in [(0x9DDC5F3146219B97u64, true), (0x79B9E4DC002B32D6u64, true), (0x2F381F8B2F0E517Au64, false)] {
        let mut xoshiro = Xoshiro256PlusPlus::seed_from_u64(seed);
        let tx = set.transaction().unwrap();
        for _i in 0u64..COUNT {
            assert_eq!(tx.contains(hex.encode(xoshiro.next_u64())).unwrap(), b);
        }
    }
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

    assert!(set.find(|item: &[u8]| item.eq(b"foo")).unwrap().is_some_and(|key| key.eq(b"foo")));
    assert!(set.find(|item: &[u8]| item.eq_ignore_ascii_case(b"FOO")).unwrap().is_some_and(|key| key.eq(b"foo")));
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

    assert!(tx.find(|item: &[u8]| item.eq(b"foo")).unwrap().is_some_and(|item| item.eq(b"foo")));
    assert!(tx.find(|item: &[u8]| item.eq_ignore_ascii_case(b"FOO")).unwrap().is_some_and(|item| item.eq(b"foo")));
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

    {
        let mut tx = set.transaction().unwrap();
        for i in 0u64..MAX_ITEMS {
            assert!(tx.insert(hex.encode_raw(mix64(i))).unwrap());
        }
    }

    assert!(!set.is_empty().unwrap());
    assert_eq!(set.len().unwrap().to_u64(), MAX_ITEMS);

    {
        let mut tx = set.transaction().unwrap();
        for i in 0u64..MAX_ITEMS {
            assert!(tx.remove(hex.encode_raw(mix64(i))).unwrap());
        }
    }

    assert!(set.is_empty().unwrap());
    assert_eq!(set.len().unwrap().to_u64(), 0);
}

#[test]
fn test_sqlite_set_blob_randomized() {
    const COUNT: u64 = 125_000u64;
    let mut set = SQLiteSet::<Blob>::new().unwrap();
    let mut temp = [0u8; 16usize];

    for seed in [0x9DDC5F3146219B97u64, 0x79B9E4DC002B32D6u64] {
        for b in [true, false] {
            let mut xoshiro = Xoshiro256PlusPlus::seed_from_u64(seed);
            let mut tx = set.transaction().unwrap();
            for _i in 0u64..COUNT {
                xoshiro.fill_bytes(&mut temp);
                assert_eq!(tx.insert(temp.as_slice()).unwrap(), b);
            }
        }
    }

    for (seed, b) in [(0x9DDC5F3146219B97u64, true), (0x79B9E4DC002B32D6u64, true), (0x2F381F8B2F0E517Au64, false)] {
        let mut xoshiro = Xoshiro256PlusPlus::seed_from_u64(seed);
        let tx = set.transaction().unwrap();
        for _i in 0u64..COUNT {
            xoshiro.fill_bytes(&mut temp);
            assert_eq!(tx.contains(temp.as_slice()).unwrap(), b);
        }
    }
}

// ---------------------------------------------------------------------------
// SQLiteSet File-based Test
// ---------------------------------------------------------------------------

#[test]
fn test_sqlite_set_filebased() {
    let temp_dir = tempdir_in(env!["CARGO_TARGET_TMPDIR"]).unwrap();
    let file_path_1 = temp_dir.path().join("sqlite_set-1.db");
    let file_path_2 = temp_dir.path().join("sqlite_set-2.db");

    assert!(!fs::exists(&file_path_1).unwrap());
    assert!(!fs::exists(&file_path_2).unwrap());

    {
        let mut set = SQLiteSet::<String>::with_file(&file_path_1).unwrap();
        {
            let mut tx = set.transaction().unwrap();
            assert!(tx.insert("foo").unwrap());
            assert!(tx.insert("baz").unwrap());
            assert!(tx.insert("BAZ").unwrap());
        }
    }

    {
        let mut set = SQLiteSet::<String>::with_file_nocase(&file_path_2).unwrap();
        {
            let mut tx = set.transaction().unwrap();
            assert!(tx.insert("bar").unwrap());
            assert!(tx.insert("qux").unwrap());
            assert!(!tx.insert("QUX").unwrap());
        }
    }

    assert!(fs::exists(&file_path_1).unwrap());
    assert!(fs::exists(&file_path_2).unwrap());

    for no_case in [false, true] {
        let set = if !no_case { SQLiteSet::<String>::with_file(&file_path_1).unwrap() } else { SQLiteSet::<String>::with_file_nocase(&file_path_1).unwrap() };
        assert!(set.contains("foo").unwrap());
        assert!(!set.contains("FOO").unwrap());
        assert!(!set.contains("bar").unwrap());
        assert!(set.contains("baz").unwrap());
        assert!(set.contains("BAZ").unwrap());
        assert!(!set.contains("qux").unwrap());
    }

    for no_case in [false, true] {
        let set = if !no_case { SQLiteSet::<String>::with_file(&file_path_2).unwrap() } else { SQLiteSet::<String>::with_file_nocase(&file_path_2).unwrap() };
        assert!(!set.contains("foo").unwrap());
        assert!(set.contains("bar").unwrap());
        assert!(set.contains("BAR").unwrap());
        assert!(!set.contains("baz").unwrap());
        assert!(set.contains("qux").unwrap());
        assert!(set.contains("QUX").unwrap());
    }
}

// ---------------------------------------------------------------------------
// SQLiteSet Multi-Thread Tests
// ---------------------------------------------------------------------------

fn sqlite_set_thread_worker<const COUNT: u64>(mut thread_set: SQLiteSet<String>, barrier: &Barrier) -> SQLiteSet<String> {
    let mut hex = HexEncoder::default();
    barrier.wait();

    {
        let mut tx = thread_set.transaction().unwrap();
        for i in 0u64..COUNT {
            assert!(tx.insert(hex.encode(mix64(i))).unwrap());
        }
    }

    thread_set
}

#[test]
fn test_sqlite_set_multithreading() {
    const COUNT: u64 = 125_000u64;
    const THREADS: usize = 8usize;

    let maps: [SQLiteSet<String>; THREADS] = array::from_fn(|_| SQLiteSet::new().unwrap());
    let barrier = Arc::new(Barrier::new(THREADS));
    let mut handles: Vec<_> = maps
        .into_iter()
        .map(|map| {
            let barrier = Arc::clone(&barrier);
            thread::spawn(move || sqlite_set_thread_worker::<COUNT>(map, &barrier))
        })
        .collect();

    for handle in handles.drain(..) {
        let thread_map = handle.join().unwrap();
        assert_eq!(thread_map.len().unwrap().to_u64(), COUNT);
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
                    _ => panic!("Unexpected SQLite failure has been encountered! [{sqlite_error}]"),
                },
                _ => panic!("Unexpected SQLiteMap has been encountered!"),
            },
            Ok(success) => assert!(success),
        }
    }
}
