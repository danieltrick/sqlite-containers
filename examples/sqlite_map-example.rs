// SQLiteMap Example
// This file is part of the 'SQLite-based containers for Rust' project (sqlite-containers)
// SPDX-License-Identifier: Unlicense

#[path = "../tests/utilities/mod.rs"]
mod utilities;

use crate::utilities::{hex_enc, mix64};
use sqlite_containers::SQLiteMap;

// ---------------------------------------------------------------------------
// SQLiteSet Example
// ---------------------------------------------------------------------------

fn main() {
    let mut hexstr_1 = [0; 16usize];
    let mut hexstr_2 = [0; 16usize];

    let mut sqlite_set = SQLiteMap::new().expect("Failed to create SQLiteSet!");
    let mut tx = sqlite_set.transaction().expect("Failed to init transaction!");

    println!("Inserting elements, please wait...");

    for n in 0..100_000_000u64 {
        let (key, value) = (hex_enc(mix64(n), &mut hexstr_1), hex_enc(mix64(!n), &mut hexstr_2));
        tx.insert(key, value).expect("Insertion failed!");
    }

    tx.commit();
    println!("{} element(s) successfully inserted.", sqlite_set.len().unwrap());
}
