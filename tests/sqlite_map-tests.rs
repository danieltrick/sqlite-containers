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
use sqlite_containers::{
    Action, Blob, Error, IterationError, SQLiteMap, SizeExt, TryClone,
    rusqlite::{
        Error as RusqliteError,
        ffi::{Error as FfiError, ErrorCode as FfiErrorCode},
    },
};
use std::{
    array,
    collections::{HashMap, HashSet},
    fs,
    sync::{Arc, Barrier, LazyLock, mpsc::sync_channel},
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

    assert!(map.insert(0i64, "12345678".to_owned()).unwrap());
    assert!(map.insert(42i64, "87654321".to_owned()).unwrap());
    assert!(!map.insert(0i64, "abcdefgh".to_owned()).unwrap());
    assert!(!map.insert(42i64, "hgfedcba".to_owned()).unwrap());
    assert!(map.insert(i64::MAX, "deadbeef".to_owned()).unwrap());
    assert!(!map.insert(i64::MAX, "stuvwxyz".to_owned()).unwrap());

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

    assert!(map.insert(i64::MIN, "cafebabe".to_owned()).unwrap());
    assert!(!map.insert(i64::MIN, "cafebabe".to_owned()).unwrap());
    assert!(map.insert(0i64, "12345678".to_owned()).unwrap());
    assert!(!map.insert(0i64, "12345678".to_owned()).unwrap());

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
        let mut hash = HashSet::new();
        map.try_for_each_key(|key| {
            assert!(hash.insert(key.to_owned()));
            Ok::<_, ()>(())
        })
        .unwrap();
        assert_eq!(hash.len(), 3);
        assert!(hash.contains(&0i64));
        assert!(hash.contains(&i64::MAX));
        assert!(hash.contains(&i64::MIN));
    }

    assert!(matches!(map.try_for_each_key(|_key| Err("Failed!")), Err(IterationError::CallbackError(_))));

    {
        let mut hash = HashMap::new();
        map.for_each_entry(|key, value| assert!(hash.insert(key.to_owned(), value.to_owned()).is_none())).unwrap();
        assert_eq!(hash.len(), 3);
        assert_eq!(hash.get(&0i64).unwrap(), "12345678");
        assert_eq!(hash.get(&i64::MAX).unwrap(), "deadbeef");
        assert_eq!(hash.get(&i64::MIN).unwrap(), "cafebabe");
    }

    {
        let mut hash = HashMap::new();
        map.try_for_each_entry(|key, value| {
            assert!(hash.insert(key.to_owned(), value.to_owned()).is_none());
            Ok::<_, ()>(())
        })
        .unwrap();
        assert_eq!(hash.len(), 3);
        assert_eq!(hash.get(&0i64).unwrap(), "12345678");
        assert_eq!(hash.get(&i64::MAX).unwrap(), "deadbeef");
        assert_eq!(hash.get(&i64::MIN).unwrap(), "cafebabe");
    }

    assert!(matches!(map.try_for_each_entry(|_key, _value| Err("Failed!")), Err(IterationError::CallbackError(_))));

    assert!(map.find_key(|key| key.eq(&0i64)).unwrap().is_some_and(|key| key == 0i64));
    assert!(map.find_key(|key| key.eq(&42i64)).unwrap().is_none());
    assert!(map.find_key(|key| key.eq(&i64::MAX)).unwrap().is_some_and(|key| key == i64::MAX));
    assert!(map.find_key(|key| key.eq(&i64::MIN)).unwrap().is_some_and(|key| key == i64::MIN));

    assert_eq!(map.find_entry(|key, _| key.eq(&0i64)).unwrap().unwrap().1, "12345678");
    assert!(map.find_entry(|key, _| key.eq(&42i64)).unwrap().is_none());
    assert_eq!(map.find_entry(|key, _| key.eq(&i64::MAX)).unwrap().unwrap().1, "deadbeef");
    assert_eq!(map.find_entry(|key, _| key.eq(&i64::MIN)).unwrap().unwrap().1, "cafebabe");

    map.update(0i64, "abcdefgh".to_owned()).unwrap();
    map.update(42i64, "hgfedcba".to_owned()).unwrap();
    map.update(i64::MAX, "stuvwxyz".to_owned()).unwrap();

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
    assert!(!map.contains_key(&8191i64).unwrap());
    assert!(!map.contains_key(&131071i64).unwrap());
    assert!(!map.contains_key(&i64::MAX).unwrap());
    assert!(!map.contains_key(&i64::MIN).unwrap());

    assert!(map.insert(0i64, "ijklmnop".to_owned()).unwrap());
    assert!(map.insert(42i64, "ponmlkji".to_owned()).unwrap());

    assert_eq!(
        map.extend(
            false,
            [(42i64, "abcdefgh".to_owned()), (8191i64, "hgfedcba".to_owned()), (131071i64, "stuvwxyz".to_owned()), (i64::MAX, "zyxwvuts".to_owned())]
        )
        .unwrap(),
        Some((3, 0))
    );

    assert_eq!(map.len().unwrap(), 5);
    assert!(!map.is_empty().unwrap());
    assert!(map.contains_key(&0i64).unwrap());
    assert!(map.contains_key(&42i64).unwrap());
    assert!(!map.contains_key(&524287).unwrap());
    assert!(map.contains_key(&8191i64).unwrap());
    assert!(map.contains_key(&131071i64).unwrap());
    assert!(map.contains_key(&i64::MAX).unwrap());
    assert!(!map.contains_key(&i64::MIN).unwrap());
    assert!(map.get(&0i64).unwrap().is_some_and(|value| value.eq("ijklmnop")));
    assert!(map.get(&42i64).unwrap().is_some_and(|value| value.eq("ponmlkji")));
    assert!(map.get(&524287).unwrap().is_none());
    assert!(map.get(&8191i64).unwrap().is_some_and(|value| value.eq("hgfedcba")));
    assert!(map.get(&131071i64).unwrap().is_some_and(|value| value.eq("stuvwxyz")));
    assert!(map.get(&i64::MAX).unwrap().is_some_and(|value| value.eq("zyxwvuts")));
    assert!(map.get(&i64::MIN).unwrap().is_none());

    assert_eq!(map.extend_ref(false, [(&42i64, "deadbeef"), (&i64::MIN, "cafebabe"), (&42i64, "dead10cc")]).unwrap(), Some((1, 0)));

    assert_eq!(map.len().unwrap(), 6);
    assert!(!map.is_empty().unwrap());
    assert!(map.contains_key(&0i64).unwrap());
    assert!(map.contains_key(&42i64).unwrap());
    assert!(!map.contains_key(&524287).unwrap());
    assert!(map.contains_key(&8191i64).unwrap());
    assert!(map.contains_key(&131071i64).unwrap());
    assert!(map.contains_key(&i64::MAX).unwrap());
    assert!(map.contains_key(&i64::MIN).unwrap());
    assert!(map.get(&0i64).unwrap().is_some_and(|value| value.eq("ijklmnop")));
    assert!(map.get(&42i64).unwrap().is_some_and(|value| value.eq("ponmlkji")));
    assert!(map.get(&524287).unwrap().is_none());
    assert!(map.get(&8191i64).unwrap().is_some_and(|value| value.eq("hgfedcba")));
    assert!(map.get(&131071i64).unwrap().is_some_and(|value| value.eq("stuvwxyz")));
    assert!(map.get(&i64::MAX).unwrap().is_some_and(|value| value.eq("zyxwvuts")));
    assert!(map.get(&i64::MIN).unwrap().is_some_and(|value| value.eq("cafebabe")));

    assert!(map.take(&524287).unwrap().is_none());
    assert!(map.take(&8191i64).unwrap().is_some_and(|value| value.eq("hgfedcba")));
    assert!(map.take(&i64::MAX).unwrap().is_some_and(|value| value.eq("zyxwvuts")));
    assert!(!map.contains_key(&8191i64).unwrap());
    assert!(!map.contains_key(&i64::MAX).unwrap());
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

    assert!(tx.insert(0i64, "12345678".to_owned()).unwrap());
    assert!(tx.insert(42i64, "87654321".to_owned()).unwrap());
    assert!(!tx.insert(0i64, "abcdefgh".to_owned()).unwrap());
    assert!(!tx.insert(42i64, "hgfedcba".to_owned()).unwrap());
    assert!(tx.insert(i64::MAX, "deadbeef".to_owned()).unwrap());
    assert!(!tx.insert(i64::MAX, "stuvwxyz".to_owned()).unwrap());

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

    assert!(tx.insert(i64::MIN, "cafebabe".to_owned()).unwrap());
    assert!(!tx.insert(i64::MIN, "cafebabe".to_owned()).unwrap());
    assert!(tx.insert(0i64, "12345678".to_owned()).unwrap());
    assert!(!tx.insert(0i64, "12345678".to_owned()).unwrap());

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
        let mut hash = HashSet::new();
        tx.try_for_each_key(|key| {
            assert!(hash.insert(key.to_owned()));
            Ok::<_, ()>(())
        })
        .unwrap();
        assert_eq!(hash.len(), 3);
        assert!(hash.contains(&0i64));
        assert!(hash.contains(&i64::MAX));
        assert!(hash.contains(&i64::MIN));
    }

    assert!(matches!(tx.try_for_each_key(|_key| Err("Failed!")), Err(IterationError::CallbackError(_))));

    {
        let mut hash = HashMap::new();
        tx.for_each_entry(|key, value| assert!(hash.insert(key.to_owned(), value.to_owned()).is_none())).unwrap();
        assert_eq!(hash.len(), 3);
        assert_eq!(hash.get(&0i64).unwrap(), "12345678");
        assert_eq!(hash.get(&i64::MAX).unwrap(), "deadbeef");
        assert_eq!(hash.get(&i64::MIN).unwrap(), "cafebabe");
    }

    {
        let mut hash = HashMap::new();
        tx.try_for_each_entry(|key, value| {
            assert!(hash.insert(key.to_owned(), value.to_owned()).is_none());
            Ok::<_, ()>(())
        })
        .unwrap();
        assert_eq!(hash.len(), 3);
        assert_eq!(hash.get(&0i64).unwrap(), "12345678");
        assert_eq!(hash.get(&i64::MAX).unwrap(), "deadbeef");
        assert_eq!(hash.get(&i64::MIN).unwrap(), "cafebabe");
    }

    assert!(matches!(tx.try_for_each_entry(|_key, _value| Err("Failed!")), Err(IterationError::CallbackError(_))));

    assert!(tx.find_key(|key| key.eq(&0i64)).unwrap().is_some_and(|key| key == 0i64));
    assert!(tx.find_key(|key| key.eq(&42i64)).unwrap().is_none());
    assert!(tx.find_key(|key| key.eq(&i64::MAX)).unwrap().is_some_and(|key| key == i64::MAX));
    assert!(tx.find_key(|key| key.eq(&i64::MIN)).unwrap().is_some_and(|key| key == i64::MIN));

    assert_eq!(tx.find_entry(|key, _| key.eq(&0i64)).unwrap().unwrap().1, "12345678");
    assert!(tx.find_entry(|key, _| key.eq(&42i64)).unwrap().is_none());
    assert_eq!(tx.find_entry(|key, _| key.eq(&i64::MAX)).unwrap().unwrap().1, "deadbeef");
    assert_eq!(tx.find_entry(|key, _| key.eq(&i64::MIN)).unwrap().unwrap().1, "cafebabe");

    tx.update(0i64, "abcdefgh".to_owned()).unwrap();
    tx.update(42i64, "hgfedcba".to_owned()).unwrap();
    tx.update(i64::MAX, "stuvwxyz".to_owned()).unwrap();

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
    assert!(!tx.contains_key(&8191i64).unwrap());
    assert!(!tx.contains_key(&131071i64).unwrap());
    assert!(!tx.contains_key(&i64::MAX).unwrap());
    assert!(!tx.contains_key(&i64::MIN).unwrap());

    assert!(tx.insert(0i64, "ijklmnop".to_owned()).unwrap());
    assert!(tx.insert(42i64, "ponmlkji".to_owned()).unwrap());

    assert_eq!(
        tx.extend(
            false,
            [(42i64, "abcdefgh".to_owned()), (8191i64, "hgfedcba".to_owned()), (131071i64, "stuvwxyz".to_owned()), (i64::MAX, "zyxwvuts".to_owned())]
        )
        .unwrap(),
        Some((3, 0))
    );

    assert_eq!(tx.len().unwrap(), 5);
    assert!(!tx.is_empty().unwrap());
    assert!(tx.contains_key(&0i64).unwrap());
    assert!(tx.contains_key(&42i64).unwrap());
    assert!(!tx.contains_key(&524287).unwrap());
    assert!(tx.contains_key(&8191i64).unwrap());
    assert!(tx.contains_key(&131071i64).unwrap());
    assert!(tx.contains_key(&i64::MAX).unwrap());
    assert!(!tx.contains_key(&i64::MIN).unwrap());
    assert!(tx.get(&0i64).unwrap().is_some_and(|value| value.eq("ijklmnop")));
    assert!(tx.get(&42i64).unwrap().is_some_and(|value| value.eq("ponmlkji")));
    assert!(tx.get(&524287).unwrap().is_none());
    assert!(tx.get(&8191i64).unwrap().is_some_and(|value| value.eq("hgfedcba")));
    assert!(tx.get(&131071i64).unwrap().is_some_and(|value| value.eq("stuvwxyz")));
    assert!(tx.get(&i64::MAX).unwrap().is_some_and(|value| value.eq("zyxwvuts")));
    assert!(tx.get(&i64::MIN).unwrap().is_none());

    assert_eq!(tx.extend_ref(false, [(&42i64, "deadbeef"), (&i64::MIN, "cafebabe"), (&42i64, "dead10cc")]).unwrap(), Some((1, 0)));

    assert!(tx.take(&524287).unwrap().is_none());
    assert!(tx.take(&8191i64).unwrap().is_some_and(|value| value.eq("hgfedcba")));
    assert!(tx.take(&i64::MAX).unwrap().is_some_and(|value| value.eq("zyxwvuts")));

    tx.commit();

    assert_eq!(map.len().unwrap(), 4);
    assert!(!map.is_empty().unwrap());
    assert!(map.contains_key(&0i64).unwrap());
    assert!(map.contains_key(&42i64).unwrap());
    assert!(!map.contains_key(&524287).unwrap());
    assert!(!map.contains_key(&8191i64).unwrap());
    assert!(map.contains_key(&131071i64).unwrap());
    assert!(!map.contains_key(&i64::MAX).unwrap());
    assert!(map.contains_key(&i64::MIN).unwrap());
    assert!(map.get(&0i64).unwrap().is_some_and(|value| value.eq("ijklmnop")));
    assert!(map.get(&42i64).unwrap().is_some_and(|value| value.eq("ponmlkji")));
    assert!(map.get(&524287).unwrap().is_none());
    assert!(map.get(&8191i64).unwrap().is_none());
    assert!(map.get(&131071i64).unwrap().is_some_and(|value| value.eq("stuvwxyz")));
    assert!(map.get(&i64::MAX).unwrap().is_none());
    assert!(map.get(&i64::MIN).unwrap().is_some_and(|value| value.eq("cafebabe")));
}

#[test]
fn test_sqlite_map_i64_string_from_entries() {
    static ENTRIES: LazyLock<[(i64, String); 6]> = LazyLock::new(|| {
        [
            (0i64, "abcdefgh".to_owned()),
            (42i64, "hgfedcba".to_owned()),
            (8191i64, "ijklmnop".to_owned()),
            (42i64, "ponmlkji".to_owned()),
            (i64::MIN, "stuvwxyz".to_owned()),
            (i64::MAX, "zyxwvuts".to_owned()),
        ]
    });

    let map = SQLiteMap::<i64, String>::try_from(ENTRIES.clone()).unwrap();

    assert_eq!(map.len().unwrap(), 5);
    assert!(map.get(&0i64).unwrap().is_some_and(|value| value.eq("abcdefgh")));
    assert!(map.get(&1i64).unwrap().is_none());
    assert!(map.get(&42i64).unwrap().is_some_and(|value| value.eq("hgfedcba")));
    assert!(map.get(&8191i64).unwrap().is_some_and(|value| value.eq("ijklmnop")));
    assert!(map.get(&131071i64).unwrap().is_none());
    assert!(map.get(&i64::MIN).unwrap().is_some_and(|value| value.eq("stuvwxyz")));
    assert!(map.get(&i64::MAX).unwrap().is_some_and(|value| value.eq("zyxwvuts")));
}

#[test]
fn test_sqlite_map_i64_string_stresstest() {
    let mut map = SQLiteMap::<i64, String>::new().unwrap();
    let mut hex = HexEncoder::default();

    {
        let mut tx = map.transaction().unwrap();
        for i in 0u64..MAX_ITEMS {
            let (key, value) = (mix64(i) as i64, hex.encode(mix64(!i)));
            assert!(tx.insert_ref(&key, value).unwrap());
        }
    }

    assert!(!map.is_empty().unwrap());
    assert_eq!(map.len().unwrap().to_u64(), MAX_ITEMS);

    {
        let tx = map.transaction().unwrap();
        for i in 0u64..MAX_ITEMS {
            let (key, value) = (mix64(i) as i64, hex.encode(mix64(!i)));
            assert_eq!(tx.get(&key).unwrap().unwrap(), value);
        }
    }

    {
        let mut tx = map.transaction().unwrap();
        for i in 0u64..MAX_ITEMS {
            assert!(tx.remove(&(mix64(i) as i64)).unwrap());
        }
    }

    assert!(map.is_empty().unwrap());
    assert_eq!(map.len().unwrap().to_u64(), 0);
}

