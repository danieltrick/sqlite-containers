// SQLiteMap
// This file is part of the 'SQLite-based containers for Rust' project (sqlite-containers)
// SPDX-License-Identifier: Unlicense

use rusqlite::OptionalExtension;

use crate::common::{Error, SizeT, check_constraint_violation};
use crate::rusqlite::{Connection, Transaction};

// ---------------------------------------------------------------------------
// Statements
// ---------------------------------------------------------------------------

const SQL_CREATE_TBL: &str = "CREATE TABLE data (key TEXT PRIMARY KEY NOT NULL, value TEXT NOT NULL) WITHOUT ROWID;";
const SQL_CREATE_NOC: &str = "CREATE TABLE data (key TEXT PRIMARY KEY NOT NULL COLLATE NOCASE, value TEXT NOT NULL) WITHOUT ROWID;";
const SQL_COUNT_KEYS: &str = "SELECT COUNT(*) FROM data;";
const SQL_INSERT_KEY: &str = "INSERT INTO data (key, value) VALUES (?1, ?2);";
const SQL_EXISTS_KEY: &str = "SELECT 1 FROM data WHERE key = ? LIMIT 1;";
const SQL_EXISTS_NOC: &str = "SELECT 1 FROM data WHERE key COLLATE NOCASE = ? LIMIT 1;";
const SQL_LOOKUP_KEY: &str = "SELECT value FROM data WHERE key = ? LIMIT 1;";
const SQL_LOOKUP_NOC: &str = "SELECT value FROM data WHERE key COLLATE NOCASE = ? LIMIT 1;";
const SQL_QUERY_KEYS: &str = "SELECT (key, value) FROM data;";
const SQL_DELETE_KEY: &str = "DELETE FROM data WHERE key = ?;";
const SQL_DELETE_NOC: &str = "DELETE FROM data WHERE key COLLATE NOCASE = ?;";
const SQL_DELETE_ALL: &str = "DELETE FROM data;";

// ---------------------------------------------------------------------------
// SQLiteMap
// ---------------------------------------------------------------------------

/// A [hash map](https://doc.rust-lang.org/std/collections/struct.HashMap.html) with [string](https://doc.rust-lang.org/beta/std/string/struct.String.html) keys and values that is backed by an SQLite in-memory database.
///
/// By default, `SQLiteMap` treats its keys as case-sensitive, but a case-insensitive variant is available. Even when using the case-sensitive map variant, for some operations a dedicated "case-insensitive" version is provided.
///
/// <div class="warning">
///
/// **Important:** If you need to perform a large number of inserts, it is *highly recommended* to start an explicit [transaction](Self::transaction) and use it for the bulk insert. Otherwise, SQLite handles each insert as a separate transaction, which can be very slow!
///
/// </div>
pub struct SQLiteMap {
    connection: Connection,
}

impl SQLiteMap {
    /// Creates a new, empty SQLite-backed hash map with case-sensitive keys.
    pub fn new() -> Result<Self, Error> {
        Ok(Self { connection: Self::initialize_connection(false)? })
    }

    /// Creates a new, empty SQLite-backed hash map with case-insensitive keys.
    pub fn with_nocase() -> Result<Self, Error> {
        Ok(Self { connection: Self::initialize_connection(true)? })
    }

    fn initialize_connection(no_case: bool) -> Result<Connection, Error> {
        let connection = Connection::open_in_memory()?;
        connection.pragma_update(None, "journal_mode", "OFF")?;
        connection.pragma_update(None, "synchronous", "OFF")?;
        connection.pragma_update(None, "temp_store", "MEMORY")?;
        if !no_case {
            connection.execute(SQL_CREATE_TBL, [])?;
        } else {
            connection.execute(SQL_CREATE_NOC, [])?;
        }
        Ok(connection)
    }

