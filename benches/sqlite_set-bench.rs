// SQLiteSet Benchmark
// This file is part of the 'SQLite-based containers for Rust' project (sqlite-containers)
// SPDX-License-Identifier: Unlicense

use criterion::{Criterion, criterion_group, criterion_main};
use sqlite_containers::SQLiteSet;
use std::hint::black_box;

// ---------------------------------------------------------------------------
// SQLiteSet Benchmark
// ---------------------------------------------------------------------------

const TARGET_ITEM_COUNT: usize = 100_000_000usize;

fn bench_sqlite_set(c: &mut Criterion) {
    c.bench_function("insert_directly", |b| {
        b.iter(|| {
            let mut sqlite_set = SQLiteSet::new().unwrap();
            for n in 0..TARGET_ITEM_COUNT {
                assert!(black_box(sqlite_set.insert(&format!("{:016X}", black_box(mix64(n as u64)))).unwrap()));
            }
            // assert_eq!(sqlite_set.len().unwrap(), TARGET_ITEM_COUNT);
        });
    });

    c.bench_function("insert_transact", |b| {
        b.iter(|| {
            let mut sqlite_set = SQLiteSet::new().unwrap();
            let mut tx = sqlite_set.transaction().unwrap();
            for n in 0..TARGET_ITEM_COUNT {
                assert!(black_box(tx.insert(&format!("{:016X}", black_box(mix64(n as u64)))).unwrap()));
            }
            tx.commit();
            // assert_eq!(sqlite_set.len().unwrap(), TARGET_ITEM_COUNT);
        });
    });

    c.bench_function("lookup_directly", |b| {
        let mut sqlite_set = SQLiteSet::new().unwrap();
        for n in 0..TARGET_ITEM_COUNT {
            assert!(black_box(sqlite_set.insert(&format!("{:016X}", black_box(mix64(n as u64)))).unwrap()));
        }
        assert_eq!(sqlite_set.len().unwrap(), TARGET_ITEM_COUNT);
        b.iter(|| {
            for n in 0..TARGET_ITEM_COUNT {
                assert!(black_box(sqlite_set.contains(&format!("{:016X}", black_box(mix64(n as u64)))).unwrap()));
            }
        });
    });

    c.bench_function("lookup_transact", |b| {
        let mut sqlite_set = SQLiteSet::new().unwrap();
        let mut tx = sqlite_set.transaction().unwrap();
        for n in 0..TARGET_ITEM_COUNT {
            assert!(black_box(tx.insert(&format!("{:016X}", black_box(mix64(n as u64)))).unwrap()));
        }
        tx.commit();
        assert_eq!(sqlite_set.len().unwrap(), TARGET_ITEM_COUNT);
        b.iter(|| {
            let tx = sqlite_set.transaction().unwrap();
            for n in 0..TARGET_ITEM_COUNT {
                assert!(black_box(tx.contains(&format!("{:016X}", black_box(mix64(n as u64)))).unwrap()));
            }
        });
    });
}

#[inline]
fn mix64(mut z: u64) -> u64 {
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
    z ^ (z >> 31)
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

criterion_group!(benches, bench_sqlite_set);
criterion_main!(benches);
