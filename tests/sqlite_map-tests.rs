// SQLiteMap Tests
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
use sqlite_containers::{Action, Blob, Error, SQLiteMap, SizeExt};
use std::{
    array,
    collections::{HashMap, HashSet},
    fs,
    sync::{Arc, Barrier, mpsc::sync_channel},
    thread,
};
use tempfile::tempdir_in;

// ---------------------------------------------------------------------------
// SQLiteMap<i64, String> Tests
// ---------------------------------------------------------------------------

#[test]
fn test_sqlite_map_i64_string() {
    let mut map = SQLiteMap::<i64, String>::new().unwrap();

    assert_eq!(map.len().unwrap(), 0);
    assert!(map.is_empty().unwrap());
    assert!(!map.contains_key(&0i64).unwrap());
    assert!(!map.contains_key(&42i64).unwrap());
    assert!(!map.contains_key(&i64::MAX).unwrap());
    assert!(!map.contains_key(&i64::MIN).unwrap());
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
    assert!(map.contains_key(&0i64).unwrap());
    assert!(map.contains_key(&42i64).unwrap());
    assert!(map.contains_key(&i64::MAX).unwrap());
    assert!(!map.contains_key(&i64::MIN).unwrap());
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
    assert!(!map.contains_key(&0i64).unwrap());
    assert!(!map.contains_key(&42i64).unwrap());
    assert!(map.contains_key(&i64::MAX).unwrap());
    assert!(!map.contains_key(&i64::MIN).unwrap());
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
    assert!(map.contains_key(&0i64).unwrap());
    assert!(!map.contains_key(&42i64).unwrap());
    assert!(map.contains_key(&i64::MAX).unwrap());
    assert!(map.contains_key(&i64::MIN).unwrap());
    assert_eq!(map.get(&0i64).unwrap().unwrap(), "12345678");
    assert!(map.get(&42i64).unwrap().is_none());
    assert_eq!(map.get(&i64::MAX).unwrap().unwrap(), "deadbeef");
    assert_eq!(map.get(&i64::MIN).unwrap().unwrap(), "cafebabe");

    {
        let mut hash = HashSet::new();
        map.for_each_key(|key| assert!(hash.insert(key.to_owned()))).unwrap();
        assert_eq!(hash.len(), 3);
        assert!(hash.contains(&0i64));
        assert!(hash.contains(&i64::MAX));
        assert!(hash.contains(&i64::MIN));
    }

    {
        let mut hash = HashMap::new();
        map.for_each_entry(|key, value| assert!(hash.insert(key.to_owned(), value.to_owned()).is_none())).unwrap();
        assert_eq!(hash.len(), 3);
        assert_eq!(hash.get(&0i64).unwrap(), "12345678");
        assert_eq!(hash.get(&i64::MAX).unwrap(), "deadbeef");
        assert_eq!(hash.get(&i64::MIN).unwrap(), "cafebabe");
    }

    assert!(map.find_key(|key| key.eq(&0i64)).unwrap().is_some_and(|key| key == 0i64));
    assert!(map.find_key(|key| key.eq(&42i64)).unwrap().is_none());
    assert!(map.find_key(|key| key.eq(&i64::MAX)).unwrap().is_some_and(|key| key == i64::MAX));
    assert!(map.find_key(|key| key.eq(&i64::MIN)).unwrap().is_some_and(|key| key == i64::MIN));

    assert_eq!(map.find_entry(|key, _| key.eq(&0i64)).unwrap().unwrap().1, "12345678");
    assert!(map.find_entry(|key, _| key.eq(&42i64)).unwrap().is_none());
    assert_eq!(map.find_entry(|key, _| key.eq(&i64::MAX)).unwrap().unwrap().1, "deadbeef");
    assert_eq!(map.find_entry(|key, _| key.eq(&i64::MIN)).unwrap().unwrap().1, "cafebabe");

    map.update(&0i64, "abcdefgh").unwrap();
    map.update(&42i64, "hgfedcba").unwrap();
    map.update(&i64::MAX, "stuvwxyz").unwrap();

    assert_eq!(map.get(&0i64).unwrap().unwrap(), "abcdefgh");
    assert_eq!(map.get(&42i64).unwrap().unwrap(), "hgfedcba");
    assert_eq!(map.get(&i64::MAX).unwrap().unwrap(), "stuvwxyz");
    assert_eq!(map.get(&i64::MIN).unwrap().unwrap(), "cafebabe");

    {
        let mut hash = HashSet::new();
        map.for_each_key(|key| assert!(hash.insert(key.to_owned()))).unwrap();
        assert_eq!(hash.len(), 4);
        assert!(hash.contains(&0i64));
        assert!(hash.contains(&42i64));
        assert!(hash.contains(&i64::MAX));
        assert!(hash.contains(&i64::MIN));
    }

    {
        let mut hash = HashMap::new();
        map.for_each_entry(|key, value| assert!(hash.insert(key.to_owned(), value.to_owned()).is_none())).unwrap();
        assert_eq!(hash.len(), 4);
        assert_eq!(hash.get(&0i64).unwrap(), "abcdefgh");
        assert_eq!(hash.get(&42i64).unwrap(), "hgfedcba");
        assert_eq!(hash.get(&i64::MAX).unwrap(), "stuvwxyz");
        assert_eq!(hash.get(&i64::MIN).unwrap(), "cafebabe");
    }

    assert!(map.find_key(|key| key.eq(&0i64)).unwrap().is_some_and(|key| key == 0i64));
    assert!(map.find_key(|key| key.eq(&42i64)).unwrap().is_some_and(|key| key == 42i64));
    assert!(map.find_key(|key| key.eq(&i64::MAX)).unwrap().is_some_and(|key| key == i64::MAX));
    assert!(map.find_key(|key| key.eq(&i64::MIN)).unwrap().is_some_and(|key| key == i64::MIN));

    assert_eq!(map.find_entry(|key, _| key.eq(&0i64)).unwrap().unwrap().1, "abcdefgh");
    assert_eq!(map.find_entry(|key, _| key.eq(&42i64)).unwrap().unwrap().1, "hgfedcba");
    assert_eq!(map.find_entry(|key, _| key.eq(&i64::MAX)).unwrap().unwrap().1, "stuvwxyz");
    assert_eq!(map.find_entry(|key, _| key.eq(&i64::MIN)).unwrap().unwrap().1, "cafebabe");

    map.clear().unwrap();

    assert_eq!(map.len().unwrap(), 0);
    assert!(map.is_empty().unwrap());
    assert!(!map.contains_key(&0i64).unwrap());
    assert!(!map.contains_key(&42i64).unwrap());
    assert!(!map.contains_key(&i64::MAX).unwrap());
    assert!(!map.contains_key(&i64::MIN).unwrap());
}

#[test]
fn test_sqlite_map_i64_string_transacted() {
    let mut map = SQLiteMap::<i64, String>::new().unwrap();
    let mut tx = map.transaction().unwrap();

    assert_eq!(tx.len().unwrap(), 0);
    assert!(tx.is_empty().unwrap());
    assert!(!tx.contains_key(&0i64).unwrap());
    assert!(!tx.contains_key(&42i64).unwrap());
    assert!(!tx.contains_key(&i64::MAX).unwrap());
    assert!(!tx.contains_key(&i64::MIN).unwrap());
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
    assert!(tx.contains_key(&0i64).unwrap());
    assert!(tx.contains_key(&42i64).unwrap());
    assert!(tx.contains_key(&i64::MAX).unwrap());
    assert!(!tx.contains_key(&i64::MIN).unwrap());
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
    assert!(!tx.contains_key(&0i64).unwrap());
    assert!(!tx.contains_key(&42i64).unwrap());
    assert!(tx.contains_key(&i64::MAX).unwrap());
    assert!(!tx.contains_key(&i64::MIN).unwrap());
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
    assert!(tx.contains_key(&0i64).unwrap());
    assert!(!tx.contains_key(&42i64).unwrap());
    assert!(tx.contains_key(&i64::MAX).unwrap());
    assert!(tx.contains_key(&i64::MIN).unwrap());
    assert_eq!(tx.get(&0i64).unwrap().unwrap(), "12345678");
    assert!(tx.get(&42i64).unwrap().is_none());
    assert_eq!(tx.get(&i64::MAX).unwrap().unwrap(), "deadbeef");
    assert_eq!(tx.get(&i64::MIN).unwrap().unwrap(), "cafebabe");

    {
        let mut hash = HashSet::new();
        tx.for_each_key(|key| assert!(hash.insert(key.to_owned()))).unwrap();
        assert_eq!(hash.len(), 3);
        assert!(hash.contains(&0i64));
        assert!(hash.contains(&i64::MAX));
        assert!(hash.contains(&i64::MIN));
    }

    {
        let mut hash = HashMap::new();
        tx.for_each_entry(|key, value| assert!(hash.insert(key.to_owned(), value.to_owned()).is_none())).unwrap();
        assert_eq!(hash.len(), 3);
        assert_eq!(hash.get(&0i64).unwrap(), "12345678");
        assert_eq!(hash.get(&i64::MAX).unwrap(), "deadbeef");
        assert_eq!(hash.get(&i64::MIN).unwrap(), "cafebabe");
    }

    assert!(tx.find_key(|key| key.eq(&0i64)).unwrap().is_some_and(|key| key == 0i64));
    assert!(tx.find_key(|key| key.eq(&42i64)).unwrap().is_none());
    assert!(tx.find_key(|key| key.eq(&i64::MAX)).unwrap().is_some_and(|key| key == i64::MAX));
    assert!(tx.find_key(|key| key.eq(&i64::MIN)).unwrap().is_some_and(|key| key == i64::MIN));

    assert_eq!(tx.find_entry(|key, _| key.eq(&0i64)).unwrap().unwrap().1, "12345678");
    assert!(tx.find_entry(|key, _| key.eq(&42i64)).unwrap().is_none());
    assert_eq!(tx.find_entry(|key, _| key.eq(&i64::MAX)).unwrap().unwrap().1, "deadbeef");
    assert_eq!(tx.find_entry(|key, _| key.eq(&i64::MIN)).unwrap().unwrap().1, "cafebabe");

    tx.update(&0i64, "abcdefgh").unwrap();
    tx.update(&42i64, "hgfedcba").unwrap();
    tx.update(&i64::MAX, "stuvwxyz").unwrap();

    assert_eq!(tx.get(&0i64).unwrap().unwrap(), "abcdefgh");
    assert_eq!(tx.get(&42i64).unwrap().unwrap(), "hgfedcba");
    assert_eq!(tx.get(&i64::MAX).unwrap().unwrap(), "stuvwxyz");
    assert_eq!(tx.get(&i64::MIN).unwrap().unwrap(), "cafebabe");

    {
        let mut hash = HashSet::new();
        tx.for_each_key(|key| assert!(hash.insert(key.to_owned()))).unwrap();
        assert_eq!(hash.len(), 4);
        assert!(hash.contains(&0i64));
        assert!(hash.contains(&42i64));
        assert!(hash.contains(&i64::MAX));
        assert!(hash.contains(&i64::MIN));
    }

    {
        let mut hash = HashMap::new();
        tx.for_each_entry(|key, value| assert!(hash.insert(key.to_owned(), value.to_owned()).is_none())).unwrap();
        assert_eq!(hash.len(), 4);
        assert_eq!(hash.get(&0i64).unwrap(), "abcdefgh");
        assert_eq!(hash.get(&42i64).unwrap(), "hgfedcba");
        assert_eq!(hash.get(&i64::MAX).unwrap(), "stuvwxyz");
        assert_eq!(hash.get(&i64::MIN).unwrap(), "cafebabe");
    }

    assert!(tx.find_key(|key| key.eq(&0i64)).unwrap().is_some_and(|key| key == 0i64));
    assert!(tx.find_key(|key| key.eq(&42i64)).unwrap().is_some_and(|key| key == 42i64));
    assert!(tx.find_key(|key| key.eq(&i64::MAX)).unwrap().is_some_and(|key| key == i64::MAX));
    assert!(tx.find_key(|key| key.eq(&i64::MIN)).unwrap().is_some_and(|key| key == i64::MIN));

    assert_eq!(tx.find_entry(|key, _| key.eq(&0i64)).unwrap().unwrap().1, "abcdefgh");
    assert_eq!(tx.find_entry(|key, _| key.eq(&42i64)).unwrap().unwrap().1, "hgfedcba");
    assert_eq!(tx.find_entry(|key, _| key.eq(&i64::MAX)).unwrap().unwrap().1, "stuvwxyz");
    assert_eq!(tx.find_entry(|key, _| key.eq(&i64::MIN)).unwrap().unwrap().1, "cafebabe");

    tx.clear().unwrap();

    assert_eq!(tx.len().unwrap(), 0);
    assert!(tx.is_empty().unwrap());
    assert!(!tx.contains_key(&0i64).unwrap());
    assert!(!tx.contains_key(&42i64).unwrap());
    assert!(!tx.contains_key(&i64::MAX).unwrap());
    assert!(!tx.contains_key(&i64::MIN).unwrap());

    assert!(tx.insert(&0i64, "ijklmnop").unwrap());
    assert!(tx.insert(&42i64, "ponmlkji").unwrap());

    tx.commit();

    assert_eq!(map.len().unwrap(), 2);
    assert!(!map.is_empty().unwrap());
    assert!(map.contains_key(&0i64).unwrap());
    assert!(map.contains_key(&42i64).unwrap());
    assert!(!map.contains_key(&i64::MAX).unwrap());
    assert!(!map.contains_key(&i64::MIN).unwrap());
    assert_eq!(map.get(&0i64).unwrap().unwrap(), "ijklmnop");
    assert_eq!(map.get(&42i64).unwrap().unwrap(), "ponmlkji");
    assert!(map.get(&i64::MAX).unwrap().is_none());
    assert!(map.get(&i64::MIN).unwrap().is_none());
}

