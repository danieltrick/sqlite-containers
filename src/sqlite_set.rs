// SQLiteSet
// This file is part of the 'SQLite-based containers for Rust' project (sqlite-containers)
// SPDX-License-Identifier: Unlicense

use crate::{
    CharType, Error, Size, SizeExt, TryClone, Type,
    common::{OPEN_FLAGS, SQL_TYPE_TEXT, SQL_TYPES, SqlType, check_constraint_violation, verify_schema},
    rusqlite::{Connection, ToSql, Transaction},
};
use cfg_if::cfg_if;
use const_format::concatcp;
use std::{borrow::Borrow, collections::BTreeMap, marker::PhantomData, path::Path, sync::LazyLock};

// ---------------------------------------------------------------------------
// Statements
// ---------------------------------------------------------------------------

const TABLE_NAME: &str = "rusqlite_set_v2";

static SQL_CREATE_TBL: LazyLock<BTreeMap<SqlType, String>> = LazyLock::new(|| {
    let mut map = BTreeMap::<SqlType, String>::new();
    for sql_type in SQL_TYPES {
        cfg_if! {
            if #[cfg(feature = "with_rowid")] {
                map.insert(sql_type, format!("CREATE TABLE IF NOT EXISTS {TABLE_NAME} (key {sql_type} PRIMARY KEY NOT NULL);"));
            } else {
                map.insert(sql_type, format!("CREATE TABLE IF NOT EXISTS {TABLE_NAME} (key {sql_type} PRIMARY KEY NOT NULL) WITHOUT ROWID;"));
            }
        }
    }
    map
});

cfg_if! {
    if #[cfg(feature = "with_rowid")] {
        const SQL_CREATE_NOC: &str = concatcp!("CREATE TABLE IF NOT EXISTS ", TABLE_NAME, " (key TEXT PRIMARY KEY NOT NULL COLLATE NOCASE);");
    } else {
        const SQL_CREATE_NOC: &str = concatcp!("CREATE TABLE IF NOT EXISTS ", TABLE_NAME, " (key TEXT PRIMARY KEY NOT NULL COLLATE NOCASE) WITHOUT ROWID;");
    }
}

