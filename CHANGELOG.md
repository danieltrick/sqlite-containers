# Change Log

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/).

## [2.1.0] - 2026-10-15

### Added

- Added functions `SQLiteSet::with_file()` and `SQLiteSet::with_file_nocase()`.
- Added functions `SQLiteMap::with_file()` and `SQLiteMap::with_file_nocase()`.
- Added functions `version()` and `sqlite_version()`.

### Changed

- Function `SQLiteSet::filter()` has been replaced with `SQLiteSet::glob()`.
- Function `SQLiteMap::filter_keys()` has been replaced with `SQLiteSet::glob_keys()`.
- Function `SQLiteMap::filter_entries()` has been replaced with `SQLiteSet::glob_entries()`.
- Functions `SQLiteSet::is_empty()` and `SQLiteMap::is_empty()` have been improved.
- Integration tests have been significantly improved.

## [2.0.0] - 2026-10-13

### Changed

- Refactored `SQLiteSet` into a generic implementation that supports keys of various types.
- Refactored `SQLiteMap` into a generic implementation that supports keys and values of various types.

### Added

- Added `SQLiteSet::filter()` to filter the contained keys using a [`LIKE`](https://www.geeksforgeeks.org/sqlite/sqlite-like-operator/)-style pattern.
- Added `SQLiteMap::filter_{entries,keys}()` to filter the contained key-value pairs using a [`LIKE`](https://www.geeksforgeeks.org/sqlite/sqlite-like-operator/)-style pattern.
- Added separate `SQLiteMap::for_each_key()` and `SQLiteMap::for_each_entry()` functions.
- Added feature `with_rowid` (disabled by default), which can be used to *disable* the `WITHOUT ROWID` optimization.

### Removed

- `SQLiteSet` and `SQLiteMap` **no** longer implement the `Default` trait.

## [1.0.0] - 2026-10-09

### Added

- This is the first public release of this project.
