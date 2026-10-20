// Common Data Types
// This file is part of the SQLite-based containers for Rust project (sqlite-containers)
// SPDX-License-Identifier: Unlicense

use crate::{
    rusqlite::{
        Error as RusqliteError, OpenFlags,
        ffi::{Error as FfiError, ErrorCode as FfiErrorCode},
        types::{FromSql, ToSql, Type as RusqliteType},
        version_number,
    },
    semver::Version,
};
use cfg_if::cfg_if;
use std::{
    fmt::{Debug, Display},
    sync::LazyLock,
};

#[cfg(any(feature = "backup", feature = "checked"))]
use crate::rusqlite::Connection;

// ---------------------------------------------------------------------------
// Flags
// ---------------------------------------------------------------------------

/// Binary Large Object (BLOB), represented as a [`Vec<u8>`](std::vec::Vec).
pub type Blob = Vec<u8>;

/// Flags to be used for [`Connection::open_with_flags()`](rusqlite::Connection::open_with_flags).
pub static OPEN_FLAGS: LazyLock<OpenFlags> = LazyLock::new(|| {
    let mut flags = OpenFlags::default();
    flags.remove(OpenFlags::SQLITE_OPEN_URI);
    flags
});

/// Flags to be used for [`Connection::open_with_flags()`](rusqlite::Connection::open_with_flags) *without* `SQLITE_OPEN_CREATE`.
pub static OPEN_FLAGS_NOCREATE: LazyLock<OpenFlags> = LazyLock::new(|| {
    let mut flags = OpenFlags::default();
    flags.remove(OpenFlags::SQLITE_OPEN_URI);
    flags.remove(OpenFlags::SQLITE_OPEN_CREATE);
    flags
});

/// Flags to be used for [`Connection::open_with_flags()`](rusqlite::Connection::open_with_flags) *without* `SQLITE_OPEN_CREATE`.
pub static OPEN_FLAGS_RDONLY: LazyLock<OpenFlags> = LazyLock::new(|| {
    let mut flags = OpenFlags::default();
    flags.remove(OpenFlags::SQLITE_OPEN_URI);
    flags.remove(OpenFlags::SQLITE_OPEN_CREATE);
    flags.remove(OpenFlags::SQLITE_OPEN_READ_WRITE);
    flags.insert(OpenFlags::SQLITE_OPEN_READ_ONLY);
    flags
});

// ---------------------------------------------------------------------------
// Sealed Type
// ---------------------------------------------------------------------------

trait Sealed {}

macro_rules! impl_sealed {
    ($($t:ty),*) => {
        $(
            impl Sealed for $t {}
        )*
    };
}

impl_sealed!(i32, u32, i64, String, Blob);

// ---------------------------------------------------------------------------
// SQL Type Holder
// ---------------------------------------------------------------------------

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub struct SqlType(pub RusqliteType);

impl SqlType {
    #[inline(always)]
    pub const fn new(value: RusqliteType) -> Self {
        Self(value)
    }
}

impl Ord for SqlType {
    #[inline]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (self.0 as usize).cmp(&(other.0 as usize))
    }
}

impl PartialOrd for SqlType {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Display for SqlType {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0.to_string().to_ascii_uppercase())
    }
}

pub const SQL_TYPE_TEXT: SqlType = SqlType::new(RusqliteType::Text);
pub static SQL_TYPES: [SqlType; 3usize] = [SqlType::new(RusqliteType::Integer), SqlType::new(RusqliteType::Text), SqlType::new(RusqliteType::Blob)];

// ---------------------------------------------------------------------------
// Key or Value Type
// ---------------------------------------------------------------------------

/// The type of the keys or values that are stored in an [`SQLiteMap`](crate::SQLiteMap) or an [`SQLiteSet`](crate::SQLiteSet).
///
/// Currently, the following types are supported:
///
/// Type                              | SQLite storage datatype
/// --------------------------------- | -----------------------
/// **`i32`**                         | `INTEGER`
/// **`u32`**                         | `INTEGER`
/// **`i64`**                         | `INTEGER`
/// **`Vec<u8>`**, alias **[`Blob`]** | `BLOB`
/// **[`String`]**                    | `TEXT`
///
/// &nbsp;
///
/// <div class="warning">
///
/// Type `u64` is **not** currently supported directly, because SQLite's `INTEGER` datatype is (at most) 64-Bit in size and is *always* signed; it can **not** store values greater than `i64::MAX`. To store `u64` keys or values, please use `value as i64` to convert the keys or value to `i64` before insertion. This can be reverted, after a key or value has been read, with `value as u64`.
///
/// </div>
#[allow(private_bounds)]
pub trait Type: FromSql + ToSql + Sealed {
    /// The SQLite storage datatype that corresponds to this type.
    const SQL_TYPE: SqlType;
}

