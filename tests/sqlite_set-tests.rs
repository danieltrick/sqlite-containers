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
use cfg_if::cfg_if;
use rand_xoshiro::{
    Xoshiro256PlusPlus,
    rand_core::{Rng, SeedableRng},
};
use sqlite_containers::{
    Blob, Error, IterationError, SQLiteSet, SizeExt, TryClone,
    rusqlite::{
        Error as RusqliteError,
        ffi::{Error as FfiError, ErrorCode as FfiErrorCode},
    },
};
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

    assert!(set.insert(0i64).unwrap());
    assert!(set.insert(42i64).unwrap());
    assert!(!set.insert(0i64).unwrap());
    assert!(!set.insert(42i64).unwrap());
    assert!(set.insert(i64::MAX).unwrap());
    assert!(!set.insert(i64::MAX).unwrap());

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

    assert!(set.insert(i64::MIN).unwrap());
    assert!(!set.insert(i64::MIN).unwrap());
    assert!(set.insert(0i64).unwrap());
    assert!(!set.insert(0i64).unwrap());

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

    {
        let mut hash = HashSet::new();
        set.try_for_each(|key| {
            assert!(hash.insert(key.to_owned()));
            Ok::<_, ()>(())
        })
        .unwrap();
        assert_eq!(hash.len(), 3);
        assert!(hash.contains(&0i64));
        assert!(hash.contains(&i64::MAX));
        assert!(hash.contains(&i64::MIN));
    }

    assert!(matches!(set.try_for_each(|_key| Err("Failed!")), Err(IterationError::CallbackError(_))));

    assert!(set.find(|item| item.eq(&0i64)).unwrap().is_some_and(|item| item == 0i64));
    assert!(set.find(|item| item.eq(&42i64)).unwrap().is_none());
    assert!(set.find(|item| item.eq(&i64::MAX)).unwrap().is_some_and(|item| item == i64::MAX));
    assert!(set.find(|item| item.eq(&i64::MIN)).unwrap().is_some_and(|item| item == i64::MIN));

    set.clear().unwrap();

    assert_eq!(set.len().unwrap(), 0);
    assert!(set.is_empty().unwrap());
    assert!(!set.contains(&0i64).unwrap());
    assert!(!set.contains(&42i64).unwrap());
    assert!(!set.contains(&8191i64).unwrap());
    assert!(!set.contains(&131071i64).unwrap());
    assert!(!set.contains(&i64::MAX).unwrap());
    assert!(!set.contains(&i64::MIN).unwrap());

    assert!(set.insert(0i64).unwrap());
    assert!(set.insert(42i64).unwrap());

    assert_eq!(set.extend([42i64, 8191i64, 131071i64]).unwrap(), 2);

    assert_eq!(set.len().unwrap(), 4);
    assert!(!set.is_empty().unwrap());
    assert!(set.contains(&0i64).unwrap());
    assert!(set.contains(&42i64).unwrap());
    assert!(set.contains(&8191i64).unwrap());
    assert!(set.contains(&131071i64).unwrap());
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

    assert!(tx.insert(0i64).unwrap());
    assert!(tx.insert(42i64).unwrap());
    assert!(!tx.insert(0i64).unwrap());
    assert!(!tx.insert(42i64).unwrap());
    assert!(tx.insert(i64::MAX).unwrap());
    assert!(!tx.insert(i64::MAX).unwrap());

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

    assert!(tx.insert(i64::MIN).unwrap());
    assert!(!tx.insert(i64::MIN).unwrap());
    assert!(tx.insert(0i64).unwrap());
    assert!(!tx.insert(0i64).unwrap());

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

    {
        let mut hash = HashSet::new();
        tx.try_for_each(|key| {
            assert!(hash.insert(key.to_owned()));
            Ok::<_, ()>(())
        })
        .unwrap();
        assert_eq!(hash.len(), 3);
        assert!(hash.contains(&0i64));
        assert!(hash.contains(&i64::MAX));
        assert!(hash.contains(&i64::MIN));
    }

    assert!(matches!(tx.try_for_each(|_key| Err("Failed!")), Err(IterationError::CallbackError(_))));

    assert!(tx.find(|item| item.eq(&0i64)).unwrap().is_some_and(|item| item == 0i64));
    assert!(tx.find(|item| item.eq(&42i64)).unwrap().is_none());
    assert!(tx.find(|item| item.eq(&i64::MAX)).unwrap().is_some_and(|item| item == i64::MAX));
    assert!(tx.find(|item| item.eq(&i64::MIN)).unwrap().is_some_and(|item| item == i64::MIN));

    tx.clear().unwrap();

    assert_eq!(tx.len().unwrap(), 0);
    assert!(tx.is_empty().unwrap());
    assert!(!tx.contains(&0i64).unwrap());
    assert!(!tx.contains(&42i64).unwrap());
    assert!(!tx.contains(&8191i64).unwrap());
    assert!(!tx.contains(&131071i64).unwrap());
    assert!(!tx.contains(&i64::MAX).unwrap());
    assert!(!tx.contains(&i64::MIN).unwrap());

    assert!(tx.insert(0i64).unwrap());
    assert!(tx.insert(42i64).unwrap());

    assert_eq!(tx.extend([42i64, 8191i64, 131071i64]).unwrap(), 2);

    tx.commit();

    assert_eq!(set.len().unwrap(), 4);
    assert!(!set.is_empty().unwrap());
    assert!(set.contains(&0i64).unwrap());
    assert!(set.contains(&42i64).unwrap());
    assert!(set.contains(&8191i64).unwrap());
    assert!(set.contains(&131071i64).unwrap());
    assert!(!set.contains(&i64::MAX).unwrap());
    assert!(!set.contains(&i64::MIN).unwrap());
}

#[test]
fn test_sqlite_set_i64_from_keys() {
    const KEYS: [i64; 6] = [0i64, 42i64, 8191i64, 42i64, i64::MIN, i64::MAX];
    let set = SQLiteSet::<i64>::try_from(KEYS).unwrap();

    assert_eq!(set.len().unwrap(), 5);
    assert!(set.contains(&0i64).unwrap());
    assert!(!set.contains(&1i64).unwrap());
    assert!(set.contains(&42i64).unwrap());
    assert!(set.contains(&8191i64).unwrap());
    assert!(!set.contains(&131071i64).unwrap());
    assert!(set.contains(&i64::MIN).unwrap());
    assert!(set.contains(&i64::MAX).unwrap());
}

