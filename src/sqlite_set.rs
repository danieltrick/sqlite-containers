// SQLiteSet
// This file is part of the 'SQLite-based containers for Rust' project (sqlite-containers)
// SPDX-License-Identifier: Unlicense

use crate::common::{Error, SizeT, check_constraint_violation};
use crate::rusqlite::{Connection, Transaction};

// ---------------------------------------------------------------------------
// Statements
// ---------------------------------------------------------------------------

const SQL_CREATE_TBL: &str = "CREATE TABLE data (key TEXT PRIMARY KEY NOT NULL) WITHOUT ROWID;";
const SQL_CREATE_NOC: &str = "CREATE TABLE data (key TEXT PRIMARY KEY NOT NULL COLLATE NOCASE) WITHOUT ROWID;";
const SQL_COUNT_KEYS: &str = "SELECT COUNT(*) FROM data;";
const SQL_INSERT_KEY: &str = "INSERT INTO data (key) VALUES (?1);";
const SQL_LOOKUP_KEY: &str = "SELECT 1 FROM data WHERE key = ? LIMIT 1;";
const SQL_LOOKUP_NOC: &str = "SELECT 1 FROM data WHERE key COLLATE NOCASE = ? LIMIT 1;";
const SQL_QUERY_KEYS: &str = "SELECT key FROM data;";
const SQL_DELETE_KEY: &str = "DELETE FROM data WHERE key = ?;";
const SQL_DELETE_NOC: &str = "DELETE FROM data WHERE key COLLATE NOCASE = ?;";
const SQL_DELETE_ALL: &str = "DELETE FROM data;";

// ---------------------------------------------------------------------------
// SQLiteSet
// ---------------------------------------------------------------------------

/// A [hash set](https://doc.rust-lang.org/std/collections/struct.HashSet.html) with [string](https://doc.rust-lang.org/beta/std/string/struct.String.html) keys that is backed by an SQLite in-memory database.
///
/// By default, `SQLiteSet` treats its keys as case-sensitive, but a case-insensitive variant is available. Even when using the case-sensitive set variant, for some operations a dedicated "case-insensitive" version is provided.
///
/// <div class="warning">
///
/// **Important:** If you need to perform a large number of inserts, it is *highly recommended* to start an explicit [transaction](Self::transaction) and use it for the bulk insert. Otherwise, SQLite handles each insert as a separate transaction, which can be very slow!
///
/// </div>
pub struct SQLiteSet {
    connection: Connection,
}

impl SQLiteSet {
    /// Creates a new, empty SQLite-backed hash set with case-sensitive keys.
    pub fn new() -> Result<Self, Error> {
        Ok(Self { connection: Self::initialize_connection(false)? })
    }

    /// Creates a new, empty SQLite-backed hash set with case-insensitive keys.
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