macro_rules! impl_type {
    ($type:ty, $sql_type:ident) => {
        impl Type for $type {
            const SQL_TYPE: SqlType = SqlType::new(RusqliteType::$sql_type);
        }
    };
}

impl_type!(i32, Integer);
impl_type!(u32, Integer);
impl_type!(i64, Integer);
impl_type!(Blob, Blob);
impl_type!(String, Text);

// ---------------------------------------------------------------------------
// Character-based Types
// ---------------------------------------------------------------------------

/// Represents a character-based type, e.g., [`String`].
///
/// Keys of these types can be treated case-sensitive (default) or case-insensitive.
pub trait CharType {}

macro_rules! impl_chartype {
    ($($t:ty),*) => {
        $(
            impl CharType for $t {}
        )*
    };
}

impl_chartype!(String);

// ---------------------------------------------------------------------------
// Size Type
// ---------------------------------------------------------------------------

cfg_if! {
    if #[cfg(target_pointer_width = "64")] {
        /// The "size" type used by [`len()`](crate::SQLiteSet::len), equal to `usize` on this platform.
        pub type Size = usize;
    } else {
        /// The "size" type used by [`len()`](crate::SQLiteSet::len), equal to `u64` on this platform.
        pub type Size = u64;
    }
}

/// Provides useful helper functions for the [`Size`](crate::Size) type.
pub trait SizeExt {
    /// Create a new [`Size`] instance from an `i64` value.
    fn from_i64(value: i64) -> Self;
    /// Create a new [`Size`] instance from an `u64` value.
    fn from_u64(value: u64) -> Self;
    /// Convert this [`Size`] instance to an `u64` value.
    fn to_u64(self) -> u64;
    /// Convert this [`Size`] instance to an `usize` value, if possible.
    fn to_usize(self) -> Option<usize>;
}

impl SizeExt for Size {
    #[inline(always)]
    fn from_i64(value: i64) -> Self {
        value.try_into().unwrap_or_default()
    }

    #[inline(always)]
    fn from_u64(value: u64) -> Self {
        value as Self
    }

    #[allow(clippy::unnecessary_cast)]
    #[inline(always)]
    fn to_u64(self) -> u64 {
        self as u64
    }

    #[inline(always)]
    fn to_usize(self) -> Option<usize> {
        cfg_if! {
            if #[cfg(target_pointer_width = "64")] {
                Some(self)
            } else {
                self.try_into().ok()
            }
        }
    }
}

// ---------------------------------------------------------------------------
// SQLite Error
// ---------------------------------------------------------------------------

/// Error type that indicates that a container operation has failed.
///
/// Container operations are generally infallible. However, because our containers are backed by SQLite databases, they may still fail due to an internal SQLite error. Such errors can indicate a bug, but may also occur in out-of-memory situations.
#[allow(dead_code)]
#[derive(Debug, PartialEq)]
pub enum Error {
    /// Indicates that the operation failed because of an SQLite error; wraps the underlying [`RusqliteError`](crate::rusqlite::Error).
    SqlError(RusqliteError),
    /// An attempt was made to open an existing container (database), but the existing database schema is incompatible.
    InvalidSchema,
    /// The [integrity check](https://www.sqlite.org/pragma.html#pragma_integrity_check) for the *file-based* container (database) file has failed. Take care!
    IntegrityCheck,
    /// An attempt was made to use a feature that is **not** supported by this build of `sqlite-containers`.
    Unsupported,
}

impl Error {
    /// Returns the wrapped [`RusqliteError`](crate::rusqlite::Error), if this `Error` originates from SQLite; otherwise `None` is returned.
    pub fn into_inner(self) -> Option<RusqliteError> {
        match self {
            Error::SqlError(inner) => Some(inner),
            _ => None,
        }
    }
}

impl From<RusqliteError> for Error {
    #[inline]
    fn from(inner: RusqliteError) -> Self {
        Self::SqlError(inner)
    }
}

// ---------------------------------------------------------------------------
// IterationError
// ---------------------------------------------------------------------------

/// The error type used by the [`try_for_each`](crate::SQLiteSet::try_for_each) function.
#[derive(Debug)]
pub enum IterationError<C> {
    /// Indicates that the iteration was aborted because the callback function returned an error.
    CallbackError(C),
    /// Indicates that a container operation has failed.
    Other(Error),
}

impl<E> From<RusqliteError> for IterationError<E> {
    #[inline]
    fn from(error: RusqliteError) -> Self {
        Self::Other(error.into())
    }
}

impl From<IterationError<Error>> for Error {
    #[inline]
    fn from(value: IterationError<Error>) -> Self {
        match value {
            IterationError::CallbackError(error) => error,
            IterationError::Other(error) => error,
        }
    }
}

