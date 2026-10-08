// SQLiteSet Tests
// This file is part of the 'SQLite-based containers for Rust' project (sqlite-containers)
// SPDX-License-Identifier: Unlicense

use des::{
    Des,
    cipher::{Block, BlockCipherEncBackend, KeyInit},
};
use sqlite_containers::SQLiteSet;
use std::collections::HashSet;

// ---------------------------------------------------------------------------
// SQLiteSet Tests
// ---------------------------------------------------------------------------

#[test]
fn test_sqlite_set() {
    let mut set = SQLiteSet::default();

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
        set.for_each(|key| assert!(hash.insert(key.to_owned()))).unwrap();
        assert_eq!(hash.len(), 3);
        assert!(hash.contains("foo"));
        assert!(hash.contains("baz"));
        assert!(hash.contains("qux"));
    }

    assert!(set.find(|item| item.eq("foo")).unwrap().is_some());
    assert!(set.find(|item| item.eq_ignore_ascii_case("FOO")).unwrap().is_some());
    assert!(set.find(|item| item.eq("bar")).unwrap().is_none());

    set.clear().unwrap();

    assert_eq!(set.len().unwrap(), 0);
    assert!(set.is_empty().unwrap());
    assert!(!set.contains("foo").unwrap());
    assert!(!set.contains("bar").unwrap());
    assert!(!set.contains("baz").unwrap());
    assert!(!set.contains("qux").unwrap());
}

#[test]
fn test_sqlite_set_transacted() {
    let mut set = SQLiteSet::new().unwrap();
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
        tx.for_each(|key| assert!(hash.insert(key.to_owned()))).unwrap();
        assert_eq!(hash.len(), 3);
        assert!(hash.contains("foo"));
        assert!(hash.contains("baz"));
        assert!(hash.contains("qux"));
    }

    assert!(tx.find(|item| item.eq("foo")).unwrap().is_some());
    assert!(tx.find(|item| item.eq_ignore_ascii_case("FOO")).unwrap().is_some());
    assert!(tx.find(|item| item.eq("bar")).unwrap().is_none());

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
fn test_sqlite_set_ignorecase() {
    let mut set_1 = SQLiteSet::new().unwrap();
    let mut set_2 = SQLiteSet::with_nocase().unwrap();

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
fn test_sqlite_set_stresstest() {
    let mut set = SQLiteSet::new().unwrap();
    const COUNT: u64 = 1_000_003u64;
    {
        let mut tx = set.transaction().unwrap();
        let cipher = Des::new_from_slice(b"12345678").unwrap();
        for i in 0u64..COUNT {
            let mut block = Block::<Des>::from(i.to_be_bytes());
            cipher.encrypt_block_inplace(&mut block);
            let key = format!("{:016X}", u64::from_be_bytes(block.into()));
            assert!(tx.insert(&key).unwrap());
        }
    }
    {
        let tx = set.transaction().unwrap();
        let cipher = Des::new_from_slice(b"12345678").unwrap();
        for i in 0u64..COUNT {
            let mut block = Block::<Des>::from(i.to_be_bytes());
            cipher.encrypt_block_inplace(&mut block);
            let key = format!("{:016X}", u64::from_be_bytes(block.into()));
            assert!(tx.contains(&key).unwrap());
        }
        for i in 0u64..COUNT {
            let mut block = Block::<Des>::from(i.checked_add(COUNT).unwrap().to_be_bytes());
            cipher.encrypt_block_inplace(&mut block);
            let key = format!("{:016X}", u64::from_be_bytes(block.into()));
            assert!(!tx.contains(&key).unwrap());
        }
    }
}
