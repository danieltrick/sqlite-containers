// SQLiteSet Example
// This file is part of the 'SQLite-based containers for Rust' project (sqlite-containers)
// SPDX-License-Identifier: Unlicense

#[path = "../tests/utilities/mod.rs"]
mod utilities;

use crate::utilities::{HexEncoder, mix64};
use sqlite_containers::SQLiteSet;

// ---------------------------------------------------------------------------
// SQLiteSet Example
// ---------------------------------------------------------------------------

fn main() {
    println!("SQLiteSet Example [{}]", env!("CARGO_PKG_VERSION"));

    let mut hex = HexEncoder::default();

    let mut sqlite_set = SQLiteSet::new().expect("Failed to create SQLiteSet!");
    let mut tx = sqlite_set.transaction().expect("Failed to init transaction!");

    println!("Inserting elements, please wait...");

    for n in 0..100_000_000u64 {
        tx.insert(hex.encode(mix64(n))).expect("Insertion failed!");
    }

    tx.commit();
    println!("{} element(s) successfully inserted.", sqlite_set.len().unwrap());
}
