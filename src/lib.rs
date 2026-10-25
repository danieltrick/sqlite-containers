// SQLite Containers
// This file is part of the 'SQLite-based containers for Rust' project (sqlite-containers)
// SPDX-License-Identifier: Unlicense

#![warn(missing_docs)]
#![cfg_attr(docsrs, feature(doc_cfg))]

//! SQLite-based containers for Rust
//! ================================
//!
//! ![SQLite-based containers for Rust](https://raw.githubusercontent.com/danieltrick/sqlite-containers/refs/heads/main/etc/images/sqlite-containers.png)
//!
//! [**`sqlite-containers`**](https://crates.io/crates/sqlite-containers) provides [*map*](https://en.wikipedia.org/wiki/Associative_array) and [*set*](https://en.wikipedia.org/wiki/Set_(abstract_data_type)) implementations that are backed by **SQLite** [in-memory](https://www.sqlite.org/inmemorydb.html) databases.
//!
//! It is designed as a drop-in replacement for [`HashMap`](std::collections::HashMap) and [`HashSet`](std::collections::HashSet).
//!
//! Leveraging the power of the SQLite engine, [`SQLiteMap`] and [`SQLiteSet`] efficiently store billions of elements!
//!
//! Quick Start
//! -----------
//!
//! This example demonstrates how to use **`SQLiteSet`**:
//!
//! ```rust
//! use sqlite_containers::SQLiteSet;
//!
//! let mut sqlite_set = SQLiteSet::<String>::new().expect("Failed to create SQLiteSet!");
//! let mut tx = sqlite_set.transaction().expect("Failed to start transaction!");
//!
//! println!("Inserting elements, please wait...");
//!
//! for n in 0..1_000_000u64 {
//!     tx.insert(&format!("{:016X}", n)).expect("Insertion failed!");
//! }
//!
//! tx.commit();
//! println!("{} element(s) successfully inserted.", sqlite_set.len().unwrap());
//! ```
//!
//! &#128073; Please refer to the [`examples`](https://github.com/danieltrick/sqlite-containers/tree/main/examples) directory for details!
//!
//! Usage instructions
//! ------------------
//!
//! In order to use **`sqlite-containers`** in your Rust project, simply add it to your `Cargo.toml` file:
//!
//! ```toml
//! [dependencies]
//! sqlite-containers = "2.3.0"
//! ```
//!
//! ### Optional Features
//!
//! `sqlite-containers` provides several optional features:
//!
//! Feature          | Default  | Description
//! ---------------- | -------- | -----------
//! **`bundled`**    | &#x2705; | If this feature is enabled (the default), `sqlite-containers` automatically compiles and links in an up to date version of the SQLite library for you. This avoids many common build issues, and avoids depending on the version of SQLite on the users system. Use `default-features = false`, if you want to disable this default behavior and instead use the "system" SQLite library.
//! **`backup`**     | &#x2705; | If this feature is enabled (the default), the [`try_clone()`](TryClone::try_clone) function can be used to create a **copy** of an existing container instance, which otherwise fails with [`Error::Unsupported`]. Please note that this feature implies the `rusqlite` feature of the same name; requires SQLite 3.6.11+.
//! **`checked`**    | &#x2705; | If this feature is enabled (the default), an additional [integrity check](https://www.sqlite.org/pragma.html#pragma_integrity_check) check is performed when opening a container (database) from a file. This may slightly slow down the opening process for *file-based* containers, but disabling the feature may allow some corruption to go undetected.
//! **`with_rowid`** | &#x274C; | This feature prevents `sqlite-containers` from using the [`WITHOUT ROWID`](https://www.sqlite.org/withoutrowid.html) optimization, allowing SQLite to implicitly create the special "rowid" column. By default, i.e., if the `with_rowid` feature is **not** explicitly enabled, `WITHOUT ROWID` *will* be used and thus the creation of a "rowid" column is suppressed. Please note that, in most cases, using the `WITHOUT ROWID` optimization (the default) should improve the performance. This means that enabling the `with_rowid` feature is **not** normally recommended! Nonetheless, in some situations `with_rowid` can be beneficial.
//! **`hooks`**      | &#x274C; | If this feature is enabled, the [`update()`](SQLiteMap::update) function returns additional information that would otherwise be unavailable. Please note that this feature implies the `with_rowid` feature. Enabling this feature may slightly degrade performance, so it should *only* be used if it is actually needed.
//!
//! Sources
//! -------
//!
//! Official Git repository mirrors:
//! - <https://github.com/danieltrick/sqlite-containers.git>
//! - <https://codeberg.org/danieltrick/sqlite-containers.git>
//! - <https://gitlab.com/danieltrick/sqlite-containers.git>
//!
//! Acknowledgement
//! ---------------
//!
//! This project is based on the [**SQLite**](https://www.sqlite.org/) library and the [**Rusqlite**](https://crates.io/crates/rusqlite) wrapper for Rust &#128571;
//!
//! License
//! -------
//!
//! The “SQLite-based Containers for Rust” project is released under the [Unlicense](https://unlicense.org/).

mod common;
mod sqlite_map;
mod sqlite_set;

pub use crate::common::{Blob, CharType, CloneWrapper, EqWrapper, Error, IterationError, Size, SizeExt, TryClone, TryEq, Type, sqlite_version, version};
pub use crate::sqlite_map::{Action, SQLiteMap, SQLiteMapTransaction};
pub use crate::sqlite_set::{SQLiteSet, SQLiteSetTransaction};

pub use rusqlite;
pub use semver;