    /// Starts a new SQLite transaction for this map.
    ///
    /// Please note that using an explicit SQLite transaction allows for much more efficient bulk inserts &#x1F680;
    ///
    /// Returns the new [`SQLiteMapTransaction`] instance.
    pub fn transaction(&mut self) -> Result<SQLiteMapTransaction<'_>, Error> {
        SQLiteMapTransaction::from(&mut self.connection)
    }

    /// Inserts the given key-value pair into the map.
    ///
    /// Returns `true`, if the map did not already contain the key; otherwise returns `false`.
    #[inline]
    pub fn insert(&mut self, key: &str, value: &str) -> Result<bool, Error> {
        let mut insert = self.connection.prepare_cached(SQL_INSERT_KEY)?;
        match insert.execute([key, value]) {
            Ok(_) => Ok(true),
            Err(error) => check_constraint_violation(error),
        }
    }

    /// Checks whether the map contains the specified key.
    ///
    /// For case-sensitive maps, the check is case-sensitive; for case-insensitive maps, the check is case-insensitive.
    ///
    /// Returns `true`, if the map contains the key; otherwise returns `false`.
    #[inline]
    pub fn contains(&self, key: &str) -> Result<bool, Error> {
        let mut contains = self.connection.prepare_cached(SQL_EXISTS_KEY)?;
        Ok(contains.exists([key])?)
    }

    /// This is the "case-insensitive" version of the [`contains()`](Self::contains) function.
    ///
    /// The check is *always* performed case-insensitive.
    ///
    /// Returns `true`, if the map contains the key; otherwise returns `false`.
    #[inline]
    pub fn contains_nocase(&self, key: &str) -> Result<bool, Error> {
        let mut contains = self.connection.prepare_cached(SQL_EXISTS_NOC)?;
        Ok(contains.exists([key])?)
    }

    /// Tries to retriece the value for the specified key.
    ///
    /// For case-sensitive maps, the key is treated as case-sensitive; for case-insensitive maps, it is treated as case-insensitive.
    ///
    /// Returns the `Some(value)`, if the map contains the key; otherwise returns `None`.
    #[inline]
    pub fn get(&self, key: &str) -> Result<Option<String>, Error> {
        let mut get = self.connection.prepare_cached(SQL_LOOKUP_KEY)?;
        Ok(get.query_one([key], |row| row.get(0)).optional()?)
    }

    /// This is the "case-insensitive" version of the [`contains()`](Self::get) function.
    ///
    /// The key is *always* treated as case-insensitive.
    ///
    /// Returns the `Some(value)`, if the map contains the key; otherwise returns `None`.
    #[inline]
    pub fn get_nocase(&self, key: &str) -> Result<Option<String>, Error> {
        let mut get = self.connection.prepare_cached(SQL_LOOKUP_NOC)?;
        Ok(get.query_one([key], |row| row.get(0)).optional()?)
    }

    /// Removes the specified key from the map, if present.
    ///
    /// For case-sensitive maps, the key is treated as case-sensitive; for case-insensitive maps, it is treated as case-insensitive.
    ///
    /// Returns `true`, if the map contained the key; otherwise returns `false`.
    #[inline]
    pub fn remove(&mut self, key: &str) -> Result<bool, Error> {
        let mut contains = self.connection.prepare_cached(SQL_DELETE_KEY)?;
        Ok(contains.execute([key])? != 0)
    }

    /// This is the "case-insensitive" version of the [`remove()`](Self::remove) function.
    ///
    /// The key is *always* treated as case-insensitive.
    ///
    /// Returns `true`, if the map contained the key; otherwise returns `false`.
    #[inline]
    pub fn remove_nocase(&mut self, key: &str) -> Result<bool, Error> {
        let mut contains = self.connection.prepare_cached(SQL_DELETE_NOC)?;
        Ok(contains.execute([key])? != 0)
    }

    /// Invokes the given `callback` function for each key-value pair that is currently contained in the map.
    ///
    /// This function does **not** guarantee a specific iteration order.
    #[inline]
    pub fn for_each<F>(&self, mut callback: F) -> Result<(), Error>
    where
        F: FnMut(&str, &str),
    {
        let mut iter = self.connection.prepare_cached(SQL_QUERY_KEYS)?;
        let mut result = iter.query([])?;
        while let Some(current_item) = result.next()? {
            let key: String = current_item.get(0)?;
            let value: String = current_item.get(1)?;
            callback(&key, &value);
        }
        Ok(())
    }

    /// Searches the map for the first key-value pair that satisfies the given `predicate`.
    ///
    /// Returns the first key-value pair that satisfies the given predicate, or `None` if **no** key-value pair satisfies the predicate or the map is empty.
    ///
    /// This function does **not** guarantee a specific iteration order.
    ///
    /// Also, the predicate is **not** always tested on *all* key-value pairs, because the function returns at the first match.
    #[inline]
    pub fn find<P>(&self, predicate: P) -> Result<Option<(String, String)>, Error>
    where
        P: Fn(&str, &str) -> bool,
    {
        let mut iter = self.connection.prepare_cached(SQL_QUERY_KEYS)?;
        let mut result = iter.query([])?;
        while let Some(current_item) = result.next()? {
            let key: String = current_item.get(0)?;
            let value: String = current_item.get(1)?;
            if predicate(&key, &value) {
                return Ok(Some((key, value)));
            }
        }
        Ok(None)
    }

    /// Returns the number of unique keys in the map.
    #[inline]
    pub fn len(&self) -> Result<SizeT, Error> {
        let mut query_count = self.connection.prepare_cached(SQL_COUNT_KEYS)?;
        let count: i64 = query_count.query_one([], |row| row.get(0))?;
        Ok(count.try_into().unwrap_or_default())
    }

    /// Returns `true` if the map contains **no** keys; otherwise returns `false`.
    #[inline]
    pub fn is_empty(&self) -> Result<bool, Error> {
        Ok(self.len()? == 0)
    }

    /// Removes *all* keys from the map.
    #[inline]
    pub fn clear(&mut self) -> Result<(), Error> {
        let mut clear = self.connection.prepare_cached(SQL_DELETE_ALL)?;
        clear.execute([])?;
        Ok(())
    }
}