#[test]
fn test_sqlite_set_i64_stresstest() {
    let mut set = SQLiteSet::<i64>::new().unwrap();

    {
        let mut tx = set.transaction().unwrap();
        for i in 0u64..MAX_ITEMS {
            assert!(tx.insert(mix64(i) as i64).unwrap());
        }
    }

    assert!(!set.is_empty().unwrap());
    assert_eq!(set.len().unwrap().to_u64(), MAX_ITEMS);

    {
        let mut tx = set.transaction().unwrap();
        for i in 0u64..MAX_ITEMS {
            let expected = mix64(i) as i64;
            assert!(tx.remove(&expected).unwrap());
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
                assert_eq!(tx.insert(xoshiro.next_u64() as i64).unwrap(), b);
            }
        }
    }

    for (seed, b) in [(0x9DDC5F3146219B97u64, true), (0x79B9E4DC002B32D6u64, true), (0x2F381F8B2F0E517Au64, false)] {
        let mut xoshiro = Xoshiro256PlusPlus::seed_from_u64(seed);
        let tx = set.transaction().unwrap();
        for _i in 0u64..COUNT {
            let expected = xoshiro.next_u64() as i64;
            assert_eq!(tx.contains(&expected).unwrap(), b);
        }
    }
}

#[test]
fn test_sqlite_set_i64_union() {
    let mut set_1 = SQLiteSet::<i64>::new().unwrap();
    let mut set_2 = SQLiteSet::<i64>::new().unwrap();

    {
        let mut tx = set_1.transaction().unwrap();
        assert!(tx.insert(0i64).unwrap());
        assert!(tx.insert(42i64).unwrap());
        assert!(tx.insert(2147483647i64).unwrap());
    }

    {
        let mut tx = set_2.transaction().unwrap();
        assert!(tx.insert(0i64).unwrap());
        assert!(tx.insert(i64::MIN).unwrap());
        assert!(tx.insert(i64::MAX).unwrap());
        assert!(tx.insert(2305843009213693951i64).unwrap());
    }

    assert!(set_1.contains(&0i64).unwrap());
    assert!(set_1.contains(&42i64).unwrap());
    assert!(!set_1.contains(&524287).unwrap());
    assert!(set_1.contains(&2147483647i64).unwrap());
    assert!(!set_1.contains(&2305843009213693951i64).unwrap());
    assert!(!set_1.contains(&i64::MIN).unwrap());
    assert!(!set_1.contains(&i64::MAX).unwrap());

    assert_eq!(set_1.union(&set_2).unwrap(), 3);

    assert!(set_1.contains(&0i64).unwrap());
    assert!(set_1.contains(&42i64).unwrap());
    assert!(!set_1.contains(&524287).unwrap());
    assert!(set_1.contains(&2147483647i64).unwrap());
    assert!(set_1.contains(&2305843009213693951i64).unwrap());
    assert!(set_1.contains(&i64::MIN).unwrap());
    assert!(set_1.contains(&i64::MAX).unwrap());
}

#[test]
fn test_sqlite_set_i64_difference() {
    let mut set_1 = SQLiteSet::<i64>::new().unwrap();
    let mut set_2 = SQLiteSet::<i64>::new().unwrap();

    {
        let mut tx = set_1.transaction().unwrap();
        assert!(tx.insert(0i64).unwrap());
        assert!(tx.insert(42i64).unwrap());
        assert!(tx.insert(2147483647i64).unwrap());
        assert!(tx.insert(i64::MIN).unwrap());
        assert!(tx.insert(i64::MAX).unwrap());
    }

    {
        let mut tx = set_2.transaction().unwrap();
        assert!(tx.insert(42i64).unwrap());
        assert!(tx.insert(2305843009213693951i64).unwrap());
        assert!(tx.insert(i64::MIN).unwrap());
    }

    assert!(set_1.contains(&0i64).unwrap());
    assert!(set_1.contains(&42i64).unwrap());
    assert!(!set_1.contains(&524287).unwrap());
    assert!(set_1.contains(&2147483647i64).unwrap());
    assert!(!set_1.contains(&2305843009213693951i64).unwrap());
    assert!(set_1.contains(&i64::MIN).unwrap());
    assert!(set_1.contains(&i64::MAX).unwrap());

    assert_eq!(set_1.difference(&set_2).unwrap(), 2);

    assert!(set_1.contains(&0i64).unwrap());
    assert!(!set_1.contains(&42i64).unwrap());
    assert!(!set_1.contains(&524287).unwrap());
    assert!(set_1.contains(&2147483647i64).unwrap());
    assert!(!set_1.contains(&2305843009213693951i64).unwrap());
    assert!(!set_1.contains(&i64::MIN).unwrap());
    assert!(set_1.contains(&i64::MAX).unwrap());
}

#[test]
fn test_sqlite_set_i64_intersection() {
    let mut set_1 = SQLiteSet::<i64>::new().unwrap();
    let mut set_2 = SQLiteSet::<i64>::new().unwrap();

    {
        let mut tx = set_1.transaction().unwrap();
        assert!(tx.insert(0i64).unwrap());
        assert!(tx.insert(42i64).unwrap());
        assert!(tx.insert(2147483647i64).unwrap());
        assert!(tx.insert(2305843009213693951i64).unwrap());
        assert!(tx.insert(i64::MIN).unwrap());
        assert!(tx.insert(i64::MAX).unwrap());
    }

    {
        let mut tx = set_2.transaction().unwrap();
        assert!(tx.insert(42i64).unwrap());
        assert!(tx.insert(2305843009213693951i64).unwrap());
        assert!(tx.insert(i64::MAX).unwrap());
    }

    let set_3 = set_1.intersection(&set_2).unwrap();
    assert_eq!(set_3.len().unwrap(), 3);

    assert!(!set_3.contains(&0i64).unwrap());
    assert!(set_3.contains(&42i64).unwrap());
    assert!(!set_3.contains(&2147483647i64).unwrap());
    assert!(set_3.contains(&2305843009213693951i64).unwrap());
    assert!(!set_3.contains(&i64::MIN).unwrap());
    assert!(set_3.contains(&i64::MAX).unwrap());
}