// ---------------------------------------------------------------------------
// TryClone Trait
// ---------------------------------------------------------------------------

/// This is the *failable* version of the standard [`Clone`](std::clone::Clone) trait.
pub trait TryClone: Sized {
    /// The error type to be returned by [`try_clone()`](Self::try_clone).
    type Error: Debug;
    /// This is the *failable* version of the standard [`clone()`](std::clone::Clone::clone) function.
    fn try_clone(&self) -> Result<Self, Self::Error>;
}

cfg_if! {
    if #[cfg(feature = "backup")] {
        use crate::rusqlite::backup::Backup;
        use std::time::Duration;

        pub(crate) fn backup_db(src: &Connection, dst: &mut Connection) -> Result<(), Error> {
            let backup = Backup::new(src, dst)?;
            Ok(backup.run_to_completion(1000i32, Duration::ZERO, None)?)
        }
    }
}

// ---------------------------------------------------------------------------
// Helper Functions
// ---------------------------------------------------------------------------

#[inline(always)]
pub(crate) fn check_constraint_violation(error: RusqliteError) -> Result<bool, Error> {
    match error {
        RusqliteError::SqliteFailure(FfiError { code: FfiErrorCode::ConstraintViolation, .. }, _) => Ok(false),
        _ => Err(error.into()),
    }
}

#[inline(always)]
fn trim_schema_str(sql: &str) -> &str {
    const PREFIX: (&str, &str) = ("CREATE TABLE", "IF NOT EXISTS");
    let sql = sql.trim_ascii();
    let sql = sql.strip_prefix(PREFIX.0).map(str::trim_ascii_start).unwrap_or(sql);
    let sql = sql.strip_prefix(PREFIX.1).map(str::trim_ascii_start).unwrap_or(sql);
    sql.strip_suffix(';').map(str::trim_ascii_end).unwrap_or(sql)
}

#[inline(always)]
pub(crate) fn verify_schema(sql_create: &str, sql_schema: &str) -> bool {
    trim_schema_str(sql_create).eq(trim_schema_str(sql_schema))
}

cfg_if! {
    if #[cfg(feature = "checked")] {
        #[inline(always)]
        fn map_check_result(check_result: Option<String>) -> Result<(), Error> {
            match check_result {
                Some(value) if value.eq_ignore_ascii_case("ok") => Ok(()),
                _ => Err(Error::IntegrityCheck),
            }
        }

        #[inline(always)]
        pub(crate) fn integrity_check(connection: &Connection) -> Result<(), Error> {
            use crate::rusqlite::OptionalExtension;
            const SQL_CHECK_DATA: &str = "PRAGMA integrity_check;";
            map_check_result(connection.query_row(SQL_CHECK_DATA, [], |row| row.get::<_, String>(0)).optional()?)
        }
    }
}

#[inline(always)]
pub(crate) fn increment(value: &mut Size) {
    *value = value.saturating_add(1)
}

// ---------------------------------------------------------------------------
// Version Getters
// ---------------------------------------------------------------------------

/// Returns the package version of the `sqlite-containers` package.
pub fn version() -> &'static Version {
    static PKG_VERSION: LazyLock<Version> = LazyLock::new(|| Version::parse(env!["CARGO_PKG_VERSION"]).expect("Failed to parse version!"));
    &PKG_VERSION
}