#[test]
fn test_sqlite_map_i64_string_randomized() {
    const COUNT: u64 = 75_000u64;
    let mut map = SQLiteMap::<i64, String>::new().unwrap();
    let mut hex = HexEncoder::default();

    for (seed, t) in [(0x9DDC5F3146219B97u64, true), (0x79B9E4DC002B32D6u64, false)] {
        for b in [true, false] {
            let mut xoshiro = Xoshiro256PlusPlus::seed_from_u64(seed);
            if !t {
                for _i in 0u64..COUNT {
                    let (key, value) = (xoshiro.next_u64() as i64, hex.encode(xoshiro.next_u64()));
                    assert_eq!(map.insert_ref(&key, value).unwrap(), b);
                }
            } else {
                let mut tx = map.transaction().unwrap();
                for _i in 0u64..COUNT {
                    let (key, value) = (xoshiro.next_u64() as i64, hex.encode(xoshiro.next_u64()));
                    assert_eq!(tx.insert_ref(&key, value).unwrap(), b);
                }
            }
        }
    }

    for (seed, b) in [(0x9DDC5F3146219B97u64, true), (0x79B9E4DC002B32D6u64, true), (0x2F381F8B2F0E517Au64, false)] {
        let mut xoshiro = Xoshiro256PlusPlus::seed_from_u64(seed);
        let tx = map.transaction().unwrap();
        for _i in 0u64..COUNT {
            if b {
                let (key, value) = (xoshiro.next_u64() as i64, hex.encode(xoshiro.next_u64()));
                assert_eq!(tx.get(&key).unwrap().unwrap(), value);
            } else {
                assert!(tx.get(&(xoshiro.next_u64() as i64)).unwrap().is_none());
                let _skip = xoshiro.next_u64();
            }
        }
    }
}

#[test]
fn test_sqlite_map_i64_string_extend() {
    let mut map_1 = SQLiteMap::<i64, String>::new().unwrap();
    let mut map_2 = SQLiteMap::<i64, String>::new().unwrap();
    let mut map_3 = SQLiteMap::<i64, String>::new().unwrap();

    {
        let mut tx = map_1.transaction().unwrap();
        assert!(tx.insert(0i64, "12345678".to_owned()).unwrap());
        assert!(tx.insert(42i64, "87654321".to_owned()).unwrap());
        assert!(tx.insert(2147483647i64, "abcdefgh".to_owned()).unwrap());
    }

    {
        let mut tx = map_2.transaction().unwrap();
        assert!(tx.insert(0i64, "12345678".to_owned()).unwrap());
        assert!(tx.insert(42i64, "87654321".to_owned()).unwrap());
        assert!(tx.insert(2147483647i64, "abcdefgh".to_owned()).unwrap());
    }

    {
        let mut tx = map_3.transaction().unwrap();
        assert!(tx.insert(0i64, "hgfedcba".to_owned()).unwrap());
        assert!(tx.insert(i64::MIN, "stuvwxyz".to_owned()).unwrap());
        assert!(tx.insert(i64::MAX, "zyxwvuts".to_owned()).unwrap());
        assert!(tx.insert(2305843009213693951i64, "deadbeef".to_owned()).unwrap());
    }

    assert!(map_1.get(&0i64).unwrap().is_some_and(|value| value.eq("12345678")));
    assert!(map_1.get(&42i64).unwrap().is_some_and(|value| value.eq("87654321")));
    assert!(map_1.get(&524287).unwrap().is_none());
    assert!(map_1.get(&2147483647i64).unwrap().is_some_and(|value| value.eq("abcdefgh")));
    assert!(map_1.get(&2305843009213693951i64).unwrap().is_none());
    assert!(map_1.get(&i64::MIN).unwrap().is_none());
    assert!(map_1.get(&i64::MAX).unwrap().is_none());

    assert_eq!(map_1.extend_map(&map_3, false).unwrap(), Some((3, 0)));

    assert!(map_1.get(&0i64).unwrap().is_some_and(|value| value.eq("12345678")));
    assert!(map_1.get(&42i64).unwrap().is_some_and(|value| value.eq("87654321")));
    assert!(map_1.get(&524287).unwrap().is_none());
    assert!(map_1.get(&2147483647i64).unwrap().is_some_and(|value| value.eq("abcdefgh")));
    assert!(map_1.get(&2305843009213693951i64).unwrap().is_some_and(|value| value.eq("deadbeef")));
    assert!(map_1.get(&i64::MIN).unwrap().is_some_and(|value| value.eq("stuvwxyz")));
    assert!(map_1.get(&i64::MAX).unwrap().is_some_and(|value| value.eq("zyxwvuts")));

    assert!(map_2.get(&0i64).unwrap().is_some_and(|value| value.eq("12345678")));
    assert!(map_2.get(&42i64).unwrap().is_some_and(|value| value.eq("87654321")));
    assert!(map_2.get(&524287).unwrap().is_none());
    assert!(map_2.get(&2147483647i64).unwrap().is_some_and(|value| value.eq("abcdefgh")));
    assert!(map_2.get(&2305843009213693951i64).unwrap().is_none());
    assert!(map_2.get(&i64::MIN).unwrap().is_none());
    assert!(map_2.get(&i64::MAX).unwrap().is_none());

    cfg_if! {
        if #[cfg(feature = "hooks")] {
            assert_eq!(map_2.extend_map(&map_3, true).unwrap(), Some((3, 1)));
        } else {
            assert_eq!(map_2.extend_map(&map_3, true).unwrap(), None);
        }
    }

    assert!(map_2.get(&0i64).unwrap().is_some_and(|value| value.eq("hgfedcba")));
    assert!(map_2.get(&42i64).unwrap().is_some_and(|value| value.eq("87654321")));
    assert!(map_2.get(&524287).unwrap().is_none());
    assert!(map_2.get(&2147483647i64).unwrap().is_some_and(|value| value.eq("abcdefgh")));
    assert!(map_2.get(&2305843009213693951i64).unwrap().is_some_and(|value| value.eq("deadbeef")));
    assert!(map_2.get(&i64::MIN).unwrap().is_some_and(|value| value.eq("stuvwxyz")));
    assert!(map_2.get(&i64::MAX).unwrap().is_some_and(|value| value.eq("zyxwvuts")));
}

#[test]
fn test_sqlite_map_i64_string_update() {
    let mut map = SQLiteMap::<i64, String>::new().unwrap();

    {
        let mut tx = map.transaction().unwrap();
        assert!(tx.insert(0i64, "12345678".to_owned()).unwrap());
        assert!(tx.insert(42i64, "87654321".to_owned()).unwrap());
        assert!(tx.insert(2147483647i64, "abcdefgh".to_owned()).unwrap());
    }

    let result_1 = map.extend(true, [(0i64, "hgfedcba".to_owned()), (42i64, "stuvwxyz".to_owned()), (i64::MAX, "zyxwvuts".to_owned())]).unwrap();
    cfg_if! {
        if #[cfg(feature = "hooks")] {
            assert_eq!(result_1, Some((1, 2)));
        } else {
            assert_eq!(result_1, None);
        }
    }

    assert!(map.get(&0i64).unwrap().is_some_and(|value| value.eq("hgfedcba")));
    assert!(map.get(&42i64).unwrap().is_some_and(|value| value.eq("stuvwxyz")));
    assert!(map.get(&2147483647i64).unwrap().is_some_and(|value| value.eq("abcdefgh")));
    assert!(map.get(&i64::MAX).unwrap().is_some_and(|value| value.eq("zyxwvuts")));

    let result_2 = map.extend_ref(true, [(&0i64, "deadbeef"), (&42i64, "cafebabe"), (&i64::MIN, "c0deba5e")]).unwrap();
    cfg_if! {
        if #[cfg(feature = "hooks")] {
            assert_eq!(result_2, Some((1, 2)));
        } else {
            assert_eq!(result_2, None);
        }
    }

    assert!(map.get(&0i64).unwrap().is_some_and(|value| value.eq("deadbeef")));
    assert!(map.get(&42i64).unwrap().is_some_and(|value| value.eq("cafebabe")));
    assert!(map.get(&2147483647i64).unwrap().is_some_and(|value| value.eq("abcdefgh")));
    assert!(map.get(&i64::MIN).unwrap().is_some_and(|value| value.eq("c0deba5e")));
    assert!(map.get(&i64::MAX).unwrap().is_some_and(|value| value.eq("zyxwvuts")));
}

#[cfg_attr(debug_assertions, ignore)]
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
                    assert!(tx.insert_ref(&hash, word).unwrap());
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

    assert!(map.insert(b"foo".to_vec(), "12345678".to_owned()).unwrap());
    assert!(map.insert(b"bar".to_vec(), "87654321".to_owned()).unwrap());
    assert!(!map.insert(b"foo".to_vec(), "abcdefgh".to_owned()).unwrap());
    assert!(!map.insert(b"bar".to_vec(), "hgfedcba".to_owned()).unwrap());
    assert!(map.insert(b"baz".to_vec(), "deadbeef".to_owned()).unwrap());
    assert!(!map.insert(b"baz".to_vec(), "stuvwxyz".to_owned()).unwrap());

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

    assert!(map.insert(b"qux".to_vec(), "cafebabe".to_owned()).unwrap());
    assert!(!map.insert(b"qux".to_vec(), "cafebabe".to_owned()).unwrap());
    assert!(map.insert(b"foo".to_vec(), "12345678".to_owned()).unwrap());
    assert!(!map.insert(b"foo".to_vec(), "12345678".to_owned()).unwrap());

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
        let mut hash = HashSet::new();
        map.try_for_each_key(|key| {
            assert!(hash.insert(key.to_owned()));
            Ok::<_, ()>(())
        })
        .unwrap();
        assert_eq!(hash.len(), 3);
        assert!(hash.contains(&b"foo"[..]));
        assert!(hash.contains(&b"baz"[..]));
        assert!(hash.contains(&b"qux"[..]));
    }

    assert!(matches!(map.try_for_each_key(|_key| Err("Failed!")), Err(IterationError::CallbackError(_))));

    {
        let mut hash = HashMap::new();
        map.for_each_entry(|key, value| assert!(hash.insert(key.to_owned(), value.to_owned()).is_none())).unwrap();
        assert_eq!(hash.len(), 3);
        assert_eq!(hash.get(&b"foo"[..]).unwrap(), "12345678");
        assert_eq!(hash.get(&b"baz"[..]).unwrap(), "deadbeef");
        assert_eq!(hash.get(&b"qux"[..]).unwrap(), "cafebabe");
    }

    {
        let mut hash = HashMap::new();
        map.try_for_each_entry(|key, value| {
            assert!(hash.insert(key.to_owned(), value.to_owned()).is_none());
            Ok::<_, ()>(())
        })
        .unwrap();
        assert_eq!(hash.len(), 3);
        assert_eq!(hash.get(&b"foo"[..]).unwrap(), "12345678");
        assert_eq!(hash.get(&b"baz"[..]).unwrap(), "deadbeef");
        assert_eq!(hash.get(&b"qux"[..]).unwrap(), "cafebabe");
    }

    assert!(matches!(map.try_for_each_entry(|_key, _value| Err("Failed!")), Err(IterationError::CallbackError(_))));

    assert!(map.find_key(|key| key.eq(b"foo")).unwrap().is_some_and(|key| key.eq(b"foo")));
    assert!(map.find_key(|key| key.eq(b"bar")).unwrap().is_none());
    assert!(map.find_key(|key| key.eq(b"baz")).unwrap().is_some_and(|key| key.eq(b"baz")));
    assert!(map.find_key(|key| key.eq(b"qux")).unwrap().is_some_and(|key| key.eq(b"qux")));

    assert_eq!(map.find_entry(|key, _| key.eq(b"foo")).unwrap().unwrap().1, "12345678");
    assert_eq!(map.find_entry(|key, _| key.eq_ignore_ascii_case(b"FOO")).unwrap().unwrap().1, "12345678");
    assert!(map.find_entry(|key, _| key.eq(b"bar")).unwrap().is_none());

    map.update(b"foo".to_vec(), "abcdefgh".to_owned()).unwrap();
    map.update(b"bar".to_vec(), "hgfedcba".to_owned()).unwrap();
    map.update(b"baz".to_vec(), "stuvwxyz".to_owned()).unwrap();

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
    assert!(!map.contains_key(&b"xyz"[..]).unwrap());

    assert!(map.insert(b"foo".to_vec(), "ijklmnop".to_owned()).unwrap());
    assert!(map.insert(b"baz".to_vec(), "ponmlkji".to_owned()).unwrap());

    assert_eq!(
        map.extend(false, [(b"bar".to_vec(), "abcdefgh".to_owned()), (b"baz".to_vec(), "hgfedcba".to_owned()), (b"qux".to_vec(), "stuvwxyz".to_owned())])
            .unwrap(),
        Some((2, 0))
    );

    assert_eq!(map.len().unwrap(), 4);
    assert!(!map.is_empty().unwrap());
    assert!(map.contains_key(&b"foo"[..]).unwrap());
    assert!(map.contains_key(&b"bar"[..]).unwrap());
    assert!(map.contains_key(&b"baz"[..]).unwrap());
    assert!(map.contains_key(&b"qux"[..]).unwrap());
    assert!(!map.contains_key(&b"xyz"[..]).unwrap());
    assert!(!map.contains_key(&b"$?!"[..]).unwrap());
    assert!(map.get(&b"foo"[..]).unwrap().is_some_and(|value| value.eq("ijklmnop")));
    assert!(map.get(&b"bar"[..]).unwrap().is_some_and(|value| value.eq("abcdefgh")));
    assert!(map.get(&b"baz"[..]).unwrap().is_some_and(|value| value.eq("ponmlkji")));
    assert!(map.get(&b"qux"[..]).unwrap().is_some_and(|value| value.eq("stuvwxyz")));
    assert!(map.get(&b"xyz"[..]).unwrap().is_none());
    assert!(map.get(&b"$?!"[..]).unwrap().is_none());

    assert_eq!(map.extend_ref(false, [(&b"foo"[..], "deadbeef"), (&b"xyz"[..], "cafebabe"), (&b"qux"[..], "dead10cc")]).unwrap(), Some((1, 0)));

    assert_eq!(map.len().unwrap(), 5);
    assert!(!map.is_empty().unwrap());
    assert!(map.contains_key(&b"foo"[..]).unwrap());
    assert!(map.contains_key(&b"bar"[..]).unwrap());
    assert!(map.contains_key(&b"baz"[..]).unwrap());
    assert!(map.contains_key(&b"qux"[..]).unwrap());
    assert!(map.contains_key(&b"xyz"[..]).unwrap());
    assert!(!map.contains_key(&b"$?!"[..]).unwrap());
    assert!(map.get(&b"foo"[..]).unwrap().is_some_and(|value| value.eq("ijklmnop")));
    assert!(map.get(&b"bar"[..]).unwrap().is_some_and(|value| value.eq("abcdefgh")));
    assert!(map.get(&b"baz"[..]).unwrap().is_some_and(|value| value.eq("ponmlkji")));
    assert!(map.get(&b"qux"[..]).unwrap().is_some_and(|value| value.eq("stuvwxyz")));
    assert!(map.get(&b"xyz"[..]).unwrap().is_some_and(|value| value.eq("cafebabe")));
    assert!(map.get(&b"$?!"[..]).unwrap().is_none());

    assert!(map.take(&b"$?!"[..]).unwrap().is_none());
    assert!(map.take(&b"qux"[..]).unwrap().is_some_and(|value| value.eq("stuvwxyz")));
    assert!(map.take(&b"bar"[..]).unwrap().is_some_and(|value| value.eq("abcdefgh")));
    assert!(!map.contains_key(&b"qux"[..]).unwrap());
    assert!(!map.contains_key(&b"bar"[..]).unwrap());
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

    assert!(tx.insert(b"foo".to_vec(), "12345678".to_owned()).unwrap());
    assert!(tx.insert(b"bar".to_vec(), "87654321".to_owned()).unwrap());
    assert!(!tx.insert(b"foo".to_vec(), "abcdefgh".to_owned()).unwrap());
    assert!(!tx.insert(b"bar".to_vec(), "hgfedcba".to_owned()).unwrap());
    assert!(tx.insert(b"baz".to_vec(), "deadbeef".to_owned()).unwrap());
    assert!(!tx.insert(b"baz".to_vec(), "stuvwxyz".to_owned()).unwrap());

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

    assert!(tx.insert(b"qux".to_vec(), "cafebabe".to_owned()).unwrap());
    assert!(!tx.insert(b"qux".to_vec(), "cafebabe".to_owned()).unwrap());
    assert!(tx.insert(b"foo".to_vec(), "12345678".to_owned()).unwrap());
    assert!(!tx.insert(b"foo".to_vec(), "12345678".to_owned()).unwrap());

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
        let mut hash = HashSet::new();
        tx.try_for_each_key(|key| {
            assert!(hash.insert(key.to_owned()));
            Ok::<_, ()>(())
        })
        .unwrap();
        assert_eq!(hash.len(), 3);
        assert!(hash.contains(&b"foo"[..]));
        assert!(hash.contains(&b"baz"[..]));
        assert!(hash.contains(&b"qux"[..]));
    }

    assert!(matches!(tx.try_for_each_key(|_key| Err("Failed!")), Err(IterationError::CallbackError(_))));

    {
        let mut hash = HashMap::new();
        tx.for_each_entry(|key, value| assert!(hash.insert(key.to_owned(), value.to_owned()).is_none())).unwrap();
        assert_eq!(hash.len(), 3);
        assert_eq!(hash.get(&b"foo"[..]).unwrap(), "12345678");
        assert_eq!(hash.get(&b"baz"[..]).unwrap(), "deadbeef");
        assert_eq!(hash.get(&b"qux"[..]).unwrap(), "cafebabe");
    }

    {
        let mut hash = HashMap::new();
        tx.try_for_each_entry(|key, value| {
            assert!(hash.insert(key.to_owned(), value.to_owned()).is_none());
            Ok::<_, ()>(())
        })
        .unwrap();
        assert_eq!(hash.len(), 3);
        assert_eq!(hash.get(&b"foo"[..]).unwrap(), "12345678");
        assert_eq!(hash.get(&b"baz"[..]).unwrap(), "deadbeef");
        assert_eq!(hash.get(&b"qux"[..]).unwrap(), "cafebabe");
    }

    assert!(matches!(tx.try_for_each_entry(|_key, _value| Err("Failed!")), Err(IterationError::CallbackError(_))));

    assert!(tx.find_key(|key| key.eq(b"foo")).unwrap().is_some_and(|key| key.eq(b"foo")));
    assert!(tx.find_key(|key| key.eq(b"bar")).unwrap().is_none());
    assert!(tx.find_key(|key| key.eq(b"baz")).unwrap().is_some_and(|key| key.eq(b"baz")));
    assert!(tx.find_key(|key| key.eq(b"qux")).unwrap().is_some_and(|key| key.eq(b"qux")));

    assert_eq!(tx.find_entry(|key, _| key.eq(b"foo")).unwrap().unwrap().1, "12345678");
    assert_eq!(tx.find_entry(|key, _| key.eq_ignore_ascii_case(b"FOO")).unwrap().unwrap().1, "12345678");
    assert!(tx.find_entry(|key, _| key.eq(b"bar")).unwrap().is_none());

    tx.update(b"foo".to_vec(), "abcdefgh".to_owned()).unwrap();
    tx.update(b"bar".to_vec(), "hgfedcba".to_owned()).unwrap();
    tx.update(b"baz".to_vec(), "stuvwxyz".to_owned()).unwrap();

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
    assert!(!tx.contains_key(&b"xyz"[..]).unwrap());

    assert!(tx.insert(b"foo".to_vec(), "ijklmnop".to_owned()).unwrap());
    assert!(tx.insert(b"baz".to_vec(), "ponmlkji".to_owned()).unwrap());

    assert_eq!(
        tx.extend(false, [(b"bar".to_vec(), "abcdefgh".to_owned()), (b"baz".to_vec(), "hgfedcba".to_owned()), (b"qux".to_vec(), "stuvwxyz".to_owned())])
            .unwrap(),
        Some((2, 0))
    );

    assert_eq!(tx.len().unwrap(), 4);
    assert!(!tx.is_empty().unwrap());
    assert!(tx.contains_key(&b"foo"[..]).unwrap());
    assert!(tx.contains_key(&b"bar"[..]).unwrap());
    assert!(tx.contains_key(&b"baz"[..]).unwrap());
    assert!(tx.contains_key(&b"qux"[..]).unwrap());
    assert!(!tx.contains_key(&b"xyz"[..]).unwrap());
    assert!(!tx.contains_key(&b"$?!"[..]).unwrap());
    assert!(tx.get(&b"foo"[..]).unwrap().is_some_and(|value| value.eq("ijklmnop")));
    assert!(tx.get(&b"bar"[..]).unwrap().is_some_and(|value| value.eq("abcdefgh")));
    assert!(tx.get(&b"baz"[..]).unwrap().is_some_and(|value| value.eq("ponmlkji")));
    assert!(tx.get(&b"qux"[..]).unwrap().is_some_and(|value| value.eq("stuvwxyz")));
    assert!(tx.get(&b"xyz"[..]).unwrap().is_none());
    assert!(tx.get(&b"$?!"[..]).unwrap().is_none());

    assert_eq!(tx.extend_ref(false, [(&b"foo"[..], "deadbeef"), (&b"xyz"[..], "cafebabe"), (&b"qux"[..], "dead10cc")]).unwrap(), Some((1, 0)));

    assert!(tx.take(&b"$?!"[..]).unwrap().is_none());
    assert!(tx.take(&b"qux"[..]).unwrap().is_some_and(|value| value.eq("stuvwxyz")));
    assert!(tx.take(&b"bar"[..]).unwrap().is_some_and(|value| value.eq("abcdefgh")));

    tx.commit();

    assert_eq!(map.len().unwrap(), 3);
    assert!(!map.is_empty().unwrap());
    assert!(map.contains_key(&b"foo"[..]).unwrap());
    assert!(!map.contains_key(&b"bar"[..]).unwrap());
    assert!(map.contains_key(&b"baz"[..]).unwrap());
    assert!(!map.contains_key(&b"qux"[..]).unwrap());
    assert!(map.contains_key(&b"xyz"[..]).unwrap());
    assert!(!map.contains_key(&b"$?!"[..]).unwrap());
    assert!(map.get(&b"foo"[..]).unwrap().is_some_and(|value| value.eq("ijklmnop")));
    assert!(map.get(&b"bar"[..]).unwrap().is_none());
    assert!(map.get(&b"baz"[..]).unwrap().is_some_and(|value| value.eq("ponmlkji")));
    assert!(map.get(&b"qux"[..]).unwrap().is_none());
    assert!(map.get(&b"xyz"[..]).unwrap().is_some_and(|value| value.eq("cafebabe")));
    assert!(map.get(&b"$?!"[..]).unwrap().is_none());
}