    /// Starts a new SQLite transaction for this set.
    ///
    /// Please note that using an explicit SQLite transaction allows for much more efficient bulk inserts &#x1F680;
    ///
    /// Returns the new [`SQLiteSetTransaction`] instance.
    pub fn transaction(&mut self) -> Result<SQLiteSetTransaction<'_>, Error> {
        SQLiteSetTransaction::from(&mut self.connection)
    }

    /// Inserts the given key into the set.
    ///
    /// Returns `true`, if the set did not already contain the key; otherwise returns `false`.
    #[inline]
    pub fn insert(&mut self, key: &str) -> Result<bool, Error> {
        let mut insert = self.connection.prepare_cached(SQL_INSERT_KEY)?;
        match insert.execute([key]) {
            Ok(_) => Ok(true),
            Err(error) => check_constraint_violation(error),
        }
    }

    /// Checks whether the set contains the specified key.
    ///
    /// For case-sensitive sets, the check is case-sensitive; for case-insensitive sets, the check is case-insensitive.
    ///
    /// Returns `true`, if the set contains the key; otherwise returns `false`.
    #[inline]
    pub fn contains(&self, key: &str) -> Result<bool, Error> {
        let mut contains = self.connection.prepare_cached(SQL_LOOKUP_KEY)?;
        Ok(contains.exists([key])?)
    }

    /// This is the "case-insensitive" version of the [`contains()`](Self::contains) function.
    ///
    /// The check is *always* performed case-insensitive.
    ///
    /// Returns `true`, if the set contains the key; otherwise returns `false`.
    #[inline]
    pub fn contains_nocase(&self, key: &str) -> Result<bool, Error> {
        let mut contains = self.connection.prepare_cached(SQL_LOOKUP_NOC)?;
        Ok(contains.exists([key])?)
    }

    /// Removes the specified key from the set, if present.
    ///
    /// For case-sensitive sets, the key is treated as case-sensitive; for case-insensitive sets, it is treated as case-insensitive.
    ///
    /// Returns `true`, if the set contained the key; otherwise returns `false`.
    #[inline]
    pub fn remove(&mut self, key: &str) -> Result<bool, Error> {
        let mut contains = self.connection.prepare_cached(SQL_DELETE_KEY)?;
        Ok(contains.execute([key])? != 0)
    }

    /// This is the "case-insensitive" version of the [`remove()`](Self::remove) function.
    ///
    /// The key is *always* treated as case-insensitive.
    ///
    /// Returns `true`, if the set contained the key; otherwise returns `false`.
    #[inline]
    pub fn remove_nocase(&mut self, key: &str) -> Result<bool, Error> {
        let mut contains = self.connection.prepare_cached(SQL_DELETE_NOC)?;
        Ok(contains.execute([key])? != 0)
    }

    /// Invokes the given `callback` function for each key that is currently contained in the set.
    ///
    /// This function does **not** guarantee a specific iteration order.
    #[inline]
    pub fn for_each<F>(&self, mut callback: F) -> Result<(), Error>
    where
        F: FnMut(&str),
    {
        let mut iter = self.connection.prepare_cached(SQL_QUERY_KEYS)?;
        let mut result = iter.query([])?;
        while let Some(current_item) = result.next()? {
            let key: String = current_item.get(0)?;
            callback(&key);
        }
        Ok(())
    }

    /// Searches the set for the first key that satisfies the given `predicate`.
    ///
    /// Returns the first key that satisfies the given predicate, or `None` if **no** key satisfies the predicate or the set is empty.
    ///
    /// This function does **not** guarantee a specific iteration order.
    ///
    /// Also, the predicate is **not** always tested on *all* keys, because the function returns at the first match.
    #[inline]
    pub fn find<P>(&self, predicate: P) -> Result<Option<String>, Error>
    where
        P: Fn(&str) -> bool,
    {
        let mut iter = self.connection.prepare_cached(SQL_QUERY_KEYS)?;
        let mut result = iter.query([])?;
        while let Some(current_item) = result.next()? {
            let key: String = current_item.get(0)?;
            if predicate(&key) {
                return Ok(Some(key));
            }
        }
        Ok(None)
    }

    /// Returns the number of unique keys in the set.
    #[inline]
    pub fn len(&self) -> Result<SizeT, Error> {
        let mut query_count = self.connection.prepare_cached(SQL_COUNT_KEYS)?;
        let count: i64 = query_count.query_one([], |row| row.get(0))?;
        Ok(count.try_into().unwrap_or_default())
    }

    /// Returns `true` if the set contains **no** keys; otherwise returns `false`.
    #[inline]
    pub fn is_empty(&self) -> Result<bool, Error> {
        Ok(self.len()? == 0)
    }

    /// Removes *all* keys from the set.
    #[inline]
    pub fn clear(&mut self) -> Result<(), Error> {
        let mut clear = self.connection.prepare_cached(SQL_DELETE_ALL)?;
        clear.execute([])?;
        Ok(())
    }
}

impl Default for SQLiteSet {
    /// Returns a new, empty set, as created by the [`SQLiteSet::new()`] function.
    ///
    /// # Panics
    ///
    /// Panics if a new `SQLiteSet` instance could **not** be created, e.g., because of an SQLite error.
    fn default() -> Self {
        Self::new().expect("Failed to create SQLiteSet instance!")
    }
}

// ---------------------------------------------------------------------------
// SQLiteSet Transaction
// ---------------------------------------------------------------------------

/// Represents an active SQLite transaction for a [`SQLiteSet`].
///
/// Most functions provided by this struct mirror the corresponding functions of the `SQLiteSet` struct.
///
/// The transaction is committed automatically when the `SQLiteSetTransaction` is dropped.
///
/// <div class="warning">
///
/// **Important:** If you need to perform a large number of inserts, it is *highly recommended* to start an explicit [transaction](SQLiteSet::transaction) and use it for the bulk insert. Otherwise, SQLite handles each insert as a separate transaction, which can be very slow!
///
/// </div>
pub struct SQLiteSetTransaction<'a> {
    transaction: Transaction<'a>,
}

