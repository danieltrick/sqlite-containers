// SQLiteSet Example
// This file is part of the 'SQLite-based containers for Rust' project (sqlite-containers)
// SPDX-License-Identifier: Unlicense

#[path = "../tests/common/utilities.rs"]
mod utilities;

use crate::utilities::{hex_encode, mix64};
use sqlite_containers::SQLiteSet;

// ---------------------------------------------------------------------------
// SQLiteSet Example
// ---------------------------------------------------------------------------

fn main() {
    let mut hexstr = [0; 16usize];

    let mut sqlite_set = SQLiteSet::new().expect("Failed to create SQLiteSet!");
    let mut tx = sqlite_set.transaction().expect("Failed to init transaction!");

    println!("Inserting elements, please wait...");

    for n in 0..100_000_000u64 {
        tx.insert(hex_encode(mix64(n), &mut hexstr)).expect("Insert failed!");
    }

    tx.commit();
    println!("{} element(s) successfully inserted.", sqlite_set.len().unwrap());
}