#[test]
fn test_sqlite_map_blob_string_from_entries() {
    static ENTRIES: LazyLock<[(Blob, String); 6]> = LazyLock::new(|| {
        [
            (b"foo".to_vec(), "abcdefgh".to_owned()),
            (b"bar".to_vec(), "hgfedcba".to_owned()),
            (b"baz".to_vec(), "ijklmnop".to_owned()),
            (b"foo".to_vec(), "ponmlkji".to_owned()),
            (b"qux".to_vec(), "stuvwxyz".to_owned()),
            (b"xyz".to_vec(), "zyxwvuts".to_owned()),
        ]
    });

    let map = SQLiteMap::<Blob, String>::try_from(ENTRIES.clone()).unwrap();

    assert_eq!(map.len().unwrap(), 5);
    assert!(map.get(&b"foo"[..]).unwrap().is_some_and(|value| value.eq("abcdefgh")));
    assert!(map.get(&b"FOO"[..]).unwrap().is_none());
    assert!(map.get(&b"bar"[..]).unwrap().is_some_and(|value| value.eq("hgfedcba")));
    assert!(map.get(&b"BAR"[..]).unwrap().is_none());
    assert!(map.get(&b"baz"[..]).unwrap().is_some_and(|value| value.eq("ijklmnop")));
    assert!(map.get(&b"qux"[..]).unwrap().is_some_and(|value| value.eq("stuvwxyz")));
    assert!(map.get(&b"xyz"[..]).unwrap().is_some_and(|value| value.eq("zyxwvuts")));
    assert!(map.get(&b"$?!"[..]).unwrap().is_none());
}

#[test]
fn test_sqlite_map_blob_string_stresstest() {
    let mut map = SQLiteMap::<Blob, String>::new().unwrap();
    let (mut hex_1, mut hex_2) = (HexEncoder::default(), HexEncoder::default());

    {
        let mut tx = map.transaction().unwrap();
        for i in 0u64..MAX_ITEMS {
            let (key, value) = (hex_1.encode_raw(mix64(i)), hex_2.encode(mix64(!i)));
            assert!(tx.insert_ref(key, value).unwrap());
        }
    }

    assert!(!map.is_empty().unwrap());
    assert_eq!(map.len().unwrap().to_u64(), MAX_ITEMS);

    {
        let tx = map.transaction().unwrap();
        for i in 0u64..MAX_ITEMS {
            let (key, value) = (hex_1.encode_raw(mix64(i)), hex_2.encode(mix64(!i)));
            assert_eq!(tx.get(key).unwrap().unwrap(), value);
        }
    }

    {
        let mut tx = map.transaction().unwrap();
        for i in 0u64..MAX_ITEMS {
            assert!(tx.remove(hex_1.encode_raw(mix64(i))).unwrap());
        }
    }

    assert!(map.is_empty().unwrap());
    assert_eq!(map.len().unwrap().to_u64(), 0);
}

#[test]
fn test_sqlite_map_blob_string_randomized() {
    const COUNT: u64 = 75_000u64;
    let mut map = SQLiteMap::<Blob, String>::new().unwrap();
    let (mut temp, mut hex) = ([0u8; 16usize], HexEncoder::default());

    for (seed, t) in [(0x9DDC5F3146219B97u64, true), (0x79B9E4DC002B32D6u64, false)] {
        for b in [true, false] {
            let mut xoshiro = Xoshiro256PlusPlus::seed_from_u64(seed);
            if !t {
                for _i in 0u64..COUNT {
                    xoshiro.fill_bytes(&mut temp);
                    assert_eq!(map.insert_ref(temp.as_slice(), hex.encode(xoshiro.next_u64())).unwrap(), b);
                }
            } else {
                let mut tx = map.transaction().unwrap();
                for _i in 0u64..COUNT {
                    xoshiro.fill_bytes(&mut temp);
                    assert_eq!(tx.insert_ref(temp.as_slice(), hex.encode(xoshiro.next_u64())).unwrap(), b);
                }
            }
        }
    }

    for (seed, b) in [(0x9DDC5F3146219B97u64, true), (0x79B9E4DC002B32D6u64, true), (0x2F381F8B2F0E517Au64, false)] {
        let mut xoshiro = Xoshiro256PlusPlus::seed_from_u64(seed);
        let tx = map.transaction().unwrap();
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

#[test]
fn test_sqlite_map_blob_string_extend() {
    let mut map_1 = SQLiteMap::<Blob, String>::new().unwrap();
    let mut map_2 = SQLiteMap::<Blob, String>::new().unwrap();
    let mut map_3 = SQLiteMap::<Blob, String>::new().unwrap();

    {
        let mut tx = map_1.transaction().unwrap();
        assert!(tx.insert(b"foo".to_vec(), "12345678".to_owned()).unwrap());
        assert!(tx.insert(b"bar".to_vec(), "87654321".to_owned()).unwrap());
        assert!(tx.insert(b"baz".to_vec(), "abcdefgh".to_owned()).unwrap());
    }

    {
        let mut tx = map_2.transaction().unwrap();
        assert!(tx.insert(b"foo".to_vec(), "12345678".to_owned()).unwrap());
        assert!(tx.insert(b"bar".to_vec(), "87654321".to_owned()).unwrap());
        assert!(tx.insert(b"baz".to_vec(), "abcdefgh".to_owned()).unwrap());
    }

    {
        let mut tx = map_3.transaction().unwrap();
        assert!(tx.insert(b"foo".to_vec(), "hgfedcba".to_owned()).unwrap());
        assert!(tx.insert(b"qux".to_vec(), "stuvwxyz".to_owned()).unwrap());
        assert!(tx.insert(b"xyz".to_vec(), "zyxwvuts".to_owned()).unwrap());
        assert!(tx.insert(b"cor".to_vec(), "deadbeef".to_owned()).unwrap());
    }

    assert!(map_1.get(&b"foo"[..]).unwrap().is_some_and(|value| value.eq("12345678")));
    assert!(map_1.get(&b"bar"[..]).unwrap().is_some_and(|value| value.eq("87654321")));
    assert!(map_1.get(&b"$?!"[..]).unwrap().is_none());
    assert!(map_1.get(&b"baz"[..]).unwrap().is_some_and(|value| value.eq("abcdefgh")));
    assert!(map_1.get(&b"qux"[..]).unwrap().is_none());
    assert!(map_1.get(&b"xyz"[..]).unwrap().is_none());
    assert!(map_1.get(&b"cor"[..]).unwrap().is_none());

    assert_eq!(map_1.extend_map(&map_3, false).unwrap(), Some((3, 0)));

    assert!(map_1.get(&b"foo"[..]).unwrap().is_some_and(|value| value.eq("12345678")));
    assert!(map_1.get(&b"bar"[..]).unwrap().is_some_and(|value| value.eq("87654321")));
    assert!(map_1.get(&b"$?!"[..]).unwrap().is_none());
    assert!(map_1.get(&b"baz"[..]).unwrap().is_some_and(|value| value.eq("abcdefgh")));
    assert!(map_1.get(&b"qux"[..]).unwrap().is_some_and(|value| value.eq("stuvwxyz")));
    assert!(map_1.get(&b"xyz"[..]).unwrap().is_some_and(|value| value.eq("zyxwvuts")));
    assert!(map_1.get(&b"cor"[..]).unwrap().is_some_and(|value| value.eq("deadbeef")));

    assert!(map_2.get(&b"foo"[..]).unwrap().is_some_and(|value| value.eq("12345678")));
    assert!(map_2.get(&b"bar"[..]).unwrap().is_some_and(|value| value.eq("87654321")));
    assert!(map_2.get(&b"$?!"[..]).unwrap().is_none());
    assert!(map_2.get(&b"baz"[..]).unwrap().is_some_and(|value| value.eq("abcdefgh")));
    assert!(map_2.get(&b"qux"[..]).unwrap().is_none());
    assert!(map_2.get(&b"xyz"[..]).unwrap().is_none());
    assert!(map_2.get(&b"cor"[..]).unwrap().is_none());

    cfg_if! {
        if #[cfg(feature = "hooks")] {
            assert_eq!(map_2.extend_map(&map_3, true).unwrap(), Some((3, 1)));
        } else {
            assert_eq!(map_2.extend_map(&map_3, true).unwrap(), None);
        }
    }

    assert!(map_2.get(&b"foo"[..]).unwrap().is_some_and(|value| value.eq("hgfedcba")));
    assert!(map_2.get(&b"bar"[..]).unwrap().is_some_and(|value| value.eq("87654321")));
    assert!(map_2.get(&b"$?!"[..]).unwrap().is_none());
    assert!(map_2.get(&b"baz"[..]).unwrap().is_some_and(|value| value.eq("abcdefgh")));
    assert!(map_2.get(&b"qux"[..]).unwrap().is_some_and(|value| value.eq("stuvwxyz")));
    assert!(map_2.get(&b"xyz"[..]).unwrap().is_some_and(|value| value.eq("zyxwvuts")));
    assert!(map_2.get(&b"cor"[..]).unwrap().is_some_and(|value| value.eq("deadbeef")));
}

#[test]
fn test_sqlite_map_blob_string_update() {
    let mut map = SQLiteMap::<Blob, String>::new().unwrap();

    {
        let mut tx = map.transaction().unwrap();
        assert!(tx.insert(b"foo".to_vec(), "12345678".to_owned()).unwrap());
        assert!(tx.insert(b"bar".to_vec(), "87654321".to_owned()).unwrap());
        assert!(tx.insert(b"baz".to_vec(), "abcdefgh".to_owned()).unwrap());
    }

    let result_1 = map
        .extend(true, [(b"bar".to_vec(), "hgfedcba".to_owned()), (b"baz".to_vec(), "stuvwxyz".to_owned()), (b"qux".to_vec(), "zyxwvuts".to_owned())])
        .unwrap();
    cfg_if! {
        if #[cfg(feature = "hooks")] {
            assert_eq!(result_1, Some((1, 2)));
        } else {
            assert_eq!(result_1, None);
        }
    }

    assert!(map.get(&b"foo"[..]).unwrap().is_some_and(|value| value.eq("12345678")));
    assert!(map.get(&b"bar"[..]).unwrap().is_some_and(|value| value.eq("hgfedcba")));
    assert!(map.get(&b"baz"[..]).unwrap().is_some_and(|value| value.eq("stuvwxyz")));
    assert!(map.get(&b"qux"[..]).unwrap().is_some_and(|value| value.eq("zyxwvuts")));

    let result_2 = map.extend_ref(true, [(&b"baz"[..], "deadbeef"), (&b"xyz"[..], "cafebabe"), (&b"bar"[..], "c0deba5e")]).unwrap();
    cfg_if! {
        if #[cfg(feature = "hooks")] {
            assert_eq!(result_2, Some((1, 2)));
        } else {
            assert_eq!(result_2, None);
        }
    }

    assert!(map.get(&b"foo"[..]).unwrap().is_some_and(|value| value.eq("12345678")));
    assert!(map.get(&b"bar"[..]).unwrap().is_some_and(|value| value.eq("c0deba5e")));
    assert!(map.get(&b"baz"[..]).unwrap().is_some_and(|value| value.eq("deadbeef")));
    assert!(map.get(&b"qux"[..]).unwrap().is_some_and(|value| value.eq("zyxwvuts")));
    assert!(map.get(&b"xyz"[..]).unwrap().is_some_and(|value| value.eq("cafebabe")));
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

    assert!(map.insert("foo".to_owned(), "12345678".to_owned()).unwrap());
    assert!(map.insert("bar".to_owned(), "87654321".to_owned()).unwrap());
    assert!(!map.insert("foo".to_owned(), "abcdefgh".to_owned()).unwrap());
    assert!(!map.insert("bar".to_owned(), "hgfedcba".to_owned()).unwrap());
    assert!(map.insert("baz".to_owned(), "deadbeef".to_owned()).unwrap());
    assert!(!map.insert("baz".to_owned(), "stuvwxyz".to_owned()).unwrap());

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

    assert!(map.insert("qux".to_owned(), "cafebabe".to_owned()).unwrap());
    assert!(!map.insert("qux".to_owned(), "cafebabe".to_owned()).unwrap());
    assert!(map.insert("foo".to_owned(), "12345678".to_owned()).unwrap());
    assert!(!map.insert("foo".to_owned(), "12345678".to_owned()).unwrap());

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
        let mut hash = HashSet::new();
        map.try_for_each_key(|key| {
            assert!(hash.insert(key.to_owned()));
            Ok::<_, ()>(())
        })
        .unwrap();
        assert_eq!(hash.len(), 3);
        assert!(hash.contains("foo"));
        assert!(hash.contains("baz"));
        assert!(hash.contains("qux"));
    }

    assert!(matches!(map.try_for_each_key(|_key| Err("Failed!")), Err(IterationError::CallbackError(_))));

    {
        let mut hash = HashMap::new();
        map.for_each_entry(|key, value| assert!(hash.insert(key.to_owned(), value.to_owned()).is_none())).unwrap();
        assert_eq!(hash.len(), 3);
        assert_eq!(hash.get("foo").unwrap(), "12345678");
        assert_eq!(hash.get("baz").unwrap(), "deadbeef");
        assert_eq!(hash.get("qux").unwrap(), "cafebabe");
    }

    {
        let mut hash = HashMap::new();
        map.try_for_each_entry(|key, value| {
            assert!(hash.insert(key.to_owned(), value.to_owned()).is_none());
            Ok::<_, ()>(())
        })
        .unwrap();
        assert_eq!(hash.len(), 3);
        assert_eq!(hash.get("foo").unwrap(), "12345678");
        assert_eq!(hash.get("baz").unwrap(), "deadbeef");
        assert_eq!(hash.get("qux").unwrap(), "cafebabe");
    }

    assert!(matches!(map.try_for_each_entry(|_key, _value| Err("Failed!")), Err(IterationError::CallbackError(_))));

    assert!(map.find_key(|key| key.eq("foo")).unwrap().is_some_and(|key| key.eq("foo")));
    assert!(map.find_key(|key| key.eq("bar")).unwrap().is_none());
    assert!(map.find_key(|key| key.eq("baz")).unwrap().is_some_and(|key| key.eq("baz")));
    assert!(map.find_key(|key| key.eq("qux")).unwrap().is_some_and(|key| key.eq("qux")));

    assert_eq!(map.find_entry(|key, _| key.eq("foo")).unwrap().unwrap().1, "12345678");
    assert_eq!(map.find_entry(|key, _| key.eq_ignore_ascii_case("FOO")).unwrap().unwrap().1, "12345678");
    assert!(map.find_entry(|key, _| key.eq("bar")).unwrap().is_none());

    map.update("foo".to_owned(), "abcdefgh".to_owned()).unwrap();
    map.update("bar".to_owned(), "hgfedcba".to_owned()).unwrap();
    map.update("baz".to_owned(), "stuvwxyz".to_owned()).unwrap();

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
    assert!(!map.contains_key("xyz").unwrap());

    assert!(map.insert("foo".to_owned(), "ijklmnop".to_owned()).unwrap());
    assert!(map.insert("baz".to_owned(), "ponmlkji".to_owned()).unwrap());

    assert_eq!(
        map.extend(false, [("bar".to_owned(), "abcdefgh".to_owned()), ("baz".to_owned(), "hgfedcba".to_owned()), ("qux".to_owned(), "stuvwxyz".to_owned())])
            .unwrap(),
        Some((2, 0))
    );

    assert_eq!(map.len().unwrap(), 4);
    assert!(!map.is_empty().unwrap());
    assert!(map.contains_key("foo").unwrap());
    assert!(map.contains_key("bar").unwrap());
    assert!(map.contains_key("baz").unwrap());
    assert!(map.contains_key("qux").unwrap());
    assert!(!map.contains_key("xyz").unwrap());
    assert!(!map.contains_key("$?!").unwrap());
    assert!(map.get("foo").unwrap().is_some_and(|value| value.eq("ijklmnop")));
    assert!(map.get("bar").unwrap().is_some_and(|value| value.eq("abcdefgh")));
    assert!(map.get("baz").unwrap().is_some_and(|value| value.eq("ponmlkji")));
    assert!(map.get("qux").unwrap().is_some_and(|value| value.eq("stuvwxyz")));
    assert!(map.get("xyz").unwrap().is_none());
    assert!(map.get("$?!").unwrap().is_none());

    assert_eq!(map.extend_ref(false, [("foo", "deadbeef"), ("xyz", "cafebabe"), ("qux", "dead10cc")]).unwrap(), Some((1, 0)));

    assert_eq!(map.len().unwrap(), 5);
    assert!(!map.is_empty().unwrap());
    assert!(map.contains_key("foo").unwrap());
    assert!(map.contains_key("bar").unwrap());
    assert!(map.contains_key("baz").unwrap());
    assert!(map.contains_key("qux").unwrap());
    assert!(map.contains_key("xyz").unwrap());
    assert!(!map.contains_key("$?!").unwrap());
    assert!(map.get("foo").unwrap().is_some_and(|value| value.eq("ijklmnop")));
    assert!(map.get("bar").unwrap().is_some_and(|value| value.eq("abcdefgh")));
    assert!(map.get("baz").unwrap().is_some_and(|value| value.eq("ponmlkji")));
    assert!(map.get("qux").unwrap().is_some_and(|value| value.eq("stuvwxyz")));
    assert!(map.get("xyz").unwrap().is_some_and(|value| value.eq("cafebabe")));
    assert!(map.get("$?!").unwrap().is_none());

    assert!(map.take("$?!").unwrap().is_none());
    assert!(map.take("qux").unwrap().is_some_and(|value| value.eq("stuvwxyz")));
    assert!(map.take("bar").unwrap().is_some_and(|value| value.eq("abcdefgh")));
    assert!(!map.contains_key("qux").unwrap());
    assert!(!map.contains_key("bar").unwrap());
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

    assert!(tx.insert("foo".to_owned(), "12345678".to_owned()).unwrap());
    assert!(tx.insert("bar".to_owned(), "87654321".to_owned()).unwrap());
    assert!(!tx.insert("foo".to_owned(), "abcdefgh".to_owned()).unwrap());
    assert!(!tx.insert("bar".to_owned(), "hgfedcba".to_owned()).unwrap());
    assert!(tx.insert("baz".to_owned(), "deadbeef".to_owned()).unwrap());
    assert!(!tx.insert("baz".to_owned(), "stuvwxyz".to_owned()).unwrap());

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

    assert!(tx.insert("qux".to_owned(), "cafebabe".to_owned()).unwrap());
    assert!(!tx.insert("qux".to_owned(), "cafebabe".to_owned()).unwrap());
    assert!(tx.insert("foo".to_owned(), "12345678".to_owned()).unwrap());
    assert!(!tx.insert("foo".to_owned(), "12345678".to_owned()).unwrap());

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
        let mut hash = HashSet::new();
        tx.try_for_each_key(|key| {
            assert!(hash.insert(key.to_owned()));
            Ok::<_, ()>(())
        })
        .unwrap();
        assert_eq!(hash.len(), 3);
        assert!(hash.contains("foo"));
        assert!(hash.contains("baz"));
        assert!(hash.contains("qux"));
    }

    assert!(matches!(tx.try_for_each_key(|_key| Err("Failed!")), Err(IterationError::CallbackError(_))));

    {
        let mut hash = HashMap::new();
        tx.for_each_entry(|key, value| assert!(hash.insert(key.to_owned(), value.to_owned()).is_none())).unwrap();
        assert_eq!(hash.len(), 3);
        assert_eq!(hash.get("foo").unwrap(), "12345678");
        assert_eq!(hash.get("baz").unwrap(), "deadbeef");
        assert_eq!(hash.get("qux").unwrap(), "cafebabe");
    }

    {
        let mut hash = HashMap::new();
        tx.try_for_each_entry(|key, value| {
            assert!(hash.insert(key.to_owned(), value.to_owned()).is_none());
            Ok::<_, ()>(())
        })
        .unwrap();
        assert_eq!(hash.len(), 3);
        assert_eq!(hash.get("foo").unwrap(), "12345678");
        assert_eq!(hash.get("baz").unwrap(), "deadbeef");
        assert_eq!(hash.get("qux").unwrap(), "cafebabe");
    }

    assert!(matches!(tx.try_for_each_entry(|_key, _value| Err("Failed!")), Err(IterationError::CallbackError(_))));

    assert!(tx.find_key(|key| key.eq("foo")).unwrap().is_some_and(|key| key.eq("foo")));
    assert!(tx.find_key(|key| key.eq("bar")).unwrap().is_none());
    assert!(tx.find_key(|key| key.eq("baz")).unwrap().is_some_and(|key| key.eq("baz")));
    assert!(tx.find_key(|key| key.eq("qux")).unwrap().is_some_and(|key| key.eq("qux")));

    assert_eq!(tx.find_entry(|key, _| key.eq("foo")).unwrap().unwrap().1, "12345678");
    assert_eq!(tx.find_entry(|key, _| key.eq_ignore_ascii_case("FOO")).unwrap().unwrap().1, "12345678");
    assert!(tx.find_entry(|key, _| key.eq("bar")).unwrap().is_none());

    tx.update("foo".to_owned(), "abcdefgh".to_owned()).unwrap();
    tx.update("bar".to_owned(), "hgfedcba".to_owned()).unwrap();
    tx.update("baz".to_owned(), "stuvwxyz".to_owned()).unwrap();

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
    assert!(!tx.contains_key("xyz").unwrap());

    assert!(tx.insert("foo".to_owned(), "ijklmnop".to_owned()).unwrap());
    assert!(tx.insert("baz".to_owned(), "ponmlkji".to_owned()).unwrap());

    assert_eq!(
        tx.extend(false, [("bar".to_owned(), "abcdefgh".to_owned()), ("baz".to_owned(), "hgfedcba".to_owned()), ("qux".to_owned(), "stuvwxyz".to_owned())])
            .unwrap(),
        Some((2, 0))
    );

    assert_eq!(tx.len().unwrap(), 4);
    assert!(!tx.is_empty().unwrap());
    assert!(tx.contains_key("foo").unwrap());
    assert!(tx.contains_key("bar").unwrap());
    assert!(tx.contains_key("baz").unwrap());
    assert!(tx.contains_key("qux").unwrap());
    assert!(!tx.contains_key("xyz").unwrap());
    assert!(!tx.contains_key("$?!").unwrap());
    assert!(tx.get("foo").unwrap().is_some_and(|value| value.eq("ijklmnop")));
    assert!(tx.get("bar").unwrap().is_some_and(|value| value.eq("abcdefgh")));
    assert!(tx.get("baz").unwrap().is_some_and(|value| value.eq("ponmlkji")));
    assert!(tx.get("qux").unwrap().is_some_and(|value| value.eq("stuvwxyz")));
    assert!(tx.get("xyz").unwrap().is_none());
    assert!(tx.get("$?!").unwrap().is_none());

    assert_eq!(tx.extend_ref(false, [("foo", "deadbeef"), ("xyz", "cafebabe"), ("qux", "dead10cc")]).unwrap(), Some((1, 0)));

    assert!(tx.take("$?!").unwrap().is_none());
    assert!(tx.take("qux").unwrap().is_some_and(|value| value.eq("stuvwxyz")));
    assert!(tx.take("bar").unwrap().is_some_and(|value| value.eq("abcdefgh")));

    tx.commit();

    assert_eq!(map.len().unwrap(), 3);
    assert!(!map.is_empty().unwrap());
    assert!(map.contains_key("foo").unwrap());
    assert!(!map.contains_key("bar").unwrap());
    assert!(map.contains_key("baz").unwrap());
    assert!(!map.contains_key("qux").unwrap());
    assert!(map.contains_key("xyz").unwrap());
    assert!(!map.contains_key("$?!").unwrap());
    assert!(map.get("foo").unwrap().is_some_and(|value| value.eq("ijklmnop")));
    assert!(map.get("bar").unwrap().is_none());
    assert!(map.get("baz").unwrap().is_some_and(|value| value.eq("ponmlkji")));
    assert!(map.get("qux").unwrap().is_none());
    assert!(map.get("xyz").unwrap().is_some_and(|value| value.eq("cafebabe")));
    assert!(map.get("$?!").unwrap().is_none());
}

