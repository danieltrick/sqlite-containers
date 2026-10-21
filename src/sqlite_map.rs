// SQLiteMap
// This file is part of the 'SQLite-based containers for Rust' project (sqlite-containers)
// SPDX-License-Identifier: Unlicense

use crate::{
    CharType, Error, IterationError, SQLiteSet, Size, SizeExt, TryClone, Type,
    common::{OPEN_FLAGS, OPEN_FLAGS_NOCREATE, OPEN_FLAGS_RDONLY, SQL_TYPE_TEXT, SQL_TYPES, SqlType, check_constraint_violation, increment, verify_schema},
    rusqlite::{Connection, DropBehavior, OptionalExtension, ToSql, Transaction, TransactionBehavior, params},
};
use cfg_if::cfg_if;
use const_format::concatcp;
use std::{borrow::Borrow, collections::BTreeMap, marker::PhantomData, path::Path, sync::LazyLock};

// ---------------------------------------------------------------------------
// Action Type
// ---------------------------------------------------------------------------

/// Represents the outcome of an [`update()`](SQLiteMap::update) operation.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Action {
    /// It is *unknown* whether a new key-value pair was inserted or the value associated with an existing key was updated.
    ///
    /// This is the expected return value, if the [`hooks`](crate#optional-features) feature is **not** enabled.
    Unknown,
    /// Indicates that a new key-value pair was inserted.
    Insert,
    /// Indicates that the value associated with an existing key was updated.
    ///
    /// Please note that [`Action::Update`](Self::Update) does **not** guarantee that the value associated with the key has actually changed &ndash; it is possible to “update” the associated value of a key to a new value that is equal to the previously associated value.
    Update,
}

cfg_if! {
    if #[cfg(feature = "hooks")] {
        use crate::rusqlite::hooks::Action as RusqliteAction;
        use std::sync::{Arc, atomic::{Ordering::Relaxed, AtomicI32}};
        struct LastActionHolder(Arc<AtomicI32>);

        impl Action {
            #[inline]
            fn from(value: &LastActionHolder) -> Self {
                const ACTION_SQLITE_INSERT: i32 = RusqliteAction::SQLITE_INSERT as i32;
                const ACTION_SQLITE_UPDATE: i32 = RusqliteAction::SQLITE_UPDATE as i32;
                match value.0.load(Relaxed) {
                    ACTION_SQLITE_INSERT => Action::Insert,
                    ACTION_SQLITE_UPDATE => Action::Update,
                    _ => Action::Unknown,
                }
            }
        }

        impl Default for LastActionHolder {
            #[inline]
            fn default() -> Self {
                Self(Arc::new(AtomicI32::new(RusqliteAction::UNKNOWN as i32)))
            }
        }
    } else {
        struct LastActionHolder;

        impl Action {
            #[inline(always)]
            const fn from(_: &LastActionHolder) -> Self {
                Action::Unknown
            }
        }

        impl LastActionHolder {
            #[inline(always)]
            pub const fn default() -> Self {
                Self
            }
        }
    }
}

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

const SQL_GET_SCHEMA: &str = concatcp!("SELECT sql FROM sqlite_schema WHERE type = 'table' AND name = '", TABLE_NAME, "';");
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
    last_action: LastActionHolder,
}

