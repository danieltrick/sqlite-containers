# Change Log

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/).

## [1.0.0] - 2026-10-09

### Added

- This is the first public release of this project.

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