#[test]
fn test_sqlite_map_string_string_ignorecase() {
    let mut map_1 = SQLiteMap::<String, String>::new().unwrap();
    let mut map_2 = SQLiteMap::<String, String>::with_nocase().unwrap();

    assert!(map_1.insert("foo".to_owned(), "12345678".to_owned()).unwrap());
    assert!(map_1.insert("FOO".to_owned(), "87654321".to_owned()).unwrap());
    assert!(map_1.insert("BAR".to_owned(), "abcdefgh".to_owned()).unwrap());
    assert!(map_1.insert("bar".to_owned(), "hgfedcba".to_owned()).unwrap());

    assert!(map_2.insert("foo".to_owned(), "12345678".to_owned()).unwrap());
    assert!(!map_2.insert("FOO".to_owned(), "87654321".to_owned()).unwrap());
    assert!(map_2.insert("BAR".to_owned(), "abcdefgh".to_owned()).unwrap());
    assert!(!map_2.insert("bar".to_owned(), "hgfedcba".to_owned()).unwrap());

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
fn test_sqlite_map_string_string_from_entries() {
    static ENTRIES: LazyLock<[(String, String); 6]> = LazyLock::new(|| {
        [
            ("foo".to_owned(), "abcdefgh".to_owned()),
            ("bar".to_owned(), "hgfedcba".to_owned()),
            ("baz".to_owned(), "ijklmnop".to_owned()),
            ("foo".to_owned(), "ponmlkji".to_owned()),
            ("FOO".to_owned(), "stuvwxyz".to_owned()),
            ("qux".to_owned(), "zyxwvuts".to_owned()),
        ]
    });

    let map_1 = SQLiteMap::<String, String>::try_from(ENTRIES.clone()).unwrap();
    let map_2 = SQLiteMap::<String, String>::try_from_nocase(ENTRIES.clone()).unwrap();

    assert_eq!(map_1.len().unwrap(), 5);
    assert!(map_1.get("foo").unwrap().is_some_and(|value| value.eq("abcdefgh")));
    assert!(map_1.get("FOO").unwrap().is_some_and(|value| value.eq("stuvwxyz")));
    assert!(map_1.get("bar").unwrap().is_some_and(|value| value.eq("hgfedcba")));
    assert!(map_1.get("BAR").unwrap().is_none());
    assert!(map_1.get("baz").unwrap().is_some_and(|value| value.eq("ijklmnop")));
    assert!(map_1.get("BAZ").unwrap().is_none());
    assert!(map_1.get("qux").unwrap().is_some_and(|value| value.eq("zyxwvuts")));
    assert!(map_1.get("QUX").unwrap().is_none());
    assert!(map_1.get("xyz").unwrap().is_none());
    assert!(map_1.get("XYZ").unwrap().is_none());

    assert_eq!(map_2.len().unwrap(), 4);
    assert!(map_2.get("foo").unwrap().is_some_and(|value| value.eq("abcdefgh")));
    assert!(map_2.get("FOO").unwrap().is_some_and(|value| value.eq("abcdefgh")));
    assert!(map_2.get("bar").unwrap().is_some_and(|value| value.eq("hgfedcba")));
    assert!(map_2.get("BAR").unwrap().is_some_and(|value| value.eq("hgfedcba")));
    assert!(map_2.get("baz").unwrap().is_some_and(|value| value.eq("ijklmnop")));
    assert!(map_2.get("BAZ").unwrap().is_some_and(|value| value.eq("ijklmnop")));
    assert!(map_2.get("qux").unwrap().is_some_and(|value| value.eq("zyxwvuts")));
    assert!(map_2.get("QUX").unwrap().is_some_and(|value| value.eq("zyxwvuts")));
    assert!(map_2.get("xyz").unwrap().is_none());
    assert!(map_2.get("XYZ").unwrap().is_none());
}

#[test]
fn test_sqlite_map_string_string_stresstest() {
    let mut map = SQLiteMap::<String, String>::new().unwrap();
    let (mut hex_1, mut hex_2) = (HexEncoder::default(), HexEncoder::default());

    {
        let mut tx = map.transaction().unwrap();
        for i in 0u64..MAX_ITEMS {
            let (key, value) = (hex_1.encode(mix64(i)), hex_2.encode(mix64(!i)));
            assert!(tx.insert_ref(key, value).unwrap());
        }
    }

    assert!(!map.is_empty().unwrap());
    assert_eq!(map.len().unwrap().to_u64(), MAX_ITEMS);

    {
        let tx = map.transaction().unwrap();
        for i in 0u64..MAX_ITEMS {
            let (key, value) = (hex_1.encode(mix64(i)), hex_2.encode(mix64(!i)));
            assert_eq!(tx.get(key).unwrap().unwrap(), value);
        }
    }

    {
        let mut tx = map.transaction().unwrap();
        for i in 0u64..MAX_ITEMS {
            assert!(tx.remove(hex_1.encode(mix64(i))).unwrap());
        }
    }

    assert!(map.is_empty().unwrap());
    assert_eq!(map.len().unwrap().to_u64(), 0);
}

#[test]
fn test_sqlite_map_string_string_randomized() {
    const COUNT: u64 = 75_000u64;
    let mut map = SQLiteMap::<String, String>::new().unwrap();
    let (mut hex_1, mut hex_2) = (HexEncoder::default(), HexEncoder::default());

    for (seed, t) in [(0x9DDC5F3146219B97u64, true), (0x79B9E4DC002B32D6u64, false)] {
        for b in [true, false] {
            let mut xoshiro = Xoshiro256PlusPlus::seed_from_u64(seed);
            if !t {
                for _i in 0u64..COUNT {
                    assert_eq!(map.insert_ref(hex_1.encode(xoshiro.next_u64()), hex_2.encode(xoshiro.next_u64())).unwrap(), b);
                }
            } else {
                let mut tx = map.transaction().unwrap();
                for _i in 0u64..COUNT {
                    assert_eq!(tx.insert_ref(hex_1.encode(xoshiro.next_u64()), hex_2.encode(xoshiro.next_u64())).unwrap(), b);
                }
            }
        }
    }

    for (seed, b) in [(0x9DDC5F3146219B97u64, true), (0x79B9E4DC002B32D6u64, true), (0x2F381F8B2F0E517Au64, false)] {
        let mut xoshiro = Xoshiro256PlusPlus::seed_from_u64(seed);
        let tx = map.transaction().unwrap();
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

#[test]
fn test_sqlite_map_string_string_extend() {
    let mut map_1 = SQLiteMap::<String, String>::new().unwrap();
    let mut map_2 = SQLiteMap::<String, String>::new().unwrap();
    let mut map_3 = SQLiteMap::<String, String>::new().unwrap();

    {
        let mut tx = map_1.transaction().unwrap();
        assert!(tx.insert("foo".to_owned(), "12345678".to_owned()).unwrap());
        assert!(tx.insert("bar".to_owned(), "87654321".to_owned()).unwrap());
        assert!(tx.insert("baz".to_owned(), "abcdefgh".to_owned()).unwrap());
    }

    {
        let mut tx = map_2.transaction().unwrap();
        assert!(tx.insert("foo".to_owned(), "12345678".to_owned()).unwrap());
        assert!(tx.insert("bar".to_owned(), "87654321".to_owned()).unwrap());
        assert!(tx.insert("baz".to_owned(), "abcdefgh".to_owned()).unwrap());
    }

    {
        let mut tx = map_3.transaction().unwrap();
        assert!(tx.insert("foo".to_owned(), "hgfedcba".to_owned()).unwrap());
        assert!(tx.insert("qux".to_owned(), "stuvwxyz".to_owned()).unwrap());
        assert!(tx.insert("xyz".to_owned(), "zyxwvuts".to_owned()).unwrap());
        assert!(tx.insert("cor".to_owned(), "deadbeef".to_owned()).unwrap());
    }

    assert!(map_1.get("foo").unwrap().is_some_and(|value| value.eq("12345678")));
    assert!(map_1.get("bar").unwrap().is_some_and(|value| value.eq("87654321")));
    assert!(map_1.get("$?!").unwrap().is_none());
    assert!(map_1.get("baz").unwrap().is_some_and(|value| value.eq("abcdefgh")));
    assert!(map_1.get("qux").unwrap().is_none());
    assert!(map_1.get("xyz").unwrap().is_none());
    assert!(map_1.get("cor").unwrap().is_none());

    assert_eq!(map_1.extend_map(&map_3, false).unwrap(), Some((3, 0)));

    assert!(map_1.get("foo").unwrap().is_some_and(|value| value.eq("12345678")));
    assert!(map_1.get("bar").unwrap().is_some_and(|value| value.eq("87654321")));
    assert!(map_1.get("$?!").unwrap().is_none());
    assert!(map_1.get("baz").unwrap().is_some_and(|value| value.eq("abcdefgh")));
    assert!(map_1.get("qux").unwrap().is_some_and(|value| value.eq("stuvwxyz")));
    assert!(map_1.get("xyz").unwrap().is_some_and(|value| value.eq("zyxwvuts")));
    assert!(map_1.get("cor").unwrap().is_some_and(|value| value.eq("deadbeef")));

    assert!(map_2.get("foo").unwrap().is_some_and(|value| value.eq("12345678")));
    assert!(map_2.get("bar").unwrap().is_some_and(|value| value.eq("87654321")));
    assert!(map_2.get("$?!").unwrap().is_none());
    assert!(map_2.get("baz").unwrap().is_some_and(|value| value.eq("abcdefgh")));
    assert!(map_2.get("qux").unwrap().is_none());
    assert!(map_2.get("xyz").unwrap().is_none());
    assert!(map_2.get("cor").unwrap().is_none());

    cfg_if! {
        if #[cfg(feature = "hooks")] {
            assert_eq!(map_2.extend_map(&map_3, true).unwrap(), Some((3, 1)));
        } else {
            assert_eq!(map_2.extend_map(&map_3, true).unwrap(), None);
        }
    }

    assert!(map_2.get("foo").unwrap().is_some_and(|value| value.eq("hgfedcba")));
    assert!(map_2.get("bar").unwrap().is_some_and(|value| value.eq("87654321")));
    assert!(map_2.get("$?!").unwrap().is_none());
    assert!(map_2.get("baz").unwrap().is_some_and(|value| value.eq("abcdefgh")));
    assert!(map_2.get("qux").unwrap().is_some_and(|value| value.eq("stuvwxyz")));
    assert!(map_2.get("xyz").unwrap().is_some_and(|value| value.eq("zyxwvuts")));
    assert!(map_2.get("cor").unwrap().is_some_and(|value| value.eq("deadbeef")));
}

#[test]
fn test_sqlite_map_string_string_update() {
    let mut map = SQLiteMap::<String, String>::new().unwrap();

    {
        let mut tx = map.transaction().unwrap();
        assert!(tx.insert("foo".to_owned(), "12345678".to_owned()).unwrap());
        assert!(tx.insert("bar".to_owned(), "87654321".to_owned()).unwrap());
        assert!(tx.insert("baz".to_owned(), "abcdefgh".to_owned()).unwrap());
    }

    let result_1 = map
        .extend(true, [("bar".to_owned(), "hgfedcba".to_owned()), ("baz".to_owned(), "stuvwxyz".to_owned()), ("qux".to_owned(), "zyxwvuts".to_owned())])
        .unwrap();
    cfg_if! {
        if #[cfg(feature = "hooks")] {
            assert_eq!(result_1, Some((1, 2)));
        } else {
            assert_eq!(result_1, None);
        }
    }

    assert!(map.get("foo").unwrap().is_some_and(|value| value.eq("12345678")));
    assert!(map.get("bar").unwrap().is_some_and(|value| value.eq("hgfedcba")));
    assert!(map.get("baz").unwrap().is_some_and(|value| value.eq("stuvwxyz")));
    assert!(map.get("qux").unwrap().is_some_and(|value| value.eq("zyxwvuts")));

    let result_2 = map.extend_ref(true, [("baz", "deadbeef"), ("xyz", "cafebabe"), ("bar", "c0deba5e")]).unwrap();
    cfg_if! {
        if #[cfg(feature = "hooks")] {
            assert_eq!(result_2, Some((1, 2)));
        } else {
            assert_eq!(result_2, None);
        }
    }

    assert!(map.get("foo").unwrap().is_some_and(|value| value.eq("12345678")));
    assert!(map.get("bar").unwrap().is_some_and(|value| value.eq("c0deba5e")));
    assert!(map.get("baz").unwrap().is_some_and(|value| value.eq("deadbeef")));
    assert!(map.get("qux").unwrap().is_some_and(|value| value.eq("zyxwvuts")));
    assert!(map.get("xyz").unwrap().is_some_and(|value| value.eq("cafebabe")));
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

    assert!(map.insert("foo".to_owned(), b"12345678".to_vec()).unwrap());
    assert!(map.insert("bar".to_owned(), b"87654321".to_vec()).unwrap());
    assert!(!map.insert("foo".to_owned(), b"abcdefgh".to_vec()).unwrap());
    assert!(!map.insert("bar".to_owned(), b"hgfedcba".to_vec()).unwrap());
    assert!(map.insert("baz".to_owned(), b"deadbeef".to_vec()).unwrap());
    assert!(!map.insert("baz".to_owned(), b"stuvwxyz".to_vec()).unwrap());

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

    assert!(map.insert("qux".to_owned(), b"cafebabe".to_vec()).unwrap());
    assert!(!map.insert("qux".to_owned(), b"cafebabe".to_vec()).unwrap());
    assert!(map.insert("foo".to_owned(), b"12345678".to_vec()).unwrap());
    assert!(!map.insert("foo".to_owned(), b"12345678".to_vec()).unwrap());

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
        let mut hash = HashSet::new();
        map.try_for_each_key(|key| {
            assert!(hash.insert(key.to_owned()));
            Ok::<_, ()>(())
        })
        .unwrap();
        assert_eq!(hash.len(), 3);
        assert!(hash.contains("foo"));
        assert!(hash.contains("baz"));
        assert!(hash.contains("qux"));
    }

    assert!(matches!(map.try_for_each_key(|_key| Err("Failed!")), Err(IterationError::CallbackError(_))));

    {
        let mut hash = HashMap::new();
        map.for_each_entry(|key, value| assert!(hash.insert(key.to_owned(), value.to_owned()).is_none())).unwrap();
        assert_eq!(hash.len(), 3);
        assert_eq!(hash.get("foo").unwrap(), &b"12345678"[..]);
        assert_eq!(hash.get("baz").unwrap(), &b"deadbeef"[..]);
        assert_eq!(hash.get("qux").unwrap(), &b"cafebabe"[..]);
    }

    {
        let mut hash = HashMap::new();
        map.try_for_each_entry(|key, value| {
            assert!(hash.insert(key.to_owned(), value.to_owned()).is_none());
            Ok::<_, ()>(())
        })
        .unwrap();
        assert_eq!(hash.len(), 3);
        assert_eq!(hash.get("foo").unwrap(), &b"12345678"[..]);
        assert_eq!(hash.get("baz").unwrap(), &b"deadbeef"[..]);
        assert_eq!(hash.get("qux").unwrap(), &b"cafebabe"[..]);
    }

    assert!(matches!(map.try_for_each_entry(|_key, _value| Err("Failed!")), Err(IterationError::CallbackError(_))));

    assert!(map.find_key(|key| key.eq("foo")).unwrap().is_some_and(|key| key.eq("foo")));
    assert!(map.find_key(|key| key.eq("bar")).unwrap().is_none());
    assert!(map.find_key(|key| key.eq("baz")).unwrap().is_some_and(|key| key.eq("baz")));
    assert!(map.find_key(|key| key.eq("qux")).unwrap().is_some_and(|key| key.eq("qux")));

    assert_eq!(map.find_entry(|key, _| key.eq("foo")).unwrap().unwrap().1, &b"12345678"[..]);
    assert_eq!(map.find_entry(|key, _| key.eq_ignore_ascii_case("FOO")).unwrap().unwrap().1, &b"12345678"[..]);
    assert!(map.find_entry(|key, _| key.eq("bar")).unwrap().is_none());

    map.update("foo".to_owned(), b"abcdefgh".to_vec()).unwrap();
    map.update("bar".to_owned(), b"hgfedcba".to_vec()).unwrap();
    map.update("baz".to_owned(), b"stuvwxyz".to_vec()).unwrap();

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
    assert!(!map.contains_key("xyz").unwrap());

    assert!(map.insert("foo".to_owned(), b"ijklmnop".to_vec()).unwrap());
    assert!(map.insert("baz".to_owned(), b"ponmlkji".to_vec()).unwrap());

    assert_eq!(
        map.extend(false, [("bar".to_owned(), b"abcdefgh".to_vec()), ("baz".to_owned(), b"hgfedcba".to_vec()), ("qux".to_owned(), b"stuvwxyz".to_vec())])
            .unwrap(),
        Some((2, 0))
    );

    assert_eq!(map.len().unwrap(), 4);
    assert!(!map.is_empty().unwrap());
    assert!(map.contains_key("foo").unwrap());
    assert!(map.contains_key("bar").unwrap());
    assert!(map.contains_key("baz").unwrap());
    assert!(map.contains_key("qux").unwrap());
    assert!(!map.contains_key("xyz").unwrap());
    assert!(!map.contains_key("$?!").unwrap());
    assert!(map.get("foo").unwrap().is_some_and(|value| value.eq(&b"ijklmnop"[..])));
    assert!(map.get("bar").unwrap().is_some_and(|value| value.eq(&b"abcdefgh"[..])));
    assert!(map.get("baz").unwrap().is_some_and(|value| value.eq(&b"ponmlkji"[..])));
    assert!(map.get("qux").unwrap().is_some_and(|value| value.eq(&b"stuvwxyz"[..])));
    assert!(map.get("xyz").unwrap().is_none());
    assert!(map.get("$?!").unwrap().is_none());

    assert_eq!(map.extend_ref(false, [("foo", &b"deadbeef"[..]), ("xyz", &b"cafebabe"[..]), ("qux", &b"dead10cc"[..])]).unwrap(), Some((1, 0)));

    assert_eq!(map.len().unwrap(), 5);
    assert!(!map.is_empty().unwrap());
    assert!(map.contains_key("foo").unwrap());
    assert!(map.contains_key("bar").unwrap());
    assert!(map.contains_key("baz").unwrap());
    assert!(map.contains_key("qux").unwrap());
    assert!(map.contains_key("xyz").unwrap());
    assert!(!map.contains_key("$?!").unwrap());
    assert!(map.get("foo").unwrap().is_some_and(|value| value.eq(b"ijklmnop")));
    assert!(map.get("bar").unwrap().is_some_and(|value| value.eq(b"abcdefgh")));
    assert!(map.get("baz").unwrap().is_some_and(|value| value.eq(b"ponmlkji")));
    assert!(map.get("qux").unwrap().is_some_and(|value| value.eq(b"stuvwxyz")));
    assert!(map.get("xyz").unwrap().is_some_and(|value| value.eq(b"cafebabe")));
    assert!(map.get("$?!").unwrap().is_none());

    assert!(map.take("$?!").unwrap().is_none());
    assert!(map.take("qux").unwrap().is_some_and(|value| value.eq(b"stuvwxyz")));
    assert!(map.take("qux").unwrap().is_none());
    assert!(map.take("BAR").unwrap().is_none());
    assert!(map.take_nocase("BAR").unwrap().is_some_and(|value| value.eq(b"abcdefgh")));
    assert!(!map.contains_key("qux").unwrap());
    assert!(!map.contains_key("bar").unwrap());
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

    assert!(tx.insert("foo".to_owned(), b"12345678".to_vec()).unwrap());
    assert!(tx.insert("bar".to_owned(), b"87654321".to_vec()).unwrap());
    assert!(!tx.insert("foo".to_owned(), b"abcdefgh".to_vec()).unwrap());
    assert!(!tx.insert("bar".to_owned(), b"hgfedcba".to_vec()).unwrap());
    assert!(tx.insert("baz".to_owned(), b"deadbeef".to_vec()).unwrap());
    assert!(!tx.insert("baz".to_owned(), b"stuvwxyz".to_vec()).unwrap());

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

    assert!(tx.insert("qux".to_owned(), b"cafebabe".to_vec()).unwrap());
    assert!(!tx.insert("qux".to_owned(), b"cafebabe".to_vec()).unwrap());
    assert!(tx.insert("foo".to_owned(), b"12345678".to_vec()).unwrap());
    assert!(!tx.insert("foo".to_owned(), b"12345678".to_vec()).unwrap());

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
        let mut hash = HashSet::new();
        tx.try_for_each_key(|key| {
            assert!(hash.insert(key.to_owned()));
            Ok::<_, ()>(())
        })
        .unwrap();
        assert_eq!(hash.len(), 3);
        assert!(hash.contains("foo"));
        assert!(hash.contains("baz"));
        assert!(hash.contains("qux"));
    }

    assert!(matches!(tx.try_for_each_key(|_key| Err("Failed!")), Err(IterationError::CallbackError(_))));

    {
        let mut hash = HashMap::new();
        tx.for_each_entry(|key, value| assert!(hash.insert(key.to_owned(), value.to_owned()).is_none())).unwrap();
        assert_eq!(hash.len(), 3);
        assert_eq!(hash.get("foo").unwrap(), &b"12345678"[..]);
        assert_eq!(hash.get("baz").unwrap(), &b"deadbeef"[..]);
        assert_eq!(hash.get("qux").unwrap(), &b"cafebabe"[..]);
    }

    {
        let mut hash = HashMap::new();
        tx.try_for_each_entry(|key, value| {
            assert!(hash.insert(key.to_owned(), value.to_owned()).is_none());
            Ok::<_, ()>(())
        })
        .unwrap();
        assert_eq!(hash.len(), 3);
        assert_eq!(hash.get("foo").unwrap(), &b"12345678"[..]);
        assert_eq!(hash.get("baz").unwrap(), &b"deadbeef"[..]);
        assert_eq!(hash.get("qux").unwrap(), &b"cafebabe"[..]);
    }

    assert!(matches!(tx.try_for_each_entry(|_key, _value| Err("Failed!")), Err(IterationError::CallbackError(_))));

    assert!(tx.find_key(|key| key.eq("foo")).unwrap().is_some_and(|key| key.eq("foo")));
    assert!(tx.find_key(|key| key.eq("bar")).unwrap().is_none());
    assert!(tx.find_key(|key| key.eq("baz")).unwrap().is_some_and(|key| key.eq("baz")));
    assert!(tx.find_key(|key| key.eq("qux")).unwrap().is_some_and(|key| key.eq("qux")));

    assert_eq!(tx.find_entry(|key, _| key.eq("foo")).unwrap().unwrap().1, &b"12345678"[..]);
    assert_eq!(tx.find_entry(|key, _| key.eq_ignore_ascii_case("FOO")).unwrap().unwrap().1, &b"12345678"[..]);
    assert!(tx.find_entry(|key, _| key.eq("bar")).unwrap().is_none());

    tx.update("foo".to_owned(), b"abcdefgh".to_vec()).unwrap();
    tx.update("bar".to_owned(), b"hgfedcba".to_vec()).unwrap();
    tx.update("baz".to_owned(), b"stuvwxyz".to_vec()).unwrap();

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
    assert!(!tx.contains_key("xyz").unwrap());

    assert!(tx.insert("foo".to_owned(), b"ijklmnop".to_vec()).unwrap());
    assert!(tx.insert("baz".to_owned(), b"ponmlkji".to_vec()).unwrap());

    assert_eq!(
        tx.extend(false, [("bar".to_owned(), b"abcdefgh".to_vec()), ("baz".to_owned(), b"hgfedcba".to_vec()), ("qux".to_owned(), b"stuvwxyz".to_vec())])
            .unwrap(),
        Some((2, 0))
    );

    assert_eq!(tx.len().unwrap(), 4);
    assert!(!tx.is_empty().unwrap());
    assert!(tx.contains_key("foo").unwrap());
    assert!(tx.contains_key("bar").unwrap());
    assert!(tx.contains_key("baz").unwrap());
    assert!(tx.contains_key("qux").unwrap());
    assert!(!tx.contains_key("xyz").unwrap());
    assert!(!tx.contains_key("$?!").unwrap());
    assert!(tx.get("foo").unwrap().is_some_and(|value| value.eq(&b"ijklmnop"[..])));
    assert!(tx.get("bar").unwrap().is_some_and(|value| value.eq(&b"abcdefgh"[..])));
    assert!(tx.get("baz").unwrap().is_some_and(|value| value.eq(&b"ponmlkji"[..])));
    assert!(tx.get("qux").unwrap().is_some_and(|value| value.eq(&b"stuvwxyz"[..])));
    assert!(tx.get("xyz").unwrap().is_none());
    assert!(tx.get("$?!").unwrap().is_none());

    assert_eq!(tx.extend_ref(false, [("foo", &b"deadbeef"[..]), ("xyz", &b"cafebabe"[..]), ("qux", &b"dead10cc"[..])]).unwrap(), Some((1, 0)));

    assert!(tx.take("$?!").unwrap().is_none());
    assert!(tx.take("qux").unwrap().is_some_and(|value| value.eq(b"stuvwxyz")));
    assert!(tx.take("qux").unwrap().is_none());
    assert!(tx.take("BAR").unwrap().is_none());
    assert!(tx.take_nocase("BAR").unwrap().is_some_and(|value| value.eq(b"abcdefgh")));

    tx.commit();

    assert_eq!(map.len().unwrap(), 3);
    assert!(!map.is_empty().unwrap());
    assert!(map.contains_key("foo").unwrap());
    assert!(!map.contains_key("bar").unwrap());
    assert!(map.contains_key("baz").unwrap());
    assert!(!map.contains_key("qux").unwrap());
    assert!(map.contains_key("xyz").unwrap());
    assert!(!map.contains_key("$?!").unwrap());
    assert!(map.get("foo").unwrap().is_some_and(|value| value.eq(b"ijklmnop")));
    assert!(map.get("bar").unwrap().is_none());
    assert!(map.get("baz").unwrap().is_some_and(|value| value.eq(b"ponmlkji")));
    assert!(map.get("qux").unwrap().is_none());
    assert!(map.get("xyz").unwrap().is_some_and(|value| value.eq(b"cafebabe")));
    assert!(map.get("$?!").unwrap().is_none());
}

