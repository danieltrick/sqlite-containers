# Change Log

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/).

## [2.2.0] - 2026-10-18

### Added

- Added new optional features `backup`, `checked` and `hooks`.
- Added new trait `TryClone`, which is currently implemented by the `SQLiteMap` and `SQLiteMap` structs.
- Added new functions `SQLiteMap::to_file()` and `SQLiteSet::to_file()`.
- Added new functions `SQLiteMap::union()` and `SQLiteSet::union()`.
- Added new functions `SQLiteMap::to_set()` and `SQLiteMap::to_set_nocase()`.
- Added new parameter `can_create` to `SQLiteMap::from_file()` and `SQLiteMap::from_file_nocase()`.
- Added new parameter `can_create` to `SQLiteSet::from_file()` and `SQLiteSet::from_file_nocase()`.
- Added new parameter `readonly` to `SQLiteMap::from_file()` and `SQLiteMap::from_file_nocase()`.
- Added new parameter `readonly` to `SQLiteSet::from_file()` and `SQLiteSet::from_file_nocase()`.
- Perform an integrity check when opening a file-based database, enabled by default (can be disabled).

### Changed

- Function `SQLiteMap::update()` now returns a value indicating whether an `INSERT` or `UPDATE` was performed.
- Functions `SQLiteMap::with_file()` was renamed to `SQLiteMap::from_file()`.
- Functions `SQLiteMap::with_file_nocase()` was renamed to `SQLiteMap::from_file_nocase()`.
- Functions `SQLiteSet::with_file()` was renamed to `SQLiteSet::from_file()`.
- Functions `SQLiteSet::with_file_nocase()` was renamed to `SQLiteSet::from_file_nocase()`.

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
