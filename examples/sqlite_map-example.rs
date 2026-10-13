// SQLiteMap Example
// This file is part of the 'SQLite-based containers for Rust' project (sqlite-containers)
// SPDX-License-Identifier: Unlicense

#[path = "../tests/utilities/utilities.rs"]
mod utilities;

use crate::utilities::{HexEncoder, mix64};
use sqlite_containers::SQLiteMap;

// ---------------------------------------------------------------------------
// SQLiteMap Example
// ---------------------------------------------------------------------------

fn main() {
    println!("SQLiteMap Example [{}]", env!("CARGO_PKG_VERSION"));

    let mut hex_1 = HexEncoder::default();
    let mut hex_2 = HexEncoder::default();

    let mut sqlite_set = SQLiteMap::<String, String>::new().expect("Failed to create map!");
    let mut tx = sqlite_set.transaction().expect("Failed to init transaction!");

    println!("Inserting key-value pairs, please wait...");

    for n in 0..10_000_000u64 {
        let (key, value) = (hex_1.encode(mix64(n)), hex_2.encode(mix64(u64::MAX - n)));
        tx.insert(key, value).expect("Insertion failed!");
    }

    tx.commit();
    println!("{} key-value pair(s) successfully inserted.", sqlite_set.len().unwrap());
}