#[test]
fn test_sqlite_map_string_blob_ignorecase() {
    let mut map_1 = SQLiteMap::<String, Blob>::new().unwrap();
    let mut map_2 = SQLiteMap::<String, Blob>::with_nocase().unwrap();

    assert!(map_1.insert("foo".to_owned(), b"12345678".to_vec()).unwrap());
    assert!(map_1.insert("FOO".to_owned(), b"87654321".to_vec()).unwrap());
    assert!(map_1.insert("BAR".to_owned(), b"abcdefgh".to_vec()).unwrap());
    assert!(map_1.insert("bar".to_owned(), b"hgfedcba".to_vec()).unwrap());

    assert!(map_2.insert("foo".to_owned(), b"12345678".to_vec()).unwrap());
    assert!(!map_2.insert("FOO".to_owned(), b"87654321".to_vec()).unwrap());
    assert!(map_2.insert("BAR".to_owned(), b"abcdefgh".to_vec()).unwrap());
    assert!(!map_2.insert("bar".to_owned(), b"hgfedcba".to_vec()).unwrap());

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
fn test_sqlite_map_string_blob_from_entries() {
    static ENTRIES: LazyLock<[(String, Blob); 6]> = LazyLock::new(|| {
        [
            ("foo".to_owned(), b"abcdefgh".to_vec()),
            ("bar".to_owned(), b"hgfedcba".to_vec()),
            ("baz".to_owned(), b"ijklmnop".to_vec()),
            ("foo".to_owned(), b"ponmlkji".to_vec()),
            ("FOO".to_owned(), b"stuvwxyz".to_vec()),
            ("qux".to_owned(), b"zyxwvuts".to_vec()),
        ]
    });

    let map_1 = SQLiteMap::<String, Blob>::try_from(ENTRIES.clone()).unwrap();
    let map_2 = SQLiteMap::<String, Blob>::try_from_nocase(ENTRIES.clone()).unwrap();

    assert_eq!(map_1.len().unwrap(), 5);
    assert!(map_1.get("foo").unwrap().is_some_and(|value| value.eq(b"abcdefgh")));
    assert!(map_1.get("FOO").unwrap().is_some_and(|value| value.eq(b"stuvwxyz")));
    assert!(map_1.get("bar").unwrap().is_some_and(|value| value.eq(b"hgfedcba")));
    assert!(map_1.get("BAR").unwrap().is_none());
    assert!(map_1.get("baz").unwrap().is_some_and(|value| value.eq(b"ijklmnop")));
    assert!(map_1.get("BAZ").unwrap().is_none());
    assert!(map_1.get("qux").unwrap().is_some_and(|value| value.eq(b"zyxwvuts")));
    assert!(map_1.get("QUX").unwrap().is_none());
    assert!(map_1.get("xyz").unwrap().is_none());
    assert!(map_1.get("XYZ").unwrap().is_none());

    assert_eq!(map_2.len().unwrap(), 4);
    assert!(map_2.get("foo").unwrap().is_some_and(|value| value.eq(b"abcdefgh")));
    assert!(map_2.get("FOO").unwrap().is_some_and(|value| value.eq(b"abcdefgh")));
    assert!(map_2.get("bar").unwrap().is_some_and(|value| value.eq(b"hgfedcba")));
    assert!(map_2.get("BAR").unwrap().is_some_and(|value| value.eq(b"hgfedcba")));
    assert!(map_2.get("baz").unwrap().is_some_and(|value| value.eq(b"ijklmnop")));
    assert!(map_2.get("BAZ").unwrap().is_some_and(|value| value.eq(b"ijklmnop")));
    assert!(map_2.get("qux").unwrap().is_some_and(|value| value.eq(b"zyxwvuts")));
    assert!(map_2.get("QUX").unwrap().is_some_and(|value| value.eq(b"zyxwvuts")));
    assert!(map_2.get("xyz").unwrap().is_none());
    assert!(map_2.get("XYZ").unwrap().is_none());
}

#[test]
fn test_sqlite_map_string_blob_stresstest() {
    let mut map = SQLiteMap::<String, Blob>::new().unwrap();
    let (mut hex_1, mut hex_2) = (HexEncoder::default(), HexEncoder::default());

    {
        let mut tx = map.transaction().unwrap();
        for i in 0u64..MAX_ITEMS {
            let (key, value) = (hex_1.encode(mix64(i)), hex_2.encode_raw(mix64(!i)));
            assert!(tx.insert_ref(key, value).unwrap());
        }
    }

    assert!(!map.is_empty().unwrap());
    assert_eq!(map.len().unwrap().to_u64(), MAX_ITEMS);

    {
        let tx = map.transaction().unwrap();
        for i in 0u64..MAX_ITEMS {
            let (key, value) = (hex_1.encode(mix64(i)), hex_2.encode_raw(mix64(!i)));
            assert_eq!(tx.get(key).unwrap().unwrap(), value);
        }
    }

    {
        let mut tx = map.transaction().unwrap();
        for i in 0u64..MAX_ITEMS {
            assert!(tx.remove(hex_1.encode(mix64(i))).unwrap());
        }
    }

    assert!(map.is_empty().unwrap());
    assert_eq!(map.len().unwrap().to_u64(), 0);
}

#[test]
fn test_sqlite_map_string_blob_randomized() {
    const COUNT: u64 = 75_000u64;
    let mut map = SQLiteMap::<String, Blob>::new().unwrap();
    let (mut hex, mut temp) = (HexEncoder::default(), [0u8; 16usize]);

    for (seed, t) in [(0x9DDC5F3146219B97u64, true), (0x79B9E4DC002B32D6u64, false)] {
        for b in [true, false] {
            let mut xoshiro = Xoshiro256PlusPlus::seed_from_u64(seed);
            if !t {
                for _i in 0u64..COUNT {
                    xoshiro.fill_bytes(&mut temp);
                    assert_eq!(map.insert_ref(hex.encode(xoshiro.next_u64()), temp.as_slice()).unwrap(), b);
                }
            } else {
                let mut tx = map.transaction().unwrap();
                for _i in 0u64..COUNT {
                    xoshiro.fill_bytes(&mut temp);
                    assert_eq!(tx.insert_ref(hex.encode(xoshiro.next_u64()), temp.as_slice()).unwrap(), b);
                }
            }
        }
    }

    for (seed, b) in [(0x9DDC5F3146219B97u64, true), (0x79B9E4DC002B32D6u64, true), (0x2F381F8B2F0E517Au64, false)] {
        let mut xoshiro = Xoshiro256PlusPlus::seed_from_u64(seed);
        let tx = map.transaction().unwrap();
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

#[test]
fn test_sqlite_map_string_blob_extend() {
    let mut map_1 = SQLiteMap::<String, Blob>::new().unwrap();
    let mut map_2 = SQLiteMap::<String, Blob>::new().unwrap();
    let mut map_3 = SQLiteMap::<String, Blob>::new().unwrap();

    {
        let mut tx = map_1.transaction().unwrap();
        assert!(tx.insert("foo".to_owned(), b"12345678".to_vec()).unwrap());
        assert!(tx.insert("bar".to_owned(), b"87654321".to_vec()).unwrap());
        assert!(tx.insert("baz".to_owned(), b"abcdefgh".to_vec()).unwrap());
    }

    {
        let mut tx = map_2.transaction().unwrap();
        assert!(tx.insert("foo".to_owned(), b"12345678".to_vec()).unwrap());
        assert!(tx.insert("bar".to_owned(), b"87654321".to_vec()).unwrap());
        assert!(tx.insert("baz".to_owned(), b"abcdefgh".to_vec()).unwrap());
    }

    {
        let mut tx = map_3.transaction().unwrap();
        assert!(tx.insert("foo".to_owned(), b"hgfedcba".to_vec()).unwrap());
        assert!(tx.insert("qux".to_owned(), b"stuvwxyz".to_vec()).unwrap());
        assert!(tx.insert("xyz".to_owned(), b"zyxwvuts".to_vec()).unwrap());
        assert!(tx.insert("cor".to_owned(), b"deadbeef".to_vec()).unwrap());
    }

    assert!(map_1.get("foo").unwrap().is_some_and(|value| value.eq(b"12345678")));
    assert!(map_1.get("bar").unwrap().is_some_and(|value| value.eq(b"87654321")));
    assert!(map_1.get("$?!").unwrap().is_none());
    assert!(map_1.get("baz").unwrap().is_some_and(|value| value.eq(b"abcdefgh")));
    assert!(map_1.get("qux").unwrap().is_none());
    assert!(map_1.get("xyz").unwrap().is_none());
    assert!(map_1.get("cor").unwrap().is_none());

    assert_eq!(map_1.extend_map(&map_3, false).unwrap(), Some((3, 0)));

    assert!(map_1.get("foo").unwrap().is_some_and(|value| value.eq(b"12345678")));
    assert!(map_1.get("bar").unwrap().is_some_and(|value| value.eq(b"87654321")));
    assert!(map_1.get("$?!").unwrap().is_none());
    assert!(map_1.get("baz").unwrap().is_some_and(|value| value.eq(b"abcdefgh")));
    assert!(map_1.get("qux").unwrap().is_some_and(|value| value.eq(b"stuvwxyz")));
    assert!(map_1.get("xyz").unwrap().is_some_and(|value| value.eq(b"zyxwvuts")));
    assert!(map_1.get("cor").unwrap().is_some_and(|value| value.eq(b"deadbeef")));

    assert!(map_2.get("foo").unwrap().is_some_and(|value| value.eq(b"12345678")));
    assert!(map_2.get("bar").unwrap().is_some_and(|value| value.eq(b"87654321")));
    assert!(map_2.get("$?!").unwrap().is_none());
    assert!(map_2.get("baz").unwrap().is_some_and(|value| value.eq(b"abcdefgh")));
    assert!(map_2.get("qux").unwrap().is_none());
    assert!(map_2.get("xyz").unwrap().is_none());
    assert!(map_2.get("cor").unwrap().is_none());

    cfg_if! {
        if #[cfg(feature = "hooks")] {
            assert_eq!(map_2.extend_map(&map_3, true).unwrap(), Some((3, 1)));
        } else {
            assert_eq!(map_2.extend_map(&map_3, true).unwrap(), None);
        }
    }

    assert!(map_2.get("foo").unwrap().is_some_and(|value| value.eq(b"hgfedcba")));
    assert!(map_2.get("bar").unwrap().is_some_and(|value| value.eq(b"87654321")));
    assert!(map_2.get("$?!").unwrap().is_none());
    assert!(map_2.get("baz").unwrap().is_some_and(|value| value.eq(b"abcdefgh")));
    assert!(map_2.get("qux").unwrap().is_some_and(|value| value.eq(b"stuvwxyz")));
    assert!(map_2.get("xyz").unwrap().is_some_and(|value| value.eq(b"zyxwvuts")));
    assert!(map_2.get("cor").unwrap().is_some_and(|value| value.eq(b"deadbeef")));
}

#[test]
fn test_sqlite_map_string_blob_update() {
    let mut map = SQLiteMap::<String, Blob>::new().unwrap();

    {
        let mut tx = map.transaction().unwrap();
        assert!(tx.insert("foo".to_owned(), b"12345678".to_vec()).unwrap());
        assert!(tx.insert("bar".to_owned(), b"87654321".to_vec()).unwrap());
        assert!(tx.insert("baz".to_owned(), b"abcdefgh".to_vec()).unwrap());
    }

    let result_1 = map
        .extend(true, [("bar".to_owned(), b"hgfedcba".to_vec()), ("baz".to_owned(), b"stuvwxyz".to_vec()), ("qux".to_owned(), b"zyxwvuts".to_vec())])
        .unwrap();
    cfg_if! {
        if #[cfg(feature = "hooks")] {
            assert_eq!(result_1, Some((1, 2)));
        } else {
            assert_eq!(result_1, None);
        }
    }

    assert!(map.get("foo").unwrap().is_some_and(|value| value.eq(b"12345678")));
    assert!(map.get("bar").unwrap().is_some_and(|value| value.eq(b"hgfedcba")));
    assert!(map.get("baz").unwrap().is_some_and(|value| value.eq(b"stuvwxyz")));
    assert!(map.get("qux").unwrap().is_some_and(|value| value.eq(b"zyxwvuts")));

    let result_2 = map.extend_ref(true, [("baz", &b"deadbeef"[..]), ("xyz", &b"cafebabe"[..]), ("bar", &b"c0deba5e"[..])]).unwrap();
    cfg_if! {
        if #[cfg(feature = "hooks")] {
            assert_eq!(result_2, Some((1, 2)));
        } else {
            assert_eq!(result_2, None);
        }
    }

    assert!(map.get("foo").unwrap().is_some_and(|value| value.eq(b"12345678")));
    assert!(map.get("bar").unwrap().is_some_and(|value| value.eq(b"c0deba5e")));
    assert!(map.get("baz").unwrap().is_some_and(|value| value.eq(b"deadbeef")));
    assert!(map.get("qux").unwrap().is_some_and(|value| value.eq(b"zyxwvuts")));
    assert!(map.get("xyz").unwrap().is_some_and(|value| value.eq(b"cafebabe")));
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

    assert!(map.insert("foo".to_owned(), 0x12345678).unwrap());
    assert!(map.insert("bar".to_owned(), 0x87654321).unwrap());
    assert!(!map.insert("foo".to_owned(), 0x9ABCDEF0).unwrap());
    assert!(!map.insert("bar".to_owned(), 0x0FEDCBA9).unwrap());
    assert!(map.insert("baz".to_owned(), 0xDEADBEEF).unwrap());
    assert!(!map.insert("baz".to_owned(), 0x8BADF00D).unwrap());

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

    assert!(map.insert("qux".to_owned(), 0xCAFEBABE).unwrap());
    assert!(!map.insert("qux".to_owned(), 0xCAFEBABE).unwrap());
    assert!(map.insert("foo".to_owned(), 0x12345678).unwrap());
    assert!(!map.insert("foo".to_owned(), 0x12345678).unwrap());

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
        let mut hash = HashSet::new();
        map.try_for_each_key(|key| {
            assert!(hash.insert(key.to_owned()));
            Ok::<_, ()>(())
        })
        .unwrap();
        assert_eq!(hash.len(), 3);
        assert!(hash.contains("foo"));
        assert!(hash.contains("baz"));
        assert!(hash.contains("qux"));
    }

    assert!(matches!(map.try_for_each_key(|_key| Err("Failed!")), Err(IterationError::CallbackError(_))));

    {
        let mut hash = HashMap::new();
        map.for_each_entry(|key, value| assert!(hash.insert(key.to_owned(), value.to_owned()).is_none())).unwrap();
        assert_eq!(hash.len(), 3);
        assert_eq!(hash.get("foo").unwrap(), &0x12345678);
        assert_eq!(hash.get("baz").unwrap(), &0xDEADBEEF);
        assert_eq!(hash.get("qux").unwrap(), &0xCAFEBABE);
    }

    {
        let mut hash = HashMap::new();
        map.try_for_each_entry(|key, value| {
            assert!(hash.insert(key.to_owned(), value.to_owned()).is_none());
            Ok::<_, ()>(())
        })
        .unwrap();
        assert_eq!(hash.len(), 3);
        assert_eq!(hash.get("foo").unwrap(), &0x12345678);
        assert_eq!(hash.get("baz").unwrap(), &0xDEADBEEF);
        assert_eq!(hash.get("qux").unwrap(), &0xCAFEBABE);
    }

    assert!(matches!(map.try_for_each_entry(|_key, _value| Err("Failed!")), Err(IterationError::CallbackError(_))));

    assert!(map.find_key(|key| key.eq("foo")).unwrap().is_some_and(|key| key.eq("foo")));
    assert!(map.find_key(|key| key.eq("bar")).unwrap().is_none());
    assert!(map.find_key(|key| key.eq("baz")).unwrap().is_some_and(|key| key.eq("baz")));
    assert!(map.find_key(|key| key.eq("qux")).unwrap().is_some_and(|key| key.eq("qux")));

    assert_eq!(map.find_entry(|key, _| key.eq("foo")).unwrap().unwrap().1, 0x12345678);
    assert_eq!(map.find_entry(|key, _| key.eq_ignore_ascii_case("FOO")).unwrap().unwrap().1, 0x12345678);
    assert!(map.find_entry(|key, _| key.eq("bar")).unwrap().is_none());

    map.update("foo".to_owned(), 0x9ABCDEF0).unwrap();
    map.update("bar".to_owned(), 0x0FEDCBA9).unwrap();
    map.update("baz".to_owned(), 0x8BADF00D).unwrap();

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
    assert!(!map.contains_key("xyz").unwrap());

    assert!(map.insert("foo".to_owned(), 0x12345678i64).unwrap());
    assert!(map.insert("baz".to_owned(), 0x87654321i64).unwrap());

    assert_eq!(
        map.extend(false, [("bar".to_owned(), 0x9ABCDEF0i64), ("baz".to_owned(), 0xCAFEBABE), ("qux".to_owned(), 0xDEADBEEFi64)]).unwrap(),
        Some((2, 0))
    );

    assert_eq!(map.len().unwrap(), 4);
    assert!(!map.is_empty().unwrap());
    assert!(map.contains_key("foo").unwrap());
    assert!(map.contains_key("bar").unwrap());
    assert!(map.contains_key("baz").unwrap());
    assert!(map.contains_key("qux").unwrap());
    assert!(!map.contains_key("xyz").unwrap());
    assert!(!map.contains_key("$?!").unwrap());
    assert!(map.get("foo").unwrap().is_some_and(|value| value.eq(&0x12345678i64)));
    assert!(map.get("bar").unwrap().is_some_and(|value| value.eq(&0x9ABCDEF0i64)));
    assert!(map.get("baz").unwrap().is_some_and(|value| value.eq(&0x87654321i64)));
    assert!(map.get("qux").unwrap().is_some_and(|value| value.eq(&0xDEADBEEFi64)));
    assert!(map.get("xyz").unwrap().is_none());
    assert!(map.get("$?!").unwrap().is_none());

    assert_eq!(map.extend_ref(false, [("foo", &0xEBEBEBEBi64), ("xyz", &0xCAFEBABEi64), ("qux", &0xDEAD10CCi64)]).unwrap(), Some((1, 0)));

    assert_eq!(map.len().unwrap(), 5);
    assert!(!map.is_empty().unwrap());
    assert!(map.contains_key("foo").unwrap());
    assert!(map.contains_key("bar").unwrap());
    assert!(map.contains_key("baz").unwrap());
    assert!(map.contains_key("qux").unwrap());
    assert!(map.contains_key("xyz").unwrap());
    assert!(!map.contains_key("$?!").unwrap());
    assert!(map.get("foo").unwrap().is_some_and(|value| value.eq(&0x12345678i64)));
    assert!(map.get("bar").unwrap().is_some_and(|value| value.eq(&0x9ABCDEF0i64)));
    assert!(map.get("baz").unwrap().is_some_and(|value| value.eq(&0x87654321i64)));
    assert!(map.get("qux").unwrap().is_some_and(|value| value.eq(&0xDEADBEEFi64)));
    assert!(map.get("xyz").unwrap().is_some_and(|value| value.eq(&0xCAFEBABEi64)));
    assert!(map.get("$?!").unwrap().is_none());

    assert!(map.take("$?!").unwrap().is_none());
    assert!(map.take("qux").unwrap().is_some_and(|value| value.eq(&0xDEADBEEFi64)));
    assert!(map.take("bar").unwrap().is_some_and(|value| value.eq(&0x9ABCDEF0i64)));
    assert!(!map.contains_key("qux").unwrap());
    assert!(!map.contains_key("bar").unwrap());
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

    assert!(tx.insert("foo".to_owned(), 0x12345678).unwrap());
    assert!(tx.insert("bar".to_owned(), 0x87654321).unwrap());
    assert!(!tx.insert("foo".to_owned(), 0x9ABCDEF0).unwrap());
    assert!(!tx.insert("bar".to_owned(), 0x0FEDCBA9).unwrap());
    assert!(tx.insert("baz".to_owned(), 0xDEADBEEF).unwrap());
    assert!(!tx.insert("baz".to_owned(), 0x8BADF00D).unwrap());

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

    assert!(tx.insert("qux".to_owned(), 0xCAFEBABE).unwrap());
    assert!(!tx.insert("qux".to_owned(), 0xCAFEBABE).unwrap());
    assert!(tx.insert("foo".to_owned(), 0x12345678).unwrap());
    assert!(!tx.insert("foo".to_owned(), 0x12345678).unwrap());

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
        let mut hash = HashSet::new();
        tx.try_for_each_key(|key| {
            assert!(hash.insert(key.to_owned()));
            Ok::<_, ()>(())
        })
        .unwrap();
        assert_eq!(hash.len(), 3);
        assert!(hash.contains("foo"));
        assert!(hash.contains("baz"));
        assert!(hash.contains("qux"));
    }

    assert!(matches!(tx.try_for_each_key(|_key| Err("Failed!")), Err(IterationError::CallbackError(_))));

    {
        let mut hash = HashMap::new();
        tx.for_each_entry(|key, value| assert!(hash.insert(key.to_owned(), value.to_owned()).is_none())).unwrap();
        assert_eq!(hash.len(), 3);
        assert_eq!(hash.get("foo").unwrap(), &0x12345678);
        assert_eq!(hash.get("baz").unwrap(), &0xDEADBEEF);
        assert_eq!(hash.get("qux").unwrap(), &0xCAFEBABE);
    }

    {
        let mut hash = HashMap::new();
        tx.try_for_each_entry(|key, value| {
            assert!(hash.insert(key.to_owned(), value.to_owned()).is_none());
            Ok::<_, ()>(())
        })
        .unwrap();
        assert_eq!(hash.len(), 3);
        assert_eq!(hash.get("foo").unwrap(), &0x12345678);
        assert_eq!(hash.get("baz").unwrap(), &0xDEADBEEF);
        assert_eq!(hash.get("qux").unwrap(), &0xCAFEBABE);
    }

    assert!(matches!(tx.try_for_each_entry(|_key, _value| Err("Failed!")), Err(IterationError::CallbackError(_))));

    assert!(tx.find_key(|key| key.eq("foo")).unwrap().is_some_and(|key| key.eq("foo")));
    assert!(tx.find_key(|key| key.eq("bar")).unwrap().is_none());
    assert!(tx.find_key(|key| key.eq("baz")).unwrap().is_some_and(|key| key.eq("baz")));
    assert!(tx.find_key(|key| key.eq("qux")).unwrap().is_some_and(|key| key.eq("qux")));

    assert_eq!(tx.find_entry(|key, _| key.eq("foo")).unwrap().unwrap().1, 0x12345678);
    assert_eq!(tx.find_entry(|key, _| key.eq_ignore_ascii_case("FOO")).unwrap().unwrap().1, 0x12345678);
    assert!(tx.find_entry(|key, _| key.eq("bar")).unwrap().is_none());

    tx.update("foo".to_owned(), 0x9ABCDEF0).unwrap();
    tx.update("bar".to_owned(), 0x0FEDCBA9).unwrap();
    tx.update("baz".to_owned(), 0x8BADF00D).unwrap();

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
    assert!(!tx.contains_key("qux").unwrap());

    assert!(tx.insert("foo".to_owned(), 0x12345678i64).unwrap());
    assert!(tx.insert("baz".to_owned(), 0x87654321i64).unwrap());

    assert_eq!(tx.extend(false, [("bar".to_owned(), 0x9ABCDEF0i64), ("baz".to_owned(), 0xCAFEBABE), ("qux".to_owned(), 0xDEADBEEFi64)]).unwrap(), Some((2, 0)));

    assert_eq!(tx.len().unwrap(), 4);
    assert!(!tx.is_empty().unwrap());
    assert!(tx.contains_key("foo").unwrap());
    assert!(tx.contains_key("bar").unwrap());
    assert!(tx.contains_key("baz").unwrap());
    assert!(tx.contains_key("qux").unwrap());
    assert!(!tx.contains_key("xyz").unwrap());
    assert!(!tx.contains_key("$?!").unwrap());
    assert!(tx.get("foo").unwrap().is_some_and(|value| value.eq(&0x12345678i64)));
    assert!(tx.get("bar").unwrap().is_some_and(|value| value.eq(&0x9ABCDEF0i64)));
    assert!(tx.get("baz").unwrap().is_some_and(|value| value.eq(&0x87654321i64)));
    assert!(tx.get("qux").unwrap().is_some_and(|value| value.eq(&0xDEADBEEFi64)));
    assert!(tx.get("xyz").unwrap().is_none());
    assert!(tx.get("$?!").unwrap().is_none());

    assert_eq!(tx.extend_ref(false, [("foo", &0xEBEBEBEBi64), ("xyz", &0xCAFEBABEi64), ("qux", &0xDEAD10CCi64)]).unwrap(), Some((1, 0)));

    assert!(tx.take("$?!").unwrap().is_none());
    assert!(tx.take("qux").unwrap().is_some_and(|value| value.eq(&0xDEADBEEFi64)));
    assert!(tx.take("bar").unwrap().is_some_and(|value| value.eq(&0x9ABCDEF0i64)));

    tx.commit();

    assert_eq!(map.len().unwrap(), 3);
    assert!(!map.is_empty().unwrap());
    assert!(map.contains_key("foo").unwrap());
    assert!(!map.contains_key("bar").unwrap());
    assert!(map.contains_key("baz").unwrap());
    assert!(!map.contains_key("qux").unwrap());
    assert!(map.contains_key("xyz").unwrap());
    assert!(!map.contains_key("$?!").unwrap());
    assert!(map.get("foo").unwrap().is_some_and(|value| value.eq(&0x12345678i64)));
    assert!(map.get("bar").unwrap().is_none());
    assert!(map.get("baz").unwrap().is_some_and(|value| value.eq(&0x87654321i64)));
    assert!(map.get("qux").unwrap().is_none());
    assert!(map.get("xyz").unwrap().is_some_and(|value| value.eq(&0xCAFEBABEi64)));
    assert!(map.get("$?!").unwrap().is_none());
}

