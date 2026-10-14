// SQLiteMap
// This file is part of the 'SQLite-based containers for Rust' project (sqlite-containers)
// SPDX-License-Identifier: Unlicense

use crate::{
    CharType, Error, Size, SizeExt, Type,
    common::{OPEN_FLAGS, SQL_TYPE_TEXT, SQL_TYPES, SqlType, check_constraint_violation},
    rusqlite::{Connection, OptionalExtension, ToSql, Transaction, params},
};
use cfg_if::cfg_if;
use const_format::concatcp;
use std::{borrow::Borrow, collections::BTreeMap, marker::PhantomData, path::Path, sync::LazyLock};

// ---------------------------------------------------------------------------
// Statements
// ---------------------------------------------------------------------------

const TABLE_NAME: &str = "rusqlite_map_v2";

static SQL_CREATE_TBL: LazyLock<BTreeMap<(SqlType, SqlType), String>> = LazyLock::new(|| {
    let mut map = BTreeMap::<(SqlType, SqlType), String>::new();
    for key_type in SQL_TYPES {
        for value_type in SQL_TYPES {
            cfg_if! {
                if #[cfg(feature = "with_rowid")] {
                    map.insert((key_type, value_type), format!("CREATE TABLE IF NOT EXISTS {TABLE_NAME} (key {key_type} PRIMARY KEY NOT NULL, value {value_type} NOT NULL);"));
                } else {
                    map.insert((key_type, value_type), format!("CREATE TABLE IF NOT EXISTS {TABLE_NAME} (key {key_type} PRIMARY KEY NOT NULL, value {value_type} NOT NULL) WITHOUT ROWID;"));
                }
            }
        }
    }
    map
});

static SQL_CREATE_NOC: LazyLock<BTreeMap<SqlType, String>> = LazyLock::new(|| {
    let mut map = BTreeMap::<SqlType, String>::new();
    for value_type in SQL_TYPES {
        cfg_if! {
            if #[cfg(feature = "with_rowid")] {
                map.insert(value_type, format!("CREATE TABLE IF NOT EXISTS {TABLE_NAME} (key TEXT PRIMARY KEY NOT NULL COLLATE NOCASE, value {value_type} NOT NULL);"));
            } else {
                map.insert(value_type, format!("CREATE TABLE IF NOT EXISTS {TABLE_NAME} (key TEXT PRIMARY KEY NOT NULL COLLATE NOCASE, value {value_type} NOT NULL) WITHOUT ROWID;"));
            }
        }
    }
    map
});

const SQL_COUNT_KEYS: &str = concatcp!("SELECT COUNT(*) FROM ", TABLE_NAME, ";");
const SQL_EXIST_KEYS: &str = concatcp!("SELECT EXISTS (SELECT 1 FROM ", TABLE_NAME, ");");
const SQL_INSERT_KEY: &str = concatcp!("INSERT INTO ", TABLE_NAME, " (key, value) VALUES (?1, ?2);");
const SQL_UPSERT_KEY: &str = concatcp!("INSERT INTO ", TABLE_NAME, " (key, value) VALUES (?1, ?2) ON CONFLICT(key) DO UPDATE SET value = excluded.value;");
const SQL_EXISTS_KEY: &str = concatcp!("SELECT 1 FROM ", TABLE_NAME, " WHERE key = ? LIMIT 1;");
const SQL_EXISTS_NOC: &str = concatcp!("SELECT 1 FROM ", TABLE_NAME, " WHERE key COLLATE NOCASE = ? LIMIT 1;");
const SQL_LOOKUP_KEY: &str = concatcp!("SELECT value FROM ", TABLE_NAME, " WHERE key = ? LIMIT 1;");
const SQL_LOOKUP_NOC: &str = concatcp!("SELECT value FROM ", TABLE_NAME, " WHERE key COLLATE NOCASE = ? LIMIT 1;");
const SQL_QUERY_KEYS: &str = concatcp!("SELECT key FROM ", TABLE_NAME, ";");
const SQL_QUERY_PAIR: &str = concatcp!("SELECT key, value FROM ", TABLE_NAME, ";");
const SQL_GLOBA_KEYS: &str = concatcp!("SELECT key FROM ", TABLE_NAME, " WHERE key GLOB ?1;");
const SQL_GLOBN_KEYS: &str = concatcp!("SELECT key FROM ", TABLE_NAME, " WHERE key NOT GLOB ?1;");
const SQL_GLOBA_PAIR: &str = concatcp!("SELECT key, value FROM ", TABLE_NAME, " WHERE key GLOB ?1;");
const SQL_GLOBN_PAIR: &str = concatcp!("SELECT key, value FROM ", TABLE_NAME, " WHERE key NOT GLOB ?1;");
const SQL_DELETE_KEY: &str = concatcp!("DELETE FROM ", TABLE_NAME, " WHERE key = ?;");
const SQL_DELETE_NOC: &str = concatcp!("DELETE FROM ", TABLE_NAME, " WHERE key COLLATE NOCASE = ?;");
const SQL_DELETE_ALL: &str = concatcp!("DELETE FROM ", TABLE_NAME, ";");