const SQL_GET_SCHEMA: &str = concatcp!("SELECT sql FROM sqlite_schema WHERE type = 'table' AND name = '", TABLE_NAME, "';");
const SQL_COUNT_KEYS: &str = concatcp!("SELECT COUNT(*) FROM ", TABLE_NAME, ";");
const SQL_EXIST_KEYS: &str = concatcp!("SELECT EXISTS (SELECT 1 FROM ", TABLE_NAME, ");");
const SQL_INSERT_KEY: &str = concatcp!("INSERT INTO ", TABLE_NAME, " (key) VALUES (?1);");
const SQL_EXISTS_KEY: &str = concatcp!("SELECT 1 FROM ", TABLE_NAME, " WHERE key = ?1 LIMIT 1;");
const SQL_EXISTS_NOC: &str = concatcp!("SELECT 1 FROM ", TABLE_NAME, " WHERE key COLLATE NOCASE = ? LIMIT 1;");
const SQL_QUERY_KEYS: &str = concatcp!("SELECT key FROM ", TABLE_NAME, ";");
const SQL_GLOBA_KEYS: &str = concatcp!("SELECT key FROM ", TABLE_NAME, " WHERE key GLOB ?1;");
const SQL_GLOBN_KEYS: &str = concatcp!("SELECT key FROM ", TABLE_NAME, " WHERE key NOT GLOB ?1;");
const SQL_DELETE_KEY: &str = concatcp!("DELETE FROM ", TABLE_NAME, " WHERE key = ?1;");
const SQL_DELETE_NOC: &str = concatcp!("DELETE FROM ", TABLE_NAME, " WHERE key COLLATE NOCASE = ?1;");
const SQL_DELETE_ALL: &str = concatcp!("DELETE FROM ", TABLE_NAME, ";");

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
    /// Creates a new, empty hash set with case-sensitive keys, backed by an *in-memory* SQLite database.
    #[inline]
    pub fn new() -> Result<Self, Error> {
        Self::from_connection(Self::create_connection::<false, &Path>(None)?)
    }

    /// Creates a new, empty hash set with case-insensitive keys, backed by an *in-memory* SQLite database.
    ///
    /// Only ASCII characters are case folded. SQLite does **not** attempt to do full UTF case folding!
    #[inline]
    pub fn with_nocase() -> Result<Self, Error>
    where
        T: CharType,
    {
        Self::from_connection(Self::create_connection::<true, &Path>(None)?)
    }

    /// Creates a new or re-opens an existing hash set with case-sensitive keys, backed by a *file-based* SQLite database.
    ///
    /// The database file is created, if it does *not* exist yet.
    ///
    /// Fails with [`Error::InvalidSchema`], if the schema of the existing database file does **not** match the expected schema.
    #[inline]
    pub fn with_file<P>(path: P) -> Result<Self, Error>
    where
        P: AsRef<Path>,
    {
        Self::from_connection(Self::create_connection::<false, _>(Some(path))?)
    }

    /// Creates a new or re-opens an existing hash set with case-insensitive keys, backed by a *file-based* SQLite database.
    ///
    /// The database file is created, if it does *not* exist yet.
    ///
    /// Fails with [`Error::InvalidSchema`], if the schema of the existing database file does **not** match the expected schema.
    ///
    /// Only ASCII characters are case folded. SQLite does **not** attempt to do full UTF case folding!
    #[inline]
    pub fn with_file_nocase<P>(path: P) -> Result<Self, Error>
    where
        T: CharType,
        P: AsRef<Path>,
    {
        Self::from_connection(Self::create_connection::<true, _>(Some(path))?)
    }

    #[inline(always)]
    fn create_connection<const NO_CASE: bool, P>(path_option: Option<P>) -> Result<Connection, Error>
    where
        P: AsRef<Path>,
    {
        let connection = Self::initialize_connection(if let Some(path) = path_option.as_ref() {
            Connection::open_with_flags(path, *OPEN_FLAGS)?
        } else {
            Connection::open_in_memory()?
        })?;

        #[cfg(not(feature = "no_check"))]
        if path_option.is_some() {
            crate::common::integrity_check(&connection)?;
        }

        let sql_create_table = if !NO_CASE {
            SQL_CREATE_TBL.get(&T::SQL_TYPE).unwrap()
        } else {
            debug_assert_eq!(T::SQL_TYPE, SQL_TYPE_TEXT);
            SQL_CREATE_NOC
        };

        connection.execute(sql_create_table, [])?;

        if path_option.is_some() {
            let sql_schema_table: String = connection.query_one(SQL_GET_SCHEMA, [], |row| row.get(0))?;
            if !verify_schema(sql_create_table, &sql_schema_table) {
                return Err(Error::InvalidSchema);
            }
        }

        Ok(connection)
    }

    #[inline(always)]
    fn initialize_connection(connection: Connection) -> Result<Connection, Error> {
        connection.pragma_update(None, "synchronous", "OFF")?;
        connection.pragma_update(None, "journal_mode", "OFF")?;
        Ok(connection)
    }

    #[inline(always)]
    fn from_connection(connection: Connection) -> Result<Self, Error> {
        Ok(Self { _type: PhantomData, connection })
    }

    /// Starts a new SQLite transaction for this set.
    ///
    /// Please note that using an explicit SQLite transaction allows for much more efficient bulk inserts &#x1F680;
    ///
    /// Returns the new [`SQLiteSetTransaction`] instance.
    #[inline]
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

    /// Invokes the given `callback` function for each key that matches the given pattern.
    ///
    /// The pattern string is interpreted according to the [UNIX *globbing*](https://man7.org/linux/man-pages/man7/glob.7.html) rules. Specifically, a `?` character (not between brackets) matches any single character, and a `*` character (not between brackets) matches any string, including the empty string. Additionally, an expression `[…]` where the first character after the leading `[` is *not* an `!` matches a single character that can be any of the characters enclosed by the brackets; and an expression `[!…]` matches a single character that is *not* matched by the expression.
    ///
    /// This function is case-sensitive and does **not** guarantee a specific iteration order.
    ///
    /// If `invert` is `true`, returns all keys that do **not** match the pattern; otherwise the matching keys are returned.
    #[inline]
    pub fn glob<F, Q>(&self, invert: bool, pattern: &Q, mut callback: F) -> Result<(), Error>
    where
        F: FnMut(T),
        T: Borrow<Q> + CharType,
        Q: ?Sized + ToSql,
    {
        let mut statement = self.connection.prepare_cached(if invert { SQL_GLOBN_KEYS } else { SQL_GLOBA_KEYS })?;
        for key in statement.query_map([pattern], |row| row.get(0))? {
            callback(key?);
        }
        Ok(())
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
        let mut statement = self.connection.prepare_cached(SQL_EXIST_KEYS)?;
        let any_elements: bool = statement.query_one([], |row| row.get(0))?;
        Ok(!any_elements)
    }

    /// Removes *all* keys from the set.
    #[inline]
    pub fn clear(&mut self) -> Result<(), Error> {
        let mut statement = self.connection.prepare_cached(SQL_DELETE_ALL)?;
        statement.execute([])?;
        Ok(())
    }
}