#[test]
fn test_sqlite_map_string_i64_ignorecase() {
    let mut map_1 = SQLiteMap::<String, i64>::new().unwrap();
    let mut map_2 = SQLiteMap::<String, i64>::with_nocase().unwrap();

    assert!(map_1.insert("foo".to_owned(), 0x12345678).unwrap());
    assert!(map_1.insert("FOO".to_owned(), 0x87654321).unwrap());
    assert!(map_1.insert("BAR".to_owned(), 0x9ABCDEF0).unwrap());
    assert!(map_1.insert("bar".to_owned(), 0x0FEDCBA9).unwrap());

    assert!(map_2.insert("foo".to_owned(), 0x12345678).unwrap());
    assert!(!map_2.insert("FOO".to_owned(), 0x87654321).unwrap());
    assert!(map_2.insert("BAR".to_owned(), 0x9ABCDEF0).unwrap());
    assert!(!map_2.insert("bar".to_owned(), 0x0FEDCBA9).unwrap());

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
fn test_sqlite_map_string_i64_from_entries() {
    static ENTRIES: LazyLock<[(String, i64); 6]> = LazyLock::new(|| {
        [
            ("foo".to_owned(), 0x12345678i64),
            ("bar".to_owned(), 0x87654321i64),
            ("baz".to_owned(), 0x90ABCDEFi64),
            ("foo".to_owned(), 0xFEDCBA09i64),
            ("FOO".to_owned(), 0xDEADBEEFi64),
            ("qux".to_owned(), 0xCAFEBABEi64),
        ]
    });

    let map_1 = SQLiteMap::<String, i64>::try_from(ENTRIES.clone()).unwrap();
    let map_2 = SQLiteMap::<String, i64>::try_from_nocase(ENTRIES.clone()).unwrap();

    assert_eq!(map_1.len().unwrap(), 5);
    assert!(map_1.get("foo").unwrap().is_some_and(|value| value.eq(&0x12345678i64)));
    assert!(map_1.get("FOO").unwrap().is_some_and(|value| value.eq(&0xDEADBEEFi64)));
    assert!(map_1.get("bar").unwrap().is_some_and(|value| value.eq(&0x87654321i64)));
    assert!(map_1.get("BAR").unwrap().is_none());
    assert!(map_1.get("baz").unwrap().is_some_and(|value| value.eq(&0x90ABCDEFi64)));
    assert!(map_1.get("BAZ").unwrap().is_none());
    assert!(map_1.get("qux").unwrap().is_some_and(|value| value.eq(&0xCAFEBABEi64)));
    assert!(map_1.get("QUX").unwrap().is_none());
    assert!(map_1.get("xyz").unwrap().is_none());
    assert!(map_1.get("XYZ").unwrap().is_none());

    assert_eq!(map_2.len().unwrap(), 4);
    assert!(map_2.get("foo").unwrap().is_some_and(|value| value.eq(&0x12345678i64)));
    assert!(map_2.get("FOO").unwrap().is_some_and(|value| value.eq(&0x12345678i64)));
    assert!(map_2.get("bar").unwrap().is_some_and(|value| value.eq(&0x87654321i64)));
    assert!(map_2.get("BAR").unwrap().is_some_and(|value| value.eq(&0x87654321i64)));
    assert!(map_2.get("baz").unwrap().is_some_and(|value| value.eq(&0x90ABCDEFi64)));
    assert!(map_2.get("BAZ").unwrap().is_some_and(|value| value.eq(&0x90ABCDEFi64)));
    assert!(map_2.get("qux").unwrap().is_some_and(|value| value.eq(&0xCAFEBABEi64)));
    assert!(map_2.get("QUX").unwrap().is_some_and(|value| value.eq(&0xCAFEBABEi64)));
    assert!(map_2.get("xyz").unwrap().is_none());
    assert!(map_2.get("XYZ").unwrap().is_none());
}

#[test]
fn test_sqlite_map_string_i64_stresstest() {
    let mut map = SQLiteMap::<String, i64>::new().unwrap();
    let mut hex = HexEncoder::default();

    {
        let mut tx = map.transaction().unwrap();
        for i in 0u64..MAX_ITEMS {
            let (key, value) = (hex.encode(mix64(i)), mix64(!i) as i64);
            assert!(tx.insert_ref(key, &value).unwrap());
        }
    }

    assert!(!map.is_empty().unwrap());
    assert_eq!(map.len().unwrap().to_u64(), MAX_ITEMS);

    {
        let tx = map.transaction().unwrap();
        for i in 0u64..MAX_ITEMS {
            let (key, value) = (hex.encode(mix64(i)), mix64(!i) as i64);
            assert_eq!(tx.get(key).unwrap().unwrap(), value);
        }
    }

    {
        let mut tx = map.transaction().unwrap();
        for i in 0u64..MAX_ITEMS {
            assert!(tx.remove(hex.encode(mix64(i))).unwrap());
        }
    }

    assert!(map.is_empty().unwrap());
    assert_eq!(map.len().unwrap().to_u64(), 0);
}

#[test]
fn test_sqlite_map_string_i64_randomized() {
    const COUNT: u64 = 75_000u64;
    let mut map = SQLiteMap::<String, i64>::new().unwrap();
    let mut hex = HexEncoder::default();

    for (seed, t) in [(0x9DDC5F3146219B97u64, true), (0x79B9E4DC002B32D6u64, false)] {
        for b in [true, false] {
            let mut xoshiro = Xoshiro256PlusPlus::seed_from_u64(seed);
            if !t {
                for _i in 0u64..COUNT {
                    let (key, value) = (hex.encode(xoshiro.next_u64()), xoshiro.next_u64() as i64);
                    assert_eq!(map.insert_ref(key, &value).unwrap(), b);
                }
            } else {
                let mut tx = map.transaction().unwrap();
                for _i in 0u64..COUNT {
                    let (key, value) = (hex.encode(xoshiro.next_u64()), xoshiro.next_u64() as i64);
                    assert_eq!(tx.insert_ref(key, &value).unwrap(), b);
                }
            }
        }
    }

    for (seed, b) in [(0x9DDC5F3146219B97u64, true), (0x79B9E4DC002B32D6u64, true), (0x2F381F8B2F0E517Au64, false)] {
        let mut xoshiro = Xoshiro256PlusPlus::seed_from_u64(seed);
        let tx = map.transaction().unwrap();
        for _i in 0u64..COUNT {
            if b {
                assert_eq!(tx.get(hex.encode(xoshiro.next_u64())).unwrap().unwrap(), xoshiro.next_u64() as i64);
            } else {
                assert!(tx.get(hex.encode(xoshiro.next_u64())).unwrap().is_none());
            }
        }
    }
}

#[test]
fn test_sqlite_map_string_i64_extend() {
    let mut map_1 = SQLiteMap::<String, i64>::new().unwrap();
    let mut map_2 = SQLiteMap::<String, i64>::new().unwrap();
    let mut map_3 = SQLiteMap::<String, i64>::new().unwrap();

    {
        let mut tx = map_1.transaction().unwrap();
        assert!(tx.insert("foo".to_owned(), 0x12345678i64).unwrap());
        assert!(tx.insert("bar".to_owned(), 0x87654321i64).unwrap());
        assert!(tx.insert("baz".to_owned(), 0x9ABCDEF0i64).unwrap());
    }

    {
        let mut tx = map_2.transaction().unwrap();
        assert!(tx.insert("foo".to_owned(), 0x12345678i64).unwrap());
        assert!(tx.insert("bar".to_owned(), 0x87654321i64).unwrap());
        assert!(tx.insert("baz".to_owned(), 0x9ABCDEF0i64).unwrap());
    }

    {
        let mut tx = map_3.transaction().unwrap();
        assert!(tx.insert("foo".to_owned(), 0x0FEDCBA9i64).unwrap());
        assert!(tx.insert("qux".to_owned(), 0xDEADBEEFi64).unwrap());
        assert!(tx.insert("xyz".to_owned(), 0xCAFEBABEi64).unwrap());
        assert!(tx.insert("cor".to_owned(), 0xFEEDFACEi64).unwrap());
    }

    assert!(map_1.get("foo").unwrap().is_some_and(|value| value.eq(&0x12345678i64)));
    assert!(map_1.get("bar").unwrap().is_some_and(|value| value.eq(&0x87654321i64)));
    assert!(map_1.get("$?!").unwrap().is_none());
    assert!(map_1.get("baz").unwrap().is_some_and(|value| value.eq(&0x9ABCDEF0i64)));
    assert!(map_1.get("qux").unwrap().is_none());
    assert!(map_1.get("xyz").unwrap().is_none());
    assert!(map_1.get("cor").unwrap().is_none());

    assert_eq!(map_1.extend_map(&map_3, false).unwrap(), Some((3, 0)));

    assert!(map_1.get("foo").unwrap().is_some_and(|value| value.eq(&0x12345678i64)));
    assert!(map_1.get("bar").unwrap().is_some_and(|value| value.eq(&0x87654321i64)));
    assert!(map_1.get("$?!").unwrap().is_none());
    assert!(map_1.get("baz").unwrap().is_some_and(|value| value.eq(&0x9ABCDEF0i64)));
    assert!(map_1.get("qux").unwrap().is_some_and(|value| value.eq(&0xDEADBEEFi64)));
    assert!(map_1.get("xyz").unwrap().is_some_and(|value| value.eq(&0xCAFEBABEi64)));
    assert!(map_1.get("cor").unwrap().is_some_and(|value| value.eq(&0xFEEDFACEi64)));

    assert!(map_2.get("foo").unwrap().is_some_and(|value| value.eq(&0x12345678i64)));
    assert!(map_2.get("bar").unwrap().is_some_and(|value| value.eq(&0x87654321i64)));
    assert!(map_2.get("$?!").unwrap().is_none());
    assert!(map_2.get("baz").unwrap().is_some_and(|value| value.eq(&0x9ABCDEF0i64)));
    assert!(map_2.get("qux").unwrap().is_none());
    assert!(map_2.get("xyz").unwrap().is_none());
    assert!(map_2.get("cor").unwrap().is_none());

    cfg_if! {
        if #[cfg(feature = "hooks")] {
            assert_eq!(map_2.extend_map(&map_3, true).unwrap(), Some((3, 1)));
        } else {
            assert_eq!(map_2.extend_map(&map_3, true).unwrap(), None);
        }
    }

    assert!(map_2.get("foo").unwrap().is_some_and(|value| value.eq(&0x0FEDCBA9i64)));
    assert!(map_2.get("bar").unwrap().is_some_and(|value| value.eq(&0x87654321i64)));
    assert!(map_2.get("$?!").unwrap().is_none());
    assert!(map_2.get("baz").unwrap().is_some_and(|value| value.eq(&0x9ABCDEF0i64)));
    assert!(map_2.get("qux").unwrap().is_some_and(|value| value.eq(&0xDEADBEEFi64)));
    assert!(map_2.get("xyz").unwrap().is_some_and(|value| value.eq(&0xCAFEBABEi64)));
    assert!(map_2.get("cor").unwrap().is_some_and(|value| value.eq(&0xFEEDFACEi64)));
}

