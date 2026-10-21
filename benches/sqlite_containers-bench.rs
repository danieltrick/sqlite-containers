// SQLite Containers Benchmark
// This file is part of the 'SQLite-based containers for Rust' project (sqlite-containers)
// SPDX-License-Identifier: Unlicense

#[path = "../tests/utilities/utilities.rs"]
mod utilities;

use crate::utilities::{HexEncoder, mix64};
use criterion::{Criterion, criterion_group, criterion_main};
use sqlite_containers::{SQLiteMap, SQLiteSet, Size, SizeExt};
use std::hint::black_box;

const TARGET_ITEM_COUNT: u64 = 25_000_000u64;

// ---------------------------------------------------------------------------
// SQLiteMap Benchmark
// ---------------------------------------------------------------------------

fn bench_sqlite_map(c: &mut Criterion) {
    let mut g = c.benchmark_group("sqlite_map");
    let mut hex_1 = HexEncoder::default();
    let mut hex_2 = HexEncoder::default();

    g.bench_function("insert_directly", |b| {
        b.iter(|| {
            let mut sqlite_map = SQLiteMap::<String, String>::new().unwrap();
            for n in 0..TARGET_ITEM_COUNT {
                assert!(black_box(sqlite_map.insert_ref(hex_1.encode(black_box(mix64(n))), hex_2.encode(black_box(mix64(!n)))).unwrap()));
            }
        });
    });

    g.bench_function("insert_transact", |b| {
        b.iter(|| {
            let mut sqlite_map = SQLiteMap::<String, String>::new().unwrap();
            let mut tx = sqlite_map.transaction().unwrap();
            for n in 0..TARGET_ITEM_COUNT {
                assert!(black_box(tx.insert_ref(hex_1.encode(black_box(mix64(n))), hex_2.encode(black_box(mix64(!n)))).unwrap()));
            }
            tx.commit();
        });
    });

    g.bench_function("lookup_directly", |b| {
        let mut sqlite_map = SQLiteMap::<String, String>::new().unwrap();
        let mut tx = sqlite_map.transaction().unwrap();
        for n in 0..TARGET_ITEM_COUNT {
            assert!(black_box(tx.insert_ref(hex_1.encode(black_box(mix64(n))), hex_2.encode(black_box(mix64(!n)))).unwrap()));
        }
        tx.commit();

        b.iter(|| {
            for n in 0..TARGET_ITEM_COUNT {
                assert_eq!(black_box(sqlite_map.get(hex_1.encode(black_box(mix64(n)))).unwrap().unwrap()), hex_2.encode(mix64(!n)));
            }
        });
    });

    g.bench_function("lookup_transact", |b| {
        let mut sqlite_map = SQLiteMap::<String, String>::new().unwrap();
        let mut tx = sqlite_map.transaction().unwrap();
        for n in 0..TARGET_ITEM_COUNT {
            assert!(black_box(tx.insert_ref(hex_1.encode(black_box(mix64(n))), hex_2.encode(black_box(mix64(!n)))).unwrap()));
        }
        tx.commit();

        b.iter(|| {
            let tx = sqlite_map.transaction().unwrap();
            for n in 0..TARGET_ITEM_COUNT {
                assert_eq!(black_box(tx.get(hex_1.encode(black_box(mix64(n)))).unwrap().unwrap()), hex_2.encode(mix64(!n)));
            }
        });
    });

    g.bench_function("length_directly", |b| {
        let mut sqlite_map = SQLiteMap::<String, String>::new().unwrap();
        let mut tx = sqlite_map.transaction().unwrap();
        for n in 0..TARGET_ITEM_COUNT {
            assert!(black_box(tx.insert_ref(hex_1.encode(black_box(mix64(n))), hex_2.encode(black_box(mix64(!n)))).unwrap()));
        }
        tx.commit();

        b.iter(|| {
            assert_eq!(black_box(sqlite_map.len().unwrap()), Size::from_u64(TARGET_ITEM_COUNT));
        });
    });

    g.bench_function("length_transact", |b| {
        let mut sqlite_map = SQLiteMap::<String, String>::new().unwrap();
        let mut tx = sqlite_map.transaction().unwrap();
        for n in 0..TARGET_ITEM_COUNT {
            assert!(black_box(tx.insert_ref(hex_1.encode(black_box(mix64(n))), hex_2.encode(black_box(mix64(!n)))).unwrap()));
        }

        b.iter(|| {
            assert_eq!(black_box(tx.len().unwrap()), Size::from_u64(TARGET_ITEM_COUNT));
        });
    });
}

