// SQLite Containers
// This file is part of the 'SQLite-based containers for Rust' project (sqlite-containers)
// SPDX-License-Identifier: Unlicense

//! SQLite-based containers for Rust
//! ================================
//!
//! **`sqlite-containers`** provides [*hash set*](https://doc.rust-lang.org/std/collections/struct.HashSet.html) and [*hash map*](https://doc.rust-lang.org/std/collections/struct.HashMap.html) implementations that are backed by **SQLite** in-memory databases.
//!
//! Leveraging the power of the SQLite engine, these containers can efficiently store billions of elements.
//!
//! Quick Start
//! -----------
//!
//! This example demonstrates how to use **[`SQLiteSet`]**:
//!
//! ```rust
//! use sqlite_containers::SQLiteSet;
//!
//! let mut sqlite_set = SQLiteSet::new().expect("Failed to create SQLiteSet!");
//! let mut tx = sqlite_set.transaction().expect("Failed to init transaction!");
//!
//! println!("Inserting elements, please wait...");
//!
//! for n in 0..100_000_000u64 {
//!     tx.insert(&format!("{:016X}", n)).expect("Insertion failed!");
//! }
//!
//! tx.commit();
//! println!("{} element(s) successfully inserted.", sqlite_set.len().unwrap());
//! ```
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
mod sqlite_set;

pub use crate::common::{Error, SizeT};
pub use crate::sqlite_set::{SQLiteSet, SQLiteSetTransaction};

pub use rusqlite;
