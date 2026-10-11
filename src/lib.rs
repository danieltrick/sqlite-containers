// SQLite Containers
// This file is part of the 'SQLite-based containers for Rust' project (sqlite-containers)
// SPDX-License-Identifier: Unlicense

//! SQLite-based containers for Rust
//! ================================
//!
//! **`sqlite-containers`** provides [*map*](https://en.wikipedia.org/wiki/Associative_array) and [*set*](https://en.wikipedia.org/wiki/Set_(abstract_data_type)) implementations that are backed by **SQLite** [in-memory](https://www.sqlite.org/inmemorydb.html) databases.
//!
//! Intended as drop-in replacements for [`HashMap`](std::collections::HashMap) and [`HashSet`](std::collections::HashSet).
//!
//! Leveraging the power of the SQLite engine, `sqlite-containers` efficiently store billions of elements &#x1F60E;
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
//! &#128073; For more details, please refer to the [`SQLiteMap`] and [`SQLiteSet`] documentation!
//!
//! Usage instructions
//! ------------------
//!
//! In order to use **`sqlite-containers`** in your Rust project, simply add it to your `Cargo.toml` file:
//!
//! ```toml
//! [dependencies]
//! sqlite-containers = "1.0.0"
//! ```
//!
//! #### Features
//!
//! * The **`bundled`** feature, which is *enabled* by default, automatically compiles and links in an up to date version of SQLite for you. This avoids many common build issues, and avoids depending on the version of SQLite on the users system.
//!
//!   Use `default-features = false`, if you want to disable this default behavior and instead use the "system" SQLite library.
//!
//! * The **`with_rowid`** feature, which is *disabled* by default, prevents `sqlite-containers` from using the [`WITHOUT ROWID`](https://www.sqlite.org/withoutrowid.html) optimization, allowing SQLite to implicitly create the special "rowid" column. By default, i.e., if the `with_rowid` feature is **not** explicitly enabled, `WITHOUT ROWID` *will* be used and thus the creation of a "rowid" column is suppressed.
//!
//!   Please note that, in most cases, using the `WITHOUT ROWID` optimization (the default) should improve the performance. This means that enabling the `with_rowid` feature is **not** normally recommended! Nonetheless, there are situations where `with_rowid` can actually be beneficial, so it is definitely worth trying for fine-tuning the performance of your application.
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

pub use crate::common::{Blob, CharType, Error, Size, SizeExt, Type};
pub use crate::sqlite_map::{SQLiteMap, SQLiteMapTransaction};
pub use crate::sqlite_set::{SQLiteSet, SQLiteSetTransaction};

pub use rusqlite;
