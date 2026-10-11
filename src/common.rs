// Common Data Types
// This file is part of the SQLite-based containers for Rust project (sqlite-containers)
// SPDX-License-Identifier: Unlicense

use std::{fmt::Display, hash::Hash};

use crate::rusqlite::{
    Error as RusqliteError, ErrorCode,
    types::{FromSql, ToSql, Type as RusqliteType},
};

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

impl_sealed!(i32, u32, i64, String, Vec<u8>);

// ---------------------------------------------------------------------------
// SQL Type Holder
// ---------------------------------------------------------------------------

#[derive(Eq, PartialEq, Copy, Clone)]
pub struct SqlType(RusqliteType);

impl SqlType {
    #[inline]
    pub const fn new(value: RusqliteType) -> Self {
        Self(value)
    }
}

impl Hash for SqlType {
    #[inline]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        (self.0 as usize).hash(state);
    }
}

impl Display for SqlType {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0.to_string().to_ascii_uppercase())
    }
}

pub static SQL_TYPES: [SqlType; 3usize] = [SqlType::new(RusqliteType::Integer), SqlType::new(RusqliteType::Text), SqlType::new(RusqliteType::Blob)];

// ---------------------------------------------------------------------------
// Key or Value Type
// ---------------------------------------------------------------------------

/// The type of the keys or values that are stored in an [`SQLiteMap`](crate::SQLiteMap) or an [`SQLiteSet`](crate::SQLiteSet).
///
/// Currently, the following types are supported:
///
/// Type          | SQLite storage datatype
/// ------------- | -----------------------
/// **`i32`**     | `INTEGER`
/// **`u32`**     | `INTEGER`
/// **`i64`**     | `INTEGER`
/// **`Vec<u8>`** | `BLOB`
/// **`String`**  | `TEXT`
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
impl_type!(Vec<u8>, Blob);
impl_type!(String, Text);

// ---------------------------------------------------------------------------
// Character-based Types
// ---------------------------------------------------------------------------

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

/// The "size" type used by [`len()`](crate::SQLiteSet::len), equal to `usize` on this platform.
#[cfg(target_pointer_width = "64")]
pub type Size = usize;
/// The "size" type used by [`len()`](crate::SQLiteSet::len), equal to `u64` on this platform.
#[cfg(not(target_pointer_width = "64"))]
pub type Size = u64;

/// Provides useful helper functions for the [`Size`](crate::Size) type.
pub trait SizeExt {
    fn from_i64(value: i64) -> Self;
    fn from_u64(value: u64) -> Self;
    fn to_u64(self) -> u64;
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

    #[cfg(target_pointer_width = "64")]
    #[inline(always)]
    fn to_usize(self) -> Option<usize> {
        Some(self)
    }

    #[cfg(not(target_pointer_width = "64"))]
    #[inline(always)]
    fn to_usize(self) -> Option<usize> {
        self.try_into().ok()
    }
}

// ---------------------------------------------------------------------------
// SQLite Error
// ---------------------------------------------------------------------------

/// Indicates that the operation failed due to an SQLite error.
///
/// Container operations are generally infallible. However, because our containers are backed by SQLite databases, they may still fail due to an internal SQLite error. Such errors can indicate a bug, but may also occur in out-of-memory situations.
///
/// Wraps the underlying [`RusqliteError`](crate::rusqlite::Error).
#[allow(dead_code)]
#[derive(Debug)]
pub struct Error(RusqliteError);

impl Error {
    pub fn into_inner(self) -> RusqliteError {
        self.0
    }
}

impl From<RusqliteError> for Error {
    #[inline]
    fn from(error: RusqliteError) -> Self {
        Self(error)
    }
}

#[inline]
pub fn check_constraint_violation(error: RusqliteError) -> Result<bool, Error> {
    match error {
        RusqliteError::SqliteFailure(sql_error, _) => match sql_error.code {
            ErrorCode::ConstraintViolation => Ok(false),
            _ => Err(error.into()),
        },
        _ => Err(error.into()),
    }
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
    fn test_error() {
        let error_1 = Error::from(RusqliteError::InvalidQuery);
        assert_eq!(error_1.into_inner(), RusqliteError::InvalidQuery);

        let error_2 = Error::from(RusqliteError::InvalidPath(PathBuf::from("filename.txt")));
        assert!(matches!(error_2.into_inner(), RusqliteError::InvalidPath(_)));

        let error_3 = Error::from(RusqliteError::SqliteFailure(FfiError::new(SQLITE_NOMEM), Some("Memory allocation fail!".to_owned())));
        assert!(matches!(error_3.into_inner(), RusqliteError::SqliteFailure(FfiError { code: FfiErrorCode::OutOfMemory, .. }, Some(_))));
    }

    #[test]
    fn test_check_constraint_violation() {
        let error_1 = check_constraint_violation(RusqliteError::InvalidQuery);
        assert!(matches!(error_1, Err(Error(..))));

        let error_2 = check_constraint_violation(RusqliteError::SqliteFailure(FfiError::new(SQLITE_CONSTRAINT), None));
        assert!(matches!(error_2, Ok(false)));

        let error_3 = check_constraint_violation(RusqliteError::SqliteFailure(FfiError::new(SQLITE_NOMEM), None));
        assert!(matches!(error_3, Err(Error(..))));
    }
}