#[test]
fn test_sqlite_map_string_i64_update() {
    let mut map = SQLiteMap::<String, i64>::new().unwrap();

    {
        let mut tx = map.transaction().unwrap();
        assert!(tx.insert("foo".to_owned(), 0x12345678i64).unwrap());
        assert!(tx.insert("bar".to_owned(), 0x87654321i64).unwrap());
        assert!(tx.insert("baz".to_owned(), 0x90ABCDEFi64).unwrap());
    }

    let result_1 = map.extend(true, [("bar".to_owned(), 0xFEDCBA09i64), ("baz".to_owned(), 0xDEADBEEFi64), ("qux".to_owned(), 0xCAFEBABEi64)]).unwrap();
    cfg_if! {
        if #[cfg(feature = "hooks")] {
            assert_eq!(result_1, Some((1, 2)));
        } else {
            assert_eq!(result_1, None);
        }
    }

    assert!(map.get("foo").unwrap().is_some_and(|value| value.eq(&0x12345678i64)));
    assert!(map.get("bar").unwrap().is_some_and(|value| value.eq(&0xFEDCBA09i64)));
    assert!(map.get("baz").unwrap().is_some_and(|value| value.eq(&0xDEADBEEFi64)));
    assert!(map.get("qux").unwrap().is_some_and(|value| value.eq(&0xCAFEBABEi64)));

    let result_2 = map.extend_ref(true, [("baz", &0xBEBEBEBEi64), ("xyz", &0xDEAD10CCi64), ("bar", &0xC0DEBA5Ei64)]).unwrap();
    cfg_if! {
        if #[cfg(feature = "hooks")] {
            assert_eq!(result_2, Some((1, 2)));
        } else {
            assert_eq!(result_2, None);
        }
    }

    assert!(map.get("foo").unwrap().is_some_and(|value| value.eq(&0x12345678i64)));
    assert!(map.get("bar").unwrap().is_some_and(|value| value.eq(&0xC0DEBA5Ei64)));
    assert!(map.get("baz").unwrap().is_some_and(|value| value.eq(&0xBEBEBEBEi64)));
    assert!(map.get("qux").unwrap().is_some_and(|value| value.eq(&0xCAFEBABEi64)));
    assert!(map.get("xyz").unwrap().is_some_and(|value| value.eq(&0xDEAD10CCi64)));
}