/// Returns the version of the underlying native SQLite library.
pub fn sqlite_version() -> &'static Version {
    static SQLITE_VERSION: LazyLock<Version> = LazyLock::new(|| {
        let version = version_number() as u64;
        Version::new(version / 1000000u64, (version % 1000000u64) / 1000u64, version % 1000u64)
    });
    &SQLITE_VERSION
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod test {
    use super::*;
    use crate::rusqlite::ffi::{Error as FfiError, ErrorCode as FfiErrorCode, SQLITE_CONSTRAINT, SQLITE_NOMEM};
    use std::path::PathBuf;

    #[cfg(target_pointer_width = "64")]
    #[test]
    fn test_size_p64() {
        assert_eq!(Size::from_i64(i64::MAX), i64::MAX as usize);
        assert_eq!(Size::from_i64(1i64), 1usize);
        assert_eq!(Size::from_i64(0i64), usize::MIN);
        assert_eq!(Size::from_i64(-1i64), usize::MIN);
        assert_eq!(Size::from_i64(i64::MIN), usize::MIN);

        assert_eq!(Size::from_u64(u64::MAX), usize::MAX);
        assert_eq!(Size::from_u64(1u64), 1usize);
        assert_eq!(Size::from_u64(u64::MIN), usize::MIN);

        assert_eq!(Size::from_i64(i64::MAX).to_u64(), i64::MAX as u64);
        assert_eq!(Size::from_i64(i64::MAX).to_usize(), Some(i64::MAX as usize));
    }

    #[cfg(not(target_pointer_width = "64"))]
    #[test]
    fn test_size_p32() {
        assert_eq!(Size::from_i64(i64::MAX), i64::MAX as u64);
        assert_eq!(Size::from_i64(1i64), 1u64);
        assert_eq!(Size::from_i64(0i64), u64::MIN);
        assert_eq!(Size::from_i64(-1i64), u64::MIN);
        assert_eq!(Size::from_i64(i64::MIN), u64::MIN);

        assert_eq!(Size::from_u64(u64::MAX), u64::MAX);
        assert_eq!(Size::from_u64(1u64), 1u64);
        assert_eq!(Size::from_u64(u64::MIN), u64::MIN);

        assert_eq!(Size::from_i64(i64::MAX).to_u64(), i64::MAX as u64);
        assert_eq!(Size::from_i64(usize::MAX as i64).to_usize(), Some(usize::MAX));
        assert!(Size::from_i64(i64::MAX).to_usize().is_none());
    }

    #[test]
    fn test_sql_type() {
        const ALL_TYPES: [RusqliteType; 3usize] = [RusqliteType::Integer, RusqliteType::Blob, RusqliteType::Text];
        for native_type in ALL_TYPES {
            let sql_type = SqlType::new(native_type);
            assert_eq!(sql_type.0, native_type);
            for other_type in ALL_TYPES {
                let other_type = SqlType::new(other_type);
                assert_eq!(sql_type.partial_cmp(&other_type).unwrap(), sql_type.cmp(&other_type));
            }
        }
    }

    #[test]
    fn test_error() {
        let error_1 = Error::from(RusqliteError::InvalidQuery);
        assert!(matches!(error_1.into_inner(), Some(RusqliteError::InvalidQuery)));

        let error_2 = Error::from(RusqliteError::InvalidPath(PathBuf::from("filename.txt")));
        assert!(matches!(error_2.into_inner(), Some(RusqliteError::InvalidPath(_))));

        let error_3 = Error::from(RusqliteError::SqliteFailure(FfiError::new(SQLITE_NOMEM), Some("Memory allocation fail!".to_owned())));
        assert!(matches!(error_3.into_inner(), Some(RusqliteError::SqliteFailure(FfiError { code: FfiErrorCode::OutOfMemory, .. }, Some(_)))));

        let error_4 = Error::InvalidSchema;
        assert!(error_4.into_inner().is_none());
    }

    #[test]
    fn test_iteration_error() {
        let error_1 = IterationError::<&str>::from(RusqliteError::InvalidQuery);
        assert!(matches!(error_1, IterationError::Other(Error::SqlError(RusqliteError::InvalidQuery))));

        let error_2 = IterationError::Other(Error::IntegrityCheck);
        let check_a: Error = error_2.into();
        assert!(matches!(check_a, Error::IntegrityCheck));

        let error_3 = IterationError::CallbackError(Error::IntegrityCheck);
        let check_b: Error = error_3.into();
        assert!(matches!(check_b, Error::IntegrityCheck));
    }

    #[test]
    fn test_check_constraint_violation() {
        let error_1 = check_constraint_violation(RusqliteError::InvalidQuery);
        assert!(matches!(error_1, Err(Error::SqlError(..))));

        let error_2 = check_constraint_violation(RusqliteError::SqliteFailure(FfiError::new(SQLITE_CONSTRAINT), None));
        assert!(matches!(error_2, Ok(false)));

        let error_3 = check_constraint_violation(RusqliteError::SqliteFailure(FfiError::new(SQLITE_NOMEM), None));
        assert!(matches!(error_3, Err(Error::SqlError(..))));
    }

    #[test]
    fn test_integrity_check_result() {
        assert_eq!(map_check_result(None), Err(Error::IntegrityCheck));
        assert_eq!(map_check_result(Some("xy".into())), Err(Error::IntegrityCheck));
        assert_eq!(map_check_result(Some("ok".into())), Ok(()));
    }

    #[test]
    fn test_get_version() {
        let (pkg_version, sqlite_version) = (version(), sqlite_version());

        eprintln!("pkg_version: {pkg_version:?}");
        eprintln!("sqlite_version: {sqlite_version:?}");

        assert_eq!(pkg_version.to_string(), env!["CARGO_PKG_VERSION"]);
        assert!(version_to_u64(sqlite_version) > 0x3_000022_000000u64);
    }

    fn version_to_u64(v: &Version) -> u64 {
        assert!(v.major <= 0x00FFFF);
        assert!(v.minor <= 0xFFFFFF);
        assert!(v.patch <= 0xFFFFFF);
        (v.major << 48) | (v.minor << 24) | v.patch
    }
}