cfg_if! {
    if #[cfg(feature = "backup")] {
        impl<T> TryClone for SQLiteSet<T>
        where
            T: Type,
        {
            type Error = Error;

            /// Creates an exact, independent *copy* of the given set instance.
            ///
            /// This function fails with [`Error::Unsupported`](crate::Error::Unsupported), if the optional [`backup`](crate#optional-features) feature is **not** enabled &#128549;
            #[inline]
            fn try_clone(&self) -> Result<Self, Self::Error> {
                let mut connection_cloned = Self::initialize_connection(Connection::open_in_memory()?)?;
                crate::common::backup_db(&self.connection, &mut connection_cloned)?;
                Self::from_connection(connection_cloned)
            }
        }
    } else {
        impl<T> TryClone for SQLiteSet<T>
        where
            T: Type,
        {
            type Error = Error;

            /// Creates an exact, independent *copy* of the given set instance.
            ///
            /// This function fails with [`Error::Unsupported`](crate::Error::Unsupported), if the optional [`backup`](crate#optional-features) feature is **not** enabled &#128549;
            #[inline]
            fn try_clone(&self) -> Result<Self, Self::Error> {
                Err(Error::Unsupported)
            }
        }
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

    /// This function is equivalent to [`SQLiteSet::glob()`].
    #[inline]
    pub fn glob<F, Q>(&self, invert: bool, pattern: &Q, mut callback: F) -> Result<(), Error>
    where
        F: FnMut(T),
        T: Borrow<Q> + CharType,
        Q: ?Sized + ToSql,
    {
        let mut statement = self.transaction.prepare_cached(if invert { SQL_GLOBN_KEYS } else { SQL_GLOBA_KEYS })?;
        for key in statement.query_map([pattern], |row| row.get(0))? {
            callback(key?);
        }
        Ok(())
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
        let mut statement = self.transaction.prepare_cached(SQL_EXIST_KEYS)?;
        let any_elements: bool = statement.query_one([], |row| row.get(0))?;
        Ok(!any_elements)
    }

    /// This function is equivalent to [`SQLiteSet::clear()`].
    #[inline]
    pub fn clear(&mut self) -> Result<(), Error> {
        let mut statement = self.transaction.prepare_cached(SQL_DELETE_ALL)?;
        statement.execute([])?;
        Ok(())
    }
}