// ---------------------------------------------------------------------------
// SQLiteMap
// ---------------------------------------------------------------------------

/// A [map](https://en.wikipedia.org/wiki/Associative_array) that stores key-value pairs with keys of type [`K`](crate::Type) and values of type [`V`](crate::Type), backed by an SQLite in-memory database.
///
/// It serves as a drop-in replacement for [`HashMap`](std::collections::HashMap).
///
/// By default, `SQLiteMap<String>` treats its keys as case-sensitive, but a case-insensitive variant is available. Even when using the case-sensitive map variant, for some operations a dedicated "case-insensitive" version is provided.
///
/// &nbsp;
///
/// <div class="warning">
///
/// For bulk inserts or updates, it is **strongly recommended** to use an explicit [transaction](Self::transaction). Without one, SQLite executes each insert or update in its own transaction, which can significantly degrade performance.
///
/// </div>
pub struct SQLiteMap<K, V>
where
    K: Type,
    V: Type,
{
    _types: (PhantomData<K>, PhantomData<V>),
    connection: Connection,
}

impl<K, V> SQLiteMap<K, V>
where
    K: Type,
    V: Type,
{
    /// Creates a new, empty hash map with case-sensitive keys, backed by an *in-memory* SQLite database.
    #[inline]
    pub fn new() -> Result<Self, Error> {
        Ok(Self { _types: (PhantomData, PhantomData), connection: Self::initialize_connection::<false, &Path>(None)? })
    }

    /// Creates a new, empty hash map with case-insensitive keys, backed by an *in-memory* SQLite database.
    ///
    /// Only ASCII characters are case folded. SQLite does **not** attempt to do full UTF case folding!
    #[inline]
    pub fn with_nocase() -> Result<Self, Error>
    where
        K: CharType,
    {
        Ok(Self { _types: (PhantomData, PhantomData), connection: Self::initialize_connection::<true, &Path>(None)? })
    }

    /// Creates a new or re-opens an existing hash map, backed by a *file-based* SQLite database.
    ///
    /// A new map is created with case-sensitive keys. However, if the database file already exists, its case-sensitivity is preserved.
    #[inline]
    pub fn with_file<P>(path: P) -> Result<Self, Error>
    where
        P: AsRef<Path>,
    {
        Ok(Self { _types: (PhantomData, PhantomData), connection: Self::initialize_connection::<false, _>(Some(path))? })
    }

    /// Creates a new or re-opens an existing hash map, backed by a *file-based* SQLite database.
    ///
    /// A new map is created with case-insensitive keys. However, if the database file already exists, its case-sensitivity is preserved.
    ///
    /// Only ASCII characters are case folded. SQLite does **not** attempt to do full UTF case folding!
    #[inline]
    pub fn with_file_nocase<P>(path: P) -> Result<Self, Error>
    where
        K: CharType,
        P: AsRef<Path>,
    {
        Ok(Self { _types: (PhantomData, PhantomData), connection: Self::initialize_connection::<true, _>(Some(path))? })
    }

    #[inline(always)]
    fn initialize_connection<const NO_CASE: bool, P>(path_option: Option<P>) -> Result<Connection, Error>
    where
        P: AsRef<Path>,
    {
        let connection = if let Some(path) = path_option.as_ref() { Connection::open_with_flags(path, *OPEN_FLAGS)? } else { Connection::open_in_memory()? };

        connection.pragma_update(None, "journal_mode", "OFF")?;
        connection.pragma_update(None, "synchronous", "OFF")?;

        if path_option.is_some() {
            connection.pragma_update(None, "temp_store", "MEMORY")?;
        }

        if !NO_CASE {
            connection.execute(SQL_CREATE_TBL.get(&(K::SQL_TYPE, V::SQL_TYPE)).unwrap(), [])?;
        } else {
            assert_eq!(K::SQL_TYPE, SQL_TYPE_TEXT);
            connection.execute(SQL_CREATE_NOC.get(&V::SQL_TYPE).unwrap(), [])?;
        }

        Ok(connection)
    }

    /// Starts a new SQLite transaction for this map.
    ///
    /// Please note that using an explicit SQLite transaction allows for much more efficient bulk inserts &#x1F680;
    ///
    /// Returns the new [`SQLiteMapTransaction`] instance.
    #[inline]
    pub fn transaction(&mut self) -> Result<SQLiteMapTransaction<'_, K, V>, Error> {
        SQLiteMapTransaction::from(&mut self.connection)
    }

    /// Inserts the given key-value pair into the map, if the specified key does *not* exist yet.
    ///
    /// If the map already contains the specified key, then its associated value is **not** changed by this operation!
    ///
    /// Returns `true`, if the key-value pair was inserted; otherwise returns `false`.
    ///
    /// Please use the [`update()`](Self::update) function to update the value associated with a key that may already exist.
    #[inline]
    pub fn insert<Q, R>(&mut self, key: &Q, value: &R) -> Result<bool, Error>
    where
        K: Borrow<Q>,
        Q: ?Sized + ToSql,
        V: Borrow<R>,
        R: ?Sized + ToSql,
    {
        let mut statement = self.connection.prepare_cached(SQL_INSERT_KEY)?;
        match statement.execute(params![key, value]) {
            Ok(_) => Ok(true),
            Err(error) => check_constraint_violation(error),
        }
    }

    /// Associates the specified value with the specified key, regardless of whether the key already exists in the map.
    ///
    /// If the map does *not* already contain the specified key, then the key is inserted automatically.
    ///
    /// **Note:** Due to limitations in SQLite, it is *not* possible to determine whether the key was inserted or updated.
    #[inline]
    pub fn update<Q, R>(&mut self, key: &Q, value: &R) -> Result<(), Error>
    where
        K: Borrow<Q>,
        Q: ?Sized + ToSql,
        V: Borrow<R>,
        R: ?Sized + ToSql,
    {
        let mut statement = self.connection.prepare_cached(SQL_UPSERT_KEY)?;
        statement.execute(params![key, value])?;
        Ok(())
    }

    /// Checks whether the map contains the specified key.
    ///
    /// For case-sensitive maps, the check is case-sensitive; for case-insensitive maps, the check is case-insensitive.
    ///
    /// Returns `true`, if the map contains the key; otherwise returns `false`.
    #[inline]
    pub fn contains_key<Q>(&self, key: &Q) -> Result<bool, Error>
    where
        K: Borrow<Q>,
        Q: ?Sized + ToSql,
    {
        let mut statement = self.connection.prepare_cached(SQL_EXISTS_KEY)?;
        Ok(statement.exists([key])?)
    }

    /// This is the "case-insensitive" version of the [`contains()`](Self::contains_key) function.
    ///
    /// The check is *always* performed case-insensitive.
    ///
    /// Returns `true`, if the map contains the key; otherwise returns `false`.
    #[inline]
    pub fn contains_key_nocase<Q>(&self, key: &Q) -> Result<bool, Error>
    where
        K: Borrow<Q> + CharType,
        Q: ?Sized + ToSql,
    {
        let mut statement = self.connection.prepare_cached(SQL_EXISTS_NOC)?;
        Ok(statement.exists([key])?)
    }

    /// Tries to retrieve the value for the specified key.
    ///
    /// For case-sensitive maps, the key is treated as case-sensitive; for case-insensitive maps, it is treated as case-insensitive.
    ///
    /// Returns the `Some(value)`, if the map contains the key; otherwise returns `None`.
    #[inline]
    pub fn get<Q>(&self, key: &Q) -> Result<Option<V>, Error>
    where
        K: Borrow<Q>,
        Q: ?Sized + ToSql,
    {
        let mut statement = self.connection.prepare_cached(SQL_LOOKUP_KEY)?;
        Ok(statement.query_one([key], |row| row.get(0)).optional()?)
    }

    /// This is the "case-insensitive" version of the [`get()`](Self::get) function.
    ///
    /// The key is *always* treated as case-insensitive.
    ///
    /// Returns the `Some(value)`, if the map contains the key; otherwise returns `None`.
    #[inline]
    pub fn get_nocase<Q>(&self, key: &Q) -> Result<Option<V>, Error>
    where
        K: Borrow<Q> + CharType,
        Q: ?Sized + ToSql,
    {
        let mut statement = self.connection.prepare_cached(SQL_LOOKUP_NOC)?;
        Ok(statement.query_one([key], |row| row.get(0)).optional()?)
    }

    /// Removes the specified key from the map, if present.
    ///
    /// For case-sensitive maps, the key is treated as case-sensitive; for case-insensitive maps, it is treated as case-insensitive.
    ///
    /// Returns `true`, if the map contained the key; otherwise returns `false`.
    #[inline]
    pub fn remove<Q>(&mut self, key: &Q) -> Result<bool, Error>
    where
        K: Borrow<Q>,
        Q: ?Sized + ToSql,
    {
        let mut statement = self.connection.prepare_cached(SQL_DELETE_KEY)?;
        Ok(statement.execute([key])? != 0)
    }

    /// This is the "case-insensitive" version of the [`remove()`](Self::remove) function.
    ///
    /// The key is *always* treated as case-insensitive.
    ///
    /// Returns `true`, if the map contained the key; otherwise returns `false`.
    #[inline]
    pub fn remove_nocase<Q>(&mut self, key: &Q) -> Result<bool, Error>
    where
        K: Borrow<Q> + CharType,
        Q: ?Sized + ToSql,
    {
        let mut statement = self.connection.prepare_cached(SQL_DELETE_NOC)?;
        Ok(statement.execute([key])? != 0)
    }

    /// Invokes the given `callback` function for each key that is currently contained in the map.
    ///
    /// This function does **not** guarantee a specific iteration order.
    #[inline]
    pub fn for_each_key<F>(&self, mut callback: F) -> Result<(), Error>
    where
        F: FnMut(K),
    {
        let mut statement = self.connection.prepare_cached(SQL_QUERY_KEYS)?;
        for key in statement.query_map([], |row| row.get(0))? {
            callback(key?);
        }
        Ok(())
    }

    /// Invokes the given `callback` function for each key-value pair that is currently contained in the map.
    ///
    /// This function does **not** guarantee a specific iteration order.
    #[inline]
    pub fn for_each_entry<F>(&self, mut callback: F) -> Result<(), Error>
    where
        F: FnMut(K, V),
    {
        let mut statement = self.connection.prepare_cached(SQL_QUERY_PAIR)?;
        let mut result = statement.query([])?;
        while let Some(key_value) = result.next()? {
            let (key, value): (K, V) = (key_value.get(0)?, key_value.get(1)?);
            callback(key, value);
        }
        Ok(())
    }

    /// Searches the map for the first key that satisfies the given `predicate`.
    ///
    /// Returns the first key that satisfies the given predicate, or `None` if none satisfies the predicate or the map is empty.
    ///
    /// This function does **not** guarantee a specific iteration order.
    ///
    /// Also, the predicate is **not** always tested on *all* keys, because the function returns at the first match.
    #[inline]
    pub fn find_key<P>(&self, predicate: P) -> Result<Option<K>, Error>
    where
        P: Fn(&K) -> bool,
    {
        let mut statement = self.connection.prepare_cached(SQL_QUERY_KEYS)?;
        for key in statement.query_map([], |row| row.get(0))? {
            let key: K = key?;
            if predicate(key.borrow()) {
                return Ok(Some(key));
            }
        }
        Ok(None)
    }

    /// Searches the map for the first key-value pair that satisfies the given `predicate`.
    ///
    /// Returns the first key-value pair that satisfies the given predicate, or `None` if none satisfies the predicate or the map is empty.
    ///
    /// This function does **not** guarantee a specific iteration order.
    ///
    /// Also, the predicate is **not** always tested on *all* key-value pairs, because the function returns at the first match.
    #[inline]
    pub fn find_entry<P>(&self, predicate: P) -> Result<Option<(K, V)>, Error>
    where
        P: Fn(&K, &V) -> bool,
    {
        let mut statement = self.connection.prepare_cached(SQL_QUERY_PAIR)?;
        let mut result = statement.query([])?;
        while let Some(key_value) = result.next()? {
            let (key, value): (K, V) = (key_value.get(0)?, key_value.get(1)?);
            if predicate(&key, &value) {
                return Ok(Some((key, value)));
            }
        }
        Ok(None)
    }

    /// Invokes the given `callback` function for each key that matches the given pattern.
    ///
    /// The pattern string is interpreted according to the [UNIX *globbing*](https://man7.org/linux/man-pages/man7/glob.7.html) rules. Specifically, a `?` character (not between brackets) matches any single character, and a `*` character (not between brackets) matches any string, including the empty string. Additionally, an expression `[…]` where the first character after the leading `[` is *not* an `!` matches a single character that can be any of the characters enclosed by the brackets; and an expression `[!…]` matches a single character that is *not* matched by the expression.
    ///
    /// This function is case-sensitive and does **not** guarantee a specific iteration order.
    ///
    /// If `invert` is `true`, returns all keys that do **not** match the pattern; otherwise the matching keys are returned.
    #[inline]
    pub fn glob_keys<F, Q>(&self, invert: bool, pattern: &Q, mut callback: F) -> Result<(), Error>
    where
        F: FnMut(K),
        K: Borrow<Q> + CharType,
        Q: ?Sized + ToSql,
    {
        let mut statement = self.connection.prepare_cached(if invert { SQL_GLOBN_KEYS } else { SQL_GLOBA_KEYS })?;
        for key in statement.query_map([pattern], |row| row.get(0))? {
            callback(key?);
        }
        Ok(())
    }

    /// Invokes the given `callback` function for each key-value pair whose key matches the given pattern.
    ///
    /// The pattern string is interpreted according to the [UNIX *globbing*](https://man7.org/linux/man-pages/man7/glob.7.html) rules. Specifically, a `?` character (not between brackets) matches any single character, and a `*` character (not between brackets) matches any string, including the empty string. Additionally, an expression `[…]` where the first character after the leading `[` is *not* an `!` matches a single character that can be any of the characters enclosed by the brackets; and an expression `[!…]` matches a single character that is *not* matched by the expression.
    ///
    /// This function is case-sensitive and does **not** guarantee a specific iteration order.
    ///
    /// If `invert` is `true`, returns all key-value pairs that do **not** match the pattern; otherwise the matching pairs are returned.
    #[inline]
    pub fn glob_entries<F, Q>(&self, invert: bool, pattern: &Q, mut callback: F) -> Result<(), Error>
    where
        F: FnMut(K, V),
        K: Borrow<Q> + CharType,
        Q: ?Sized + ToSql,
    {
        let mut statement = self.connection.prepare_cached(if invert { SQL_GLOBN_PAIR } else { SQL_GLOBA_PAIR })?;
        let mut result = statement.query([pattern])?;
        while let Some(key_value) = result.next()? {
            let (key, value): (K, V) = (key_value.get(0)?, key_value.get(1)?);
            callback(key, value);
        }
        Ok(())
    }

    /// Returns the number of unique keys in the map.
    #[inline]
    pub fn len(&self) -> Result<Size, Error> {
        let mut statement = self.connection.prepare_cached(SQL_COUNT_KEYS)?;
        let count: i64 = statement.query_one([], |row| row.get(0))?;
        Ok(Size::from_i64(count))
    }

    /// Returns `true` if the map contains **no** keys; otherwise returns `false`.
    #[inline]
    pub fn is_empty(&self) -> Result<bool, Error> {
        let mut statement = self.connection.prepare_cached(SQL_EXIST_KEYS)?;
        let any_elements: bool = statement.query_one([], |row| row.get(0))?;
        Ok(!any_elements)
    }

    /// Removes *all* keys from the map.
    #[inline]
    pub fn clear(&mut self) -> Result<(), Error> {
        let mut statement = self.connection.prepare_cached(SQL_DELETE_ALL)?;
        statement.execute([])?;
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// SQLiteMap Transaction
// ---------------------------------------------------------------------------

/// Represents an active SQLite transaction for a [`SQLiteMap`].
///
/// Most functions provided by this struct mirror the corresponding functions of the `SQLiteMap` struct.
///
/// The transaction is committed when the `SQLiteMapTransaction` is dropped.
pub struct SQLiteMapTransaction<'a, K, V>
where
    K: Type,
    V: Type,
{
    _types: (PhantomData<K>, PhantomData<V>),
    transaction: Transaction<'a>,
}

impl<'a, K, V> SQLiteMapTransaction<'a, K, V>
where
    K: Type,
    V: Type,
{
    #[inline]
    fn from(connection: &'a mut Connection) -> Result<Self, Error> {
        let mut transaction = connection.transaction()?;
        transaction.set_drop_behavior(rusqlite::DropBehavior::Commit);
        Ok(Self { _types: (PhantomData, PhantomData), transaction })
    }

    /// Drops the `SQLiteMapTransaction`, thereby committing the SQLite transaction.
    #[inline]
    pub fn commit(self) {}

    /// This function is equivalent to [`SQLiteMap::insert()`].
    #[inline]
    pub fn insert<Q, R>(&mut self, key: &Q, value: &R) -> Result<bool, Error>
    where
        K: Borrow<Q>,
        Q: ?Sized + ToSql,
        V: Borrow<R>,
        R: ?Sized + ToSql,
    {
        let mut statement = self.transaction.prepare_cached(SQL_INSERT_KEY)?;
        match statement.execute(params![key, value]) {
            Ok(_) => Ok(true),
            Err(error) => check_constraint_violation(error),
        }
    }

    /// This function is equivalent to [`SQLiteMap::update()`].
    #[inline]
    pub fn update<Q, R>(&mut self, key: &Q, value: &R) -> Result<(), Error>
    where
        K: Borrow<Q>,
        Q: ?Sized + ToSql,
        V: Borrow<R>,
        R: ?Sized + ToSql,
    {
        let mut statement = self.transaction.prepare_cached(SQL_UPSERT_KEY)?;
        statement.execute(params![key, value])?;
        Ok(())
    }

    /// This function is equivalent to [`SQLiteMap::contains_key()`].
    #[inline]
    pub fn contains_key<Q>(&self, key: &Q) -> Result<bool, Error>
    where
        K: Borrow<Q>,
        Q: ?Sized + ToSql,
    {
        let mut statement = self.transaction.prepare_cached(SQL_EXISTS_KEY)?;
        Ok(statement.exists([key])?)
    }

    /// This function is equivalent to [`SQLiteMap::contains_key_nocase()`].
    #[inline]
    pub fn contains_key_nocase<Q>(&self, key: &Q) -> Result<bool, Error>
    where
        K: Borrow<Q> + CharType,
        Q: ?Sized + ToSql,
    {
        let mut statement = self.transaction.prepare_cached(SQL_EXISTS_NOC)?;
        Ok(statement.exists([key])?)
    }

    /// This function is equivalent to [`SQLiteMap::get()`].
    #[inline]
    pub fn get<Q>(&self, key: &Q) -> Result<Option<V>, Error>
    where
        K: Borrow<Q>,
        Q: ?Sized + ToSql,
    {
        let mut statement = self.transaction.prepare_cached(SQL_LOOKUP_KEY)?;
        Ok(statement.query_one([key], |row| row.get(0)).optional()?)
    }

    /// This function is equivalent to [`SQLiteMap::get_nocase()`].
    #[inline]
    pub fn get_nocase<Q>(&self, key: &Q) -> Result<Option<V>, Error>
    where
        K: Borrow<Q> + CharType,
        Q: ?Sized + ToSql,
    {
        let mut statement = self.transaction.prepare_cached(SQL_LOOKUP_NOC)?;
        Ok(statement.query_one([key], |row| row.get(0)).optional()?)
    }

    /// This function is equivalent to [`SQLiteMap::remove()`].
    #[inline]
    pub fn remove<Q>(&mut self, key: &Q) -> Result<bool, Error>
    where
        K: Borrow<Q>,
        Q: ?Sized + ToSql,
    {
        let mut statement = self.transaction.prepare_cached(SQL_DELETE_KEY)?;
        Ok(statement.execute([key])? != 0)
    }

    /// This function is equivalent to [`SQLiteMap::remove_nocase()`].
    #[inline]
    pub fn remove_nocase<Q>(&mut self, key: &Q) -> Result<bool, Error>
    where
        K: Borrow<Q> + CharType,
        Q: ?Sized + ToSql,
    {
        let mut statement = self.transaction.prepare_cached(SQL_DELETE_NOC)?;
        Ok(statement.execute([key])? != 0)
    }

    /// This function is equivalent to [`SQLiteMap::for_each_key()`].
    #[inline]
    pub fn for_each_key<F>(&self, mut callback: F) -> Result<(), Error>
    where
        F: FnMut(K),
    {
        let mut statement = self.transaction.prepare_cached(SQL_QUERY_KEYS)?;
        for key in statement.query_map([], |row| row.get(0))? {
            callback(key?);
        }
        Ok(())
    }

    /// This function is equivalent to [`SQLiteMap::for_each_entry()`].
    #[inline]
    pub fn for_each_entry<F>(&self, mut callback: F) -> Result<(), Error>
    where
        F: FnMut(K, V),
    {
        let mut statement = self.transaction.prepare_cached(SQL_QUERY_PAIR)?;
        let mut result = statement.query([])?;
        while let Some(key_value) = result.next()? {
            let (key, value): (K, V) = (key_value.get(0)?, key_value.get(1)?);
            callback(key, value);
        }
        Ok(())
    }

    /// This function is equivalent to [`SQLiteMap::find_key()`].
    #[inline]
    pub fn find_key<P>(&self, predicate: P) -> Result<Option<K>, Error>
    where
        P: Fn(&K) -> bool,
    {
        let mut statement = self.transaction.prepare_cached(SQL_QUERY_KEYS)?;
        for key in statement.query_map([], |row| row.get(0))? {
            let key: K = key?;
            if predicate(key.borrow()) {
                return Ok(Some(key));
            }
        }
        Ok(None)
    }

    /// This function is equivalent to [`SQLiteMap::find_entry()`].
    #[inline]
    pub fn find_entry<P>(&self, predicate: P) -> Result<Option<(K, V)>, Error>
    where
        P: Fn(&K, &V) -> bool,
    {
        let mut statement = self.transaction.prepare_cached(SQL_QUERY_PAIR)?;
        let mut result = statement.query([])?;
        while let Some(key_value) = result.next()? {
            let (key, value): (K, V) = (key_value.get(0)?, key_value.get(1)?);
            if predicate(&key, &value) {
                return Ok(Some((key, value)));
            }
        }
        Ok(None)
    }

    /// This function is equivalent to [`SQLiteMap::glob_keys()`].
    #[inline]
    pub fn glob_keys<F, Q>(&self, invert: bool, pattern: &Q, mut callback: F) -> Result<(), Error>
    where
        F: FnMut(K),
        K: Borrow<Q> + CharType,
        Q: ?Sized + ToSql,
    {
        let mut statement = self.transaction.prepare_cached(if invert { SQL_GLOBN_KEYS } else { SQL_GLOBA_KEYS })?;
        for key in statement.query_map([pattern], |row| row.get(0))? {
            callback(key?);
        }
        Ok(())
    }

    /// This function is equivalent to [`SQLiteMap::glob_entries()`].
    #[inline]
    pub fn glob_entries<F, Q>(&self, invert: bool, pattern: &Q, mut callback: F) -> Result<(), Error>
    where
        F: FnMut(K, V),
        K: Borrow<Q> + CharType,
        Q: ?Sized + ToSql,
    {
        let mut statement = self.transaction.prepare_cached(if invert { SQL_GLOBN_PAIR } else { SQL_GLOBA_PAIR })?;
        let mut result = statement.query([pattern])?;
        while let Some(key_value) = result.next()? {
            let (key, value): (K, V) = (key_value.get(0)?, key_value.get(1)?);
            callback(key, value);
        }
        Ok(())
    }

    /// This function is equivalent to [`SQLiteMap::len()`].
    #[inline]
    pub fn len(&self) -> Result<Size, Error> {
        let mut statement = self.transaction.prepare_cached(SQL_COUNT_KEYS)?;
        let count: i64 = statement.query_one([], |row| row.get(0))?;
        Ok(Size::from_i64(count))
    }

    /// This function is equivalent to [`SQLiteMap::is_empty()`].
    #[inline]
    pub fn is_empty(&self) -> Result<bool, Error> {
        let mut statement = self.transaction.prepare_cached(SQL_EXIST_KEYS)?;
        let any_elements: bool = statement.query_one([], |row| row.get(0))?;
        Ok(!any_elements)
    }

    /// This function is equivalent to [`SQLiteMap::clear()`].
    #[inline]
    pub fn clear(&mut self) -> Result<(), Error> {
        let mut statement = self.transaction.prepare_cached(SQL_DELETE_ALL)?;
        statement.execute([])?;
        Ok(())
    }
}