impl Default for SQLiteMap {
    /// Returns a new, empty map, as created by the [`SQLiteMap::new()`] function.
    ///
    /// # Panics
    ///
    /// Panics if a new `SQLiteMap` instance could **not** be created, e.g., because of an SQLite error.
    fn default() -> Self {
        Self::new().expect("Failed to create SQLiteMap instance!")
    }
}

// ---------------------------------------------------------------------------
// SQLiteMap Transaction
// ---------------------------------------------------------------------------

/// Represents an active SQLite transaction for a [`SQLiteMap`].
///
/// Most functions provided by this struct mirror the corresponding functions of the `SQLiteMap` struct.
///
/// The transaction is committed automatically when the `SQLiteMapTransaction` is dropped.
///
/// <div class="warning">
///
/// **Important:** If you need to perform a large number of inserts, it is *highly recommended* to start an explicit [transaction](SQLiteMap::transaction) and use it for the bulk insert. Otherwise, SQLite handles each insert as a separate transaction, which can be very slow!
///
/// </div>
pub struct SQLiteMapTransaction<'a> {
    transaction: Transaction<'a>,
}

impl<'a> SQLiteMapTransaction<'a> {
    fn from(connection: &'a mut Connection) -> Result<Self, Error> {
        let mut transaction = connection.transaction()?;
        transaction.set_drop_behavior(rusqlite::DropBehavior::Commit);
        Ok(Self { transaction })
    }

    /// Drops the `SQLiteMapTransaction`, thereby committing the SQLite transaction.
    pub fn commit(self) {}

    /// This function is equivalent to [`SQLiteMap::insert()`].
    #[inline]
    pub fn insert(&mut self, key: &str, value: &str) -> Result<bool, Error> {
        let mut insert = self.transaction.prepare_cached(SQL_INSERT_KEY)?;
        match insert.execute([key, value]) {
            Ok(_) => Ok(true),
            Err(error) => check_constraint_violation(error),
        }
    }

