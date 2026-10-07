// SQLiteSet Example
// This file is part of the 'SQLite-based containers for Rust' project (sqlite-containers)
// SPDX-License-Identifier: Unlicense

use sqlite_containers::SQLiteSet;

fn main() {
    let mut sqlite_set = SQLiteSet::new().expect("Failed to create SQLiteSet!");
    let mut tx = sqlite_set.transaction().expect("Failed to init transaction!");

    println!("Inserting elements, please wait...");

    for n in 0..100_000_000u64 {
        tx.insert(&format!("{:016X}", mix64(n))).expect("Insertion failed!");
    }

    tx.commit();
    println!("{} element(s) successfully inserted.", sqlite_set.len().unwrap());
}

#[inline]
fn mix64(mut z: u64) -> u64 {
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
    z ^ (z >> 31)
}