#[test]
fn test_sqlite_map_i64_string_stresstest() {
    let mut set = SQLiteMap::<i64, String>::new().unwrap();
    let mut hex = HexEncoder::default();

    {
        let mut tx = set.transaction().unwrap();
        for i in 0u64..MAX_ITEMS {
            let (key, value) = (mix64(i) as i64, hex.encode(mix64(!i)));
            assert!(tx.insert(&key, value).unwrap());
        }
    }

    assert!(!set.is_empty().unwrap());
    assert_eq!(set.len().unwrap().to_u64(), MAX_ITEMS);

    {
        let tx = set.transaction().unwrap();
        for i in 0u64..MAX_ITEMS {
            let (key, value) = (mix64(i) as i64, hex.encode(mix64(!i)));
            assert_eq!(tx.get(&key).unwrap().unwrap(), value);
        }
    }

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
fn test_sqlite_map_i64_string_randomized() {
    const COUNT: u64 = 125_000u64;
    let mut set = SQLiteMap::<i64, String>::new().unwrap();
    let mut hex = HexEncoder::default();

    for seed in [0x9DDC5F3146219B97u64, 0x79B9E4DC002B32D6u64] {
        for b in [true, false] {
            let mut xoshiro = Xoshiro256PlusPlus::seed_from_u64(seed);
            let mut tx = set.transaction().unwrap();
            for _i in 0u64..COUNT {
                assert_eq!(tx.insert(&(xoshiro.next_u64() as i64), hex.encode(xoshiro.next_u64())).unwrap(), b);
            }
        }
    }

    for (seed, b) in [(0x9DDC5F3146219B97u64, true), (0x79B9E4DC002B32D6u64, true), (0x2F381F8B2F0E517Au64, false)] {
        let mut xoshiro = Xoshiro256PlusPlus::seed_from_u64(seed);
        let tx = set.transaction().unwrap();
        for _i in 0u64..COUNT {
            if b {
                assert_eq!(tx.get(&(xoshiro.next_u64() as i64)).unwrap().unwrap(), hex.encode(xoshiro.next_u64()));
            } else {
                assert!(tx.get(&(xoshiro.next_u64() as i64)).unwrap().is_none());
                let _skip = xoshiro.next_u64();
            }
        }
    }
}

#[cfg(not(debug_assertions))]
#[test]
fn test_sqlite_map_i64_string_words() {
    use xxhash_rust::xxh64::xxh64;
    for lang in [random_word::Lang::De, random_word::Lang::En] {
        let word_list = random_word::all(lang);
        for seed_1 in 0u64..5u64 {
            let mut map = SQLiteMap::<i64, String>::new().unwrap();
            {
                let mut tx = map.transaction().unwrap();
                for &word in word_list {
                    let hash = xxh64(word.as_bytes(), seed_1) as i64;
                    assert!(tx.insert(&hash, word).unwrap());
                }
            }
            for seed_2 in 0u64..5u64 {
                let tx = map.transaction().unwrap();
                if seed_1 == seed_2 {
                    for &word in word_list {
                        let hash = xxh64(word.as_bytes(), seed_2) as i64;
                        assert!(tx.contains_key(&hash).unwrap());
                        assert_eq!(tx.get(&hash).unwrap().unwrap(), word);
                    }
                } else {
                    for &word in word_list {
                        let hash = xxh64(word.as_bytes(), seed_2) as i64;
                        assert!(!tx.contains_key(&hash).unwrap());
                        assert!(tx.get(&hash).unwrap().is_none());
                    }
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// SQLiteMap<Blob, String> Tests
// ---------------------------------------------------------------------------

#[test]
fn test_sqlite_map_blob_string() {
    let mut map = SQLiteMap::<Blob, String>::new().unwrap();

    assert_eq!(map.len().unwrap(), 0);
    assert!(map.is_empty().unwrap());
    assert!(!map.contains_key(&b"foo"[..]).unwrap());
    assert!(!map.contains_key(&b"bar"[..]).unwrap());
    assert!(!map.contains_key(&b"baz"[..]).unwrap());
    assert!(!map.contains_key(&b"qux"[..]).unwrap());
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
    assert!(map.contains_key(&b"foo"[..]).unwrap());
    assert!(!map.contains_key(&b"FOO"[..]).unwrap());
    assert!(map.contains_key(&b"bar"[..]).unwrap());
    assert!(!map.contains_key(&b"BAR"[..]).unwrap());
    assert!(map.contains_key(&b"baz"[..]).unwrap());
    assert!(!map.contains_key(&b"BAZ"[..]).unwrap());
    assert!(!map.contains_key(&b"qux"[..]).unwrap());
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
    assert!(!map.contains_key(&b"foo"[..]).unwrap());
    assert!(!map.contains_key(&b"bar"[..]).unwrap());
    assert!(map.contains_key(&b"baz"[..]).unwrap());
    assert!(!map.contains_key(&b"BAZ"[..]).unwrap());
    assert!(!map.contains_key(&b"qux"[..]).unwrap());
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
    assert!(map.contains_key(&b"foo"[..]).unwrap());
    assert!(!map.contains_key(&b"FOO"[..]).unwrap());
    assert!(!map.contains_key(&b"bar"[..]).unwrap());
    assert!(map.contains_key(&b"baz"[..]).unwrap());
    assert!(!map.contains_key(&b"BAZ"[..]).unwrap());
    assert!(map.contains_key(&b"qux"[..]).unwrap());
    assert!(!map.contains_key(&b"QUX"[..]).unwrap());
    assert_eq!(map.get(&b"foo"[..]).unwrap().unwrap(), "12345678");
    assert!(map.get(&b"FOO"[..]).unwrap().is_none());
    assert!(map.get(&b"bar"[..]).unwrap().is_none());
    assert_eq!(map.get(&b"baz"[..]).unwrap().unwrap(), "deadbeef");
    assert!(map.get(&b"BAZ"[..]).unwrap().is_none());
    assert_eq!(map.get(&b"qux"[..]).unwrap().unwrap(), "cafebabe");
    assert!(map.get(&b"QUX"[..]).unwrap().is_none());

    {
        let mut hash = HashSet::new();
        map.for_each_key(|key| assert!(hash.insert(key.to_owned()))).unwrap();
        assert_eq!(hash.len(), 3);
        assert!(hash.contains(&b"foo"[..]));
        assert!(hash.contains(&b"baz"[..]));
        assert!(hash.contains(&b"qux"[..]));
    }

    {
        let mut hash = HashMap::new();
        map.for_each_entry(|key, value| assert!(hash.insert(key.to_owned(), value.to_owned()).is_none())).unwrap();
        assert_eq!(hash.len(), 3);
        assert_eq!(hash.get(&b"foo"[..]).unwrap(), "12345678");
        assert_eq!(hash.get(&b"baz"[..]).unwrap(), "deadbeef");
        assert_eq!(hash.get(&b"qux"[..]).unwrap(), "cafebabe");
    }

    assert!(map.find_key(|key| key.eq(b"foo")).unwrap().is_some_and(|key| key.eq(b"foo")));
    assert!(map.find_key(|key| key.eq(b"bar")).unwrap().is_none());
    assert!(map.find_key(|key| key.eq(b"baz")).unwrap().is_some_and(|key| key.eq(b"baz")));
    assert!(map.find_key(|key| key.eq(b"qux")).unwrap().is_some_and(|key| key.eq(b"qux")));

    assert_eq!(map.find_entry(|key, _| key.eq(b"foo")).unwrap().unwrap().1, "12345678");
    assert_eq!(map.find_entry(|key, _| key.eq_ignore_ascii_case(b"FOO")).unwrap().unwrap().1, "12345678");
    assert!(map.find_entry(|key, _| key.eq(b"bar")).unwrap().is_none());

    map.update(&b"foo"[..], "abcdefgh").unwrap();
    map.update(&b"bar"[..], "hgfedcba").unwrap();
    map.update(&b"baz"[..], "stuvwxyz").unwrap();

    assert_eq!(map.get(&b"foo"[..]).unwrap().unwrap(), "abcdefgh");
    assert_eq!(map.get(&b"bar"[..]).unwrap().unwrap(), "hgfedcba");
    assert_eq!(map.get(&b"baz"[..]).unwrap().unwrap(), "stuvwxyz");
    assert_eq!(map.get(&b"qux"[..]).unwrap().unwrap(), "cafebabe");

    {
        let mut hash = HashSet::new();
        map.for_each_key(|key| assert!(hash.insert(key.to_owned()))).unwrap();
        assert_eq!(hash.len(), 4);
        assert!(hash.contains(&b"foo"[..]));
        assert!(hash.contains(&b"bar"[..]));
        assert!(hash.contains(&b"baz"[..]));
        assert!(hash.contains(&b"qux"[..]));
    }

    {
        let mut hash = HashMap::new();
        map.for_each_entry(|key, value| assert!(hash.insert(key.to_owned(), value.to_owned()).is_none())).unwrap();
        assert_eq!(hash.len(), 4);
        assert_eq!(hash.get(&b"foo"[..]).unwrap(), "abcdefgh");
        assert_eq!(hash.get(&b"bar"[..]).unwrap(), "hgfedcba");
        assert_eq!(hash.get(&b"baz"[..]).unwrap(), "stuvwxyz");
        assert_eq!(hash.get(&b"qux"[..]).unwrap(), "cafebabe");
    }

    assert!(map.find_key(|key| key.eq(b"foo")).unwrap().is_some_and(|key| key.eq(b"foo")));
    assert!(map.find_key(|key| key.eq(b"bar")).unwrap().is_some_and(|key| key.eq(b"bar")));
    assert!(map.find_key(|key| key.eq(b"baz")).unwrap().is_some_and(|key| key.eq(b"baz")));
    assert!(map.find_key(|key| key.eq(b"qux")).unwrap().is_some_and(|key| key.eq(b"qux")));

    assert_eq!(map.find_entry(|key, _| key.eq(b"foo")).unwrap().unwrap().1, "abcdefgh");
    assert_eq!(map.find_entry(|key, _| key.eq_ignore_ascii_case(b"FOO")).unwrap().unwrap().1, "abcdefgh");
    assert_eq!(map.find_entry(|key, _| key.eq(b"bar")).unwrap().unwrap().1, "hgfedcba");
    assert_eq!(map.find_entry(|key, _| key.eq_ignore_ascii_case(b"BAR")).unwrap().unwrap().1, "hgfedcba");
    assert!(map.find_entry(|key, _| key.eq(b"xyz")).unwrap().is_none());

    map.clear().unwrap();

    assert_eq!(map.len().unwrap(), 0);
    assert!(map.is_empty().unwrap());
    assert!(!map.contains_key(&b"foo"[..]).unwrap());
    assert!(!map.contains_key(&b"bar"[..]).unwrap());
    assert!(!map.contains_key(&b"baz"[..]).unwrap());
    assert!(!map.contains_key(&b"qux"[..]).unwrap());
}

#[test]
fn test_sqlite_map_blob_string_transacted() {
    let mut map = SQLiteMap::<Blob, String>::new().unwrap();
    let mut tx = map.transaction().unwrap();

    assert_eq!(tx.len().unwrap(), 0);
    assert!(tx.is_empty().unwrap());
    assert!(!tx.contains_key(&b"foo"[..]).unwrap());
    assert!(!tx.contains_key(&b"bar"[..]).unwrap());
    assert!(!tx.contains_key(&b"baz"[..]).unwrap());
    assert!(!tx.contains_key(&b"qux"[..]).unwrap());
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
    assert!(tx.contains_key(&b"foo"[..]).unwrap());
    assert!(!tx.contains_key(&b"FOO"[..]).unwrap());
    assert!(tx.contains_key(&b"bar"[..]).unwrap());
    assert!(!tx.contains_key(&b"BAR"[..]).unwrap());
    assert!(tx.contains_key(&b"baz"[..]).unwrap());
    assert!(!tx.contains_key(&b"BAZ"[..]).unwrap());
    assert!(!tx.contains_key(&b"qux"[..]).unwrap());
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
    assert!(!tx.contains_key(&b"foo"[..]).unwrap());
    assert!(!tx.contains_key(&b"bar"[..]).unwrap());
    assert!(tx.contains_key(&b"baz"[..]).unwrap());
    assert!(!tx.contains_key(&b"BAZ"[..]).unwrap());
    assert!(!tx.contains_key(&b"qux"[..]).unwrap());
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
    assert!(tx.contains_key(&b"foo"[..]).unwrap());
    assert!(!tx.contains_key(&b"FOO"[..]).unwrap());
    assert!(!tx.contains_key(&b"bar"[..]).unwrap());
    assert!(tx.contains_key(&b"baz"[..]).unwrap());
    assert!(!tx.contains_key(&b"BAZ"[..]).unwrap());
    assert!(tx.contains_key(&b"qux"[..]).unwrap());
    assert!(!tx.contains_key(&b"QUX"[..]).unwrap());
    assert_eq!(tx.get(&b"foo"[..]).unwrap().unwrap(), "12345678");
    assert!(tx.get(&b"FOO"[..]).unwrap().is_none());
    assert!(tx.get(&b"bar"[..]).unwrap().is_none());
    assert_eq!(tx.get(&b"baz"[..]).unwrap().unwrap(), "deadbeef");
    assert!(tx.get(&b"BAZ"[..]).unwrap().is_none());
    assert_eq!(tx.get(&b"qux"[..]).unwrap().unwrap(), "cafebabe");
    assert!(tx.get(&b"QUX"[..]).unwrap().is_none());

    {
        let mut hash = HashSet::new();
        tx.for_each_key(|key| assert!(hash.insert(key.to_owned()))).unwrap();
        assert_eq!(hash.len(), 3);
        assert!(hash.contains(&b"foo"[..]));
        assert!(hash.contains(&b"baz"[..]));
        assert!(hash.contains(&b"qux"[..]));
    }

    {
        let mut hash = HashMap::new();
        tx.for_each_entry(|key, value| assert!(hash.insert(key.to_owned(), value.to_owned()).is_none())).unwrap();
        assert_eq!(hash.len(), 3);
        assert_eq!(hash.get(&b"foo"[..]).unwrap(), "12345678");
        assert_eq!(hash.get(&b"baz"[..]).unwrap(), "deadbeef");
        assert_eq!(hash.get(&b"qux"[..]).unwrap(), "cafebabe");
    }

    assert!(tx.find_key(|key| key.eq(b"foo")).unwrap().is_some_and(|key| key.eq(b"foo")));
    assert!(tx.find_key(|key| key.eq(b"bar")).unwrap().is_none());
    assert!(tx.find_key(|key| key.eq(b"baz")).unwrap().is_some_and(|key| key.eq(b"baz")));
    assert!(tx.find_key(|key| key.eq(b"qux")).unwrap().is_some_and(|key| key.eq(b"qux")));

    assert_eq!(tx.find_entry(|key, _| key.eq(b"foo")).unwrap().unwrap().1, "12345678");
    assert_eq!(tx.find_entry(|key, _| key.eq_ignore_ascii_case(b"FOO")).unwrap().unwrap().1, "12345678");
    assert!(tx.find_entry(|key, _| key.eq(b"bar")).unwrap().is_none());

    tx.update(&b"foo"[..], "abcdefgh").unwrap();
    tx.update(&b"bar"[..], "hgfedcba").unwrap();
    tx.update(&b"baz"[..], "stuvwxyz").unwrap();

    assert_eq!(tx.get(&b"foo"[..]).unwrap().unwrap(), "abcdefgh");
    assert_eq!(tx.get(&b"bar"[..]).unwrap().unwrap(), "hgfedcba");
    assert_eq!(tx.get(&b"baz"[..]).unwrap().unwrap(), "stuvwxyz");
    assert_eq!(tx.get(&b"qux"[..]).unwrap().unwrap(), "cafebabe");

    {
        let mut hash = HashSet::new();
        tx.for_each_key(|key| assert!(hash.insert(key.to_owned()))).unwrap();
        assert_eq!(hash.len(), 4);
        assert!(hash.contains(&b"foo"[..]));
        assert!(hash.contains(&b"bar"[..]));
        assert!(hash.contains(&b"baz"[..]));
        assert!(hash.contains(&b"qux"[..]));
    }

    {
        let mut hash = HashMap::new();
        tx.for_each_entry(|key, value| assert!(hash.insert(key.to_owned(), value.to_owned()).is_none())).unwrap();
        assert_eq!(hash.len(), 4);
        assert_eq!(hash.get(&b"foo"[..]).unwrap(), "abcdefgh");
        assert_eq!(hash.get(&b"bar"[..]).unwrap(), "hgfedcba");
        assert_eq!(hash.get(&b"baz"[..]).unwrap(), "stuvwxyz");
        assert_eq!(hash.get(&b"qux"[..]).unwrap(), "cafebabe");
    }

    assert!(tx.find_key(|key| key.eq(b"foo")).unwrap().is_some_and(|key| key.eq(b"foo")));
    assert!(tx.find_key(|key| key.eq(b"bar")).unwrap().is_some_and(|key| key.eq(b"bar")));
    assert!(tx.find_key(|key| key.eq(b"baz")).unwrap().is_some_and(|key| key.eq(b"baz")));
    assert!(tx.find_key(|key| key.eq(b"qux")).unwrap().is_some_and(|key| key.eq(b"qux")));

    assert_eq!(tx.find_entry(|key, _| key.eq(b"foo")).unwrap().unwrap().1, "abcdefgh");
    assert_eq!(tx.find_entry(|key, _| key.eq_ignore_ascii_case(b"FOO")).unwrap().unwrap().1, "abcdefgh");
    assert_eq!(tx.find_entry(|key, _| key.eq(b"bar")).unwrap().unwrap().1, "hgfedcba");
    assert_eq!(tx.find_entry(|key, _| key.eq_ignore_ascii_case(b"BAR")).unwrap().unwrap().1, "hgfedcba");
    assert!(tx.find_entry(|key, _| key.eq(b"xyz")).unwrap().is_none());

    tx.clear().unwrap();

    assert_eq!(tx.len().unwrap(), 0);
    assert!(tx.is_empty().unwrap());
    assert!(!tx.contains_key(&b"foo"[..]).unwrap());
    assert!(!tx.contains_key(&b"bar"[..]).unwrap());
    assert!(!tx.contains_key(&b"baz"[..]).unwrap());
    assert!(!tx.contains_key(&b"qux"[..]).unwrap());

    assert!(tx.insert(&b"foo"[..], "ijklmnop").unwrap());
    assert!(tx.insert(&b"bar"[..], "ponmlkji").unwrap());

    tx.commit();

    assert_eq!(map.len().unwrap(), 2);
    assert!(!map.is_empty().unwrap());
    assert!(map.contains_key(&b"foo"[..]).unwrap());
    assert!(map.contains_key(&b"bar"[..]).unwrap());
    assert!(!map.contains_key(&b"baz"[..]).unwrap());
    assert!(!map.contains_key(&b"qux"[..]).unwrap());
    assert_eq!(map.get(&b"foo"[..]).unwrap().unwrap(), "ijklmnop");
    assert_eq!(map.get(&b"bar"[..]).unwrap().unwrap(), "ponmlkji");
    assert!(map.get(&b"baz"[..]).unwrap().is_none());
    assert!(map.get(&b"qux"[..]).unwrap().is_none());
}

#[test]
fn test_sqlite_map_blob_string_stresstest() {
    let mut set = SQLiteMap::<Blob, String>::new().unwrap();
    let (mut hex_1, mut hex_2) = (HexEncoder::default(), HexEncoder::default());

    {
        let mut tx = set.transaction().unwrap();
        for i in 0u64..MAX_ITEMS {
            let (key, value) = (hex_1.encode_raw(mix64(i)), hex_2.encode(mix64(!i)));
            assert!(tx.insert(key, value).unwrap());
        }
    }

    assert!(!set.is_empty().unwrap());
    assert_eq!(set.len().unwrap().to_u64(), MAX_ITEMS);

    {
        let tx = set.transaction().unwrap();
        for i in 0u64..MAX_ITEMS {
            let (key, value) = (hex_1.encode_raw(mix64(i)), hex_2.encode(mix64(!i)));
            assert_eq!(tx.get(key).unwrap().unwrap(), value);
        }
    }

    {
        let mut tx = set.transaction().unwrap();
        for i in 0u64..MAX_ITEMS {
            assert!(tx.remove(hex_1.encode_raw(mix64(i))).unwrap());
        }
    }

    assert!(set.is_empty().unwrap());
    assert_eq!(set.len().unwrap().to_u64(), 0);
}

#[test]
fn test_sqlite_map_blob_string_randomized() {
    const COUNT: u64 = 125_000u64;
    let mut set = SQLiteMap::<Blob, String>::new().unwrap();
    let (mut temp, mut hex) = ([0u8; 16usize], HexEncoder::default());

    for seed in [0x9DDC5F3146219B97u64, 0x79B9E4DC002B32D6u64] {
        for b in [true, false] {
            let mut xoshiro = Xoshiro256PlusPlus::seed_from_u64(seed);
            let mut tx = set.transaction().unwrap();
            for _i in 0u64..COUNT {
                xoshiro.fill_bytes(&mut temp);
                assert_eq!(tx.insert(temp.as_slice(), hex.encode(xoshiro.next_u64())).unwrap(), b);
            }
        }
    }

    for (seed, b) in [(0x9DDC5F3146219B97u64, true), (0x79B9E4DC002B32D6u64, true), (0x2F381F8B2F0E517Au64, false)] {
        let mut xoshiro = Xoshiro256PlusPlus::seed_from_u64(seed);
        let tx = set.transaction().unwrap();
        for _i in 0u64..COUNT {
            xoshiro.fill_bytes(&mut temp);
            if b {
                assert_eq!(tx.get(temp.as_slice()).unwrap().unwrap(), hex.encode(xoshiro.next_u64()));
            } else {
                assert!(tx.get(temp.as_slice()).unwrap().is_none());
                let _skip = xoshiro.next_u64();
            }
        }
    }
}

// ---------------------------------------------------------------------------
// SQLiteMap<String, String> Tests
// ---------------------------------------------------------------------------

#[test]
fn test_sqlite_map_string_string() {
    let mut map = SQLiteMap::<String, String>::new().unwrap();

    assert_eq!(map.len().unwrap(), 0);
    assert!(map.is_empty().unwrap());
    assert!(!map.contains_key("foo").unwrap());
    assert!(!map.contains_key("bar").unwrap());
    assert!(!map.contains_key("baz").unwrap());
    assert!(!map.contains_key("qux").unwrap());
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
    assert!(map.contains_key("foo").unwrap());
    assert!(!map.contains_key("FOO").unwrap());
    assert!(map.contains_key_nocase("FOO").unwrap());
    assert!(map.contains_key("bar").unwrap());
    assert!(!map.contains_key("BAR").unwrap());
    assert!(map.contains_key_nocase("BAR").unwrap());
    assert!(map.contains_key("baz").unwrap());
    assert!(!map.contains_key("BAZ").unwrap());
    assert!(map.contains_key_nocase("BAZ").unwrap());
    assert!(!map.contains_key("qux").unwrap());
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
    assert!(!map.contains_key("foo").unwrap());
    assert!(!map.contains_key("bar").unwrap());
    assert!(map.contains_key("baz").unwrap());
    assert!(!map.contains_key("BAZ").unwrap());
    assert!(map.contains_key_nocase("BAZ").unwrap());
    assert!(!map.contains_key("qux").unwrap());
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
    assert!(map.contains_key("foo").unwrap());
    assert!(!map.contains_key("FOO").unwrap());
    assert!(map.contains_key_nocase("FOO").unwrap());
    assert!(!map.contains_key("bar").unwrap());
    assert!(map.contains_key("baz").unwrap());
    assert!(!map.contains_key("BAZ").unwrap());
    assert!(map.contains_key_nocase("BAZ").unwrap());
    assert!(map.contains_key("qux").unwrap());
    assert!(!map.contains_key("QUX").unwrap());
    assert!(map.contains_key_nocase("QUX").unwrap());
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
        let mut hash = HashSet::new();
        map.for_each_key(|key| assert!(hash.insert(key.to_owned()))).unwrap();
        assert_eq!(hash.len(), 3);
        assert!(hash.contains("foo"));
        assert!(hash.contains("baz"));
        assert!(hash.contains("qux"));
    }

    {
        let mut hash = HashMap::new();
        map.for_each_entry(|key, value| assert!(hash.insert(key.to_owned(), value.to_owned()).is_none())).unwrap();
        assert_eq!(hash.len(), 3);
        assert_eq!(hash.get("foo").unwrap(), "12345678");
        assert_eq!(hash.get("baz").unwrap(), "deadbeef");
        assert_eq!(hash.get("qux").unwrap(), "cafebabe");
    }

    assert!(map.find_key(|key| key.eq("foo")).unwrap().is_some_and(|key| key.eq("foo")));
    assert!(map.find_key(|key| key.eq("bar")).unwrap().is_none());
    assert!(map.find_key(|key| key.eq("baz")).unwrap().is_some_and(|key| key.eq("baz")));
    assert!(map.find_key(|key| key.eq("qux")).unwrap().is_some_and(|key| key.eq("qux")));

    assert_eq!(map.find_entry(|key, _| key.eq("foo")).unwrap().unwrap().1, "12345678");
    assert_eq!(map.find_entry(|key, _| key.eq_ignore_ascii_case("FOO")).unwrap().unwrap().1, "12345678");
    assert!(map.find_entry(|key, _| key.eq("bar")).unwrap().is_none());

    map.update("foo", "abcdefgh").unwrap();
    map.update("bar", "hgfedcba").unwrap();
    map.update("baz", "stuvwxyz").unwrap();

    assert_eq!(map.get("foo").unwrap().unwrap(), "abcdefgh");
    assert_eq!(map.get("bar").unwrap().unwrap(), "hgfedcba");
    assert_eq!(map.get("baz").unwrap().unwrap(), "stuvwxyz");
    assert_eq!(map.get("qux").unwrap().unwrap(), "cafebabe");

    {
        let mut hash = HashSet::new();
        map.for_each_key(|key| assert!(hash.insert(key.to_owned()))).unwrap();
        assert_eq!(hash.len(), 4);
        assert!(hash.contains("foo"));
        assert!(hash.contains("bar"));
        assert!(hash.contains("baz"));
        assert!(hash.contains("qux"));
    }

    {
        let mut hash = HashMap::new();
        map.for_each_entry(|key, value| assert!(hash.insert(key.to_owned(), value.to_owned()).is_none())).unwrap();
        assert_eq!(hash.len(), 4);
        assert_eq!(hash.get("foo").unwrap(), "abcdefgh");
        assert_eq!(hash.get("bar").unwrap(), "hgfedcba");
        assert_eq!(hash.get("baz").unwrap(), "stuvwxyz");
        assert_eq!(hash.get("qux").unwrap(), "cafebabe");
    }

    assert!(map.find_key(|key| key.eq("foo")).unwrap().is_some_and(|key| key.eq("foo")));
    assert!(map.find_key(|key| key.eq("bar")).unwrap().is_some_and(|key| key.eq("bar")));
    assert!(map.find_key(|key| key.eq("baz")).unwrap().is_some_and(|key| key.eq("baz")));
    assert!(map.find_key(|key| key.eq("qux")).unwrap().is_some_and(|key| key.eq("qux")));

    assert_eq!(map.find_entry(|key, _| key.eq("foo")).unwrap().unwrap().1, "abcdefgh");
    assert_eq!(map.find_entry(|key, _| key.eq_ignore_ascii_case("FOO")).unwrap().unwrap().1, "abcdefgh");
    assert_eq!(map.find_entry(|key, _| key.eq("bar")).unwrap().unwrap().1, "hgfedcba");
    assert_eq!(map.find_entry(|key, _| key.eq_ignore_ascii_case("BAR")).unwrap().unwrap().1, "hgfedcba");
    assert!(map.find_entry(|key, _| key.eq("xyz")).unwrap().is_none());

    {
        let mut hash = HashSet::new();
        map.glob_keys(false, "f*", |key| assert!(hash.insert(key))).unwrap();
        assert_eq!(hash.len(), 1);
        assert!(hash.contains("foo"));
    }

    {
        let mut hash = HashSet::new();
        map.glob_keys(false, "ba?", |key| assert!(hash.insert(key))).unwrap();
        assert_eq!(hash.len(), 2);
        assert!(hash.contains("bar"));
        assert!(hash.contains("baz"));
    }

    {
        let mut hash = HashSet::new();
        map.glob_keys(true, "ba?", |key| assert!(hash.insert(key))).unwrap();
        assert_eq!(hash.len(), 2);
        assert!(hash.contains("foo"));
        assert!(hash.contains("qux"));
    }

    {
        let mut hash = HashSet::new();
        map.glob_entries(false, "f*", |key, _| assert!(hash.insert(key))).unwrap();
        assert_eq!(hash.len(), 1);
        assert!(hash.contains("foo"));
    }

    {
        let mut hash = HashSet::new();
        map.glob_entries(false, "ba?", |key, _| assert!(hash.insert(key))).unwrap();
        assert_eq!(hash.len(), 2);
        assert!(hash.contains("bar"));
        assert!(hash.contains("baz"));
    }

    {
        let mut hash = HashSet::new();
        map.glob_entries(true, "ba?", |key, _| assert!(hash.insert(key))).unwrap();
        assert_eq!(hash.len(), 2);
        assert!(hash.contains("foo"));
        assert!(hash.contains("qux"));
    }

    map.clear().unwrap();

    assert_eq!(map.len().unwrap(), 0);
    assert!(map.is_empty().unwrap());
    assert!(!map.contains_key("foo").unwrap());
    assert!(!map.contains_key("bar").unwrap());
    assert!(!map.contains_key("baz").unwrap());
    assert!(!map.contains_key("qux").unwrap());
}

#[test]
fn test_sqlite_map_string_string_transacted() {
    let mut map = SQLiteMap::<String, String>::new().unwrap();
    let mut tx = map.transaction().unwrap();

    assert_eq!(tx.len().unwrap(), 0);
    assert!(tx.is_empty().unwrap());
    assert!(!tx.contains_key("foo").unwrap());
    assert!(!tx.contains_key("bar").unwrap());
    assert!(!tx.contains_key("baz").unwrap());
    assert!(!tx.contains_key("qux").unwrap());
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
    assert!(tx.contains_key("foo").unwrap());
    assert!(!tx.contains_key("FOO").unwrap());
    assert!(tx.contains_key_nocase("FOO").unwrap());
    assert!(tx.contains_key("bar").unwrap());
    assert!(!tx.contains_key("BAR").unwrap());
    assert!(tx.contains_key_nocase("BAR").unwrap());
    assert!(tx.contains_key("baz").unwrap());
    assert!(!tx.contains_key("BAZ").unwrap());
    assert!(tx.contains_key_nocase("BAZ").unwrap());
    assert!(!tx.contains_key("qux").unwrap());
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
    assert!(!tx.contains_key("foo").unwrap());
    assert!(!tx.contains_key("bar").unwrap());
    assert!(tx.contains_key("baz").unwrap());
    assert!(!tx.contains_key("BAZ").unwrap());
    assert!(tx.contains_key_nocase("BAZ").unwrap());
    assert!(!tx.contains_key("qux").unwrap());
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
    assert!(tx.contains_key("foo").unwrap());
    assert!(!tx.contains_key("FOO").unwrap());
    assert!(tx.contains_key_nocase("FOO").unwrap());
    assert!(!tx.contains_key("bar").unwrap());
    assert!(tx.contains_key("baz").unwrap());
    assert!(!tx.contains_key("BAZ").unwrap());
    assert!(tx.contains_key_nocase("BAZ").unwrap());
    assert!(tx.contains_key("qux").unwrap());
    assert!(!tx.contains_key("QUX").unwrap());
    assert!(tx.contains_key_nocase("QUX").unwrap());
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
        let mut hash = HashSet::new();
        tx.for_each_key(|key| assert!(hash.insert(key.to_owned()))).unwrap();
        assert_eq!(hash.len(), 3);
        assert!(hash.contains("foo"));
        assert!(hash.contains("baz"));
        assert!(hash.contains("qux"));
    }

    {
        let mut hash = HashMap::new();
        tx.for_each_entry(|key, value| assert!(hash.insert(key.to_owned(), value.to_owned()).is_none())).unwrap();
        assert_eq!(hash.len(), 3);
        assert_eq!(hash.get("foo").unwrap(), "12345678");
        assert_eq!(hash.get("baz").unwrap(), "deadbeef");
        assert_eq!(hash.get("qux").unwrap(), "cafebabe");
    }

    assert!(tx.find_key(|key| key.eq("foo")).unwrap().is_some_and(|key| key.eq("foo")));
    assert!(tx.find_key(|key| key.eq("bar")).unwrap().is_none());
    assert!(tx.find_key(|key| key.eq("baz")).unwrap().is_some_and(|key| key.eq("baz")));
    assert!(tx.find_key(|key| key.eq("qux")).unwrap().is_some_and(|key| key.eq("qux")));

    assert_eq!(tx.find_entry(|key, _| key.eq("foo")).unwrap().unwrap().1, "12345678");
    assert_eq!(tx.find_entry(|key, _| key.eq_ignore_ascii_case("FOO")).unwrap().unwrap().1, "12345678");
    assert!(tx.find_entry(|key, _| key.eq("bar")).unwrap().is_none());

    tx.update("foo", "abcdefgh").unwrap();
    tx.update("bar", "hgfedcba").unwrap();
    tx.update("baz", "stuvwxyz").unwrap();

    assert_eq!(tx.get("foo").unwrap().unwrap(), "abcdefgh");
    assert_eq!(tx.get("bar").unwrap().unwrap(), "hgfedcba");
    assert_eq!(tx.get("baz").unwrap().unwrap(), "stuvwxyz");
    assert_eq!(tx.get("qux").unwrap().unwrap(), "cafebabe");

    {
        let mut hash = HashSet::new();
        tx.for_each_key(|key| assert!(hash.insert(key.to_owned()))).unwrap();
        assert_eq!(hash.len(), 4);
        assert!(hash.contains("foo"));
        assert!(hash.contains("bar"));
        assert!(hash.contains("baz"));
        assert!(hash.contains("qux"));
    }

    {
        let mut hash = HashMap::new();
        tx.for_each_entry(|key, value| assert!(hash.insert(key.to_owned(), value.to_owned()).is_none())).unwrap();
        assert_eq!(hash.len(), 4);
        assert_eq!(hash.get("foo").unwrap(), "abcdefgh");
        assert_eq!(hash.get("bar").unwrap(), "hgfedcba");
        assert_eq!(hash.get("baz").unwrap(), "stuvwxyz");
        assert_eq!(hash.get("qux").unwrap(), "cafebabe");
    }

    assert!(tx.find_key(|key| key.eq("foo")).unwrap().is_some_and(|key| key.eq("foo")));
    assert!(tx.find_key(|key| key.eq("bar")).unwrap().is_some_and(|key| key.eq("bar")));
    assert!(tx.find_key(|key| key.eq("baz")).unwrap().is_some_and(|key| key.eq("baz")));
    assert!(tx.find_key(|key| key.eq("qux")).unwrap().is_some_and(|key| key.eq("qux")));

    assert_eq!(tx.find_entry(|key, _| key.eq("foo")).unwrap().unwrap().1, "abcdefgh");
    assert_eq!(tx.find_entry(|key, _| key.eq_ignore_ascii_case("FOO")).unwrap().unwrap().1, "abcdefgh");
    assert_eq!(tx.find_entry(|key, _| key.eq("bar")).unwrap().unwrap().1, "hgfedcba");
    assert_eq!(tx.find_entry(|key, _| key.eq_ignore_ascii_case("BAR")).unwrap().unwrap().1, "hgfedcba");
    assert!(tx.find_entry(|key, _| key.eq("xyz")).unwrap().is_none());

    {
        let mut hash = HashSet::new();
        tx.glob_keys(false, "f*", |key| assert!(hash.insert(key))).unwrap();
        assert_eq!(hash.len(), 1);
        assert!(hash.contains("foo"));
    }

    {
        let mut hash = HashSet::new();
        tx.glob_keys(false, "ba?", |key| assert!(hash.insert(key))).unwrap();
        assert_eq!(hash.len(), 2);
        assert!(hash.contains("bar"));
        assert!(hash.contains("baz"));
    }

    {
        let mut hash = HashSet::new();
        tx.glob_keys(true, "ba?", |key| assert!(hash.insert(key))).unwrap();
        assert_eq!(hash.len(), 2);
        assert!(hash.contains("foo"));
        assert!(hash.contains("qux"));
    }

    {
        let mut hash = HashSet::new();
        tx.glob_entries(false, "f*", |key, _| assert!(hash.insert(key))).unwrap();
        assert_eq!(hash.len(), 1);
        assert!(hash.contains("foo"));
    }

    {
        let mut hash = HashSet::new();
        tx.glob_entries(false, "ba?", |key, _| assert!(hash.insert(key))).unwrap();
        assert_eq!(hash.len(), 2);
        assert!(hash.contains("bar"));
        assert!(hash.contains("baz"));
    }

    {
        let mut hash = HashSet::new();
        tx.glob_entries(true, "ba?", |key, _| assert!(hash.insert(key))).unwrap();
        assert_eq!(hash.len(), 2);
        assert!(hash.contains("foo"));
        assert!(hash.contains("qux"));
    }

    tx.clear().unwrap();

    assert_eq!(tx.len().unwrap(), 0);
    assert!(tx.is_empty().unwrap());
    assert!(!tx.contains_key("foo").unwrap());
    assert!(!tx.contains_key("bar").unwrap());
    assert!(!tx.contains_key("baz").unwrap());
    assert!(!tx.contains_key("qux").unwrap());

    assert!(tx.insert("foo", "ijklmnop").unwrap());
    assert!(tx.insert("bar", "ponmlkji").unwrap());

    tx.commit();

    assert_eq!(map.len().unwrap(), 2);
    assert!(!map.is_empty().unwrap());
    assert!(map.contains_key("foo").unwrap());
    assert!(map.contains_key("bar").unwrap());
    assert!(!map.contains_key("baz").unwrap());
    assert!(!map.contains_key("qux").unwrap());
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

    assert!(map_1.contains_key("foo").unwrap());
    assert!(!map_1.contains_key("FOO").unwrap());
    assert!(!map_1.contains_key("bar").unwrap());
    assert!(map_1.contains_key("BAR").unwrap());
    assert_eq!(map_1.get("foo").unwrap().unwrap(), "12345678");
    assert!(map_1.get("FOO").unwrap().is_none());
    assert!(map_1.get("bar").unwrap().is_none());
    assert_eq!(map_1.get("BAR").unwrap().unwrap(), "abcdefgh");

    assert!(map_2.contains_key("foo").unwrap());
    assert!(map_2.contains_key("FOO").unwrap());
    assert!(map_2.contains_key("bar").unwrap());
    assert!(map_2.contains_key("BAR").unwrap());
    assert_eq!(map_2.get("foo").unwrap().unwrap(), "12345678");
    assert_eq!(map_2.get("FOO").unwrap().unwrap(), "12345678");
    assert_eq!(map_2.get("bar").unwrap().unwrap(), "abcdefgh");
    assert_eq!(map_2.get("BAR").unwrap().unwrap(), "abcdefgh");

    assert!(map_2.remove("FOO").unwrap());
    assert!(map_2.remove("bar").unwrap());

    assert!(!map_2.contains_key("foo").unwrap());
    assert!(!map_2.contains_key("FOO").unwrap());
    assert!(!map_2.contains_key("bar").unwrap());
    assert!(!map_2.contains_key("BAR").unwrap());
    assert!(map_2.get("foo").unwrap().is_none());
    assert!(map_2.get("FOO").unwrap().is_none());
    assert!(map_2.get("bar").unwrap().is_none());
    assert!(map_2.get("BAR").unwrap().is_none());
}

#[test]
fn test_sqlite_map_string_string_stresstest() {
    let mut set = SQLiteMap::<String, String>::new().unwrap();
    let (mut hex_1, mut hex_2) = (HexEncoder::default(), HexEncoder::default());

    {
        let mut tx = set.transaction().unwrap();
        for i in 0u64..MAX_ITEMS {
            let (key, value) = (hex_1.encode(mix64(i)), hex_2.encode(mix64(!i)));
            assert!(tx.insert(key, value).unwrap());
        }
    }

    assert!(!set.is_empty().unwrap());
    assert_eq!(set.len().unwrap().to_u64(), MAX_ITEMS);

    {
        let tx = set.transaction().unwrap();
        for i in 0u64..MAX_ITEMS {
            let (key, value) = (hex_1.encode(mix64(i)), hex_2.encode(mix64(!i)));
            assert_eq!(tx.get(key).unwrap().unwrap(), value);
        }
    }

    {
        let mut tx = set.transaction().unwrap();
        for i in 0u64..MAX_ITEMS {
            assert!(tx.remove(hex_1.encode(mix64(i))).unwrap());
        }
    }

    assert!(set.is_empty().unwrap());
    assert_eq!(set.len().unwrap().to_u64(), 0);
}

#[test]
fn test_sqlite_map_string_string_randomized() {
    const COUNT: u64 = 125_000u64;
    let mut set = SQLiteMap::<String, String>::new().unwrap();
    let (mut hex_1, mut hex_2) = (HexEncoder::default(), HexEncoder::default());

    for seed in [0x9DDC5F3146219B97u64, 0x79B9E4DC002B32D6u64] {
        for b in [true, false] {
            let mut xoshiro = Xoshiro256PlusPlus::seed_from_u64(seed);
            let mut tx = set.transaction().unwrap();
            for _i in 0u64..COUNT {
                assert_eq!(tx.insert(hex_1.encode(xoshiro.next_u64()), hex_2.encode(xoshiro.next_u64())).unwrap(), b);
            }
        }
    }

    for (seed, b) in [(0x9DDC5F3146219B97u64, true), (0x79B9E4DC002B32D6u64, true), (0x2F381F8B2F0E517Au64, false)] {
        let mut xoshiro = Xoshiro256PlusPlus::seed_from_u64(seed);
        let tx = set.transaction().unwrap();
        for _i in 0u64..COUNT {
            if b {
                assert_eq!(tx.get(hex_1.encode(xoshiro.next_u64())).unwrap().unwrap(), hex_2.encode(xoshiro.next_u64()));
            } else {
                assert!(tx.get(hex_1.encode(xoshiro.next_u64())).unwrap().is_none());
                let _skip = xoshiro.next_u64();
            }
        }
    }
}

// ---------------------------------------------------------------------------
// SQLiteMap<String, Blob> Tests
// ---------------------------------------------------------------------------

#[test]
fn test_sqlite_map_string_blob() {
    let mut map = SQLiteMap::<String, Blob>::new().unwrap();

    assert_eq!(map.len().unwrap(), 0);
    assert!(map.is_empty().unwrap());
    assert!(!map.contains_key("foo").unwrap());
    assert!(!map.contains_key("bar").unwrap());
    assert!(!map.contains_key("baz").unwrap());
    assert!(!map.contains_key("qux").unwrap());
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
    assert!(map.contains_key("foo").unwrap());
    assert!(!map.contains_key("FOO").unwrap());
    assert!(map.contains_key_nocase("FOO").unwrap());
    assert!(map.contains_key("bar").unwrap());
    assert!(!map.contains_key("BAR").unwrap());
    assert!(map.contains_key_nocase("BAR").unwrap());
    assert!(map.contains_key("baz").unwrap());
    assert!(!map.contains_key("BAZ").unwrap());
    assert!(map.contains_key_nocase("BAZ").unwrap());
    assert!(!map.contains_key("qux").unwrap());
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
    assert!(!map.contains_key("foo").unwrap());
    assert!(!map.contains_key("bar").unwrap());
    assert!(map.contains_key("baz").unwrap());
    assert!(!map.contains_key("BAZ").unwrap());
    assert!(map.contains_key_nocase("BAZ").unwrap());
    assert!(!map.contains_key("qux").unwrap());
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
    assert!(map.contains_key("foo").unwrap());
    assert!(!map.contains_key("FOO").unwrap());
    assert!(map.contains_key_nocase("FOO").unwrap());
    assert!(!map.contains_key("bar").unwrap());
    assert!(map.contains_key("baz").unwrap());
    assert!(!map.contains_key("BAZ").unwrap());
    assert!(map.contains_key_nocase("BAZ").unwrap());
    assert!(map.contains_key("qux").unwrap());
    assert!(!map.contains_key("QUX").unwrap());
    assert!(map.contains_key_nocase("QUX").unwrap());
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
        let mut hash = HashSet::new();
        map.for_each_key(|key| assert!(hash.insert(key.to_owned()))).unwrap();
        assert_eq!(hash.len(), 3);
        assert!(hash.contains("foo"));
        assert!(hash.contains("baz"));
        assert!(hash.contains("qux"));
    }

    {
        let mut hash = HashMap::new();
        map.for_each_entry(|key, value| assert!(hash.insert(key.to_owned(), value.to_owned()).is_none())).unwrap();
        assert_eq!(hash.len(), 3);
        assert_eq!(hash.get("foo").unwrap(), &b"12345678"[..]);
        assert_eq!(hash.get("baz").unwrap(), &b"deadbeef"[..]);
        assert_eq!(hash.get("qux").unwrap(), &b"cafebabe"[..]);
    }

    assert!(map.find_key(|key| key.eq("foo")).unwrap().is_some_and(|key| key.eq("foo")));
    assert!(map.find_key(|key| key.eq("bar")).unwrap().is_none());
    assert!(map.find_key(|key| key.eq("baz")).unwrap().is_some_and(|key| key.eq("baz")));
    assert!(map.find_key(|key| key.eq("qux")).unwrap().is_some_and(|key| key.eq("qux")));

    assert_eq!(map.find_entry(|key, _| key.eq("foo")).unwrap().unwrap().1, &b"12345678"[..]);
    assert_eq!(map.find_entry(|key, _| key.eq_ignore_ascii_case("FOO")).unwrap().unwrap().1, &b"12345678"[..]);
    assert!(map.find_entry(|key, _| key.eq("bar")).unwrap().is_none());

    map.update("foo", &b"abcdefgh"[..]).unwrap();
    map.update("bar", &b"hgfedcba"[..]).unwrap();
    map.update("baz", &b"stuvwxyz"[..]).unwrap();

    assert_eq!(map.get("foo").unwrap().unwrap(), &b"abcdefgh"[..]);
    assert_eq!(map.get("bar").unwrap().unwrap(), &b"hgfedcba"[..]);
    assert_eq!(map.get("baz").unwrap().unwrap(), &b"stuvwxyz"[..]);
    assert_eq!(map.get("qux").unwrap().unwrap(), &b"cafebabe"[..]);

    {
        let mut hash = HashSet::new();
        map.for_each_key(|key| assert!(hash.insert(key.to_owned()))).unwrap();
        assert_eq!(hash.len(), 4);
        assert!(hash.contains("foo"));
        assert!(hash.contains("bar"));
        assert!(hash.contains("baz"));
        assert!(hash.contains("qux"));
    }

    {
        let mut hash = HashMap::new();
        map.for_each_entry(|key, value| assert!(hash.insert(key.to_owned(), value.to_owned()).is_none())).unwrap();
        assert_eq!(hash.len(), 4);
        assert_eq!(hash.get("foo").unwrap(), &b"abcdefgh"[..]);
        assert_eq!(hash.get("bar").unwrap(), &b"hgfedcba"[..]);
        assert_eq!(hash.get("baz").unwrap(), &b"stuvwxyz"[..]);
        assert_eq!(hash.get("qux").unwrap(), &b"cafebabe"[..]);
    }

    assert!(map.find_key(|key| key.eq("foo")).unwrap().is_some_and(|key| key.eq("foo")));
    assert!(map.find_key(|key| key.eq("bar")).unwrap().is_some_and(|key| key.eq("bar")));
    assert!(map.find_key(|key| key.eq("baz")).unwrap().is_some_and(|key| key.eq("baz")));
    assert!(map.find_key(|key| key.eq("qux")).unwrap().is_some_and(|key| key.eq("qux")));

    assert_eq!(map.find_entry(|key, _| key.eq("foo")).unwrap().unwrap().1, &b"abcdefgh"[..]);
    assert_eq!(map.find_entry(|key, _| key.eq_ignore_ascii_case("FOO")).unwrap().unwrap().1, &b"abcdefgh"[..]);
    assert_eq!(map.find_entry(|key, _| key.eq("bar")).unwrap().unwrap().1, &b"hgfedcba"[..]);
    assert_eq!(map.find_entry(|key, _| key.eq_ignore_ascii_case("BAR")).unwrap().unwrap().1, &b"hgfedcba"[..]);
    assert!(map.find_entry(|key, _| key.eq("xyz")).unwrap().is_none());

    {
        let mut hash = HashSet::new();
        map.glob_keys(false, "f*", |key| assert!(hash.insert(key))).unwrap();
        assert_eq!(hash.len(), 1);
        assert!(hash.contains("foo"));
    }

    {
        let mut hash = HashSet::new();
        map.glob_keys(false, "ba?", |key| assert!(hash.insert(key))).unwrap();
        assert_eq!(hash.len(), 2);
        assert!(hash.contains("bar"));
        assert!(hash.contains("baz"));
    }

    {
        let mut hash = HashSet::new();
        map.glob_keys(true, "ba?", |key| assert!(hash.insert(key))).unwrap();
        assert_eq!(hash.len(), 2);
        assert!(hash.contains("foo"));
        assert!(hash.contains("qux"));
    }

    {
        let mut hash = HashSet::new();
        map.glob_entries(false, "f*", |key, _| assert!(hash.insert(key))).unwrap();
        assert_eq!(hash.len(), 1);
        assert!(hash.contains("foo"));
    }

    {
        let mut hash = HashSet::new();
        map.glob_entries(false, "ba?", |key, _| assert!(hash.insert(key))).unwrap();
        assert_eq!(hash.len(), 2);
        assert!(hash.contains("bar"));
        assert!(hash.contains("baz"));
    }

    {
        let mut hash = HashSet::new();
        map.glob_entries(true, "ba?", |key, _| assert!(hash.insert(key))).unwrap();
        assert_eq!(hash.len(), 2);
        assert!(hash.contains("foo"));
        assert!(hash.contains("qux"));
    }

    map.clear().unwrap();

    assert_eq!(map.len().unwrap(), 0);
    assert!(map.is_empty().unwrap());
    assert!(!map.contains_key("foo").unwrap());
    assert!(!map.contains_key("bar").unwrap());
    assert!(!map.contains_key("baz").unwrap());
    assert!(!map.contains_key("qux").unwrap());
}

#[test]
fn test_sqlite_map_string_blob_transacted() {
    let mut map = SQLiteMap::<String, Blob>::new().unwrap();
    let mut tx = map.transaction().unwrap();

    assert_eq!(tx.len().unwrap(), 0);
    assert!(tx.is_empty().unwrap());
    assert!(!tx.contains_key("foo").unwrap());
    assert!(!tx.contains_key("bar").unwrap());
    assert!(!tx.contains_key("baz").unwrap());
    assert!(!tx.contains_key("qux").unwrap());
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
    assert!(tx.contains_key("foo").unwrap());
    assert!(!tx.contains_key("FOO").unwrap());
    assert!(tx.contains_key_nocase("FOO").unwrap());
    assert!(tx.contains_key("bar").unwrap());
    assert!(!tx.contains_key("BAR").unwrap());
    assert!(tx.contains_key_nocase("BAR").unwrap());
    assert!(tx.contains_key("baz").unwrap());
    assert!(!tx.contains_key("BAZ").unwrap());
    assert!(tx.contains_key_nocase("BAZ").unwrap());
    assert!(!tx.contains_key("qux").unwrap());
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
    assert!(!tx.contains_key("foo").unwrap());
    assert!(!tx.contains_key("bar").unwrap());
    assert!(tx.contains_key("baz").unwrap());
    assert!(!tx.contains_key("BAZ").unwrap());
    assert!(tx.contains_key_nocase("BAZ").unwrap());
    assert!(!tx.contains_key("qux").unwrap());
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
    assert!(tx.contains_key("foo").unwrap());
    assert!(!tx.contains_key("FOO").unwrap());
    assert!(tx.contains_key_nocase("FOO").unwrap());
    assert!(!tx.contains_key("bar").unwrap());
    assert!(tx.contains_key("baz").unwrap());
    assert!(!tx.contains_key("BAZ").unwrap());
    assert!(tx.contains_key_nocase("BAZ").unwrap());
    assert!(tx.contains_key("qux").unwrap());
    assert!(!tx.contains_key("QUX").unwrap());
    assert!(tx.contains_key_nocase("QUX").unwrap());
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
        let mut hash = HashSet::new();
        tx.for_each_key(|key| assert!(hash.insert(key.to_owned()))).unwrap();
        assert_eq!(hash.len(), 3);
        assert!(hash.contains("foo"));
        assert!(hash.contains("baz"));
        assert!(hash.contains("qux"));
    }

    {
        let mut hash = HashMap::new();
        tx.for_each_entry(|key, value| assert!(hash.insert(key.to_owned(), value.to_owned()).is_none())).unwrap();
        assert_eq!(hash.len(), 3);
        assert_eq!(hash.get("foo").unwrap(), &b"12345678"[..]);
        assert_eq!(hash.get("baz").unwrap(), &b"deadbeef"[..]);
        assert_eq!(hash.get("qux").unwrap(), &b"cafebabe"[..]);
    }

    assert!(tx.find_key(|key| key.eq("foo")).unwrap().is_some_and(|key| key.eq("foo")));
    assert!(tx.find_key(|key| key.eq("bar")).unwrap().is_none());
    assert!(tx.find_key(|key| key.eq("baz")).unwrap().is_some_and(|key| key.eq("baz")));
    assert!(tx.find_key(|key| key.eq("qux")).unwrap().is_some_and(|key| key.eq("qux")));

    assert_eq!(tx.find_entry(|key, _| key.eq("foo")).unwrap().unwrap().1, &b"12345678"[..]);
    assert_eq!(tx.find_entry(|key, _| key.eq_ignore_ascii_case("FOO")).unwrap().unwrap().1, &b"12345678"[..]);
    assert!(tx.find_entry(|key, _| key.eq("bar")).unwrap().is_none());

    tx.update("foo", &b"abcdefgh"[..]).unwrap();
    tx.update("bar", &b"hgfedcba"[..]).unwrap();
    tx.update("baz", &b"stuvwxyz"[..]).unwrap();

    assert_eq!(tx.get("foo").unwrap().unwrap(), &b"abcdefgh"[..]);
    assert_eq!(tx.get("bar").unwrap().unwrap(), &b"hgfedcba"[..]);
    assert_eq!(tx.get("baz").unwrap().unwrap(), &b"stuvwxyz"[..]);
    assert_eq!(tx.get("qux").unwrap().unwrap(), &b"cafebabe"[..]);

    {
        let mut hash = HashSet::new();
        tx.for_each_key(|key| assert!(hash.insert(key.to_owned()))).unwrap();
        assert_eq!(hash.len(), 4);
        assert!(hash.contains("foo"));
        assert!(hash.contains("bar"));
        assert!(hash.contains("baz"));
        assert!(hash.contains("qux"));
    }

    {
        let mut hash = HashMap::new();
        tx.for_each_entry(|key, value| assert!(hash.insert(key.to_owned(), value.to_owned()).is_none())).unwrap();
        assert_eq!(hash.len(), 4);
        assert_eq!(hash.get("foo").unwrap(), &b"abcdefgh"[..]);
        assert_eq!(hash.get("bar").unwrap(), &b"hgfedcba"[..]);
        assert_eq!(hash.get("baz").unwrap(), &b"stuvwxyz"[..]);
        assert_eq!(hash.get("qux").unwrap(), &b"cafebabe"[..]);
    }

    assert!(tx.find_key(|key| key.eq("foo")).unwrap().is_some_and(|key| key.eq("foo")));
    assert!(tx.find_key(|key| key.eq("bar")).unwrap().is_some_and(|key| key.eq("bar")));
    assert!(tx.find_key(|key| key.eq("baz")).unwrap().is_some_and(|key| key.eq("baz")));
    assert!(tx.find_key(|key| key.eq("qux")).unwrap().is_some_and(|key| key.eq("qux")));

    assert_eq!(tx.find_entry(|key, _| key.eq("foo")).unwrap().unwrap().1, &b"abcdefgh"[..]);
    assert_eq!(tx.find_entry(|key, _| key.eq_ignore_ascii_case("FOO")).unwrap().unwrap().1, &b"abcdefgh"[..]);
    assert_eq!(tx.find_entry(|key, _| key.eq("bar")).unwrap().unwrap().1, &b"hgfedcba"[..]);
    assert_eq!(tx.find_entry(|key, _| key.eq_ignore_ascii_case("BAR")).unwrap().unwrap().1, &b"hgfedcba"[..]);
    assert!(tx.find_entry(|key, _| key.eq("xyz")).unwrap().is_none());

    {
        let mut hash = HashSet::new();
        tx.glob_keys(false, "f*", |key| assert!(hash.insert(key))).unwrap();
        assert_eq!(hash.len(), 1);
        assert!(hash.contains("foo"));
    }

    {
        let mut hash = HashSet::new();
        tx.glob_keys(false, "ba?", |key| assert!(hash.insert(key))).unwrap();
        assert_eq!(hash.len(), 2);
        assert!(hash.contains("bar"));
        assert!(hash.contains("baz"));
    }

    {
        let mut hash = HashSet::new();
        tx.glob_keys(true, "ba?", |key| assert!(hash.insert(key))).unwrap();
        assert_eq!(hash.len(), 2);
        assert!(hash.contains("foo"));
        assert!(hash.contains("qux"));
    }

    {
        let mut hash = HashSet::new();
        tx.glob_entries(false, "f*", |key, _| assert!(hash.insert(key))).unwrap();
        assert_eq!(hash.len(), 1);
        assert!(hash.contains("foo"));
    }

    {
        let mut hash = HashSet::new();
        tx.glob_entries(false, "ba?", |key, _| assert!(hash.insert(key))).unwrap();
        assert_eq!(hash.len(), 2);
        assert!(hash.contains("bar"));
        assert!(hash.contains("baz"));
    }

    {
        let mut hash = HashSet::new();
        tx.glob_entries(true, "ba?", |key, _| assert!(hash.insert(key))).unwrap();
        assert_eq!(hash.len(), 2);
        assert!(hash.contains("foo"));
        assert!(hash.contains("qux"));
    }

    tx.clear().unwrap();

    assert_eq!(tx.len().unwrap(), 0);
    assert!(tx.is_empty().unwrap());
    assert!(!tx.contains_key("foo").unwrap());
    assert!(!tx.contains_key("bar").unwrap());
    assert!(!tx.contains_key("baz").unwrap());
    assert!(!tx.contains_key("qux").unwrap());

    assert!(tx.insert("foo", &b"ijklmnop"[..]).unwrap());
    assert!(tx.insert("bar", &b"ponmlkji"[..]).unwrap());

    tx.commit();

    assert_eq!(map.len().unwrap(), 2);
    assert!(!map.is_empty().unwrap());
    assert!(map.contains_key("foo").unwrap());
    assert!(map.contains_key("bar").unwrap());
    assert!(!map.contains_key("baz").unwrap());
    assert!(!map.contains_key("qux").unwrap());
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

    assert!(map_1.contains_key("foo").unwrap());
    assert!(!map_1.contains_key("FOO").unwrap());
    assert!(!map_1.contains_key("bar").unwrap());
    assert!(map_1.contains_key("BAR").unwrap());
    assert_eq!(map_1.get("foo").unwrap().unwrap(), &b"12345678"[..]);
    assert!(map_1.get("FOO").unwrap().is_none());
    assert!(map_1.get("bar").unwrap().is_none());
    assert_eq!(map_1.get("BAR").unwrap().unwrap(), &b"abcdefgh"[..]);

    assert!(map_2.contains_key("foo").unwrap());
    assert!(map_2.contains_key("FOO").unwrap());
    assert!(map_2.contains_key("bar").unwrap());
    assert!(map_2.contains_key("BAR").unwrap());
    assert_eq!(map_2.get("foo").unwrap().unwrap(), &b"12345678"[..]);
    assert_eq!(map_2.get("FOO").unwrap().unwrap(), &b"12345678"[..]);
    assert_eq!(map_2.get("bar").unwrap().unwrap(), &b"abcdefgh"[..]);
    assert_eq!(map_2.get("BAR").unwrap().unwrap(), &b"abcdefgh"[..]);

    assert!(map_2.remove("FOO").unwrap());
    assert!(map_2.remove("bar").unwrap());

    assert!(!map_2.contains_key("foo").unwrap());
    assert!(!map_2.contains_key("FOO").unwrap());
    assert!(!map_2.contains_key("bar").unwrap());
    assert!(!map_2.contains_key("BAR").unwrap());
    assert!(map_2.get("foo").unwrap().is_none());
    assert!(map_2.get("FOO").unwrap().is_none());
    assert!(map_2.get("bar").unwrap().is_none());
    assert!(map_2.get("BAR").unwrap().is_none());
}

#[test]
fn test_sqlite_map_string_blob_stresstest() {
    let mut set = SQLiteMap::<String, Blob>::new().unwrap();
    let (mut hex_1, mut hex_2) = (HexEncoder::default(), HexEncoder::default());

    {
        let mut tx = set.transaction().unwrap();
        for i in 0u64..MAX_ITEMS {
            let (key, value) = (hex_1.encode(mix64(i)), hex_2.encode_raw(mix64(!i)));
            assert!(tx.insert(key, value).unwrap());
        }
    }

    assert!(!set.is_empty().unwrap());
    assert_eq!(set.len().unwrap().to_u64(), MAX_ITEMS);

    {
        let tx = set.transaction().unwrap();
        for i in 0u64..MAX_ITEMS {
            let (key, value) = (hex_1.encode(mix64(i)), hex_2.encode_raw(mix64(!i)));
            assert_eq!(tx.get(key).unwrap().unwrap(), value);
        }
    }

    {
        let mut tx = set.transaction().unwrap();
        for i in 0u64..MAX_ITEMS {
            assert!(tx.remove(hex_1.encode(mix64(i))).unwrap());
        }
    }

    assert!(set.is_empty().unwrap());
    assert_eq!(set.len().unwrap().to_u64(), 0);
}

#[test]
fn test_sqlite_map_string_blob_randomized() {
    const COUNT: u64 = 125_000u64;
    let mut set = SQLiteMap::<String, Blob>::new().unwrap();
    let (mut hex, mut temp) = (HexEncoder::default(), [0u8; 16usize]);

    for seed in [0x9DDC5F3146219B97u64, 0x79B9E4DC002B32D6u64] {
        for b in [true, false] {
            let mut xoshiro = Xoshiro256PlusPlus::seed_from_u64(seed);
            let mut tx = set.transaction().unwrap();
            for _i in 0u64..COUNT {
                xoshiro.fill_bytes(&mut temp);
                assert_eq!(tx.insert(hex.encode(xoshiro.next_u64()), temp.as_slice()).unwrap(), b);
            }
        }
    }

    for (seed, b) in [(0x9DDC5F3146219B97u64, true), (0x79B9E4DC002B32D6u64, true), (0x2F381F8B2F0E517Au64, false)] {
        let mut xoshiro = Xoshiro256PlusPlus::seed_from_u64(seed);
        let tx = set.transaction().unwrap();
        for _i in 0u64..COUNT {
            xoshiro.fill_bytes(&mut temp);
            if b {
                assert_eq!(tx.get(hex.encode(xoshiro.next_u64())).unwrap().unwrap(), temp.as_slice());
            } else {
                assert!(tx.get(hex.encode(xoshiro.next_u64())).unwrap().is_none());
            }
        }
    }
}

// ---------------------------------------------------------------------------
// SQLiteMap<String, i64> Tests
// ---------------------------------------------------------------------------

#[test]
fn test_sqlite_map_string_i64() {
    let mut map = SQLiteMap::<String, i64>::new().unwrap();

    assert_eq!(map.len().unwrap(), 0);
    assert!(map.is_empty().unwrap());
    assert!(!map.contains_key("foo").unwrap());
    assert!(!map.contains_key("bar").unwrap());
    assert!(!map.contains_key("baz").unwrap());
    assert!(!map.contains_key("qux").unwrap());
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
    assert!(map.contains_key("foo").unwrap());
    assert!(!map.contains_key("FOO").unwrap());
    assert!(map.contains_key_nocase("FOO").unwrap());
    assert!(map.contains_key("bar").unwrap());
    assert!(!map.contains_key("BAR").unwrap());
    assert!(map.contains_key_nocase("BAR").unwrap());
    assert!(map.contains_key("baz").unwrap());
    assert!(!map.contains_key("BAZ").unwrap());
    assert!(map.contains_key_nocase("BAZ").unwrap());
    assert!(!map.contains_key("qux").unwrap());
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
    assert!(!map.contains_key("foo").unwrap());
    assert!(!map.contains_key("bar").unwrap());
    assert!(map.contains_key("baz").unwrap());
    assert!(!map.contains_key("BAZ").unwrap());
    assert!(map.contains_key_nocase("BAZ").unwrap());
    assert!(!map.contains_key("qux").unwrap());
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
    assert!(map.contains_key("foo").unwrap());
    assert!(!map.contains_key("FOO").unwrap());
    assert!(map.contains_key_nocase("FOO").unwrap());
    assert!(!map.contains_key("bar").unwrap());
    assert!(map.contains_key("baz").unwrap());
    assert!(!map.contains_key("BAZ").unwrap());
    assert!(map.contains_key_nocase("BAZ").unwrap());
    assert!(map.contains_key("qux").unwrap());
    assert!(!map.contains_key("QUX").unwrap());
    assert!(map.contains_key_nocase("QUX").unwrap());
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
        let mut hash = HashSet::new();
        map.for_each_key(|key| assert!(hash.insert(key.to_owned()))).unwrap();
        assert_eq!(hash.len(), 3);
        assert!(hash.contains("foo"));
        assert!(hash.contains("baz"));
        assert!(hash.contains("qux"));
    }

    {
        let mut hash = HashMap::new();
        map.for_each_entry(|key, value| assert!(hash.insert(key.to_owned(), value.to_owned()).is_none())).unwrap();
        assert_eq!(hash.len(), 3);
        assert_eq!(hash.get("foo").unwrap(), &0x12345678);
        assert_eq!(hash.get("baz").unwrap(), &0xDEADBEEF);
        assert_eq!(hash.get("qux").unwrap(), &0xCAFEBABE);
    }

    assert!(map.find_key(|key| key.eq("foo")).unwrap().is_some_and(|key| key.eq("foo")));
    assert!(map.find_key(|key| key.eq("bar")).unwrap().is_none());
    assert!(map.find_key(|key| key.eq("baz")).unwrap().is_some_and(|key| key.eq("baz")));
    assert!(map.find_key(|key| key.eq("qux")).unwrap().is_some_and(|key| key.eq("qux")));

    assert_eq!(map.find_entry(|key, _| key.eq("foo")).unwrap().unwrap().1, 0x12345678);
    assert_eq!(map.find_entry(|key, _| key.eq_ignore_ascii_case("FOO")).unwrap().unwrap().1, 0x12345678);
    assert!(map.find_entry(|key, _| key.eq("bar")).unwrap().is_none());

    map.update("foo", &0x9ABCDEF0).unwrap();
    map.update("bar", &0x0FEDCBA9).unwrap();
    map.update("baz", &0x8BADF00D).unwrap();

    assert_eq!(map.get("foo").unwrap().unwrap(), 0x9ABCDEF0);
    assert_eq!(map.get("bar").unwrap().unwrap(), 0x0FEDCBA9);
    assert_eq!(map.get("baz").unwrap().unwrap(), 0x8BADF00D);
    assert_eq!(map.get("qux").unwrap().unwrap(), 0xCAFEBABE);

    {
        let mut hash = HashSet::new();
        map.for_each_key(|key| assert!(hash.insert(key.to_owned()))).unwrap();
        assert_eq!(hash.len(), 4);
        assert!(hash.contains("foo"));
        assert!(hash.contains("bar"));
        assert!(hash.contains("baz"));
        assert!(hash.contains("qux"));
    }

    {
        let mut hash = HashMap::new();
        map.for_each_entry(|key, value| assert!(hash.insert(key.to_owned(), value.to_owned()).is_none())).unwrap();
        assert_eq!(hash.len(), 4);
        assert_eq!(hash.get("foo").unwrap(), &0x9ABCDEF0);
        assert_eq!(hash.get("bar").unwrap(), &0x0FEDCBA9);
        assert_eq!(hash.get("baz").unwrap(), &0x8BADF00D);
        assert_eq!(hash.get("qux").unwrap(), &0xCAFEBABE);
    }

    assert!(map.find_key(|key| key.eq("foo")).unwrap().is_some_and(|key| key.eq("foo")));
    assert!(map.find_key(|key| key.eq("bar")).unwrap().is_some_and(|key| key.eq("bar")));
    assert!(map.find_key(|key| key.eq("baz")).unwrap().is_some_and(|key| key.eq("baz")));
    assert!(map.find_key(|key| key.eq("qux")).unwrap().is_some_and(|key| key.eq("qux")));

    assert_eq!(map.find_entry(|key, _| key.eq("foo")).unwrap().unwrap().1, 0x9ABCDEF0);
    assert_eq!(map.find_entry(|key, _| key.eq_ignore_ascii_case("FOO")).unwrap().unwrap().1, 0x9ABCDEF0);
    assert_eq!(map.find_entry(|key, _| key.eq("bar")).unwrap().unwrap().1, 0x0FEDCBA9);
    assert_eq!(map.find_entry(|key, _| key.eq_ignore_ascii_case("BAR")).unwrap().unwrap().1, 0x0FEDCBA9);
    assert!(map.find_entry(|key, _| key.eq("xyz")).unwrap().is_none());

    {
        let mut hash = HashSet::new();
        map.glob_keys(false, "f*", |key| assert!(hash.insert(key))).unwrap();
        assert_eq!(hash.len(), 1);
        assert!(hash.contains("foo"));
    }

    {
        let mut hash = HashSet::new();
        map.glob_keys(false, "ba?", |key| assert!(hash.insert(key))).unwrap();
        assert_eq!(hash.len(), 2);
        assert!(hash.contains("bar"));
        assert!(hash.contains("baz"));
    }

    {
        let mut hash = HashSet::new();
        map.glob_keys(true, "ba?", |key| assert!(hash.insert(key))).unwrap();
        assert_eq!(hash.len(), 2);
        assert!(hash.contains("foo"));
        assert!(hash.contains("qux"));
    }

    {
        let mut hash = HashSet::new();
        map.glob_entries(false, "f*", |key, _| assert!(hash.insert(key))).unwrap();
        assert_eq!(hash.len(), 1);
        assert!(hash.contains("foo"));
    }

    {
        let mut hash = HashSet::new();
        map.glob_entries(false, "ba?", |key, _| assert!(hash.insert(key))).unwrap();
        assert_eq!(hash.len(), 2);
        assert!(hash.contains("bar"));
        assert!(hash.contains("baz"));
    }

    {
        let mut hash = HashSet::new();
        map.glob_entries(true, "ba?", |key, _| assert!(hash.insert(key))).unwrap();
        assert_eq!(hash.len(), 2);
        assert!(hash.contains("foo"));
        assert!(hash.contains("qux"));
    }

    map.clear().unwrap();

    assert_eq!(map.len().unwrap(), 0);
    assert!(map.is_empty().unwrap());
    assert!(!map.contains_key("foo").unwrap());
    assert!(!map.contains_key("bar").unwrap());
    assert!(!map.contains_key("baz").unwrap());
    assert!(!map.contains_key("qux").unwrap());
}

#[test]
fn test_sqlite_map_string_i64_transacted() {
    let mut map = SQLiteMap::<String, i64>::new().unwrap();
    let mut tx = map.transaction().unwrap();

    assert_eq!(tx.len().unwrap(), 0);
    assert!(tx.is_empty().unwrap());
    assert!(!tx.contains_key("foo").unwrap());
    assert!(!tx.contains_key("bar").unwrap());
    assert!(!tx.contains_key("baz").unwrap());
    assert!(!tx.contains_key("qux").unwrap());
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
    assert!(tx.contains_key("foo").unwrap());
    assert!(!tx.contains_key("FOO").unwrap());
    assert!(tx.contains_key_nocase("FOO").unwrap());
    assert!(tx.contains_key("bar").unwrap());
    assert!(!tx.contains_key("BAR").unwrap());
    assert!(tx.contains_key_nocase("BAR").unwrap());
    assert!(tx.contains_key("baz").unwrap());
    assert!(!tx.contains_key("BAZ").unwrap());
    assert!(tx.contains_key_nocase("BAZ").unwrap());
    assert!(!tx.contains_key("qux").unwrap());
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
    assert!(!tx.contains_key("foo").unwrap());
    assert!(!tx.contains_key("bar").unwrap());
    assert!(tx.contains_key("baz").unwrap());
    assert!(!tx.contains_key("BAZ").unwrap());
    assert!(tx.contains_key_nocase("BAZ").unwrap());
    assert!(!tx.contains_key("qux").unwrap());
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
    assert!(tx.contains_key("foo").unwrap());
    assert!(!tx.contains_key("FOO").unwrap());
    assert!(tx.contains_key_nocase("FOO").unwrap());
    assert!(!tx.contains_key("bar").unwrap());
    assert!(tx.contains_key("baz").unwrap());
    assert!(!tx.contains_key("BAZ").unwrap());
    assert!(tx.contains_key_nocase("BAZ").unwrap());
    assert!(tx.contains_key("qux").unwrap());
    assert!(!tx.contains_key("QUX").unwrap());
    assert!(tx.contains_key_nocase("QUX").unwrap());
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
        let mut hash = HashSet::new();
        tx.for_each_key(|key| assert!(hash.insert(key.to_owned()))).unwrap();
        assert_eq!(hash.len(), 3);
        assert!(hash.contains("foo"));
        assert!(hash.contains("baz"));
        assert!(hash.contains("qux"));
    }

    {
        let mut hash = HashMap::new();
        tx.for_each_entry(|key, value| assert!(hash.insert(key.to_owned(), value.to_owned()).is_none())).unwrap();
        assert_eq!(hash.len(), 3);
        assert_eq!(hash.get("foo").unwrap(), &0x12345678);
        assert_eq!(hash.get("baz").unwrap(), &0xDEADBEEF);
        assert_eq!(hash.get("qux").unwrap(), &0xCAFEBABE);
    }

    assert!(tx.find_key(|key| key.eq("foo")).unwrap().is_some_and(|key| key.eq("foo")));
    assert!(tx.find_key(|key| key.eq("bar")).unwrap().is_none());
    assert!(tx.find_key(|key| key.eq("baz")).unwrap().is_some_and(|key| key.eq("baz")));
    assert!(tx.find_key(|key| key.eq("qux")).unwrap().is_some_and(|key| key.eq("qux")));

    assert_eq!(tx.find_entry(|key, _| key.eq("foo")).unwrap().unwrap().1, 0x12345678);
    assert_eq!(tx.find_entry(|key, _| key.eq_ignore_ascii_case("FOO")).unwrap().unwrap().1, 0x12345678);
    assert!(tx.find_entry(|key, _| key.eq("bar")).unwrap().is_none());

    tx.update("foo", &0x9ABCDEF0).unwrap();
    tx.update("bar", &0x0FEDCBA9).unwrap();
    tx.update("baz", &0x8BADF00D).unwrap();

    assert_eq!(tx.get("foo").unwrap().unwrap(), 0x9ABCDEF0);
    assert_eq!(tx.get("bar").unwrap().unwrap(), 0x0FEDCBA9);
    assert_eq!(tx.get("baz").unwrap().unwrap(), 0x8BADF00D);
    assert_eq!(tx.get("qux").unwrap().unwrap(), 0xCAFEBABE);

    {
        let mut hash = HashSet::new();
        tx.for_each_key(|key| assert!(hash.insert(key.to_owned()))).unwrap();
        assert_eq!(hash.len(), 4);
        assert!(hash.contains("foo"));
        assert!(hash.contains("bar"));
        assert!(hash.contains("baz"));
        assert!(hash.contains("qux"));
    }

    {
        let mut hash = HashMap::new();
        tx.for_each_entry(|key, value| assert!(hash.insert(key.to_owned(), value.to_owned()).is_none())).unwrap();
        assert_eq!(hash.len(), 4);
        assert_eq!(hash.get("foo").unwrap(), &0x9ABCDEF0);
        assert_eq!(hash.get("bar").unwrap(), &0x0FEDCBA9);
        assert_eq!(hash.get("baz").unwrap(), &0x8BADF00D);
        assert_eq!(hash.get("qux").unwrap(), &0xCAFEBABE);
    }

    assert!(tx.find_key(|key| key.eq("foo")).unwrap().is_some_and(|key| key.eq("foo")));
    assert!(tx.find_key(|key| key.eq("bar")).unwrap().is_some_and(|key| key.eq("bar")));
    assert!(tx.find_key(|key| key.eq("baz")).unwrap().is_some_and(|key| key.eq("baz")));
    assert!(tx.find_key(|key| key.eq("qux")).unwrap().is_some_and(|key| key.eq("qux")));

    assert_eq!(tx.find_entry(|key, _| key.eq("foo")).unwrap().unwrap().1, 0x9ABCDEF0);
    assert_eq!(tx.find_entry(|key, _| key.eq_ignore_ascii_case("FOO")).unwrap().unwrap().1, 0x9ABCDEF0);
    assert_eq!(tx.find_entry(|key, _| key.eq("bar")).unwrap().unwrap().1, 0x0FEDCBA9);
    assert_eq!(tx.find_entry(|key, _| key.eq_ignore_ascii_case("BAR")).unwrap().unwrap().1, 0x0FEDCBA9);
    assert!(tx.find_entry(|key, _| key.eq("xyz")).unwrap().is_none());

    {
        let mut hash = HashSet::new();
        tx.glob_keys(false, "f*", |key| assert!(hash.insert(key))).unwrap();
        assert_eq!(hash.len(), 1);
        assert!(hash.contains("foo"));
    }

    {
        let mut hash = HashSet::new();
        tx.glob_keys(false, "ba?", |key| assert!(hash.insert(key))).unwrap();
        assert_eq!(hash.len(), 2);
        assert!(hash.contains("bar"));
        assert!(hash.contains("baz"));
    }

    {
        let mut hash = HashSet::new();
        tx.glob_keys(true, "ba?", |key| assert!(hash.insert(key))).unwrap();
        assert_eq!(hash.len(), 2);
        assert!(hash.contains("foo"));
        assert!(hash.contains("qux"));
    }

    {
        let mut hash = HashSet::new();
        tx.glob_entries(false, "f*", |key, _| assert!(hash.insert(key))).unwrap();
        assert_eq!(hash.len(), 1);
        assert!(hash.contains("foo"));
    }

    {
        let mut hash = HashSet::new();
        tx.glob_entries(false, "ba?", |key, _| assert!(hash.insert(key))).unwrap();
        assert_eq!(hash.len(), 2);
        assert!(hash.contains("bar"));
        assert!(hash.contains("baz"));
    }

    {
        let mut hash = HashSet::new();
        tx.glob_entries(true, "ba?", |key, _| assert!(hash.insert(key))).unwrap();
        assert_eq!(hash.len(), 2);
        assert!(hash.contains("foo"));
        assert!(hash.contains("qux"));
    }

    tx.clear().unwrap();

    assert_eq!(tx.len().unwrap(), 0);
    assert!(tx.is_empty().unwrap());
    assert!(!tx.contains_key("foo").unwrap());
    assert!(!tx.contains_key("bar").unwrap());
    assert!(!tx.contains_key("baz").unwrap());
    assert!(!tx.contains_key("qux").unwrap());

    assert!(tx.insert("foo", &i64::MAX).unwrap());
    assert!(tx.insert("bar", &i64::MIN).unwrap());

    tx.commit();

    assert_eq!(map.len().unwrap(), 2);
    assert!(!map.is_empty().unwrap());
    assert!(map.contains_key("foo").unwrap());
    assert!(map.contains_key("bar").unwrap());
    assert!(!map.contains_key("baz").unwrap());
    assert!(!map.contains_key("qux").unwrap());
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

    assert!(map_1.contains_key("foo").unwrap());
    assert!(!map_1.contains_key("FOO").unwrap());
    assert!(!map_1.contains_key("bar").unwrap());
    assert!(map_1.contains_key("BAR").unwrap());
    assert_eq!(map_1.get("foo").unwrap().unwrap(), 0x12345678);
    assert!(map_1.get("FOO").unwrap().is_none());
    assert!(map_1.get("bar").unwrap().is_none());
    assert_eq!(map_1.get("BAR").unwrap().unwrap(), 0x9ABCDEF0);

    assert!(map_2.contains_key("foo").unwrap());
    assert!(map_2.contains_key("FOO").unwrap());
    assert!(map_2.contains_key("bar").unwrap());
    assert!(map_2.contains_key("BAR").unwrap());
    assert_eq!(map_2.get("foo").unwrap().unwrap(), 0x12345678);
    assert_eq!(map_2.get("FOO").unwrap().unwrap(), 0x12345678);
    assert_eq!(map_2.get("bar").unwrap().unwrap(), 0x9ABCDEF0);
    assert_eq!(map_2.get("BAR").unwrap().unwrap(), 0x9ABCDEF0);

    assert!(map_2.remove("FOO").unwrap());
    assert!(map_2.remove("bar").unwrap());

    assert!(!map_2.contains_key("foo").unwrap());
    assert!(!map_2.contains_key("FOO").unwrap());
    assert!(!map_2.contains_key("bar").unwrap());
    assert!(!map_2.contains_key("BAR").unwrap());
    assert!(map_2.get("foo").unwrap().is_none());
    assert!(map_2.get("FOO").unwrap().is_none());
    assert!(map_2.get("bar").unwrap().is_none());
    assert!(map_2.get("BAR").unwrap().is_none());
}

