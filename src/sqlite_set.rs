// SQLiteSet
// This file is part of the 'SQLite-based containers for Rust' project (sqlite-containers)
// SPDX-License-Identifier: Unlicense

use crate::{
    CharType, Error, IterationError, SQLiteMap, Size, SizeExt, TryClone, TryEq, Type,
    common::{SQL_TYPE_TEXT, SQL_TYPES, SqlType, check_constraint, create_new_connection, increment},
    rusqlite::{Connection, DropBehavior, ToSql, Transaction},
};
use cfg_if::cfg_if;
use const_format::concatcp;
use std::{borrow::Borrow, collections::BTreeMap, marker::PhantomData, path::Path, ptr::addr_eq, sync::LazyLock};

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
const SQL_TEST_EMPTY: &str = concatcp!("SELECT EXISTS (SELECT 1 FROM ", TABLE_NAME, ");");
const SQL_INSERT_KEY: &str = concatcp!("INSERT INTO ", TABLE_NAME, " (key) VALUES (?1);");
const SQL_EXISTS_KEY: &str = concatcp!("SELECT 1 FROM ", TABLE_NAME, " WHERE key = ?1 LIMIT 1;");
const SQL_EXISTS_NOC: &str = concatcp!("SELECT 1 FROM ", TABLE_NAME, " WHERE key COLLATE NOCASE = ? LIMIT 1;");
const SQL_QUERY_KEYS: &str = concatcp!("SELECT key FROM ", TABLE_NAME, ";");
const SQL_GLOBA_KEYS: &str = concatcp!("SELECT key FROM ", TABLE_NAME, " WHERE key GLOB ?1;");
const SQL_GLOBN_KEYS: &str = concatcp!("SELECT key FROM ", TABLE_NAME, " WHERE key NOT GLOB ?1;");
const SQL_DELETE_KEY: &str = concatcp!("DELETE FROM ", TABLE_NAME, " WHERE key = ?1;");
const SQL_DELETE_NOC: &str = concatcp!("DELETE FROM ", TABLE_NAME, " WHERE key COLLATE NOCASE = ?1;");
const SQL_DELETE_ALL: &str = concatcp!("DELETE FROM ", TABLE_NAME, ";");
const SQL_STORE_INTO: &str = concatcp!("VACUUM INTO ?1;");

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
        Ok(Self::from_connection(Self::create_connection::<false, &Path>(None, false, false)?))
    }

    /// Creates a new, empty hash set with case-insensitive keys, backed by an *in-memory* SQLite database.
    ///
    /// Only ASCII characters are case folded. SQLite does **not** attempt to do full UTF case folding!
    #[inline]
    pub fn with_nocase() -> Result<Self, Error>
    where
        T: CharType,
    {
        Ok(Self::from_connection(Self::create_connection::<true, &Path>(None, false, false)?))
    }

    #[inline(always)]
    fn from_connection(connection: Connection) -> Self {
        Self { _type: PhantomData, connection }
    }

    /// Creates a new hash set with case-sensitive keys containing all keys yielded by the given [iterable](std::iter::IntoIterator).
    ///
    /// The new hash set is backed by an *in-memory* SQLite database.
    #[inline]
    pub fn try_from<I>(keys: I) -> Result<Self, Error>
    where
        I: IntoIterator<Item = T>,
    {
        let mut new_set = Self::new()?;
        new_set.extend(keys)?;
        Ok(new_set)
    }

    /// Creates a new hash set with case-insensitive keys containing all keys yielded by the given [iterable](std::iter::IntoIterator).
    ///
    /// The new hash set is backed by an *in-memory* SQLite database.
    ///
    /// Only ASCII characters are case folded. SQLite does **not** attempt to do full UTF case folding!
    #[inline]
    pub fn try_from_nocase<I>(keys: I) -> Result<Self, Error>
    where
        I: IntoIterator<Item = T>,
        T: CharType,
    {
        let mut new_set = Self::with_nocase()?;
        new_set.extend(keys)?;
        Ok(new_set)
    }

    /// Creates a new or re-opens an existing hash set with case-sensitive keys, backed by a *file-based* SQLite database.
    ///
    /// If `readonly` is `true`, the database file is opened in “read-only” mode.
    ///
    /// If `can_create` is `true` (and if `readonly` is `false`), a new empty database file is created, if the file does *not* exist yet.
    ///
    /// Returns [`Error::InvalidSchema`], if the database file already exists but its schema does **not** match the schema expected by this type of set. This means that you **must** re-open a database file with the *same* type of set that was used to create it.
    #[inline]
    pub fn try_from_file<P>(path: P, readonly: bool, can_create: bool) -> Result<Self, Error>
    where
        P: AsRef<Path>,
    {
        Ok(Self::from_connection(Self::create_connection::<false, _>(Some(path), readonly, can_create)?))
    }

    /// Creates a new or re-opens an existing hash set with case-insensitive keys, backed by a *file-based* SQLite database.
    ///
    /// If `readonly` is `true`, the database file is opened in “read-only” mode.
    ///
    /// If `can_create` is `true` (and if `readonly` is `false`), a new empty database file is created, if the file does *not* exist yet.
    ///
    /// Returns [`Error::InvalidSchema`], if the database file already exists but its schema does **not** match the schema expected by this type of set. This means that you **must** re-open a database file with the *same* type of set that was used to create it.
    ///
    /// Only ASCII characters are case folded. SQLite does **not** attempt to do full UTF case folding!
    #[inline]
    pub fn try_from_file_nocase<P>(path: P, readonly: bool, can_create: bool) -> Result<Self, Error>
    where
        T: CharType,
        P: AsRef<Path>,
    {
        Ok(Self::from_connection(Self::create_connection::<true, _>(Some(path), readonly, can_create)?))
    }

    #[inline(always)]
    fn create_connection<const NO_CASE: bool, P>(path_option: Option<P>, readonly: bool, can_create: bool) -> Result<Connection, Error>
    where
        P: AsRef<Path>,
    {
        let sql_create_table = if !NO_CASE {
            SQL_CREATE_TBL.get(&T::SQL_TYPE).unwrap()
        } else {
            debug_assert_eq!(T::SQL_TYPE, SQL_TYPE_TEXT);
            SQL_CREATE_NOC
        };

        create_new_connection(sql_create_table, SQL_GET_SCHEMA, path_option, readonly, can_create)
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

    /// This is the “read-only” version of the [`transaction()`](Self::transaction) function.
    #[inline]
    pub(crate) fn transaction_readonly(&self) -> Result<SQLiteSetTransaction<'_, T>, Error> {
        SQLiteSetTransaction::from_immutable(&self.connection)
    }

    /// Inserts the given key into the set.
    ///
    /// Returns `true`, if the set did not already contain the key; otherwise returns `false`.
    #[inline]
    pub fn insert(&mut self, key: T) -> Result<bool, Error> {
        let mut statement = self.connection.prepare_cached(SQL_INSERT_KEY)?;
        Ok(check_constraint(statement.execute([key]))?)
    }

    /// Inserts the given key reference into the set.
    ///
    /// Returns `true`, if the set did not already contain the key; otherwise returns `false`.
    #[inline]
    pub fn insert_ref<Q>(&mut self, key: &Q) -> Result<bool, Error>
    where
        T: Borrow<Q>,
        Q: ?Sized + ToSql,
    {
        let mut statement = self.connection.prepare_cached(SQL_INSERT_KEY)?;
        Ok(check_constraint(statement.execute([key]))?)
    }

    /// Inserts all keys yielded by the given [iterable](std::iter::IntoIterator) into the set, skipping duplicate keys.
    ///
    /// Returns the number of new keys that have been inserted.
    #[inline]
    pub fn extend<I>(&mut self, keys: I) -> Result<Size, Error>
    where
        I: IntoIterator<Item = T>,
    {
        self.transaction()?.extend(keys)
    }

    /// Inserts all keys yielded by the given [iterable](std::iter::IntoIterator) into the set, skipping duplicate keys.
    ///
    /// Returns the number of new keys that have been inserted.
    #[inline]
    pub fn extend_ref<'b, I, Q>(&mut self, keys: I) -> Result<Size, Error>
    where
        I: IntoIterator<Item = &'b Q>,
        T: Borrow<Q>,
        Q: ?Sized + ToSql + 'b,
    {
        self.transaction()?.extend_ref(keys)
    }

    /// Inserts all keys from another set into this set, ignoring existing keys.
    ///
    /// Returns the number of new keys that have been inserted.
    #[inline]
    pub fn extend_set(&mut self, other: &Self) -> Result<Size, Error> {
        self.transaction()?.extend_set(other)
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

    /// Removes *all* keys that are yielded by the given [iterable](std::iter::IntoIterator), ignoring non-existing keys.
    ///
    /// For case-sensitive sets, the keys are treated as case-sensitive; for case-insensitive sets, they are treated as case-insensitive.
    ///
    /// Returns the number of keys that have been removed.
    #[inline]
    pub fn remove_all<I>(&mut self, keys: I) -> Result<Size, Error>
    where
        I: IntoIterator<Item = T>,
    {
        self.transaction()?.remove_all(keys)
    }

    /// Removes *all* keys that are yielded by the given [iterable](std::iter::IntoIterator), ignoring non-existing keys.
    ///
    /// The keys are treated as case-insensitive.
    ///
    /// Returns the number of keys that have been removed.
    #[inline]
    pub fn remove_all_nocase<I>(&mut self, keys: I) -> Result<Size, Error>
    where
        I: IntoIterator<Item = T>,
        T: CharType,
    {
        self.transaction()?.remove_all_nocase(keys)
    }

    /// Removes *all* keys that exist in another set from this sets, ignoring non-existing keys.
    ///
    /// For case-sensitive sets, the keys are treated as case-sensitive; for case-insensitive sets, they are treated as case-insensitive.
    ///
    /// Returns the number of keys that have been removed.
    #[inline]
    pub fn remove_set(&mut self, other: &Self) -> Result<Size, Error> {
        self.transaction()?.remove_set(other)
    }

    /// Removes *all* keys that exist in another set from this sets, ignoring non-existing keys.
    ///
    /// The keys are treated as case-insensitive.
    ///
    /// Returns the number of keys that have been removed.
    #[inline]
    pub fn remove_set_nocase(&mut self, other: &Self) -> Result<Size, Error>
    where
        T: CharType,
    {
        self.transaction()?.remove_set_nocase(other)
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

    /// This is the *failable* version of the [`for_each()`](Self::for_each) function.
    #[inline]
    pub fn try_for_each<F, E>(&self, mut callback: F) -> Result<(), IterationError<E>>
    where
        F: FnMut(T) -> Result<(), E>,
    {
        let mut statement = self.connection.prepare_cached(SQL_QUERY_KEYS)?;
        for key in statement.query_map([], |row| row.get(0))? {
            if let Err(error) = callback(key?) {
                return Err(IterationError::CallbackError(error));
            }
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
        let mut statement = self.connection.prepare_cached(SQL_TEST_EMPTY)?;
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

    /// Creates a new set containing all keys that are contained in both, this set *and* the other set.
    ///
    /// The created set is case-sensitive.
    #[inline]
    pub fn and(&mut self, other: &Self) -> Result<SQLiteSet<T>, Error> {
        let tx_1 = other.transaction_readonly()?;
        let mut statement_query = self.connection.prepare_cached(SQL_QUERY_KEYS)?;
        let mut statement_find = tx_1.transaction.prepare_cached(SQL_EXISTS_KEY)?;
        let mut new_map = SQLiteSet::new()?;
        {
            let tx_2 = new_map.transaction()?;
            let mut statement_insert = tx_2.transaction.prepare_cached(SQL_INSERT_KEY)?;
            for key in statement_query.query_map([], |row| row.get::<_, T>(0))? {
                let key = key?;
                if statement_find.exists([&key])? {
                    check_constraint(statement_insert.execute([key]))?;
                }
            }
        }
        Ok(new_map)
    }

    /// Creates a new set containing all keys that are contained in both, this set *and* the other set.
    ///
    /// The created set is case-insensitive.
    ///
    /// Only ASCII characters are case folded. SQLite does **not** attempt to do full UTF case folding!
    #[inline]
    pub fn and_nocase(&mut self, other: &Self) -> Result<SQLiteSet<T>, Error>
    where
        T: CharType,
    {
        let tx_1 = other.transaction_readonly()?;
        let mut statement_query = self.connection.prepare_cached(SQL_QUERY_KEYS)?;
        let mut statement_find = tx_1.transaction.prepare_cached(SQL_EXISTS_NOC)?;
        let mut new_map = SQLiteSet::with_nocase()?;
        {
            let tx_2 = new_map.transaction()?;
            let mut statement_insert = tx_2.transaction.prepare_cached(SQL_INSERT_KEY)?;
            for key in statement_query.query_map([], |row| row.get::<_, T>(0))? {
                let key = key?;
                if statement_find.exists([&key])? {
                    check_constraint(statement_insert.execute([key]))?;
                }
            }
        }
        Ok(new_map)
    }

    /// Creates a new set containing all keys that are contained in this set *or* in the other set.
    ///
    /// The created set is case-sensitive.
    #[inline]
    pub fn or(&mut self, other: &Self) -> Result<SQLiteSet<T>, Error> {
        let mut statement_query_1 = self.connection.prepare_cached(SQL_QUERY_KEYS)?;
        let mut statement_query_2 = other.connection.prepare_cached(SQL_QUERY_KEYS)?;
        let mut new_map = SQLiteSet::new()?;
        let iter_1 = statement_query_1.query_map([], |row| row.get::<_, T>(0))?;
        let iter_2 = statement_query_2.query_map([], |row| row.get::<_, T>(0))?;
        {
            let tx = new_map.transaction()?;
            let mut statement_insert = tx.transaction.prepare_cached(SQL_INSERT_KEY)?;
            for key in iter_1.chain(iter_2) {
                check_constraint(statement_insert.execute([key?]))?;
            }
        }
        Ok(new_map)
    }

    /// Creates a new set containing all keys that are contained in this set *or* the other set.
    ///
    /// The created set is case-insensitive.
    ///
    /// Only ASCII characters are case folded. SQLite does **not** attempt to do full UTF case folding!
    #[inline]
    pub fn or_nocase(&mut self, other: &Self) -> Result<SQLiteSet<T>, Error>
    where
        T: CharType,
    {
        let mut statement_query_1 = self.connection.prepare_cached(SQL_QUERY_KEYS)?;
        let mut statement_query_2 = other.connection.prepare_cached(SQL_QUERY_KEYS)?;
        let mut new_map = SQLiteSet::with_nocase()?;
        let iter_1 = statement_query_1.query_map([], |row| row.get::<_, T>(0))?;
        let iter_2 = statement_query_2.query_map([], |row| row.get::<_, T>(0))?;
        {
            let tx = new_map.transaction()?;
            let mut statement_insert = tx.transaction.prepare_cached(SQL_INSERT_KEY)?;
            for key in iter_1.chain(iter_2) {
                check_constraint(statement_insert.execute([key?]))?;
            }
        }
        Ok(new_map)
    }

    /// Creates a new set containing all keys that are contained in this set but **not** in the other set.
    ///
    /// The created set is case-sensitive.
    #[inline]
    pub fn sub(&mut self, other: &Self) -> Result<SQLiteSet<T>, Error> {
        let tx_1 = other.transaction_readonly()?;
        let mut statement_query = self.connection.prepare_cached(SQL_QUERY_KEYS)?;
        let mut statement_find = tx_1.transaction.prepare_cached(SQL_EXISTS_KEY)?;
        let mut new_map = SQLiteSet::new()?;
        {
            let tx_2 = new_map.transaction()?;
            let mut statement_insert = tx_2.transaction.prepare_cached(SQL_INSERT_KEY)?;
            for key in statement_query.query_map([], |row| row.get::<_, T>(0))? {
                let key = key?;
                if !statement_find.exists([&key])? {
                    check_constraint(statement_insert.execute([key]))?;
                }
            }
        }
        Ok(new_map)
    }

    /// Creates a new set containing all keys that are contained in this set but **not** in the other set.
    ///
    /// The created set is case-insensitive.
    ///
    /// Only ASCII characters are case folded. SQLite does **not** attempt to do full UTF case folding!
    #[inline]
    pub fn sub_nocase(&mut self, other: &Self) -> Result<SQLiteSet<T>, Error>
    where
        T: CharType,
    {
        let tx_1 = other.transaction_readonly()?;
        let mut statement_query = self.connection.prepare_cached(SQL_QUERY_KEYS)?;
        let mut statement_find = tx_1.transaction.prepare_cached(SQL_EXISTS_NOC)?;
        let mut new_map = SQLiteSet::with_nocase()?;
        {
            let tx_2 = new_map.transaction()?;
            let mut statement_insert = tx_2.transaction.prepare_cached(SQL_INSERT_KEY)?;
            for key in statement_query.query_map([], |row| row.get::<_, T>(0))? {
                let key = key?;
                if !statement_find.exists([&key])? {
                    check_constraint(statement_insert.execute([key]))?;
                }
            }
        }
        Ok(new_map)
    }

    /// Creates a new set containing all keys that are contained in *one* of the sets but **not** in both.
    ///
    /// The created set is case-sensitive.
    #[inline]
    pub fn xor(&mut self, other: &Self) -> Result<SQLiteSet<T>, Error> {
        let tx_1 = self.transaction()?;
        let tx_2 = other.transaction_readonly()?;
        let mut statement_query_1 = tx_1.transaction.prepare_cached(SQL_QUERY_KEYS)?;
        let mut statement_find_1 = tx_1.transaction.prepare_cached(SQL_EXISTS_KEY)?;
        let mut statement_query_2 = tx_2.transaction.prepare_cached(SQL_QUERY_KEYS)?;
        let mut statement_find_2 = tx_2.transaction.prepare_cached(SQL_EXISTS_KEY)?;
        let mut new_map = SQLiteSet::new()?;
        {
            let tx_3 = new_map.transaction()?;
            let mut statement_insert = tx_3.transaction.prepare_cached(SQL_INSERT_KEY)?;
            for key in statement_query_1.query_map([], |row| row.get::<_, T>(0))? {
                let key = key?;
                if !statement_find_2.exists([&key])? {
                    check_constraint(statement_insert.execute([key]))?;
                }
            }
            for key in statement_query_2.query_map([], |row| row.get::<_, T>(0))? {
                let key = key?;
                if !statement_find_1.exists([&key])? {
                    check_constraint(statement_insert.execute([key]))?;
                }
            }
        }
        Ok(new_map)
    }

    /// Creates a new set containing all keys that are contained in *one* of the sets but **not** in both.
    ///
    /// The created set is case-insensitive.
    ///
    /// Only ASCII characters are case folded. SQLite does **not** attempt to do full UTF case folding!
    #[inline]
    pub fn xor_nocase(&mut self, other: &Self) -> Result<SQLiteSet<T>, Error>
    where
        T: CharType,
    {
        let tx_1 = self.transaction()?;
        let tx_2 = other.transaction_readonly()?;
        let mut statement_query_1 = tx_1.transaction.prepare_cached(SQL_QUERY_KEYS)?;
        let mut statement_find_1 = tx_1.transaction.prepare_cached(SQL_EXISTS_NOC)?;
        let mut statement_query_2 = tx_2.transaction.prepare_cached(SQL_QUERY_KEYS)?;
        let mut statement_find_2 = tx_2.transaction.prepare_cached(SQL_EXISTS_NOC)?;
        let mut new_map = SQLiteSet::with_nocase()?;
        {
            let tx_3 = new_map.transaction()?;
            let mut statement_insert = tx_3.transaction.prepare_cached(SQL_INSERT_KEY)?;
            for key in statement_query_1.query_map([], |row| row.get::<_, T>(0))? {
                let key = key?;
                if !statement_find_2.exists([&key])? {
                    check_constraint(statement_insert.execute([key]))?;
                }
            }
            for key in statement_query_2.query_map([], |row| row.get::<_, T>(0))? {
                let key = key?;
                if !statement_find_1.exists([&key])? {
                    check_constraint(statement_insert.execute([key]))?;
                }
            }
        }
        Ok(new_map)
    }

    /// Creates a new [`SQLiteMap`] that contains *all* keys contained in this set.
    ///
    /// The given `mapping` function is used to generate the map’s values. If it returns `Some(value)` for a key, that key is inserted into the map with the returned `value` as its associated value. If it returns `None`, the key is **not** inserted into the map.
    ///
    /// The created map is case-sensitive.
    #[inline]
    pub fn to_map<F, V>(&mut self, mapping: F) -> Result<SQLiteMap<T, V>, Error>
    where
        F: Fn(&T) -> Option<V>,
        V: Type,
    {
        let mut map = SQLiteMap::<T, V>::new()?;
        map.transaction()?.extend_set(self, mapping)?;
        Ok(map)
    }

    /// Creates a new [`SQLiteMap`] that contains *all* keys contained in this set.
    ///
    /// The given `mapping` function is used to generate the map’s values. If it returns `Some(value)` for a key, that key is inserted into the map with the returned `value` as its associated value. If it returns `None`, the key is **not** inserted into the map.
    ///
    /// The created map is case-insensitive.
    ///
    /// Only ASCII characters are case folded. SQLite does **not** attempt to do full UTF case folding!
    #[inline]
    pub fn to_map_nocase<F, V>(&mut self, mapping: F) -> Result<SQLiteMap<T, V>, Error>
    where
        F: Fn(&T) -> Option<V>,
        V: Type,
        T: CharType,
    {
        let mut map = SQLiteMap::<T, V>::with_nocase()?;
        map.transaction()?.extend_set(self, mapping)?;
        Ok(map)
    }

    /// Stores all contents of this set into the specified output database file.
    ///
    /// The file will be replaced, if it already exists!
    #[inline]
    pub fn to_file<P>(&self, path: P) -> Result<(), Error>
    where
        P: AsRef<Path>,
    {
        let file = path.as_ref().to_str().ok_or(Error::InvalidPath)?;
        let mut statement = self.connection.prepare_cached(SQL_STORE_INTO)?;
        statement.execute([file])?;
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// SQLiteMap Traits
// ---------------------------------------------------------------------------

impl<T> TryEq for SQLiteSet<T>
where
    T: Type,
{
    type Error = Error;

    /// Implementation of the “is equal to” operator for [`SQLiteSet`].
    ///
    /// Two sets are considered equal if and only if they contain the same keys, i.e., every key in the first set also exists in the second set and vice versa. The keys are matched using the case-sensitivity of the underlying sets.
    fn try_eq(&self, other: &Self) -> Result<bool, Error> {
        if addr_eq(self, other) {
            return Ok(true);
        }
        let tx_1 = self.transaction_readonly()?;
        let tx_2 = other.transaction_readonly()?;
        let mut statement_query_1 = tx_1.transaction.prepare_cached(SQL_QUERY_KEYS)?;
        let mut statement_find_1 = tx_1.transaction.prepare_cached(SQL_EXISTS_KEY)?;
        let mut statement_query_2 = tx_2.transaction.prepare_cached(SQL_QUERY_KEYS)?;
        let mut statement_find_2 = tx_2.transaction.prepare_cached(SQL_EXISTS_KEY)?;
        for key in statement_query_1.query_map([], |row| row.get(0))? {
            let key: T = key?;
            if !statement_find_2.exists([key])? {
                return Ok(false);
            }
        }
        for key in statement_query_2.query_map([], |row| row.get(0))? {
            let key: T = key?;
            if !statement_find_1.exists([key])? {
                return Ok(false);
            }
        }
        Ok(true)
    }
}

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
        cfg_if! {
            if #[cfg(feature = "backup")] {
                Ok(Self::from_connection(crate::common::duplicate_db(&self.connection)?))
            } else {
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
        transaction.set_drop_behavior(DropBehavior::Commit);
        Ok(Self { _type: PhantomData, transaction })
    }

    #[inline]
    fn from_immutable(connection: &'a Connection) -> Result<Self, Error> {
        let mut transaction = connection.unchecked_transaction()?;
        transaction.set_drop_behavior(DropBehavior::Commit);
        Ok(Self { _type: PhantomData, transaction })
    }

    /// Drops the `SQLiteSetTransaction`, thereby committing the SQLite transaction.
    #[inline]
    pub fn commit(self) {}

    /// This function is equivalent to [`SQLiteSet::insert()`].
    #[inline]
    pub fn insert(&mut self, key: T) -> Result<bool, Error> {
        let mut statement = self.transaction.prepare_cached(SQL_INSERT_KEY)?;
        Ok(check_constraint(statement.execute([key]))?)
    }

    /// This function is equivalent to [`SQLiteSet::insert_ref()`].
    #[inline]
    pub fn insert_ref<Q>(&mut self, key: &Q) -> Result<bool, Error>
    where
        T: Borrow<Q>,
        Q: ?Sized + ToSql,
    {
        let mut statement = self.transaction.prepare_cached(SQL_INSERT_KEY)?;
        Ok(check_constraint(statement.execute([key]))?)
    }

    /// This function is equivalent to [`SQLiteSet::extend()`].
    #[inline]
    pub fn extend<I>(&mut self, keys: I) -> Result<Size, Error>
    where
        I: IntoIterator<Item = T>,
    {
        let mut statement = self.transaction.prepare_cached(SQL_INSERT_KEY)?;
        let mut counter = Size::default();
        for key in keys {
            if check_constraint(statement.execute([key]))? {
                increment(&mut counter);
            }
        }
        Ok(counter)
    }

    /// This function is equivalent to [`SQLiteSet::extend_ref()`].
    #[inline]
    pub fn extend_ref<'b, I, Q>(&mut self, keys: I) -> Result<Size, Error>
    where
        I: IntoIterator<Item = &'b Q>,
        T: Borrow<Q>,
        Q: ?Sized + ToSql + 'b,
    {
        let mut statement = self.transaction.prepare_cached(SQL_INSERT_KEY)?;
        let mut counter = Size::default();
        for key in keys {
            if check_constraint(statement.execute([key]))? {
                increment(&mut counter);
            }
        }
        Ok(counter)
    }

    /// This function is equivalent to [`SQLiteSet::extend_set()`].
    #[inline]
    pub fn extend_set(&mut self, other: &SQLiteSet<T>) -> Result<Size, Error> {
        let mut statement_insert = self.transaction.prepare_cached(SQL_INSERT_KEY)?;
        let mut statement_reader = other.connection.prepare_cached(SQL_QUERY_KEYS)?;
        let mut counter = Size::default();
        for key in statement_reader.query_map([], |row| row.get::<_, T>(0))? {
            if check_constraint(statement_insert.execute([key?]))? {
                increment(&mut counter);
            }
        }
        Ok(counter)
    }

    /// Insert all keys from the given [`SQLiteMap`] into this set.
    #[inline]
    pub(crate) fn extend_map<V>(&mut self, map: &SQLiteMap<T, V>) -> Result<(), Error>
    where
        V: Type,
    {
        let mut statement = self.transaction.prepare_cached(SQL_INSERT_KEY)?;
        let trx = map.transaction_readonly()?;
        trx.try_for_each_key(|key| {
            check_constraint(statement.execute([key]))?;
            Ok(())
        })
        .map_err(|err| err.into())
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

    /// This function is equivalent to [`SQLiteSet::remove_all()`].
    #[inline]
    pub fn remove_all<I>(&mut self, keys: I) -> Result<Size, Error>
    where
        I: IntoIterator<Item = T>,
    {
        let mut statement = self.transaction.prepare_cached(SQL_DELETE_KEY)?;
        let mut counter = Size::default();
        for key in keys {
            if statement.execute([key])? != 0 {
                increment(&mut counter);
            }
        }
        Ok(counter)
    }

    /// This function is equivalent to [`SQLiteSet::remove_all_nocase()`].
    #[inline]
    pub fn remove_all_nocase<I>(&mut self, keys: I) -> Result<Size, Error>
    where
        I: IntoIterator<Item = T>,
        T: CharType,
    {
        let mut statement = self.transaction.prepare_cached(SQL_DELETE_NOC)?;
        let mut counter = Size::default();
        for key in keys {
            if statement.execute([key])? != 0 {
                increment(&mut counter);
            }
        }
        Ok(counter)
    }

    /// This function is equivalent to [`SQLiteSet::remove_set()`].
    #[inline]
    pub fn remove_set(&mut self, other: &SQLiteSet<T>) -> Result<Size, Error> {
        let mut statement_delete = self.transaction.prepare_cached(SQL_DELETE_KEY)?;
        let mut statement_query = other.connection.prepare_cached(SQL_QUERY_KEYS)?;
        let mut counter = Size::default();
        for key in statement_query.query_map([], |row| row.get::<_, T>(0))? {
            if statement_delete.execute([key?])? != 0 {
                increment(&mut counter);
            }
        }
        Ok(counter)
    }

    /// This function is equivalent to [`SQLiteSet::remove_set_nocase()`].
    #[inline]
    pub fn remove_set_nocase(&mut self, other: &SQLiteSet<T>) -> Result<Size, Error>
    where
        T: CharType,
    {
        let mut statement_delete = self.transaction.prepare_cached(SQL_DELETE_NOC)?;
        let mut statement_query = other.connection.prepare_cached(SQL_QUERY_KEYS)?;
        let mut counter = Size::default();
        for key in statement_query.query_map([], |row| row.get::<_, T>(0))? {
            if statement_delete.execute([key?])? != 0 {
                increment(&mut counter);
            }
        }
        Ok(counter)
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

    /// This is the *failable* version of the [`for_each()`](Self::for_each) function.
    #[inline]
    pub fn try_for_each<F, E>(&self, mut callback: F) -> Result<(), IterationError<E>>
    where
        F: FnMut(T) -> Result<(), E>,
    {
        let mut statement = self.transaction.prepare_cached(SQL_QUERY_KEYS)?;
        for key in statement.query_map([], |row| row.get(0))? {
            if let Err(error) = callback(key?) {
                return Err(IterationError::CallbackError(error));
            }
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
        let mut statement = self.transaction.prepare_cached(SQL_TEST_EMPTY)?;
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