// ---------------------------------------------------------------------------
// SQLiteMap File-based Test
// ---------------------------------------------------------------------------

#[test]
fn test_sqlite_map_filebased() {
    let temp_dir = tempdir_in(env!["CARGO_TARGET_TMPDIR"]).unwrap();
    let file_path_1 = temp_dir.path().join("sqlite_map-1.db");
    let file_path_2 = temp_dir.path().join("sqlite_map-2.db");

    assert!(!fs::exists(&file_path_1).unwrap());
    assert!(!fs::exists(&file_path_2).unwrap());

    assert!(matches!(
        SQLiteMap::<String, String>::try_from_file(&file_path_1, false, false),
        Err(Error::SqlError(RusqliteError::SqliteFailure(FfiError { code: FfiErrorCode::CannotOpen, .. }, _)))
    ));
    assert!(matches!(
        SQLiteMap::<String, String>::try_from_file(&file_path_2, false, false),
        Err(Error::SqlError(RusqliteError::SqliteFailure(FfiError { code: FfiErrorCode::CannotOpen, .. }, _)))
    ));

    assert!(matches!(
        SQLiteMap::<String, String>::try_from_file(&file_path_1, true, true),
        Err(Error::SqlError(RusqliteError::SqliteFailure(FfiError { code: FfiErrorCode::CannotOpen, .. }, _)))
    ));
    assert!(matches!(
        SQLiteMap::<String, String>::try_from_file(&file_path_2, true, true),
        Err(Error::SqlError(RusqliteError::SqliteFailure(FfiError { code: FfiErrorCode::CannotOpen, .. }, _)))
    ));

    {
        let mut map = SQLiteMap::<String, String>::try_from_file(&file_path_1, false, true).unwrap();
        {
            let mut tx = map.transaction().unwrap();
            assert!(tx.insert("foo".to_owned(), "12345678".to_owned()).unwrap());
            assert!(tx.insert("baz".to_owned(), "87654321".to_owned()).unwrap());
            assert!(tx.insert("BAZ".to_owned(), "abcdefgh".to_owned()).unwrap());
        }
    }

    {
        let mut map = SQLiteMap::<String, String>::try_from_file_nocase(&file_path_2, false, true).unwrap();
        {
            let mut tx = map.transaction().unwrap();
            assert!(tx.insert("bar".to_owned(), "hgfedcba".to_owned()).unwrap());
            assert!(tx.insert("qux".to_owned(), "stuvwxyz".to_owned()).unwrap());
            assert!(!tx.insert("QUX".to_owned(), "zyxwvuts".to_owned()).unwrap());
        }
    }

    assert!(fs::exists(&file_path_1).unwrap());
    assert!(fs::exists(&file_path_2).unwrap());

    {
        let mut map = SQLiteMap::<String, String>::try_from_file(&file_path_1, true, false).unwrap();
        assert!(map.get("foo").unwrap().is_some_and(|value| value.eq("12345678")));
        assert!(matches!(
            map.insert("xyz".to_owned(), "feedface".to_owned()),
            Err(Error::SqlError(RusqliteError::SqliteFailure(FfiError { code: FfiErrorCode::ReadOnly, .. }, _)))
        ));
    }

    {
        let mut map = SQLiteMap::<String, String>::try_from_file(&file_path_1, false, false).unwrap();
        assert!(map.insert("xyz".to_owned(), "feedface".to_owned()).unwrap());
    }

    {
        let map = SQLiteMap::<String, String>::try_from_file(&file_path_1, false, false).unwrap();
        assert!(map.get("foo").unwrap().is_some_and(|value| value.eq("12345678")));
        assert!(map.get("FOO").unwrap().is_none());
        assert!(map.get("bar").unwrap().is_none());
        assert!(map.get("baz").unwrap().is_some_and(|value| value.eq("87654321")));
        assert!(map.get("BAZ").unwrap().is_some_and(|value| value.eq("abcdefgh")));
        assert!(map.get("qux").unwrap().is_none());
        assert!(map.get("xyz").unwrap().is_some_and(|value| value.eq("feedface")));
    }

    {
        let map = SQLiteMap::<String, String>::try_from_file_nocase(&file_path_2, false, false).unwrap();
        assert!(map.get("foo").unwrap().is_none());
        assert!(map.get("bar").unwrap().is_some_and(|value| value.eq("hgfedcba")));
        assert!(map.get("BAR").unwrap().is_some_and(|value| value.eq("hgfedcba")));
        assert!(map.get("baz").unwrap().is_none());
        assert!(map.get("qux").unwrap().is_some_and(|value| value.eq("stuvwxyz")));
        assert!(map.get("QUX").unwrap().is_some_and(|value| value.eq("stuvwxyz")));
        assert!(map.get("xyz").unwrap().is_none());
    }

    assert!(matches!(SQLiteMap::<String, String>::try_from_file_nocase(&file_path_1, false, false), Err(Error::InvalidSchema)));
    assert!(matches!(SQLiteMap::<String, String>::try_from_file(&file_path_2, false, false), Err(Error::InvalidSchema)));
}

#[cfg(feature = "checked")]
#[test]
fn test_sqlite_map_filebased_integrity() {
    use std::{
        fs::OpenOptions,
        io::{Read, Seek, SeekFrom, Write},
    };

    let temp_dir = tempdir_in(env!["CARGO_TARGET_TMPDIR"]).unwrap();
    let file_path = temp_dir.path().join("sqlite_map.db");

    assert!(!fs::exists(&file_path).unwrap());

    {
        let mut map = SQLiteMap::<String, String>::try_from_file(&file_path, false, true).unwrap();
        {
            let mut tx = map.transaction().unwrap();
            assert!(tx.insert("foo".to_owned(), "12345678".to_owned()).unwrap());
            assert!(tx.insert("FOO".to_owned(), "87654321".to_owned()).unwrap());
            assert!(tx.insert("bar".to_owned(), "abcdefgh".to_owned()).unwrap());
            assert!(tx.insert("BAR".to_owned(), "hgfedcba".to_owned()).unwrap());
            assert!(tx.insert("baz".to_owned(), "stuvwxyz".to_owned()).unwrap());
            assert!(tx.insert("BAZ".to_owned(), "zyxwvuts".to_owned()).unwrap());
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

    assert!(matches!(SQLiteMap::<String, String>::try_from_file(&file_path, false, false), Err(Error::IntegrityCheck)));
}

#[cfg(feature = "backup")]
#[test]
fn test_sqlite_map_filebased_tofile() {
    let temp_dir = tempdir_in(env!["CARGO_TARGET_TMPDIR"]).unwrap();
    let file_path_1 = temp_dir.path().join("sqlite_map-1.db");
    let file_path_2 = temp_dir.path().join("sqlite_map-2.db");

    assert!(!fs::exists(&file_path_1).unwrap());
    assert!(!fs::exists(&file_path_2).unwrap());

    {
        let mut map = SQLiteMap::<String, String>::new().unwrap();
        {
            let mut tx = map.transaction().unwrap();
            assert!(tx.insert("foo".to_owned(), "12345678".to_owned()).unwrap());
            assert!(tx.insert("FOO".to_owned(), "87654321".to_owned()).unwrap());
            assert!(tx.insert("bar".to_owned(), "abcdefgh".to_owned()).unwrap());
            assert!(tx.insert("BAR".to_owned(), "hgfedcba".to_owned()).unwrap());
            assert!(tx.insert("baz".to_owned(), "stuvwxyz".to_owned()).unwrap());
            assert!(tx.insert("BAZ".to_owned(), "zyxwvuts".to_owned()).unwrap());
        }
        map.to_file(&file_path_1).unwrap();
    }

    {
        let map = SQLiteMap::<String, String>::try_from_file(&file_path_1, false, false).unwrap();
        assert!(map.get("foo").unwrap().is_some_and(|value| value.eq("12345678")));
        assert!(map.get("FOO").unwrap().is_some_and(|value| value.eq("87654321")));
        assert!(map.get("bar").unwrap().is_some_and(|value| value.eq("abcdefgh")));
        assert!(map.get("BAR").unwrap().is_some_and(|value| value.eq("hgfedcba")));
        assert!(map.get("baz").unwrap().is_some_and(|value| value.eq("stuvwxyz")));
        assert!(map.get("BAZ").unwrap().is_some_and(|value| value.eq("zyxwvuts")));
        assert!(map.get("qux").unwrap().is_none());
        assert!(map.get("xyz").unwrap().is_none());

        assert!(matches!(map.to_file(&file_path_1), Err(Error::SqlError(RusqliteError::SqliteFailure(..)))));
        assert!(map.to_file(&file_path_2).is_ok());
    }

    {
        let map = SQLiteMap::<String, String>::try_from_file(&file_path_2, true, false).unwrap();
        assert!(map.get("foo").unwrap().is_some_and(|value| value.eq("12345678")));
        assert!(map.get("FOO").unwrap().is_some_and(|value| value.eq("87654321")));
        assert!(map.get("bar").unwrap().is_some_and(|value| value.eq("abcdefgh")));
        assert!(map.get("BAR").unwrap().is_some_and(|value| value.eq("hgfedcba")));
        assert!(map.get("baz").unwrap().is_some_and(|value| value.eq("stuvwxyz")));
        assert!(map.get("BAZ").unwrap().is_some_and(|value| value.eq("zyxwvuts")));
        assert!(map.get("qux").unwrap().is_none());
        assert!(map.get("xyz").unwrap().is_none());
    }
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
            assert!(tx.insert_ref(key, value).unwrap());
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
                    assert_eq!(map.update_ref(key, value).unwrap(), Action::Insert);
                    action_counter[0] = action_counter[0].checked_add(1u64).unwrap();
                } else {
                    assert_eq!(map.update_ref(key, value).unwrap(), Action::Unknown);
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
                    assert_eq!(map.update_ref(key, value).unwrap(), Action::Update);
                    action_counter[0] = action_counter[0].checked_sub(1u64).unwrap();
                } else {
                    assert_eq!(map.update_ref(key, value).unwrap(), Action::Insert);
                    action_counter[1] = action_counter[1].checked_sub(1u64).unwrap();
                }
            } else {
                assert_eq!(map.update_ref(key, value).unwrap(), Action::Unknown);
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
                        assert_eq!(tx.update_ref(key, value).unwrap(), Action::Insert);
                        action_counter[0] = action_counter[0].checked_add(1u64).unwrap();
                    } else {
                        assert_eq!(tx.update_ref(key, value).unwrap(), Action::Unknown);
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
                        assert_eq!(tx.update_ref(key, value).unwrap(), Action::Update);
                        action_counter[0] = action_counter[0].checked_sub(1u64).unwrap();
                    } else {
                        assert_eq!(tx.update_ref(key, value).unwrap(), Action::Insert);
                        action_counter[1] = action_counter[1].checked_sub(1u64).unwrap();
                    }
                } else {
                    assert_eq!(tx.update_ref(key, value).unwrap(), Action::Unknown);
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
// SQLiteMap ToSet Test
// ---------------------------------------------------------------------------

#[test]
fn test_sqlite_map_to_set() {
    let mut map = SQLiteMap::<String, String>::new().unwrap();

    {
        let mut tx = map.transaction().unwrap();
        assert!(tx.insert("foo".to_owned(), "12345678".to_owned()).unwrap());
        assert!(tx.insert("FOO".to_owned(), "87654321".to_owned()).unwrap());
        assert!(tx.insert("bar".to_owned(), "abcdefgh".to_owned()).unwrap());
        assert!(tx.insert("BAR".to_owned(), "hgfedcba".to_owned()).unwrap());
        assert!(tx.insert("baz".to_owned(), "stuvwxyz".to_owned()).unwrap());
        assert!(tx.insert("BAZ".to_owned(), "zyxwvuts".to_owned()).unwrap());
    }

    let set_1 = map.to_set().unwrap();
    let set_2 = map.to_set_nocase().unwrap();

    assert_eq!(set_1.len().unwrap(), 6);
    assert_eq!(set_2.len().unwrap(), 3);

    assert!(set_1.contains("foo").unwrap());
    assert!(set_1.contains("FOO").unwrap());
    assert!(set_1.contains("bar").unwrap());
    assert!(set_1.contains("BAR").unwrap());
    assert!(set_1.contains("baz").unwrap());
    assert!(set_1.contains("BAZ").unwrap());

    assert!(set_2.contains("foo").unwrap());
    assert!(set_2.contains("bar").unwrap());
    assert!(set_2.contains("baz").unwrap());
}

// ---------------------------------------------------------------------------
// SQLiteMap Clone Test
// ---------------------------------------------------------------------------

cfg_if! {
    if #[cfg(feature = "backup")] {
        #[test]
        fn test_sqlite_map_clone() {
            let mut map = SQLiteMap::<String, String>::new().unwrap();

            {
                let mut tx = map.transaction().unwrap();
                assert!(tx.insert("foo".to_owned(), "12345678".to_owned()).unwrap());
                assert!(tx.insert("baz".to_owned(), "87654321".to_owned()).unwrap());
                assert!(tx.insert("BAZ".to_owned(), "abcdefgh".to_owned()).unwrap());
            }

            let mut map_clone = map.try_clone().unwrap();

            {
                let mut tx = map.transaction().unwrap();
                assert!(tx.insert("bar".to_owned(), "hgfedcba".to_owned()).unwrap());
                assert!(tx.insert("QUX".to_owned(), "stuvwxyz".to_owned()).unwrap());
                assert!(tx.update("foo".to_owned(), "deadbeef".to_owned()).is_ok());
            }

            {
                let mut tx = map_clone.transaction().unwrap();
                assert!(tx.insert("BAR".to_owned(), "zyxwvuts".to_owned()).unwrap());
                assert!(tx.insert("qux".to_owned(), "cafebabe".to_owned()).unwrap());
                assert!(tx.update("baz".to_owned(), "abad1dea".to_owned()).is_ok());
            }

            assert!(map.get("foo").unwrap().is_some_and(|value| value.eq("deadbeef")));
            assert!(map.get("baz").unwrap().is_some_and(|value| value.eq("87654321")));
            assert!(map.get("BAZ").unwrap().is_some_and(|value| value.eq("abcdefgh")));
            assert!(map.get("bar").unwrap().is_some_and(|value| value.eq("hgfedcba")));
            assert!(map.get("BAR").unwrap().is_none());
            assert!(map.get("QUX").unwrap().is_some_and(|value| value.eq("stuvwxyz")));
            assert!(map.get("qux").unwrap().is_none());

            assert!(map_clone.get("foo").unwrap().is_some_and(|value| value.eq("12345678")));
            assert!(map_clone.get("baz").unwrap().is_some_and(|value| value.eq("abad1dea")));
            assert!(map_clone.get("BAZ").unwrap().is_some_and(|value| value.eq("abcdefgh")));
            assert!(map_clone.get("bar").unwrap().is_none());
            assert!(map_clone.get("BAR").unwrap().is_some_and(|value| value.eq("zyxwvuts")));
            assert!(map_clone.get("QUX").unwrap().is_none());
            assert!(map_clone.get("qux").unwrap().is_some_and(|value| value.eq("cafebabe")));

            cfg_if! {
                if #[cfg(feature = "hooks")] {
                    let mut tx = map_clone.transaction().unwrap();
                    assert_eq!(tx.update("xyz".to_owned(), "deadc0de".to_owned()).unwrap(), Action::Insert);
                    assert_eq!(tx.update("xyz".to_owned(), "c0deba5e".to_owned()).unwrap(), Action::Update);
                }
            }
        }
    } else {
        #[test]
        fn test_sqlite_map_clone() {
            let mut map = SQLiteMap::<String, String>::new().unwrap();

            {
                let mut tx = map.transaction().unwrap();
                assert!(tx.insert("foo".to_owned(), "12345678".to_owned()).unwrap());
                assert!(tx.insert("baz".to_owned(), "87654321".to_owned()).unwrap());
                assert!(tx.insert("BAZ".to_owned(), "abcdefgh".to_owned()).unwrap());
            }

            assert!(matches!(map.try_clone(), Err(Error::Unsupported)));
        }
    }
}

// ---------------------------------------------------------------------------
// SQLiteMap Test to Destruction
// ---------------------------------------------------------------------------

#[ignore]
#[test]
fn test_sqlite_map_to_destruction() {
    let mut map = SQLiteMap::<String, String>::new().unwrap();
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

    let mut tx = map.transaction().unwrap();

    loop {
        let (key, value) = channel_rx.recv().expect("Out of items!");
        match tx.insert_ref(&key, &value) {
            Err(error) => match error.into_inner() {
                Some(sqlite_containers::rusqlite::Error::SqliteFailure(sqlite_error, _)) => match sqlite_error.code {
                    sqlite_containers::rusqlite::ErrorCode::OutOfMemory => break,
                    sqlite_containers::rusqlite::ErrorCode::DiskFull => break,
                    _ => panic!("Unexpected SQLite failure has been encountered! [{sqlite_error}]"),
                },
                _ => panic!("Unexpected SQLiteMap error has been encountered!"),
            },
            Ok(success) => assert!(success),
        }
    }
}