#[test]
fn test_sqlite_map_string_i64_stresstest() {
    let mut set = SQLiteMap::<String, i64>::new().unwrap();
    let mut hex = HexEncoder::default();

    {
        let mut tx = set.transaction().unwrap();
        for i in 0u64..MAX_ITEMS {
            let (key, value) = (hex.encode(mix64(i)), mix64(!i) as i64);
            assert!(tx.insert(key, &value).unwrap());
        }
    }

    assert!(!set.is_empty().unwrap());
    assert_eq!(set.len().unwrap().to_u64(), MAX_ITEMS);

    {
        let tx = set.transaction().unwrap();
        for i in 0u64..MAX_ITEMS {
            let (key, value) = (hex.encode(mix64(i)), mix64(!i) as i64);
            assert_eq!(tx.get(key).unwrap().unwrap(), value);
        }
    }

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
fn test_sqlite_map_string_i64_randomized() {
    const COUNT: u64 = 125_000u64;
    let mut set = SQLiteMap::<String, i64>::new().unwrap();
    let mut hex = HexEncoder::default();

    for seed in [0x9DDC5F3146219B97u64, 0x79B9E4DC002B32D6u64] {
        for b in [true, false] {
            let mut xoshiro = Xoshiro256PlusPlus::seed_from_u64(seed);
            let mut tx = set.transaction().unwrap();
            for _i in 0u64..COUNT {
                assert_eq!(tx.insert(hex.encode(xoshiro.next_u64()), &(xoshiro.next_u64() as i64)).unwrap(), b);
            }
        }
    }

    for (seed, b) in [(0x9DDC5F3146219B97u64, true), (0x79B9E4DC002B32D6u64, true), (0x2F381F8B2F0E517Au64, false)] {
        let mut xoshiro = Xoshiro256PlusPlus::seed_from_u64(seed);
        let tx = set.transaction().unwrap();
        for _i in 0u64..COUNT {
            if b {
                assert_eq!(tx.get(hex.encode(xoshiro.next_u64())).unwrap().unwrap(), xoshiro.next_u64() as i64);
            } else {
                assert!(tx.get(hex.encode(xoshiro.next_u64())).unwrap().is_none());
            }
        }
    }
}

// ---------------------------------------------------------------------------
// SQLiteSet File-based Test
// ---------------------------------------------------------------------------

#[test]
fn test_sqlite_map_filebased() {
    let temp_dir = tempdir_in(env!["CARGO_TARGET_TMPDIR"]).unwrap();
    let file_path_1 = temp_dir.path().join("sqlite_map-1.db");
    let file_path_2 = temp_dir.path().join("sqlite_map-2.db");

    assert!(!fs::exists(&file_path_1).unwrap());
    assert!(!fs::exists(&file_path_2).unwrap());

    {
        let mut set = SQLiteMap::<String, String>::with_file(&file_path_1).unwrap();
        {
            let mut tx = set.transaction().unwrap();
            assert!(tx.insert("foo", "12345678").unwrap());
            assert!(tx.insert("baz", "87654321").unwrap());
            assert!(tx.insert("BAZ", "abcdefgh").unwrap());
        }
    }

    {
        let mut set = SQLiteMap::<String, String>::with_file_nocase(&file_path_2).unwrap();
        {
            let mut tx = set.transaction().unwrap();
            assert!(tx.insert("bar", "hgfedcba").unwrap());
            assert!(tx.insert("qux", "stuvwxyz").unwrap());
            assert!(!tx.insert("QUX", "zyxwvuts").unwrap());
        }
    }

    assert!(fs::exists(&file_path_1).unwrap());
    assert!(fs::exists(&file_path_2).unwrap());

    {
        let set = SQLiteMap::<String, String>::with_file(&file_path_1).unwrap();
        assert!(set.get("foo").unwrap().is_some_and(|value| value.eq("12345678")));
        assert!(set.get("FOO").unwrap().is_none());
        assert!(set.get("bar").unwrap().is_none());
        assert!(set.get("baz").unwrap().is_some_and(|value| value.eq("87654321")));
        assert!(set.get("BAZ").unwrap().is_some_and(|value| value.eq("abcdefgh")));
        assert!(set.get("qux").unwrap().is_none());
    }

    {
        let set = SQLiteMap::<String, String>::with_file_nocase(&file_path_2).unwrap();
        assert!(set.get("foo").unwrap().is_none());
        assert!(set.get("bar").unwrap().is_some_and(|value| value.eq("hgfedcba")));
        assert!(set.get("BAR").unwrap().is_some_and(|value| value.eq("hgfedcba")));
        assert!(set.get("baz").unwrap().is_none());
        assert!(set.get("qux").unwrap().is_some_and(|value| value.eq("stuvwxyz")));
        assert!(set.get("QUX").unwrap().is_some_and(|value| value.eq("stuvwxyz")));
    }

    assert!(matches!(SQLiteMap::<String, String>::with_file_nocase(&file_path_1), Err(Error::InvalidSchema)));
    assert!(matches!(SQLiteMap::<String, String>::with_file(&file_path_2), Err(Error::InvalidSchema)));
}

// ---------------------------------------------------------------------------
// SQLiteMap Multi-Thread Tests
// ---------------------------------------------------------------------------

fn sqlite_map_thread_worker<const COUNT: u64>(mut thread_map: SQLiteMap<String, String>, barrier: &Barrier) -> SQLiteMap<String, String> {
    let mut hex_1 = HexEncoder::default();
    let mut hex_2 = HexEncoder::default();

    barrier.wait();

    {
        let mut tx = thread_map.transaction().unwrap();
        for i in 0u64..COUNT {
            let (key, value) = (hex_1.encode(mix64(i)), hex_2.encode(mix64(u64::MAX - i)));
            assert!(tx.insert(key, value).unwrap());
        }
    }

    thread_map
}

#[test]
fn test_sqlite_map_multithreading() {
    const COUNT: u64 = 125_000u64;
    const THREADS: usize = 8usize;

    let maps: [SQLiteMap<String, String>; THREADS] = array::from_fn(|_| SQLiteMap::new().unwrap());
    let barrier = Arc::new(Barrier::new(THREADS));
    let mut handles: Vec<_> = maps
        .into_iter()
        .map(|map| {
            let barrier = Arc::clone(&barrier);
            thread::spawn(move || sqlite_map_thread_worker::<COUNT>(map, &barrier))
        })
        .collect();

    for handle in handles.drain(..) {
        let thread_map = handle.join().unwrap();
        assert_eq!(thread_map.len().unwrap().to_u64(), COUNT);
    }
}

// ---------------------------------------------------------------------------
// SQLiteMap Update-Hook Test
// ---------------------------------------------------------------------------

#[test]
fn test_sqlite_map_updatehook() {
    let mut map = SQLiteMap::<String, String>::new().unwrap();
    let mut hex_1 = HexEncoder::default();
    let mut hex_2 = HexEncoder::default();

    #[cfg(feature = "hooks")]
    let mut action_counter = [0u64, 0u64];

    let p_sieve = primal::Sieve::new(125_000usize);
    let upper_bound = p_sieve.upper_bound() as u64;

    for i in 0u64..=upper_bound {
        if !p_sieve.is_prime(i.try_into().unwrap()) {
            let (key, value) = (hex_1.encode(mix64(i)), hex_2.encode(mix64(!i)));
            cfg_if! {
                if #[cfg(feature = "hooks")] {
                    assert_eq!(map.update(key, value).unwrap(), Action::Insert);
                    action_counter[0] = action_counter[0].checked_add(1u64).unwrap();
                } else {
                    assert_eq!(map.update(key, value).unwrap(), Action::Unknown);
                }
            }
        } else {
            cfg_if! {
                if #[cfg(feature = "hooks")] {
                    action_counter[1] = action_counter[1].checked_add(1u64).unwrap();
                }
            }
        }
    }

    cfg_if! {
        if #[cfg(feature = "hooks")] {
            assert_ne!(action_counter[0], 0u64);
            assert_ne!(action_counter[1], 0u64);
        }
    }

    for i in 0u64..=upper_bound {
        let (key, value) = (hex_1.encode(mix64(i)), hex_2.encode(!mix64(!i)));
        cfg_if! {
            if #[cfg(feature = "hooks")] {
                if !p_sieve.is_prime(i.try_into().unwrap()) {
                    assert_eq!(map.update(key, value).unwrap(), Action::Update);
                    action_counter[0] = action_counter[0].checked_sub(1u64).unwrap();
                } else {
                    assert_eq!(map.update(key, value).unwrap(), Action::Insert);
                    action_counter[1] = action_counter[1].checked_sub(1u64).unwrap();
                }
            } else {
                assert_eq!(map.update(key, value).unwrap(), Action::Unknown);
            }
        }
    }

    cfg_if! {
        if #[cfg(feature = "hooks")] {
            assert_eq!(action_counter[0], 0u64);
            assert_eq!(action_counter[1], 0u64);
        }
    }
}

