// SQLiteSet
// This file is part of the 'SQLite-based containers for Rust' project (sqlite-containers)
// SPDX-License-Identifier: Unlicense

use crate::{
    CharType, Error, Size, SizeExt, Type,
    common::{SQL_TYPE_TEXT, SQL_TYPES, SqlType, check_constraint_violation},
    rusqlite::{Connection, ToSql, Transaction},
};
use cfg_if::cfg_if;
use std::{borrow::Borrow, collections::BTreeMap, marker::PhantomData, sync::LazyLock};

// ---------------------------------------------------------------------------
// Statements
// ---------------------------------------------------------------------------

static SQL_CREATE_TBL: LazyLock<BTreeMap<SqlType, String>> = LazyLock::new(|| {
    let mut map = BTreeMap::<SqlType, String>::new();
    for sql_type in SQL_TYPES {
        cfg_if! {
            if #[cfg(feature = "with_rowid")] {
                map.insert(sql_type, format!("CREATE TABLE data (key {} PRIMARY KEY NOT NULL);", sql_type));
            } else {
                map.insert(sql_type, format!("CREATE TABLE data (key {} PRIMARY KEY NOT NULL) WITHOUT ROWID;", sql_type));
            }
        }
    }
    map
});

cfg_if! {
    if #[cfg(feature = "with_rowid")] {
        const SQL_CREATE_NOC: &str = "CREATE TABLE data (key TEXT PRIMARY KEY NOT NULL COLLATE NOCASE);";
    } else {
        const SQL_CREATE_NOC: &str = "CREATE TABLE data (key TEXT PRIMARY KEY NOT NULL COLLATE NOCASE) WITHOUT ROWID;";
    }
}

const SQL_COUNT_KEYS: &str = "SELECT COUNT(*) FROM data;";
const SQL_INSERT_KEY: &str = "INSERT INTO data (key) VALUES (?1);";
const SQL_EXISTS_KEY: &str = "SELECT 1 FROM data WHERE key = ? LIMIT 1;";
const SQL_EXISTS_NOC: &str = "SELECT 1 FROM data WHERE key COLLATE NOCASE = ? LIMIT 1;";
const SQL_QUERY_KEYS: &str = "SELECT key FROM data;";
const SQL_DELETE_KEY: &str = "DELETE FROM data WHERE key = ?;";
const SQL_DELETE_NOC: &str = "DELETE FROM data WHERE key COLLATE NOCASE = ?;";
const SQL_DELETE_ALL: &str = "DELETE FROM data;";

// ---------------------------------------------------------------------------
// SQLiteSet
// ---------------------------------------------------------------------------

/// A [set](https://en.wikipedia.org/wiki/Set_(abstract_data_type)) that stores keys of type [`T`](crate::Type), backed by an SQLite in-memory database.
///
/// It serves as a drop-in replacement for [`HashSet`](std::collections::HashSet).
///
/// By default, `SQLiteSet<String>` treats its keys as case-sensitive, but a case-insensitive variant is available. Even when using the case-sensitive set variant, for some operations a dedicated "case-insensitive" version is provided.
///
/// &nbsp;
///
/// <div class="warning">
///
/// For bulk inserts or updates, it is **strongly recommended** to use an explicit [transaction](Self::transaction). Without one, SQLite executes each insert or update in its own transaction, which can significantly degrade performance.
///
/// </div>
pub struct SQLiteSet<T>
where
    T: Type,
{
    _type: PhantomData<T>,
    connection: Connection,
}