// ---------------------------------------------------------------------------
// SQLiteSet Benchmark
// ---------------------------------------------------------------------------

fn bench_sqlite_set(c: &mut Criterion) {
    let mut g = c.benchmark_group("sqlite_set");
    let mut hex = HexEncoder::default();

    g.bench_function("insert_directly", |b| {
        b.iter(|| {
            let mut sqlite_set = SQLiteSet::<String>::new().unwrap();
            for n in 0..TARGET_ITEM_COUNT {
                assert!(black_box(sqlite_set.insert_ref(hex.encode(black_box(mix64(n)))).unwrap()));
            }
        });
    });

    g.bench_function("insert_transact", |b| {
        b.iter(|| {
            let mut sqlite_set = SQLiteSet::<String>::new().unwrap();
            let mut tx = sqlite_set.transaction().unwrap();
            for n in 0..TARGET_ITEM_COUNT {
                assert!(black_box(tx.insert_ref(hex.encode(black_box(mix64(n)))).unwrap()));
            }
            tx.commit();
        });
    });

    g.bench_function("lookup_directly", |b| {
        let mut sqlite_set = SQLiteSet::<String>::new().unwrap();
        let mut tx = sqlite_set.transaction().unwrap();
        for n in 0..TARGET_ITEM_COUNT {
            assert!(black_box(tx.insert_ref(hex.encode(black_box(mix64(n)))).unwrap()));
        }
        tx.commit();

        b.iter(|| {
            for n in 0..TARGET_ITEM_COUNT {
                assert!(black_box(sqlite_set.contains(hex.encode(black_box(mix64(n)))).unwrap()));
            }
        });
    });

    g.bench_function("lookup_transact", |b| {
        let mut sqlite_set = SQLiteSet::<String>::new().unwrap();
        let mut tx = sqlite_set.transaction().unwrap();
        for n in 0..TARGET_ITEM_COUNT {
            assert!(black_box(tx.insert_ref(hex.encode(black_box(mix64(n)))).unwrap()));
        }
        tx.commit();

        b.iter(|| {
            let tx = sqlite_set.transaction().unwrap();
            for n in 0..TARGET_ITEM_COUNT {
                assert!(black_box(tx.contains(hex.encode(black_box(mix64(n)))).unwrap()));
            }
        });
    });

    g.bench_function("length_directly", |b| {
        let mut sqlite_set = SQLiteSet::<String>::new().unwrap();
        let mut tx = sqlite_set.transaction().unwrap();
        for n in 0..TARGET_ITEM_COUNT {
            assert!(black_box(tx.insert_ref(hex.encode(black_box(mix64(n)))).unwrap()));
        }
        tx.commit();

        b.iter(|| {
            assert_eq!(black_box(sqlite_set.len().unwrap()), Size::from_u64(TARGET_ITEM_COUNT));
        });
    });

    g.bench_function("length_transact", |b| {
        let mut sqlite_set = SQLiteSet::<String>::new().unwrap();
        let mut tx = sqlite_set.transaction().unwrap();
        for n in 0..TARGET_ITEM_COUNT {
            assert!(black_box(tx.insert_ref(hex.encode(black_box(mix64(n)))).unwrap()));
        }

        b.iter(|| {
            assert_eq!(black_box(tx.len().unwrap()), Size::from_u64(TARGET_ITEM_COUNT));
        });
    });
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

criterion_group!(benches, bench_sqlite_map, bench_sqlite_set);
criterion_main!(benches);