#[test]
fn test_sqlite_map_updatehook_transacted() {
    let mut map = SQLiteMap::<String, String>::new().unwrap();
    let mut hex_1 = HexEncoder::default();
    let mut hex_2 = HexEncoder::default();

    #[cfg(feature = "hooks")]
    let mut action_counter = [0u64, 0u64];

    let p_sieve = primal::Sieve::new(125_000usize);
    let upper_bound = p_sieve.upper_bound() as u64;

    {
        let mut tx = map.transaction().unwrap();
        for i in 0u64..=upper_bound {
            if !p_sieve.is_prime(i.try_into().unwrap()) {
                let (key, value) = (hex_1.encode(mix64(i)), hex_2.encode(mix64(!i)));
                cfg_if! {
                    if #[cfg(feature = "hooks")] {
                        assert_eq!(tx.update(key, value).unwrap(), Action::Insert);
                        action_counter[0] = action_counter[0].checked_add(1u64).unwrap();
                    } else {
                        assert_eq!(tx.update(key, value).unwrap(), Action::Unknown);
                    }
                }
            } else {
                cfg_if! {
                    if #[cfg(feature = "hooks")] {
                        action_counter[1] = action_counter[1].checked_add(1u64).unwrap();
                    }
                }
            }
        }
    }

    cfg_if! {
        if #[cfg(feature = "hooks")] {
            assert_ne!(action_counter[0], 0u64);
            assert_ne!(action_counter[1], 0u64);
        }
    }

    {
        let mut tx = map.transaction().unwrap();
        for i in 0u64..=upper_bound {
            let (key, value) = (hex_1.encode(mix64(i)), hex_2.encode(!mix64(!i)));
            cfg_if! {
                if #[cfg(feature = "hooks")] {
                    if !p_sieve.is_prime(i.try_into().unwrap()) {
                        assert_eq!(tx.update(key, value).unwrap(), Action::Update);
                        action_counter[0] = action_counter[0].checked_sub(1u64).unwrap();
                    } else {
                        assert_eq!(tx.update(key, value).unwrap(), Action::Insert);
                        action_counter[1] = action_counter[1].checked_sub(1u64).unwrap();
                    }
                } else {
                    assert_eq!(tx.update(key, value).unwrap(), Action::Unknown);
                }
            }
        }
    }

    cfg_if! {
        if #[cfg(feature = "hooks")] {
            assert_eq!(action_counter[0], 0u64);
            assert_eq!(action_counter[1], 0u64);
        }
    }
}

// ---------------------------------------------------------------------------
// SQLiteMap Test to Destruction
// ---------------------------------------------------------------------------

#[ignore]
#[test]
fn test_sqlite_map_to_destruction() {
    let mut set = SQLiteMap::<String, String>::new().unwrap();
    let (channel_tx, channel_rx) = sync_channel::<(String, String)>(1024usize);

    let _handle = thread::spawn(move || {
        let mut hex_1 = HexEncoder::default();
        let mut hex_2 = HexEncoder::default();
        for i in 0u64..=u64::MAX {
            let (key, value) = (hex_1.encode(mix64(i)), hex_2.encode(mix64(!i)));
            if channel_tx.send((key.to_owned(), value.to_owned())).is_err() {
                break;
            }
        }
    });

    let mut tx = set.transaction().unwrap();

    loop {
        let (key, value) = channel_rx.recv().expect("Out of items!");
        match tx.insert(&key, &value) {
            Err(error) => match error.into_inner() {
                Some(sqlite_containers::rusqlite::Error::SqliteFailure(sqlite_error, _)) => match sqlite_error.code {
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
