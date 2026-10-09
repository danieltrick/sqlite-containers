// Common Data Types
// This file is part of the SQLite-based containers for Rust project (sqlite-containers)
// SPDX-License-Identifier: Unlicense

use crate::rusqlite::{Error as RusqliteError, ErrorCode};

// ---------------------------------------------------------------------------
// Size Type
// ---------------------------------------------------------------------------

/// The "size" type used by [`len()`](crate::SQLiteSet::len), equal to `usize` on this platform.
#[cfg(target_pointer_width = "64")]
pub type SizeT = usize;
/// The "size" type used by [`len()`](crate::SQLiteSet::len), equal to `u64` on this platform.
#[cfg(not(target_pointer_width = "64"))]
pub type SizeT = u64;

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

#[cfg(test)]
mod test {
    use super::*;
    use crate::rusqlite::ffi::{Error as FfiError, ErrorCode as FfiErrorCode, SQLITE_CONSTRAINT, SQLITE_NOMEM};
    use std::path::PathBuf;

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