#[cfg(not(debug_assertions))]
#[test]
fn test_sqlite_set_i64_string_words() {
    use xxhash_rust::xxh64::xxh64;
    for lang in [random_word::Lang::De, random_word::Lang::En] {
        let word_list = random_word::all(lang);
        for seed_1 in 0u64..5u64 {
            let mut set = SQLiteSet::<i64>::new().unwrap();
            {
                let mut tx = set.transaction().unwrap();
                for &word in word_list {
                    assert!(tx.insert(xxh64(word.as_bytes(), seed_1) as i64).unwrap());
                }
            }
            for seed_2 in 0u64..5u64 {
                let tx = set.transaction().unwrap();
                if seed_1 == seed_2 {
                    for &word in word_list {
                        let rexpected_hash = xxh64(word.as_bytes(), seed_2) as i64;
                        assert!(tx.contains(&rexpected_hash).unwrap());
                    }
                } else {
                    for &word in word_list {
                        let rexpected_hash = xxh64(word.as_bytes(), seed_2) as i64;
                        assert!(!tx.contains(&rexpected_hash).unwrap());
                    }
                }
            }
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

    assert!(set.insert("foo".to_owned()).unwrap());
    assert!(set.insert("bar".to_owned()).unwrap());
    assert!(!set.insert("foo".to_owned()).unwrap());
    assert!(!set.insert("bar".to_owned()).unwrap());
    assert!(set.insert("baz".to_owned()).unwrap());
    assert!(!set.insert("baz".to_owned()).unwrap());

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

    assert!(set.insert("qux".to_owned()).unwrap());
    assert!(!set.insert("qux".to_owned()).unwrap());
    assert!(set.insert("foo".to_owned()).unwrap());
    assert!(!set.insert("foo".to_owned()).unwrap());

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

    {
        let mut hash = HashSet::new();
        set.try_for_each(|key| {
            assert!(hash.insert(key.to_owned()));
            Ok::<_, ()>(())
        })
        .unwrap();
        assert_eq!(hash.len(), 3);
        assert!(hash.contains("foo"));
        assert!(hash.contains("baz"));
        assert!(hash.contains("qux"));
    }

    assert!(matches!(set.try_for_each(|_key| Err("Failed!")), Err(IterationError::CallbackError(_))));

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
    assert!(!set.contains("xyz").unwrap());

    assert!(set.insert("foo".to_owned()).unwrap());
    assert!(set.insert("baz".to_owned()).unwrap());

    assert_eq!(set.extend(["bar", "baz", "qux"].map(str::to_owned)).unwrap(), 2);

    assert_eq!(set.len().unwrap(), 4);
    assert!(!set.is_empty().unwrap());
    assert!(set.contains("foo").unwrap());
    assert!(set.contains("bar").unwrap());
    assert!(set.contains("baz").unwrap());
    assert!(set.contains("qux").unwrap());
    assert!(!set.contains("xyz").unwrap());
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

    assert!(tx.insert("foo".to_owned()).unwrap());
    assert!(tx.insert("bar".to_owned()).unwrap());
    assert!(!tx.insert("foo".to_owned()).unwrap());
    assert!(!tx.insert("bar".to_owned()).unwrap());
    assert!(tx.insert("baz".to_owned()).unwrap());
    assert!(!tx.insert("baz".to_owned()).unwrap());

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

    assert!(tx.insert("qux".to_owned()).unwrap());
    assert!(!tx.insert("qux".to_owned()).unwrap());
    assert!(tx.insert("foo".to_owned()).unwrap());
    assert!(!tx.insert("foo".to_owned()).unwrap());

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

    {
        let mut hash = HashSet::new();
        tx.try_for_each(|key| {
            assert!(hash.insert(key.to_owned()));
            Ok::<_, ()>(())
        })
        .unwrap();
        assert_eq!(hash.len(), 3);
        assert!(hash.contains("foo"));
        assert!(hash.contains("baz"));
        assert!(hash.contains("qux"));
    }

    assert!(matches!(tx.try_for_each(|_key| Err("Failed!")), Err(IterationError::CallbackError(_))));

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
    assert!(!tx.contains("xyz").unwrap());

    assert!(tx.insert("foo".to_owned()).unwrap());
    assert!(tx.insert("baz".to_owned()).unwrap());

    assert_eq!(tx.extend(["bar", "baz", "qux"].map(str::to_owned)).unwrap(), 2);

    tx.commit();

    assert_eq!(set.len().unwrap(), 4);
    assert!(!set.is_empty().unwrap());
    assert!(set.contains("foo").unwrap());
    assert!(set.contains("bar").unwrap());
    assert!(set.contains("baz").unwrap());
    assert!(set.contains("qux").unwrap());
    assert!(!set.contains("xyz").unwrap());
}

#[test]
fn test_sqlite_set_string_ignorecase() {
    let mut set_1 = SQLiteSet::<String>::new().unwrap();
    let mut set_2 = SQLiteSet::<String>::with_nocase().unwrap();

    assert!(set_1.insert("foo".to_owned()).unwrap());
    assert!(set_1.insert("FOO".to_owned()).unwrap());
    assert!(set_1.insert("BAR".to_owned()).unwrap());
    assert!(set_1.insert("bar".to_owned()).unwrap());

    assert!(set_2.insert("foo".to_owned()).unwrap());
    assert!(!set_2.insert("FOO".to_owned()).unwrap());
    assert!(set_2.insert("BAR".to_owned()).unwrap());
    assert!(!set_2.insert("bar".to_owned()).unwrap());

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
fn test_sqlite_set_string_from_keys() {
    let keys_1: [String; 6] = ["foo", "foo", "bar", "BAR", "baz", "qux"].map(str::to_string);
    let keys_2 = keys_1.clone();
    let set_1 = SQLiteSet::<String>::try_from(keys_1).unwrap();
    let set_2 = SQLiteSet::<String>::try_from_nocase(keys_2).unwrap();

    assert_eq!(set_1.len().unwrap(), 5);
    assert!(set_1.contains("foo").unwrap());
    assert!(!set_1.contains("FOO").unwrap());
    assert!(set_1.contains("bar").unwrap());
    assert!(set_1.contains("BAR").unwrap());
    assert!(set_1.contains("baz").unwrap());
    assert!(!set_1.contains("BAZ").unwrap());
    assert!(set_1.contains("qux").unwrap());
    assert!(!set_1.contains("QUX").unwrap());
    assert!(!set_1.contains("xyz").unwrap());

    assert_eq!(set_2.len().unwrap(), 4);
    assert!(set_2.contains("foo").unwrap());
    assert!(set_2.contains("FOO").unwrap());
    assert!(set_2.contains("bar").unwrap());
    assert!(set_2.contains("BAR").unwrap());
    assert!(set_2.contains("baz").unwrap());
    assert!(set_2.contains("BAZ").unwrap());
    assert!(set_2.contains("qux").unwrap());
    assert!(set_2.contains("QUX").unwrap());
    assert!(!set_2.contains("xyz").unwrap());
}

#[test]
fn test_sqlite_set_string_stresstest() {
    let mut set = SQLiteSet::<String>::new().unwrap();
    let mut hex = HexEncoder::default();

    {
        let mut tx = set.transaction().unwrap();
        for i in 0u64..MAX_ITEMS {
            assert!(tx.insert_ref(hex.encode(mix64(i))).unwrap());
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
    const COUNT: u64 = 75_000u64;
    let mut set = SQLiteSet::<String>::new().unwrap();
    let mut hex = HexEncoder::default();

    for (seed, b) in [(0x9DDC5F3146219B97u64, true), (0x79B9E4DC002B32D6u64, false)] {
        for c in [true, false] {
            let mut xoshiro = Xoshiro256PlusPlus::seed_from_u64(seed);
            if b {
                for _i in 0u64..COUNT {
                    assert_eq!(set.insert_ref(hex.encode(xoshiro.next_u64())).unwrap(), c);
                }
            } else {
                let mut tx = set.transaction().unwrap();
                for _i in 0u64..COUNT {
                    assert_eq!(tx.insert_ref(hex.encode(xoshiro.next_u64())).unwrap(), c);
                }
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

#[test]
fn test_sqlite_set_string_union() {
    let mut set_1 = SQLiteSet::<String>::new().unwrap();
    let mut set_2 = SQLiteSet::<String>::new().unwrap();

    {
        let mut tx = set_1.transaction().unwrap();
        assert!(tx.insert("foo".to_owned()).unwrap());
        assert!(tx.insert("bar".to_owned()).unwrap());
        assert!(tx.insert("baz".to_owned()).unwrap());
    }

    {
        let mut tx = set_2.transaction().unwrap();
        assert!(tx.insert("foo".to_owned()).unwrap());
        assert!(tx.insert("qux".to_owned()).unwrap());
        assert!(tx.insert("xyz".to_owned()).unwrap());
        assert!(tx.insert("cor".to_owned()).unwrap());
    }

    assert!(set_1.contains("foo").unwrap());
    assert!(set_1.contains("bar").unwrap());
    assert!(set_1.contains("baz").unwrap());
    assert!(!set_1.contains("qux").unwrap());
    assert!(!set_1.contains("xyz").unwrap());
    assert!(!set_1.contains("$?!").unwrap());
    assert!(!set_1.contains("cor").unwrap());

    assert_eq!(set_1.union(&set_2).unwrap(), 3);

    assert!(set_1.contains("foo").unwrap());
    assert!(set_1.contains("bar").unwrap());
    assert!(set_1.contains("baz").unwrap());
    assert!(set_1.contains("qux").unwrap());
    assert!(set_1.contains("xyz").unwrap());
    assert!(!set_1.contains("$?!").unwrap());
    assert!(set_1.contains("cor").unwrap());
}

#[test]
fn test_sqlite_set_string_difference() {
    let mut set_1 = SQLiteSet::<String>::new().unwrap();
    let mut set_2 = SQLiteSet::<String>::new().unwrap();

    {
        let mut tx = set_1.transaction().unwrap();
        assert!(tx.insert("foo".to_owned()).unwrap());
        assert!(tx.insert("bar".to_owned()).unwrap());
        assert!(tx.insert("baz".to_owned()).unwrap());
        assert!(tx.insert("qux".to_owned()).unwrap());
        assert!(tx.insert("xyz".to_owned()).unwrap());
    }

    {
        let mut tx = set_2.transaction().unwrap();
        assert!(tx.insert("baz".to_owned()).unwrap());
        assert!(tx.insert("xyz".to_owned()).unwrap());
        assert!(tx.insert("$?!".to_owned()).unwrap());
    }

    assert!(set_1.contains("foo").unwrap());
    assert!(set_1.contains("bar").unwrap());
    assert!(set_1.contains("baz").unwrap());
    assert!(set_1.contains("qux").unwrap());
    assert!(set_1.contains("xyz").unwrap());
    assert!(!set_1.contains("$?!").unwrap());

    assert_eq!(set_1.difference(&set_2).unwrap(), 2);

    assert!(set_1.contains("foo").unwrap());
    assert!(set_1.contains("bar").unwrap());
    assert!(!set_1.contains("baz").unwrap());
    assert!(set_1.contains("qux").unwrap());
    assert!(!set_1.contains("xyz").unwrap());
    assert!(!set_1.contains("$?!").unwrap());
}

#[test]
fn test_sqlite_set_string_intersection() {
    let mut set_1 = SQLiteSet::<String>::new().unwrap();
    let mut set_2 = SQLiteSet::<String>::new().unwrap();

    {
        let mut tx = set_1.transaction().unwrap();
        assert!(tx.insert("foo".to_owned()).unwrap());
        assert!(tx.insert("bar".to_owned()).unwrap());
        assert!(tx.insert("qux".to_owned()).unwrap());
        assert!(tx.insert("xyz".to_owned()).unwrap());
    }

    {
        let mut tx = set_2.transaction().unwrap();
        assert!(tx.insert("bar".to_owned()).unwrap());
        assert!(tx.insert("baz".to_owned()).unwrap());
        assert!(tx.insert("qux".to_owned()).unwrap());
        assert!(tx.insert("xyz".to_owned()).unwrap());
        assert!(tx.insert("$?!".to_owned()).unwrap());
    }

    let set_3 = set_1.intersection(&set_2).unwrap();
    assert_eq!(set_3.len().unwrap(), 3);

    assert!(!set_3.contains("foo").unwrap());
    assert!(set_3.contains("bar").unwrap());
    assert!(!set_3.contains("baz").unwrap());
    assert!(set_3.contains("qux").unwrap());
    assert!(set_3.contains("xyz").unwrap());
    assert!(!set_3.contains("$?!").unwrap());
}

#[test]
fn test_sqlite_set_string_intersection_nocase() {
    let mut set_1 = SQLiteSet::<String>::new().unwrap();
    let mut set_2 = SQLiteSet::<String>::new().unwrap();

    {
        let mut tx = set_1.transaction().unwrap();
        assert!(tx.insert("foo".to_owned()).unwrap());
        assert!(tx.insert("BAR".to_owned()).unwrap());
        assert!(tx.insert("qux".to_owned()).unwrap());
        assert!(tx.insert("xyz".to_owned()).unwrap());
        assert!(tx.insert("XYZ".to_owned()).unwrap());
    }

    {
        let mut tx = set_2.transaction().unwrap();
        assert!(tx.insert("bar".to_owned()).unwrap());
        assert!(tx.insert("baz".to_owned()).unwrap());
        assert!(tx.insert("qux".to_owned()).unwrap());
        assert!(tx.insert("xyz".to_owned()).unwrap());
        assert!(tx.insert("$?!".to_owned()).unwrap());
    }

    let set_3 = set_1.intersection_nocase(&set_2).unwrap();
    assert_eq!(set_3.len().unwrap(), 3);

    assert!(!set_3.contains("foo").unwrap());
    assert!(set_3.contains("bar").unwrap());
    assert!(set_3.contains("BAR").unwrap());
    assert!(!set_3.contains("baz").unwrap());
    assert!(set_3.contains("qux").unwrap());
    assert!(set_3.contains("QUX").unwrap());
    assert!(set_3.contains("xyz").unwrap());
    assert!(set_3.contains("XYZ").unwrap());
    assert!(!set_3.contains("$?!").unwrap());
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

    assert!(set.insert(b"foo".to_vec()).unwrap());
    assert!(set.insert(b"bar".to_vec()).unwrap());
    assert!(!set.insert(b"foo".to_vec()).unwrap());
    assert!(!set.insert(b"bar".to_vec()).unwrap());
    assert!(set.insert(b"baz".to_vec()).unwrap());
    assert!(!set.insert(b"baz".to_vec()).unwrap());

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

    assert!(set.insert(b"qux".to_vec()).unwrap());
    assert!(!set.insert(b"qux".to_vec()).unwrap());
    assert!(set.insert(b"foo".to_vec()).unwrap());
    assert!(!set.insert(b"foo".to_vec()).unwrap());

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

    {
        let mut hash = HashSet::new();
        set.try_for_each(|key| {
            assert!(hash.insert(key.to_owned()));
            Ok::<_, ()>(())
        })
        .unwrap();
        assert_eq!(hash.len(), 3);
        assert!(hash.contains::<[u8]>(b"foo"));
        assert!(hash.contains::<[u8]>(b"baz"));
        assert!(hash.contains::<[u8]>(b"qux"));
    }

    assert!(matches!(set.try_for_each(|_key| Err("Failed!")), Err(IterationError::CallbackError(_))));

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
    assert!(!set.contains::<[u8]>(b"xyz").unwrap());

    assert!(set.insert(b"foo".to_vec()).unwrap());
    assert!(set.insert(b"baz".to_vec()).unwrap());

    assert_eq!(set.extend([b"bar", b"baz", b"qux"].map(Blob::from)).unwrap(), 2);

    assert_eq!(set.len().unwrap(), 4);
    assert!(!set.is_empty().unwrap());
    assert!(set.contains::<[u8]>(b"foo").unwrap());
    assert!(set.contains::<[u8]>(b"bar").unwrap());
    assert!(set.contains::<[u8]>(b"baz").unwrap());
    assert!(set.contains::<[u8]>(b"qux").unwrap());
    assert!(!set.contains::<[u8]>(b"xyz").unwrap());
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

    assert!(tx.insert(b"foo".to_vec()).unwrap());
    assert!(tx.insert(b"bar".to_vec()).unwrap());
    assert!(!tx.insert(b"foo".to_vec()).unwrap());
    assert!(!tx.insert(b"bar".to_vec()).unwrap());
    assert!(tx.insert(b"baz".to_vec()).unwrap());
    assert!(!tx.insert(b"baz".to_vec()).unwrap());

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

    assert!(tx.insert(b"qux".to_vec()).unwrap());
    assert!(!tx.insert(b"qux".to_vec()).unwrap());
    assert!(tx.insert(b"foo".to_vec()).unwrap());
    assert!(!tx.insert(b"foo".to_vec()).unwrap());

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

    {
        let mut hash = HashSet::new();
        tx.try_for_each(|key| {
            assert!(hash.insert(key.to_owned()));
            Ok::<_, ()>(())
        })
        .unwrap();
        assert_eq!(hash.len(), 3);
        assert!(hash.contains::<[u8]>(b"foo"));
        assert!(hash.contains::<[u8]>(b"baz"));
        assert!(hash.contains::<[u8]>(b"qux"));
    }

    assert!(matches!(tx.try_for_each(|_key| Err("Failed!")), Err(IterationError::CallbackError(_))));

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
    assert!(!tx.contains::<[u8]>(b"xyz").unwrap());

    assert!(tx.insert(b"foo".to_vec()).unwrap());
    assert!(tx.insert(b"baz".to_vec()).unwrap());

    assert_eq!(tx.extend([b"bar", b"baz", b"qux"].map(Blob::from)).unwrap(), 2);

    tx.commit();

    assert_eq!(set.len().unwrap(), 4);
    assert!(!set.is_empty().unwrap());
    assert!(set.contains::<[u8]>(b"foo").unwrap());
    assert!(set.contains::<[u8]>(b"bar").unwrap());
    assert!(set.contains::<[u8]>(b"baz").unwrap());
    assert!(set.contains::<[u8]>(b"qux").unwrap());
    assert!(!set.contains::<[u8]>(b"xyz").unwrap());
}

#[test]
fn test_sqlite_set_blob_from_keys() {
    let keys: [Blob; 6] = [b"foo", b"foo", b"bar", b"BAR", b"baz", b"qux"].map(Blob::from);
    let set = SQLiteSet::<Blob>::try_from(keys).unwrap();

    assert_eq!(set.len().unwrap(), 5);
    assert!(set.contains(&b"foo"[..]).unwrap());
    assert!(!set.contains(&b"FOO"[..]).unwrap());
    assert!(set.contains(&b"bar"[..]).unwrap());
    assert!(set.contains(&b"BAR"[..]).unwrap());
    assert!(set.contains(&b"baz"[..]).unwrap());
    assert!(!set.contains(&b"BAZ"[..]).unwrap());
    assert!(set.contains(&b"qux"[..]).unwrap());
    assert!(!set.contains(&b"QUX"[..]).unwrap());
    assert!(!set.contains(&b"xyz"[..]).unwrap());
}

#[test]
fn test_sqlite_set_blob_stresstest() {
    let mut set = SQLiteSet::<Blob>::new().unwrap();
    let mut hex = HexEncoder::default();

    {
        let mut tx = set.transaction().unwrap();
        for i in 0u64..MAX_ITEMS {
            assert!(tx.insert_ref(hex.encode_raw(mix64(i))).unwrap());
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
                assert_eq!(tx.insert_ref(temp.as_slice()).unwrap(), b);
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

#[test]
fn test_sqlite_set_blob_union() {
    let mut set_1 = SQLiteSet::<Blob>::new().unwrap();
    let mut set_2 = SQLiteSet::<Blob>::new().unwrap();

    {
        let mut tx = set_1.transaction().unwrap();
        assert!(tx.insert(b"foo".to_vec()).unwrap());
        assert!(tx.insert(b"bar".to_vec()).unwrap());
        assert!(tx.insert(b"baz".to_vec()).unwrap());
    }

    {
        let mut tx = set_2.transaction().unwrap();
        assert!(tx.insert(b"foo".to_vec()).unwrap());
        assert!(tx.insert(b"qux".to_vec()).unwrap());
        assert!(tx.insert(b"xyz".to_vec()).unwrap());
        assert!(tx.insert(b"cor".to_vec()).unwrap());
    }

    assert!(set_1.contains(&b"foo"[..]).unwrap());
    assert!(set_1.contains(&b"bar"[..]).unwrap());
    assert!(set_1.contains(&b"baz"[..]).unwrap());
    assert!(!set_1.contains(&b"qux"[..]).unwrap());
    assert!(!set_1.contains(&b"xyz"[..]).unwrap());
    assert!(!set_1.contains(&b"$?!"[..]).unwrap());
    assert!(!set_1.contains(&b"cor"[..]).unwrap());

    assert_eq!(set_1.union(&set_2).unwrap(), 3);

    assert!(set_1.contains(&b"foo"[..]).unwrap());
    assert!(set_1.contains(&b"bar"[..]).unwrap());
    assert!(set_1.contains(&b"baz"[..]).unwrap());
    assert!(set_1.contains(&b"qux"[..]).unwrap());
    assert!(set_1.contains(&b"xyz"[..]).unwrap());
    assert!(!set_1.contains(&b"$?!"[..]).unwrap());
    assert!(set_1.contains(&b"cor"[..]).unwrap());
}

#[test]
fn test_sqlite_set_blob_difference() {
    let mut set_1 = SQLiteSet::<Blob>::new().unwrap();
    let mut set_2 = SQLiteSet::<Blob>::new().unwrap();

    {
        let mut tx = set_1.transaction().unwrap();
        assert!(tx.insert(b"foo".to_vec()).unwrap());
        assert!(tx.insert(b"bar".to_vec()).unwrap());
        assert!(tx.insert(b"baz".to_vec()).unwrap());
        assert!(tx.insert(b"qux".to_vec()).unwrap());
        assert!(tx.insert(b"xyz".to_vec()).unwrap());
    }

    {
        let mut tx = set_2.transaction().unwrap();
        assert!(tx.insert(b"baz".to_vec()).unwrap());
        assert!(tx.insert(b"xyz".to_vec()).unwrap());
        assert!(tx.insert(b"$?!".to_vec()).unwrap());
    }

    assert!(set_1.contains(&b"foo"[..]).unwrap());
    assert!(set_1.contains(&b"bar"[..]).unwrap());
    assert!(set_1.contains(&b"baz"[..]).unwrap());
    assert!(set_1.contains(&b"qux"[..]).unwrap());
    assert!(set_1.contains(&b"xyz"[..]).unwrap());
    assert!(!set_1.contains(&b"$?!"[..]).unwrap());

    assert_eq!(set_1.difference(&set_2).unwrap(), 2);

    assert!(set_1.contains(&b"foo"[..]).unwrap());
    assert!(set_1.contains(&b"bar"[..]).unwrap());
    assert!(!set_1.contains(&b"baz"[..]).unwrap());
    assert!(set_1.contains(&b"qux"[..]).unwrap());
    assert!(!set_1.contains(&b"xyz"[..]).unwrap());
    assert!(!set_1.contains(&b"$?!"[..]).unwrap());
}

#[test]
fn test_sqlite_set_blob_intersection() {
    let mut set_1 = SQLiteSet::<Blob>::new().unwrap();
    let mut set_2 = SQLiteSet::<Blob>::new().unwrap();

    {
        let mut tx = set_1.transaction().unwrap();
        assert!(tx.insert(b"foo".to_vec()).unwrap());
        assert!(tx.insert(b"bar".to_vec()).unwrap());
        assert!(tx.insert(b"qux".to_vec()).unwrap());
        assert!(tx.insert(b"xyz".to_vec()).unwrap());
    }

    {
        let mut tx = set_2.transaction().unwrap();
        assert!(tx.insert(b"bar".to_vec()).unwrap());
        assert!(tx.insert(b"baz".to_vec()).unwrap());
        assert!(tx.insert(b"qux".to_vec()).unwrap());
        assert!(tx.insert(b"xyz".to_vec()).unwrap());
        assert!(tx.insert(b"$?!".to_vec()).unwrap());
    }

    let set_3 = set_1.intersection(&set_2).unwrap();
    assert_eq!(set_3.len().unwrap(), 3);

    assert!(!set_3.contains(&b"foo"[..]).unwrap());
    assert!(set_3.contains(&b"bar"[..]).unwrap());
    assert!(!set_3.contains(&b"baz"[..]).unwrap());
    assert!(set_3.contains(&b"qux"[..]).unwrap());
    assert!(set_3.contains(&b"xyz"[..]).unwrap());
    assert!(!set_3.contains(&b"$?!"[..]).unwrap());
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

    assert!(matches!(
        SQLiteSet::<String>::try_from_file(&file_path_1, false, false),
        Err(Error::SqlError(RusqliteError::SqliteFailure(FfiError { code: FfiErrorCode::CannotOpen, .. }, _)))
    ));
    assert!(matches!(
        SQLiteSet::<String>::try_from_file(&file_path_2, false, false),
        Err(Error::SqlError(RusqliteError::SqliteFailure(FfiError { code: FfiErrorCode::CannotOpen, .. }, _)))
    ));

    assert!(matches!(
        SQLiteSet::<String>::try_from_file(&file_path_1, true, true),
        Err(Error::SqlError(RusqliteError::SqliteFailure(FfiError { code: FfiErrorCode::CannotOpen, .. }, _)))
    ));
    assert!(matches!(
        SQLiteSet::<String>::try_from_file(&file_path_2, true, true),
        Err(Error::SqlError(RusqliteError::SqliteFailure(FfiError { code: FfiErrorCode::CannotOpen, .. }, _)))
    ));

    {
        let mut set = SQLiteSet::<String>::try_from_file(&file_path_1, false, true).unwrap();
        {
            let mut tx = set.transaction().unwrap();
            assert!(tx.insert("foo".to_owned()).unwrap());
            assert!(tx.insert("baz".to_owned()).unwrap());
            assert!(tx.insert("BAZ".to_owned()).unwrap());
        }
    }

    {
        let mut set = SQLiteSet::<String>::try_from_file_nocase(&file_path_2, false, true).unwrap();
        {
            let mut tx = set.transaction().unwrap();
            assert!(tx.insert("bar".to_owned()).unwrap());
            assert!(tx.insert("qux".to_owned()).unwrap());
            assert!(!tx.insert("QUX".to_owned()).unwrap());
        }
    }

    assert!(fs::exists(&file_path_1).unwrap());
    assert!(fs::exists(&file_path_2).unwrap());

    {
        let mut set = SQLiteSet::<String>::try_from_file(&file_path_1, true, false).unwrap();
        assert!(set.contains("foo").unwrap());
        assert!(matches!(set.insert("xyz".to_owned()), Err(Error::SqlError(RusqliteError::SqliteFailure(FfiError { code: FfiErrorCode::ReadOnly, .. }, _)))));
    }

    {
        let mut set = SQLiteSet::<String>::try_from_file(&file_path_1, false, false).unwrap();
        assert!(set.insert("xyz".to_owned()).unwrap());
    }

    {
        let set = SQLiteSet::<String>::try_from_file(&file_path_1, false, false).unwrap();
        assert!(set.contains("foo").unwrap());
        assert!(!set.contains("FOO").unwrap());
        assert!(!set.contains("bar").unwrap());
        assert!(set.contains("baz").unwrap());
        assert!(set.contains("BAZ").unwrap());
        assert!(!set.contains("qux").unwrap());
        assert!(set.contains("xyz").unwrap());
    }

    {
        let set = SQLiteSet::<String>::try_from_file_nocase(&file_path_2, false, false).unwrap();
        assert!(!set.contains("foo").unwrap());
        assert!(set.contains("bar").unwrap());
        assert!(set.contains("BAR").unwrap());
        assert!(!set.contains("baz").unwrap());
        assert!(set.contains("qux").unwrap());
        assert!(set.contains("QUX").unwrap());
        assert!(!set.contains("xyz").unwrap());
    }

    assert!(matches!(SQLiteSet::<String>::try_from_file_nocase(&file_path_1, false, false), Err(Error::InvalidSchema)));
    assert!(matches!(SQLiteSet::<String>::try_from_file(&file_path_2, false, false), Err(Error::InvalidSchema)));
}

#[cfg(feature = "checked")]
#[test]
fn test_sqlite_set_filebased_integrity() {
    use std::{
        fs::OpenOptions,
        io::{Read, Seek, SeekFrom, Write},
    };

    let temp_dir = tempdir_in(env!["CARGO_TARGET_TMPDIR"]).unwrap();
    let file_path = temp_dir.path().join("sqlite_set.db");

    assert!(!fs::exists(&file_path).unwrap());

    {
        let mut set = SQLiteSet::<String>::try_from_file(&file_path, false, true).unwrap();
        {
            let mut tx = set.transaction().unwrap();
            assert!(tx.insert("foo".to_owned()).unwrap());
            assert!(tx.insert("Foo".to_owned()).unwrap());
            assert!(tx.insert("bar".to_owned()).unwrap());
            assert!(tx.insert("BAR".to_owned()).unwrap());
            assert!(tx.insert("baz".to_owned()).unwrap());
            assert!(tx.insert("BAZ".to_owned()).unwrap());
        }
    }

    {
        const FILE_OFFSET: u64 = 4097u64;
        let mut byte = [0u8; 1usize];
        let mut file = OpenOptions::new().read(true).write(true).open(&file_path).unwrap();
        file.seek(SeekFrom::Start(FILE_OFFSET)).unwrap();
        file.read_exact(&mut byte).unwrap();
        byte[0] ^= 0xFFu8;
        file.seek(SeekFrom::Start(FILE_OFFSET)).unwrap();
        file.write_all(&byte).unwrap();
        file.sync_all().unwrap();
    }

    assert!(matches!(SQLiteSet::<String>::try_from_file(&file_path, false, false), Err(Error::IntegrityCheck)));
}

#[cfg(feature = "backup")]
#[test]
fn test_sqlite_set_filebased_tofile() {
    let temp_dir = tempdir_in(env!["CARGO_TARGET_TMPDIR"]).unwrap();
    let file_path = temp_dir.path().join("sqlite_set.db");

    assert!(!fs::exists(&file_path).unwrap());

    {
        let mut set = SQLiteSet::<String>::new().unwrap();
        {
            let mut tx = set.transaction().unwrap();
            assert!(tx.insert("foo".to_owned()).unwrap());
            assert!(tx.insert("FOO".to_owned()).unwrap());
            assert!(tx.insert("bar".to_owned()).unwrap());
            assert!(tx.insert("BAR".to_owned()).unwrap());
            assert!(tx.insert("baz".to_owned()).unwrap());
            assert!(tx.insert("BAZ".to_owned()).unwrap());
        }
        set.to_file(&file_path).unwrap();
    }

    {
        let set = SQLiteSet::<String>::try_from_file(&file_path, false, false).unwrap();
        assert!(set.contains("foo").unwrap());
        assert!(set.contains("FOO").unwrap());
        assert!(set.contains("bar").unwrap());
        assert!(set.contains("BAR").unwrap());
        assert!(set.contains("baz").unwrap());
        assert!(set.contains("BAZ").unwrap());
        assert!(!set.contains("qux").unwrap());
        assert!(!set.contains("xyz").unwrap());
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
            assert!(tx.insert_ref(hex.encode(mix64(i))).unwrap());
        }
    }

    thread_set
}

#[test]
fn test_sqlite_set_multithreading() {
    const COUNT: u64 = 125_000u64;
    const THREADS: usize = 8usize;

    let sets: [SQLiteSet<String>; THREADS] = array::from_fn(|_| SQLiteSet::new().unwrap());
    let barrier = Arc::new(Barrier::new(THREADS));
    let mut handles: Vec<_> = sets
        .into_iter()
        .map(|set| {
            let barrier = Arc::clone(&barrier);
            thread::spawn(move || sqlite_set_thread_worker::<COUNT>(set, &barrier))
        })
        .collect();

    for handle in handles.drain(..) {
        let thread_set = handle.join().unwrap();
        assert_eq!(thread_set.len().unwrap().to_u64(), COUNT);
    }
}

// ---------------------------------------------------------------------------
// SQLiteMap ToSet Test
// ---------------------------------------------------------------------------

#[test]
fn test_sqlite_set_to_map_i64() {
    let mut set = SQLiteSet::<i64>::new().unwrap();

    {
        let mut tx = set.transaction().unwrap();
        assert!(tx.insert(0i64).unwrap());
        assert!(tx.insert(42i64).unwrap());
        assert!(tx.insert(2147483647i64).unwrap());
        assert!(tx.insert(i64::MIN).unwrap());
        assert!(tx.insert(i64::MAX).unwrap());
    }

    let map = set.to_map(|key| if *key != 2147483647i64 { Some(mix64(*key as u64).to_string()) } else { None }).unwrap();

    assert_eq!(map.len().unwrap(), 4);
    assert!(map.get(&0i64).unwrap().is_some_and(|value| value.eq("0")));
    assert!(map.get(&42i64).unwrap().is_some_and(|value| value.eq("12058926934050108962")));
    assert!(map.get(&2147483647i64).unwrap().is_none());
    assert!(map.get(&i64::MIN).unwrap().is_some_and(|value| value.eq("2720858781877447050")));
    assert!(map.get(&i64::MAX).unwrap().is_some_and(|value| value.eq("6514504133438201533")));
}

#[test]
fn test_sqlite_set_to_map_string() {
    #[inline]
    fn reverse_str(str: &str) -> String {
        str.chars().rev().collect()
    }

    let mut set = SQLiteSet::<String>::new().unwrap();

    {
        let mut tx = set.transaction().unwrap();
        assert!(tx.insert("foo".to_owned()).unwrap());
        assert!(tx.insert("bar".to_owned()).unwrap());
        assert!(tx.insert("qux".to_owned()).unwrap());
        assert!(tx.insert("FOO".to_owned()).unwrap());
        assert!(tx.insert("BAR".to_owned()).unwrap());
        assert!(tx.insert("QUX".to_owned()).unwrap());
    }

    let map_1 = set.to_map(|key| if !key.eq_ignore_ascii_case("qux") { Some(format!("~{}~", reverse_str(key))) } else { None }).unwrap();
    let map_2 = set.to_map_nocase(|key| if !key.eq_ignore_ascii_case("qux") { Some(format!("~{}~", reverse_str(key))) } else { None }).unwrap();

    assert_eq!(map_1.len().unwrap(), 4);
    assert!(map_1.get("foo").unwrap().is_some_and(|value| value.eq("~oof~")));
    assert!(map_1.get("FOO").unwrap().is_some_and(|value| value.eq("~OOF~")));
    assert!(map_1.get("bar").unwrap().is_some_and(|value| value.eq("~rab~")));
    assert!(map_1.get("BAR").unwrap().is_some_and(|value| value.eq("~RAB~")));
    assert!(map_1.get("qux").unwrap().is_none());
    assert!(map_1.get("QUX").unwrap().is_none());

    assert_eq!(map_2.len().unwrap(), 2);
    assert!(map_2.get("foo").unwrap().is_some_and(|value| value.eq("~oof~") || value.eq("~OOF~")));
    assert!(map_2.get("FOO").unwrap().is_some_and(|value| value.eq("~oof~") || value.eq("~OOF~")));
    assert!(map_2.get("bar").unwrap().is_some_and(|value| value.eq("~rab~") || value.eq("~RAB~")));
    assert!(map_2.get("BAR").unwrap().is_some_and(|value| value.eq("~rab~") || value.eq("~RAB~")));
    assert!(map_2.get("qux").unwrap().is_none());
    assert!(map_2.get("QUX").unwrap().is_none());
}

// ---------------------------------------------------------------------------
// SQLiteSet Clone Test
// ---------------------------------------------------------------------------

cfg_if! {
    if #[cfg(feature = "backup")] {
        #[test]
        fn test_sqlite_set_clone() {
            let mut set = SQLiteSet::<String>::new().unwrap();

            {
                let mut tx = set.transaction().unwrap();
                assert!(tx.insert("foo".to_owned()).unwrap());
                assert!(tx.insert("baz".to_owned()).unwrap());
                assert!(tx.insert("BAZ".to_owned()).unwrap());
            }

            let mut set_clone = set.try_clone().unwrap();

            {
                let mut tx = set.transaction().unwrap();
                assert!(tx.insert("bar".to_owned()).unwrap());
                assert!(tx.insert("QUX".to_owned()).unwrap());
            }

            {
                let mut tx = set_clone.transaction().unwrap();
                assert!(tx.insert("BAR".to_owned()).unwrap());
                assert!(tx.insert("qux".to_owned()).unwrap());
            }

            assert!(set.contains("foo").unwrap());
            assert!(set.contains("baz").unwrap());
            assert!(set.contains("BAZ").unwrap());
            assert!(set.contains("bar").unwrap());
            assert!(!set.contains("BAR").unwrap());
            assert!(set.contains("QUX").unwrap());
            assert!(!set.contains("qux").unwrap());

            assert!(set_clone.contains("foo").unwrap());
            assert!(set_clone.contains("baz").unwrap());
            assert!(set_clone.contains("BAZ").unwrap());
            assert!(!set_clone.contains("bar").unwrap());
            assert!(set_clone.contains("BAR").unwrap());
            assert!(!set_clone.contains("QUX").unwrap());
            assert!(set_clone.contains("qux").unwrap());
        }
    } else {
        #[test]
        fn test_sqlite_set_clone() {
            let mut set = SQLiteSet::<String>::new().unwrap();

            {
                let mut tx = set.transaction().unwrap();
                assert!(tx.insert("foo".to_owned()).unwrap());
                assert!(tx.insert("baz".to_owned()).unwrap());
                assert!(tx.insert("BAZ".to_owned()).unwrap());
            }

            assert!(matches!(set.try_clone(), Err(Error::Unsupported)));
        }
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
        match tx.insert(key) {
            Err(error) => match error.into_inner() {
                Some(sqlite_containers::rusqlite::Error::SqliteFailure(sqlite_error, _)) => match sqlite_error.code {
                    sqlite_containers::rusqlite::ErrorCode::OutOfMemory => break,
                    sqlite_containers::rusqlite::ErrorCode::DiskFull => break,
                    _ => panic!("Unexpected SQLite failure has been encountered! [{sqlite_error}]"),
                },
                _ => panic!("Unexpected SQLiteSet error has been encountered!"),
            },
            Ok(success) => assert!(success),
        }
    }
}