impl<'a> SQLiteSetTransaction<'a> {
    fn from(connection: &'a mut Connection) -> Result<Self, Error> {
        let mut transaction = connection.transaction()?;
        transaction.set_drop_behavior(rusqlite::DropBehavior::Commit);
        Ok(Self { transaction })
    }

    /// Drops the `SQLiteSetTransaction`, thereby committing the SQLite transaction.
    pub fn commit(self) {}

    /// This function is equivalent to [`SQLiteSet::insert()`].
    #[inline]
    pub fn insert(&mut self, key: &str) -> Result<bool, Error> {
        let mut insert = self.transaction.prepare_cached(SQL_INSERT_KEY)?;
        match insert.execute([key]) {
            Ok(_) => Ok(true),
            Err(error) => check_constraint_violation(error),
        }
    }

    /// This function is equivalent to [`SQLiteSet::contains()`].
    #[inline]
    pub fn contains(&self, key: &str) -> Result<bool, Error> {
        let mut contains = self.transaction.prepare_cached(SQL_LOOKUP_KEY)?;
        Ok(contains.exists([key])?)
    }

    /// This function is equivalent to [`SQLiteSet::contains_nocase()`].
    #[inline]
    pub fn contains_nocase(&self, key: &str) -> Result<bool, Error> {
        let mut contains = self.transaction.prepare_cached(SQL_LOOKUP_NOC)?;
        Ok(contains.exists([key])?)
    }

    /// This function is equivalent to [`SQLiteSet::remove()`].
    #[inline]
    pub fn remove(&mut self, key: &str) -> Result<bool, Error> {
        let mut contains = self.transaction.prepare_cached(SQL_DELETE_KEY)?;
        Ok(contains.execute([key])? != 0)
    }

    /// This function is equivalent to [`SQLiteSet::remove_nocase()`].
    #[inline]
    pub fn remove_nocase(&mut self, key: &str) -> Result<bool, Error> {
        let mut contains = self.transaction.prepare_cached(SQL_DELETE_NOC)?;
        Ok(contains.execute([key])? != 0)
    }

    /// This function is equivalent to [`SQLiteSet::for_each()`].
    #[inline]
    pub fn for_each<F>(&self, mut callback: F) -> Result<(), Error>
    where
        F: FnMut(&str),
    {
        let mut iter = self.transaction.prepare_cached(SQL_QUERY_KEYS)?;
        let mut result = iter.query([])?;
        while let Some(current_item) = result.next()? {
            let key: String = current_item.get(0)?;
            callback(&key);
        }
        Ok(())
    }

    /// This function is equivalent to [`SQLiteSet::find()`].
    #[inline]
    pub fn find<P>(&self, predicate: P) -> Result<Option<String>, Error>
    where
        P: Fn(&str) -> bool,
    {
        let mut iter = self.transaction.prepare_cached(SQL_QUERY_KEYS)?;
        let mut result = iter.query([])?;
        while let Some(current_item) = result.next()? {
            let key: String = current_item.get(0)?;
            if predicate(&key) {
                return Ok(Some(key));
            }
        }
        Ok(None)
    }

    /// This function is equivalent to [`SQLiteSet::len()`].
    #[inline]
    pub fn len(&self) -> Result<SizeT, Error> {
        let mut query_count = self.transaction.prepare_cached(SQL_COUNT_KEYS)?;
        let count: i64 = query_count.query_one([], |row| row.get(0))?;
        Ok(count.try_into().unwrap_or_default())
    }

    /// This function is equivalent to [`SQLiteSet::is_empty()`].
    #[inline]
    pub fn is_empty(&self) -> Result<bool, Error> {
        Ok(self.len()? == 0)
    }

    /// This function is equivalent to [`SQLiteSet::clear()`].
    #[inline]
    pub fn clear(&mut self) -> Result<(), Error> {
        let mut clear = self.transaction.prepare_cached(SQL_DELETE_ALL)?;
        clear.execute([])?;
        Ok(())
    }
}
