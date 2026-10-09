// SQLite Containers Benchmark
// This file is part of the 'SQLite-based containers for Rust' project (sqlite-containers)
// SPDX-License-Identifier: Unlicense

#[path = "../tests/utilities/mod.rs"]
mod utilities;

use crate::utilities::{hex_enc, mix64};
use criterion::{Criterion, criterion_group, criterion_main};
use sqlite_containers::{SQLiteMap, SQLiteSet, SizeT};
use std::hint::black_box;

const TARGET_ITEM_COUNT: u64 = 25_000_000u64;

// ---------------------------------------------------------------------------
// SQLiteMap Benchmark
// ---------------------------------------------------------------------------

fn bench_sqlite_map(c: &mut Criterion) {
    let mut hexstr_1 = [0; 16usize];
    let mut hexstr_2 = [0; 16usize];

    c.bench_function("insert_directly", |b| {
        b.iter(|| {
            let mut sqlite_map = SQLiteMap::new().unwrap();
            for n in 0..TARGET_ITEM_COUNT {
                assert!(black_box(sqlite_map.insert(hex_enc(black_box(mix64(n)), &mut hexstr_1), hex_enc(black_box(mix64(!n)), &mut hexstr_2)).unwrap()));
            }
        });
    });

    c.bench_function("insert_transact", |b| {
        b.iter(|| {
            let mut sqlite_map = SQLiteMap::new().unwrap();
            let mut tx = sqlite_map.transaction().unwrap();
            for n in 0..TARGET_ITEM_COUNT {
                assert!(black_box(tx.insert(hex_enc(black_box(mix64(n)), &mut hexstr_1), hex_enc(black_box(mix64(!n)), &mut hexstr_2)).unwrap()));
            }
            tx.commit();
        });
    });

    c.bench_function("lookup_directly", |b| {
        let mut sqlite_map = SQLiteMap::new().unwrap();
        for n in 0..TARGET_ITEM_COUNT {
            assert!(black_box(sqlite_map.insert(hex_enc(black_box(mix64(n)), &mut hexstr_1), hex_enc(black_box(mix64(!n)), &mut hexstr_2)).unwrap()));
        }

        b.iter(|| {
            for n in 0..TARGET_ITEM_COUNT {
                assert!(black_box(sqlite_map.contains(hex_enc(black_box(mix64(n)), &mut hexstr_1)).unwrap()));
            }
        });
    });

    c.bench_function("lookup_transact", |b| {
        let mut sqlite_map = SQLiteMap::new().unwrap();
        let mut tx = sqlite_map.transaction().unwrap();
        for n in 0..TARGET_ITEM_COUNT {
            assert!(black_box(tx.insert(hex_enc(black_box(mix64(n)), &mut hexstr_1), hex_enc(black_box(mix64(!n)), &mut hexstr_2)).unwrap()));
        }
        tx.commit();

        b.iter(|| {
            let tx = sqlite_map.transaction().unwrap();
            for n in 0..TARGET_ITEM_COUNT {
                assert!(black_box(tx.contains(hex_enc(black_box(mix64(n)), &mut hexstr_1)).unwrap()));
            }
        });
    });

    c.bench_function("length_directly", |b| {
        let mut sqlite_map = SQLiteMap::new().unwrap();
        let mut tx = sqlite_map.transaction().unwrap();
        for n in 0..TARGET_ITEM_COUNT {
            assert!(black_box(tx.insert(hex_enc(black_box(mix64(n)), &mut hexstr_1), hex_enc(black_box(mix64(!n)), &mut hexstr_2)).unwrap()));
        }
        tx.commit();

        b.iter(|| {
            assert_eq!(black_box(sqlite_map.len().unwrap()), TARGET_ITEM_COUNT as SizeT);
        });
    });

    c.bench_function("length_transact", |b| {
        let mut sqlite_map = SQLiteMap::new().unwrap();
        let mut tx = sqlite_map.transaction().unwrap();
        for n in 0..TARGET_ITEM_COUNT {
            assert!(black_box(tx.insert(hex_enc(black_box(mix64(n)), &mut hexstr_1), hex_enc(black_box(mix64(!n)), &mut hexstr_2)).unwrap()));
        }

        b.iter(|| {
            assert_eq!(black_box(tx.len().unwrap()), TARGET_ITEM_COUNT as SizeT);
        });
    });
}

// ---------------------------------------------------------------------------
// SQLiteSet Benchmark
// ---------------------------------------------------------------------------

fn bench_sqlite_set(c: &mut Criterion) {
    let mut hexstr = [0; 16usize];

    c.bench_function("insert_directly", |b| {
        b.iter(|| {
            let mut sqlite_set = SQLiteSet::new().unwrap();
            for n in 0..TARGET_ITEM_COUNT {
                assert!(black_box(sqlite_set.insert(hex_enc(black_box(mix64(n)), &mut hexstr)).unwrap()));
            }
        });
    });

    c.bench_function("insert_transact", |b| {
        b.iter(|| {
            let mut sqlite_set = SQLiteSet::new().unwrap();
            let mut tx = sqlite_set.transaction().unwrap();
            for n in 0..TARGET_ITEM_COUNT {
                assert!(black_box(tx.insert(hex_enc(black_box(mix64(n)), &mut hexstr)).unwrap()));
            }
            tx.commit();
        });
    });

    c.bench_function("lookup_directly", |b| {
        let mut sqlite_set = SQLiteSet::new().unwrap();
        for n in 0..TARGET_ITEM_COUNT {
            assert!(black_box(sqlite_set.insert(hex_enc(black_box(mix64(n)), &mut hexstr)).unwrap()));
        }

        b.iter(|| {
            for n in 0..TARGET_ITEM_COUNT {
                assert!(black_box(sqlite_set.contains(hex_enc(black_box(mix64(n)), &mut hexstr)).unwrap()));
            }
        });
    });

    c.bench_function("lookup_transact", |b| {
        let mut sqlite_set = SQLiteSet::new().unwrap();
        let mut tx = sqlite_set.transaction().unwrap();
        for n in 0..TARGET_ITEM_COUNT {
            assert!(black_box(tx.insert(hex_enc(black_box(mix64(n)), &mut hexstr)).unwrap()));
        }
        tx.commit();

        b.iter(|| {
            let tx = sqlite_set.transaction().unwrap();
            for n in 0..TARGET_ITEM_COUNT {
                assert!(black_box(tx.contains(hex_enc(black_box(mix64(n)), &mut hexstr)).unwrap()));
            }
        });
    });

    c.bench_function("length_directly", |b| {
        let mut sqlite_set = SQLiteSet::new().unwrap();
        let mut tx = sqlite_set.transaction().unwrap();
        for n in 0..TARGET_ITEM_COUNT {
            assert!(black_box(tx.insert(hex_enc(black_box(mix64(n)), &mut hexstr)).unwrap()));
        }
        tx.commit();

        b.iter(|| {
            assert_eq!(black_box(sqlite_set.len().unwrap()), TARGET_ITEM_COUNT as SizeT);
        });
    });

    c.bench_function("length_transact", |b| {
        let mut sqlite_set = SQLiteSet::new().unwrap();
        let mut tx = sqlite_set.transaction().unwrap();
        for n in 0..TARGET_ITEM_COUNT {
            assert!(black_box(tx.insert(hex_enc(black_box(mix64(n)), &mut hexstr)).unwrap()));
        }

        b.iter(|| {
            assert_eq!(black_box(tx.len().unwrap()), TARGET_ITEM_COUNT as SizeT);
        });
    });
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

criterion_group!(benches, bench_sqlite_map, bench_sqlite_set);
criterion_main!(benches);