impl<T> SQLiteSet<T>
where
    T: Type,
{
    /// Creates a new, empty SQLite-backed hash set with case-sensitive keys.
    #[inline]
    pub fn new() -> Result<Self, Error> {
        Ok(Self { _type: PhantomData, connection: Self::initialize_connection::<false>()? })
    }

    /// Creates a new, empty SQLite-backed hash set with case-insensitive keys.
    #[inline]
    pub fn with_nocase() -> Result<Self, Error>
    where
        T: CharType,
    {
        Ok(Self { _type: PhantomData, connection: Self::initialize_connection::<true>()? })
    }

    #[inline]
    fn initialize_connection<const NO_CASE: bool>() -> Result<Connection, Error> {
        let connection = Connection::open_in_memory()?;
        connection.pragma_update(None, "journal_mode", "OFF")?;
        connection.pragma_update(None, "synchronous", "OFF")?;
        connection.pragma_update(None, "temp_store", "MEMORY")?;
        if !NO_CASE {
            connection.execute(SQL_CREATE_TBL.get(&T::SQL_TYPE).unwrap(), [])?;
        } else {
            assert_eq!(T::SQL_TYPE, SQL_TYPE_TEXT);
            connection.execute(SQL_CREATE_NOC, [])?;
        }
        Ok(connection)
    }

    /// Starts a new SQLite transaction for this set.
    ///
    /// Please note that using an explicit SQLite transaction allows for much more efficient bulk inserts &#x1F680;
    ///
    /// Returns the new [`SQLiteSetTransaction`] instance.
    pub fn transaction(&mut self) -> Result<SQLiteSetTransaction<'_, T>, Error> {
        SQLiteSetTransaction::from(&mut self.connection)
    }

    /// Inserts the given key into the set.
    ///
    /// Returns `true`, if the set did not already contain the key; otherwise returns `false`.
    #[inline]
    pub fn insert<Q>(&mut self, key: &Q) -> Result<bool, Error>
    where
        T: Borrow<Q>,
        Q: ?Sized + ToSql,
    {
        let mut statement = self.connection.prepare_cached(SQL_INSERT_KEY)?;
        match statement.execute([key]) {
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
    pub fn contains<Q>(&self, key: &Q) -> Result<bool, Error>
    where
        T: Borrow<Q>,
        Q: ?Sized + ToSql,
    {
        let mut statement = self.connection.prepare_cached(SQL_EXISTS_KEY)?;
        Ok(statement.exists([key])?)
    }

    /// This is the "case-insensitive" version of the [`contains()`](Self::contains) function.
    ///
    /// The check is *always* performed case-insensitive.
    ///
    /// Returns `true`, if the set contains the key; otherwise returns `false`.
    #[inline]
    pub fn contains_nocase<Q>(&self, key: &Q) -> Result<bool, Error>
    where
        T: Borrow<Q> + CharType,
        Q: ?Sized + ToSql,
    {
        let mut statement = self.connection.prepare_cached(SQL_EXISTS_NOC)?;
        Ok(statement.exists([key])?)
    }

    /// Removes the specified key from the set, if present.
    ///
    /// For case-sensitive sets, the key is treated as case-sensitive; for case-insensitive sets, it is treated as case-insensitive.
    ///
    /// Returns `true`, if the set contained the key; otherwise returns `false`.
    #[inline]
    pub fn remove<Q>(&mut self, key: &Q) -> Result<bool, Error>
    where
        T: Borrow<Q>,
        Q: ?Sized + ToSql,
    {
        let mut statement = self.connection.prepare_cached(SQL_DELETE_KEY)?;
        Ok(statement.execute([key])? != 0)
    }

    /// This is the "case-insensitive" version of the [`remove()`](Self::remove) function.
    ///
    /// The key is *always* treated as case-insensitive.
    ///
    /// Returns `true`, if the set contained the key; otherwise returns `false`.
    #[inline]
    pub fn remove_nocase<Q>(&mut self, key: &Q) -> Result<bool, Error>
    where
        T: Borrow<Q> + CharType,
        Q: ?Sized + ToSql,
    {
        let mut statement = self.connection.prepare_cached(SQL_DELETE_NOC)?;
        Ok(statement.execute([key])? != 0)
    }

    /// Invokes the given `callback` function for each key that is currently contained in the set.
    ///
    /// This function does **not** guarantee a specific iteration order.
    #[inline]
    pub fn for_each<F>(&self, mut callback: F) -> Result<(), Error>
    where
        F: FnMut(T),
    {
        let mut statement = self.connection.prepare_cached(SQL_QUERY_KEYS)?;
        for key in statement.query_map([], |row| row.get(0))? {
            callback(key?);
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
    pub fn find<P, Q>(&self, predicate: P) -> Result<Option<T>, Error>
    where
        P: Fn(&Q) -> bool,
        T: Borrow<Q>,
        Q: ?Sized,
    {
        let mut statement = self.connection.prepare_cached(SQL_QUERY_KEYS)?;
        for key in statement.query_map([], |row| row.get(0))? {
            let key: T = key?;
            if predicate(key.borrow()) {
                return Ok(Some(key));
            }
        }
        Ok(None)
    }

    /// Returns the number of unique keys in the set.
    #[inline]
    pub fn len(&self) -> Result<Size, Error> {
        let mut statement = self.connection.prepare_cached(SQL_COUNT_KEYS)?;
        let count: i64 = statement.query_one([], |row| row.get(0))?;
        Ok(Size::from_i64(count))
    }

    /// Returns `true` if the set contains **no** keys; otherwise returns `false`.
    #[inline]
    pub fn is_empty(&self) -> Result<bool, Error> {
        Ok(self.len()? == 0)
    }

    /// Removes *all* keys from the set.
    #[inline]
    pub fn clear(&mut self) -> Result<(), Error> {
        let mut statement = self.connection.prepare_cached(SQL_DELETE_ALL)?;
        statement.execute([])?;
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// SQLiteSet Transaction
// ---------------------------------------------------------------------------

/// Represents an active SQLite transaction for a [`SQLiteSet`].
///
/// Most functions provided by this struct mirror the corresponding functions of the `SQLiteSet` struct.
///
/// The transaction is committed when the `SQLiteSetTransaction` is dropped.
pub struct SQLiteSetTransaction<'a, T>
where
    T: Type,
{
    _type: PhantomData<T>,
    transaction: Transaction<'a>,
}

impl<'a, T> SQLiteSetTransaction<'a, T>
where
    T: Type,
{
    #[inline]
    fn from(connection: &'a mut Connection) -> Result<Self, Error> {
        let mut transaction = connection.transaction()?;
        transaction.set_drop_behavior(rusqlite::DropBehavior::Commit);
        Ok(Self { _type: PhantomData, transaction })
    }

    /// Drops the `SQLiteSetTransaction`, thereby committing the SQLite transaction.
    #[inline]
    pub fn commit(self) {}

    /// This function is equivalent to [`SQLiteSet::insert()`].
    #[inline]
    pub fn insert<Q>(&mut self, key: &Q) -> Result<bool, Error>
    where
        T: Borrow<Q>,
        Q: ?Sized + ToSql,
    {
        let mut statement = self.transaction.prepare_cached(SQL_INSERT_KEY)?;
        match statement.execute([key]) {
            Ok(_) => Ok(true),
            Err(error) => check_constraint_violation(error),
        }
    }

    /// This function is equivalent to [`SQLiteSet::contains()`].
    #[inline]
    pub fn contains<Q>(&self, key: &Q) -> Result<bool, Error>
    where
        T: Borrow<Q>,
        Q: ?Sized + ToSql,
    {
        let mut statement = self.transaction.prepare_cached(SQL_EXISTS_KEY)?;
        Ok(statement.exists([key])?)
    }

    /// This function is equivalent to [`SQLiteSet::contains_nocase()`].
    #[inline]
    pub fn contains_nocase<Q>(&self, key: &Q) -> Result<bool, Error>
    where
        T: Borrow<Q> + CharType,
        Q: ?Sized + ToSql,
    {
        let mut statement = self.transaction.prepare_cached(SQL_EXISTS_NOC)?;
        Ok(statement.exists([key])?)
    }

    /// This function is equivalent to [`SQLiteSet::remove()`].
    #[inline]
    pub fn remove<Q>(&mut self, key: &Q) -> Result<bool, Error>
    where
        T: Borrow<Q>,
        Q: ?Sized + ToSql,
    {
        let mut statement = self.transaction.prepare_cached(SQL_DELETE_KEY)?;
        Ok(statement.execute([key])? != 0)
    }

    /// This function is equivalent to [`SQLiteSet::remove_nocase()`].
    #[inline]
    pub fn remove_nocase<Q>(&mut self, key: &Q) -> Result<bool, Error>
    where
        T: Borrow<Q> + CharType,
        Q: ?Sized + ToSql,
    {
        let mut statement = self.transaction.prepare_cached(SQL_DELETE_NOC)?;
        Ok(statement.execute([key])? != 0)
    }

    /// This function is equivalent to [`SQLiteSet::for_each()`].
    #[inline]
    pub fn for_each<F>(&self, mut callback: F) -> Result<(), Error>
    where
        F: FnMut(T),
    {
        let mut statement = self.transaction.prepare_cached(SQL_QUERY_KEYS)?;
        for key in statement.query_map([], |row| row.get(0))? {
            callback(key?);
        }
        Ok(())
    }

    /// This function is equivalent to [`SQLiteSet::find()`].
    #[inline]
    pub fn find<P, Q>(&self, predicate: P) -> Result<Option<T>, Error>
    where
        P: Fn(&Q) -> bool,
        T: Borrow<Q>,
        Q: ?Sized,
    {
        let mut statement = self.transaction.prepare_cached(SQL_QUERY_KEYS)?;
        for key in statement.query_map([], |row| row.get(0))? {
            let key: T = key?;
            if predicate(key.borrow()) {
                return Ok(Some(key));
            }
        }
        Ok(None)
    }

    /// This function is equivalent to [`SQLiteSet::len()`].
    #[inline]
    pub fn len(&self) -> Result<Size, Error> {
        let mut statement = self.transaction.prepare_cached(SQL_COUNT_KEYS)?;
        let count: i64 = statement.query_one([], |row| row.get(0))?;
        Ok(Size::from_i64(count))
    }

    /// This function is equivalent to [`SQLiteSet::is_empty()`].
    #[inline]
    pub fn is_empty(&self) -> Result<bool, Error> {
        Ok(self.len()? == 0)
    }

    /// This function is equivalent to [`SQLiteSet::clear()`].
    #[inline]
    pub fn clear(&mut self) -> Result<(), Error> {
        let mut statement = self.transaction.prepare_cached(SQL_DELETE_ALL)?;
        statement.execute([])?;
        Ok(())
    }
}