impl<K, V> SQLiteMap<K, V>
where
    K: Type,
    V: Type,
{
    /// Creates a new, empty hash map with case-sensitive keys, backed by an *in-memory* SQLite database.
    #[allow(clippy::let_unit_value)]
    #[inline]
    pub fn new() -> Result<Self, Error> {
        Self::from_connection(Self::create_connection::<false, &Path>(None, false, false)?)
    }

    /// Creates a new, empty hash map with case-insensitive keys, backed by an *in-memory* SQLite database.
    ///
    /// Only ASCII characters are case folded. SQLite does **not** attempt to do full UTF case folding!
    #[allow(clippy::let_unit_value)]
    #[inline]
    pub fn with_nocase() -> Result<Self, Error>
    where
        K: CharType,
    {
        Self::from_connection(Self::create_connection::<true, &Path>(None, false, false)?)
    }

    /// Creates a new map with case-sensitive keys containing all key-value pairs yielded by the given [iterable](std::iter::IntoIterator).
    ///
    /// The new hash map is backed by an *in-memory* SQLite database.
    #[inline]
    pub fn try_from<I>(entries: I) -> Result<Self, Error>
    where
        I: IntoIterator<Item = (K, V)>,
    {
        let mut set = Self::new()?;
        set.extend(entries)?;
        Ok(set)
    }

    /// Creates a map set with case-insensitive keys containing all key-value pairs yielded by the given [iterable](std::iter::IntoIterator).
    ///
    /// The new hash map is backed by an *in-memory* SQLite database.
    ///
    /// Only ASCII characters are case folded. SQLite does **not** attempt to do full UTF case folding!
    #[inline]
    pub fn try_from_nocase<I>(entries: I) -> Result<Self, Error>
    where
        I: IntoIterator<Item = (K, V)>,
        K: CharType,
    {
        let mut set = Self::with_nocase()?;
        set.extend(entries)?;
        Ok(set)
    }

    /// Creates a new or re-opens an existing hash map with case-sensitive keys, backed by a *file-based* SQLite database.
    ///
    /// If `readonly` is `true`, the database file is opened in “read-only” mode.
    ///
    /// If `can_create` is `true` (and if `readonly` is `false`), a new empty database file is created, if the file does *not* exist yet.
    ///
    /// Returns [`Error::InvalidSchema`], if the database file already exists but its schema does **not** match the schema expected by this type of map. This means that you **must** re-open a database file with the *same* type of map that was used to create it.
    #[inline]
    pub fn try_from_file<P>(path: P, readonly: bool, can_create: bool) -> Result<Self, Error>
    where
        P: AsRef<Path>,
    {
        Self::from_connection(Self::create_connection::<false, _>(Some(path), readonly, can_create)?)
    }

    /// Creates a new or re-opens an existing hash map with case-insensitive keys, backed by a *file-based* SQLite database.
    ///
    /// If `readonly` is `true`, the database file is opened in “read-only” mode.
    ///
    /// If `can_create` is `true` (and if `readonly` is `false`), a new empty database file is created if the file does *not* exist yet.
    ///
    /// Returns [`Error::InvalidSchema`], if the database file already exists but its schema does **not** match the schema expected by this type of map. This means that you **must** re-open a database file with the *same* type of map that was used to create it.
    ///
    /// Only ASCII characters are case folded. SQLite does **not** attempt to do full UTF case folding!
    #[inline]
    pub fn try_from_file_nocase<P>(path: P, readonly: bool, can_create: bool) -> Result<Self, Error>
    where
        K: CharType,
        P: AsRef<Path>,
    {
        Self::from_connection(Self::create_connection::<true, _>(Some(path), readonly, can_create)?)
    }

    #[inline]
    fn create_connection<const NO_CASE: bool, P>(path_option: Option<P>, readonly: bool, can_create: bool) -> Result<Connection, Error>
    where
        P: AsRef<Path>,
    {
        let connection = if let Some(path) = path_option.as_ref() {
            Connection::open_with_flags(path, if !readonly { if can_create { *OPEN_FLAGS } else { *OPEN_FLAGS_NOCREATE } } else { *OPEN_FLAGS_RDONLY })?
        } else {
            Connection::open_in_memory()?
        };

        let connection = Self::initialize_connection(connection)?;

        #[cfg(feature = "checked")]
        if path_option.is_some() {
            crate::common::integrity_check(&connection)?;
        }

        let sql_create_table = if !NO_CASE {
            SQL_CREATE_TBL.get(&(K::SQL_TYPE, V::SQL_TYPE)).unwrap()
        } else {
            debug_assert_eq!(K::SQL_TYPE, SQL_TYPE_TEXT);
            SQL_CREATE_NOC.get(&V::SQL_TYPE).unwrap()
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
    fn initialize_connection(mut connection: Connection) -> Result<Connection, Error> {
        connection.set_transaction_behavior(TransactionBehavior::Exclusive);
        connection.pragma_update(None, "synchronous", "OFF")?;
        connection.pragma_update(None, "journal_mode", "OFF")?;
        Ok(connection)
    }

    #[inline(always)]
    fn from_connection(connection: Connection) -> Result<Self, Error> {
        let last_action = LastActionHolder::default();
        Self::install_hook(&connection, &last_action);
        Ok(Self { _types: (PhantomData, PhantomData), connection, last_action })
    }

    cfg_if! {
        if #[cfg(feature = "hooks")] {
            #[inline(always)]
            fn install_hook(connection: &Connection, last_action: &LastActionHolder) {
                let last_action = Arc::clone(&last_action.0);
                connection
                    .update_hook(Some(move |action, _db: &str, _table: &str, _rowid: i64| {
                        last_action.store(action as i32, Relaxed);
                    }))
                    .unwrap();
            }
        } else {
            #[inline(always)]
            fn install_hook(_: &Connection, _: &LastActionHolder) {}
        }
    }

    /// Starts a new SQLite transaction for this map.
    ///
    /// Please note that using an explicit SQLite transaction allows for much more efficient bulk inserts &#x1F680;
    ///
    /// Returns the new [`SQLiteMapTransaction`] instance.
    #[inline]
    pub fn transaction(&mut self) -> Result<SQLiteMapTransaction<'_, K, V>, Error> {
        SQLiteMapTransaction::from(&mut self.connection, &self.last_action)
    }

    /// This is the “read-only” version of the [`transaction()`](Self::transaction) function.
    #[inline]
    pub(crate) fn transaction_readonly(&self) -> Result<SQLiteMapTransaction<'_, K, V>, Error> {
        SQLiteMapTransaction::from_immutable(&self.connection, &self.last_action)
    }

    /// Inserts the given key-value pair into the map, if the specified key does *not* exist yet.
    ///
    /// If the map already contains the specified key, then its associated value is **not** changed by this operation!
    ///
    /// Returns `true`, if the key-value pair was inserted; otherwise returns `false`.
    ///
    /// Please use the [`update()`](Self::update) function to update the value associated with a key that may already exist.
    #[inline]
    pub fn insert(&mut self, key: K, value: V) -> Result<bool, Error> {
        let mut statement = self.connection.prepare_cached(SQL_INSERT_KEY)?;
        match statement.execute(params![key, value]) {
            Ok(_) => Ok(true),
            Err(error) => check_constraint_violation(error),
        }
    }

    /// Inserts the given key-value pair into the map, if the specified key does *not* exist yet.
    ///
    /// If the map already contains the specified key, then its associated value is **not** changed by this operation!
    ///
    /// Returns `true`, if the key-value pair was inserted; otherwise returns `false`.
    ///
    /// Please use the [`update()`](Self::update) function to update the value associated with a key that may already exist.
    #[inline]
    pub fn insert_ref<Q, R>(&mut self, key: &Q, value: &R) -> Result<bool, Error>
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
    /// Provided that the [`hooks`](crate#optional-features) feature is enabled, the function returns either [`Action::Insert`] or [`Action::Update`], depending on whether a new key-value pair was inserted or the associated value of an existing key was updated
    ///
    /// If `hooks` is **not** enabled, the function *always* returns [`Action::Unknown`], but, apart from that, it works identically &#128522;
    #[inline]
    pub fn update(&mut self, key: K, value: V) -> Result<Action, Error> {
        let mut statement = self.connection.prepare_cached(SQL_UPSERT_KEY)?;
        statement.execute(params![key, value])?;
        Ok(Action::from(&self.last_action))
    }

    /// Associates the specified value with the specified key, regardless of whether the key already exists in the map.
    ///
    /// If the map does *not* already contain the specified key, then the key is inserted automatically.
    ///
    /// Provided that the [`hooks`](crate#optional-features) feature is enabled, the function returns either [`Action::Insert`] or [`Action::Update`], depending on whether a new key-value pair was inserted or the associated value of an existing key was updated
    ///
    /// If `hooks` is **not** enabled, the function *always* returns [`Action::Unknown`], but, apart from that, it works identically &#128522;
    #[inline]
    pub fn update_ref<Q, R>(&mut self, key: &Q, value: &R) -> Result<Action, Error>
    where
        K: Borrow<Q>,
        Q: ?Sized + ToSql,
        V: Borrow<R>,
        R: ?Sized + ToSql,
    {
        let mut statement = self.connection.prepare_cached(SQL_UPSERT_KEY)?;
        statement.execute(params![key, value])?;
        Ok(Action::from(&self.last_action))
    }

    /// Inserts all key-value pairs yielded by the given [iterable](std::iter::IntoIterator) into the map, skipping existing keys.
    ///
    /// Returns the number of new key-value pairs that have been inserted.
    #[inline]
    pub fn extend<I>(&mut self, entries: I) -> Result<Size, Error>
    where
        I: IntoIterator<Item = (K, V)>,
    {
        let trx = self.transaction()?;
        let mut statement = trx.transaction.prepare_cached(SQL_INSERT_KEY)?;
        let mut counter = Size::default();
        for (key, value) in entries {
            if match statement.execute(params![key, value]) {
                Ok(_) => true,
                Err(error) => check_constraint_violation(error)?,
            } {
                increment(&mut counter);
            }
        }
        Ok(counter)
    }

    /// Inserts all key-value pairs yielded by the given [iterable](std::iter::IntoIterator) into the map, skipping existing keys.
    ///
    /// Returns the number of new key-value pairs that have been inserted.
    #[inline]
    pub fn extend_ref<'a, I, Q, R>(&mut self, entries: I) -> Result<Size, Error>
    where
        I: IntoIterator<Item = (&'a Q, &'a R)>,
        K: Borrow<Q>,
        Q: ?Sized + ToSql + 'a,
        V: Borrow<R>,
        R: ?Sized + ToSql + 'a,
    {
        let trx = self.transaction()?;
        let mut statement = trx.transaction.prepare_cached(SQL_INSERT_KEY)?;
        let mut counter = Size::default();
        for (key, value) in entries {
            if match statement.execute(params![key, value]) {
                Ok(_) => true,
                Err(error) => check_constraint_violation(error)?,
            } {
                increment(&mut counter);
            }
        }
        Ok(counter)
    }

    /// Inserts all key-value pairs from another map into this map.
    ///
    /// If `update` is `true`, this function updates the values associated with existing keys in this map using the corresponding values from the other map. Otherwise, the values associated with existing keys are **not** changed.
    ///
    /// Returns `Some((inserted, updated))`, i.e., the number of new keys inserted and existing keys updated, respectively.
    ///
    /// If [`hooks`](crate#optional-features) is **not** enabled and if `update` is `true`, the function *always* returns `None`, though &#128517;
    #[inline]
    pub fn union(&mut self, other: &Self, update: bool) -> Result<Option<(Size, Size)>, Error> {
        let trx = self.transaction()?;
        let mut statement_insert = trx.transaction.prepare_cached(if update { SQL_UPSERT_KEY } else { SQL_INSERT_KEY })?;
        let mut statement_query = other.connection.prepare_cached(SQL_QUERY_PAIR)?;
        let mut result = statement_query.query([])?;
        let mut counter = (Size::default(), Size::default());
        while let Some(key_value) = result.next()? {
            let (key, value): (K, V) = (key_value.get(0)?, key_value.get(1)?);
            if !update {
                match statement_insert.execute(params![key, value]) {
                    Ok(_) => increment(&mut counter.0),
                    Err(error) => {
                        check_constraint_violation(error)?;
                    }
                }
            } else {
                statement_insert.execute(params![key, value])?;
                #[cfg(feature = "hooks")]
                update_counter(&mut counter, Action::from(trx.last_action));
            }
        }
        cfg_if! {
            if #[cfg(feature = "hooks")] {
                Ok(Some(counter))
            } else {
                Ok(if !update { Some(counter) } else { None })
            }
        }
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

    /// This is the *failable* version of the [`for_each_key()`](Self::for_each_key) function.
    #[inline]
    pub fn try_for_each_key<F, E>(&self, mut callback: F) -> Result<(), IterationError<E>>
    where
        F: FnMut(K) -> Result<(), E>,
    {
        let mut statement = self.connection.prepare_cached(SQL_QUERY_KEYS)?;
        for key in statement.query_map([], |row| row.get(0))? {
            if let Err(error) = callback(key?) {
                return Err(IterationError::CallbackError(error));
            }
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

    /// This is the *failable* version of the [`for_each_entry()`](Self::for_each_entry) function.
    #[inline]
    pub fn try_for_each_entry<F, E>(&self, mut callback: F) -> Result<(), IterationError<E>>
    where
        F: FnMut(K, V) -> Result<(), E>,
    {
        let mut statement = self.connection.prepare_cached(SQL_QUERY_PAIR)?;
        let mut result = statement.query([])?;
        while let Some(key_value) = result.next()? {
            let (key, value): (K, V) = (key_value.get(0)?, key_value.get(1)?);
            if let Err(error) = callback(key, value) {
                return Err(IterationError::CallbackError(error));
            }
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

    /// Creates a new [`SQLiteSet`] that contains *all* keys contained in this map.
    ///
    /// The created set is case-sensitive.
    #[inline]
    pub fn to_set(&self) -> Result<SQLiteSet<K>, Error> {
        let mut key_set = SQLiteSet::<K>::new()?;
        key_set.transaction()?.extend_from_map(self)?;
        Ok(key_set)
    }

    /// Creates a new [`SQLiteSet`] that contains *all* keys contained in this map.
    ///
    /// The created set is case-insensitive.
    ///
    /// Only ASCII characters are case folded. SQLite does **not** attempt to do full UTF case folding!
    #[inline]
    pub fn to_set_nocase(&mut self) -> Result<SQLiteSet<K>, Error>
    where
        K: CharType,
    {
        let mut key_set = SQLiteSet::<K>::with_nocase()?;
        key_set.transaction()?.extend_from_map(self)?;
        Ok(key_set)
    }

    #[cfg(feature = "backup")]
    #[inline]
    /// Stores all contents of this map in the specified database file.
    ///
    /// The file will be replaced, if it already exists!
    pub fn to_file<P>(&self, path: P) -> Result<(), Error>
    where
        P: AsRef<Path>,
    {
        let mut connection_out = Self::initialize_connection(Connection::open_with_flags(path, *OPEN_FLAGS)?)?;
        crate::common::backup_db(&self.connection, &mut connection_out)?;
        Ok(())
    }
}

cfg_if! {
    if #[cfg(feature = "backup")] {
        impl<K, V> TryClone for SQLiteMap<K, V>
        where
            K: Type,
            V: Type,
        {
            type Error = Error;

            /// Creates an exact, independent *copy* of the given map instance.
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
        impl<K, V> TryClone for SQLiteMap<K, V>
        where
            K: Type,
            V: Type,
        {
            type Error = Error;

            /// Creates an exact, independent *copy* of the given map instance.
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
    last_action: &'a LastActionHolder,
}

impl<'a, K, V> SQLiteMapTransaction<'a, K, V>
where
    K: Type,
    V: Type,
{
    #[inline]
    fn from(connection: &'a mut Connection, last_action: &'a LastActionHolder) -> Result<Self, Error> {
        let mut transaction = connection.transaction()?;
        transaction.set_drop_behavior(DropBehavior::Commit);
        Ok(Self { _types: (PhantomData, PhantomData), transaction, last_action })
    }

    #[inline]
    fn from_immutable(connection: &'a Connection, last_action: &'a LastActionHolder) -> Result<Self, Error> {
        let mut transaction = connection.unchecked_transaction()?;
        transaction.set_drop_behavior(DropBehavior::Commit);
        Ok(Self { _types: (PhantomData, PhantomData), transaction, last_action })
    }

    /// Drops the `SQLiteMapTransaction`, thereby committing the SQLite transaction.
    #[inline]
    pub fn commit(self) {}

    /// This function is equivalent to [`SQLiteMap::insert()`].
    #[inline]
    pub fn insert(&mut self, key: K, value: V) -> Result<bool, Error> {
        let mut statement = self.transaction.prepare_cached(SQL_INSERT_KEY)?;
        match statement.execute(params![key, value]) {
            Ok(_) => Ok(true),
            Err(error) => check_constraint_violation(error),
        }
    }

    /// This function is equivalent to [`SQLiteMap::insert_ref()`].
    #[inline]
    pub fn insert_ref<Q, R>(&mut self, key: &Q, value: &R) -> Result<bool, Error>
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
    pub fn update(&mut self, key: K, value: V) -> Result<Action, Error> {
        let mut statement = self.transaction.prepare_cached(SQL_UPSERT_KEY)?;
        statement.execute(params![key, value])?;
        Ok(Action::from(self.last_action))
    }

    /// This function is equivalent to [`SQLiteMap::update_ref()`].
    #[inline]
    pub fn update_ref<Q, R>(&mut self, key: &Q, value: &R) -> Result<Action, Error>
    where
        K: Borrow<Q>,
        Q: ?Sized + ToSql,
        V: Borrow<R>,
        R: ?Sized + ToSql,
    {
        let mut statement = self.transaction.prepare_cached(SQL_UPSERT_KEY)?;
        statement.execute(params![key, value])?;
        Ok(Action::from(self.last_action))
    }

    /// This function is equivalent to [`SQLiteMap::extend()`].
    #[inline]
    pub fn extend<I>(&mut self, keys: I) -> Result<Size, Error>
    where
        I: IntoIterator<Item = (K, V)>,
    {
        let mut statement = self.transaction.prepare_cached(SQL_INSERT_KEY)?;
        let mut counter = Size::default();
        for (key, value) in keys {
            if match statement.execute(params![key, value]) {
                Ok(_) => true,
                Err(error) => check_constraint_violation(error)?,
            } {
                increment(&mut counter);
            }
        }
        Ok(counter)
    }

    /// This function is equivalent to [`SQLiteMap::extend_ref()`].
    #[inline]
    pub fn extend_ref<'b, I, Q, R>(&mut self, entries: I) -> Result<Size, Error>
    where
        I: IntoIterator<Item = (&'a Q, &'a R)>,
        K: Borrow<Q>,
        Q: ?Sized + ToSql + 'a,
        V: Borrow<R>,
        R: ?Sized + ToSql + 'a,
    {
        let mut statement = self.transaction.prepare_cached(SQL_INSERT_KEY)?;
        let mut counter = Size::default();
        for (key, value) in entries {
            if match statement.execute(params![key, value]) {
                Ok(_) => true,
                Err(error) => check_constraint_violation(error)?,
            } {
                increment(&mut counter);
            }
        }
        Ok(counter)
    }

    /// Insert all keys from the given [`SQLiteSet`] into this set.
    #[inline]
    pub(crate) fn extend_from_set<F>(&mut self, set: &mut SQLiteSet<K>, mapping: F) -> Result<(), Error>
    where
        F: Fn(&K) -> Option<V>,
    {
        let mut statement = self.transaction.prepare_cached(SQL_INSERT_KEY)?;
        let trx = set.transaction()?;
        trx.try_for_each(|key| match mapping(&key) {
            Some(value) => match statement.execute(params![key, value]) {
                Ok(_) => Ok(()),
                Err(error) => check_constraint_violation(error).map(|_| ()),
            },
            None => Ok(()),
        })
        .map_err(|error| error.into())
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

    /// This is the *failable* version of the [`for_each_key()`](Self::for_each_key) function.
    #[inline]
    pub fn try_for_each_key<F, E>(&self, mut callback: F) -> Result<(), IterationError<E>>
    where
        F: FnMut(K) -> Result<(), E>,
    {
        let mut statement = self.transaction.prepare_cached(SQL_QUERY_KEYS)?;
        for key in statement.query_map([], |row| row.get(0))? {
            if let Err(error) = callback(key?) {
                return Err(IterationError::CallbackError(error));
            }
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

    /// This is the *failable* version of the [`for_each_entry()`](Self::for_each_entry) function.
    #[inline]
    pub fn try_for_each_entry<F, E>(&self, mut callback: F) -> Result<(), IterationError<E>>
    where
        F: FnMut(K, V) -> Result<(), E>,
    {
        let mut statement = self.transaction.prepare_cached(SQL_QUERY_PAIR)?;
        let mut result = statement.query([])?;
        while let Some(key_value) = result.next()? {
            let (key, value): (K, V) = (key_value.get(0)?, key_value.get(1)?);
            if let Err(error) = callback(key, value) {
                return Err(IterationError::CallbackError(error));
            }
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

// ---------------------------------------------------------------------------
// Auxilary Functions
// ---------------------------------------------------------------------------

#[cfg(feature = "hooks")]
#[inline(always)]
fn update_counter(counter: &mut (Size, Size), action: Action) {
    match action {
        Action::Insert => increment(&mut counter.0),
        Action::Update => increment(&mut counter.1),
        Action::Unknown => (), /*hint::cold_path()*/
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod test {
    use super::*;

    #[cfg(feature = "hooks")]
    #[test]
    fn test_actions() {
        let action_holder = LastActionHolder::default();
        for value in [RusqliteAction::SQLITE_INSERT, RusqliteAction::SQLITE_UPDATE, RusqliteAction::UNKNOWN] {
            action_holder.0.store(value as i32, Relaxed);
            match Action::from(&action_holder) {
                Action::Insert => assert_eq!(value, RusqliteAction::SQLITE_INSERT),
                Action::Update => assert_eq!(value, RusqliteAction::SQLITE_UPDATE),
                Action::Unknown => {
                    assert_ne!(value, RusqliteAction::SQLITE_INSERT);
                    assert_ne!(value, RusqliteAction::SQLITE_UPDATE);
                }
            }
        }
    }

    #[cfg(not(feature = "hooks"))]
    #[test]
    fn test_actions() {
        let action_holder = LastActionHolder::default();
        assert!(matches!(Action::from(&action_holder), Action::Unknown));
    }

    #[cfg(feature = "hooks")]
    #[test]
    fn test_counter() {
        let mut counter = Default::default();
        for i in 0..8 {
            update_counter(&mut counter, Action::Unknown);
            assert_eq!(counter, (i, i));
            update_counter(&mut counter, Action::Insert);
            assert_eq!(counter, (i + 1, i));
            update_counter(&mut counter, Action::Update);
            assert_eq!(counter, (i + 1, i + 1));
        }
    }
}