    /// This function is equivalent to [`SQLiteMap::contains()`].
    #[inline]
    pub fn contains(&self, key: &str) -> Result<bool, Error> {
        let mut contains = self.transaction.prepare_cached(SQL_EXISTS_KEY)?;
        Ok(contains.exists([key])?)
    }

    /// This function is equivalent to [`SQLiteMap::contains_nocase()`].
    #[inline]
    pub fn contains_nocase(&self, key: &str) -> Result<bool, Error> {
        let mut contains = self.transaction.prepare_cached(SQL_EXISTS_NOC)?;
        Ok(contains.exists([key])?)
    }

    /// This function is equivalent to [`SQLiteMap::get()`].
    #[inline]
    pub fn get(&self, key: &str) -> Result<Option<String>, Error> {
        let mut get = self.transaction.prepare_cached(SQL_LOOKUP_KEY)?;
        Ok(get.query_one([key], |row| row.get(0)).optional()?)
    }

    /// This function is equivalent to [`SQLiteMap::get_nocase()`].
    #[inline]
    pub fn get_nocase(&self, key: &str) -> Result<Option<String>, Error> {
        let mut get = self.transaction.prepare_cached(SQL_LOOKUP_NOC)?;
        Ok(get.query_one([key], |row| row.get(0)).optional()?)
    }

    /// This function is equivalent to [`SQLiteMap::remove()`].
    #[inline]
    pub fn remove(&mut self, key: &str) -> Result<bool, Error> {
        let mut contains = self.transaction.prepare_cached(SQL_DELETE_KEY)?;
        Ok(contains.execute([key])? != 0)
    }

    /// This function is equivalent to [`SQLiteMap::remove_nocase()`].
    #[inline]
    pub fn remove_nocase(&mut self, key: &str) -> Result<bool, Error> {
        let mut contains = self.transaction.prepare_cached(SQL_DELETE_NOC)?;
        Ok(contains.execute([key])? != 0)
    }

    /// This function is equivalent to [`SQLiteMap::for_each()`].
    #[inline]
    pub fn for_each<F>(&self, mut callback: F) -> Result<(), Error>
    where
        F: FnMut(&str, &str),
    {
        let mut iter = self.transaction.prepare_cached(SQL_QUERY_KEYS)?;
        let mut result = iter.query([])?;
        while let Some(current_item) = result.next()? {
            let key: String = current_item.get(0)?;
            let value: String = current_item.get(1)?;
            callback(&key, &value);
        }
        Ok(())
    }

    /// This function is equivalent to [`SQLiteMap::find()`].
    #[inline]
    pub fn find<P>(&self, predicate: P) -> Result<Option<(String, String)>, Error>
    where
        P: Fn(&str, &str) -> bool,
    {
        let mut iter = self.transaction.prepare_cached(SQL_QUERY_KEYS)?;
        let mut result = iter.query([])?;
        while let Some(current_item) = result.next()? {
            let key: String = current_item.get(0)?;
            let value: String = current_item.get(1)?;
            if predicate(&key, &value) {
                return Ok(Some((key, value)));
            }
        }
        Ok(None)
    }

    /// This function is equivalent to [`SQLiteMap::len()`].
    #[inline]
    pub fn len(&self) -> Result<SizeT, Error> {
        let mut query_count = self.transaction.prepare_cached(SQL_COUNT_KEYS)?;
        let count: i64 = query_count.query_one([], |row| row.get(0))?;
        Ok(count.try_into().unwrap_or_default())
    }

    /// This function is equivalent to [`SQLiteMap::is_empty()`].
    #[inline]
    pub fn is_empty(&self) -> Result<bool, Error> {
        Ok(self.len()? == 0)
    }

    /// This function is equivalent to [`SQLiteMap::clear()`].
    #[inline]
    pub fn clear(&mut self) -> Result<(), Error> {
        let mut clear = self.transaction.prepare_cached(SQL_DELETE_ALL)?;
        clear.execute([])?;
        Ok(())
    }
}
