// SQLiteSet Benchmark
// This file is part of the 'SQLite-based containers for Rust' project (sqlite-containers)
// SPDX-License-Identifier: Unlicense

#[path = "../tests/utilities/mod.rs"]
mod utilities;

use crate::utilities::{hex_encode, mix64};
use criterion::{Criterion, criterion_group, criterion_main};
use sqlite_containers::SQLiteSet;
use std::hint::black_box;

// ---------------------------------------------------------------------------
// SQLiteSet Benchmark
// ---------------------------------------------------------------------------

const TARGET_ITEM_COUNT: u64 = 25_000_000u64;

fn bench_sqlite_set(c: &mut Criterion) {
    let mut hexstr = [0; 16usize];

    c.bench_function("insert_directly", |b| {
        b.iter(|| {
            let mut sqlite_set = SQLiteSet::new().unwrap();
            for n in 0..TARGET_ITEM_COUNT {
                assert!(black_box(sqlite_set.insert(hex_encode(black_box(mix64(n)), &mut hexstr)).unwrap()));
            }
        });
    });

    c.bench_function("insert_transact", |b| {
        b.iter(|| {
            let mut sqlite_set = SQLiteSet::new().unwrap();
            let mut tx = sqlite_set.transaction().unwrap();
            for n in 0..TARGET_ITEM_COUNT {
                assert!(black_box(tx.insert(hex_encode(black_box(mix64(n)), &mut hexstr)).unwrap()));
            }
            tx.commit();
        });
    });

    c.bench_function("lookup_directly", |b| {
        let mut sqlite_set = SQLiteSet::new().unwrap();
        for n in 0..TARGET_ITEM_COUNT {
            assert!(black_box(sqlite_set.insert(hex_encode(black_box(mix64(n)), &mut hexstr)).unwrap()));
        }

        b.iter(|| {
            for n in 0..TARGET_ITEM_COUNT {
                assert!(black_box(sqlite_set.contains(hex_encode(black_box(mix64(n)), &mut hexstr)).unwrap()));
            }
        });
    });

    c.bench_function("lookup_transact", |b| {
        let mut sqlite_set = SQLiteSet::new().unwrap();
        let mut tx = sqlite_set.transaction().unwrap();
        for n in 0..TARGET_ITEM_COUNT {
            assert!(black_box(tx.insert(hex_encode(black_box(mix64(n)), &mut hexstr)).unwrap()));
        }
        tx.commit();

        b.iter(|| {
            let tx = sqlite_set.transaction().unwrap();
            for n in 0..TARGET_ITEM_COUNT {
                assert!(black_box(tx.contains(hex_encode(black_box(mix64(n)), &mut hexstr)).unwrap()));
            }
        });
    });
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

criterion_group!(benches, bench_sqlite_set);
criterion_main!(benches);
