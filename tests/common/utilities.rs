// Benchmark Utilities
// This file is part of the 'SQLite-based containers for Rust' project (sqlite-containers)
// SPDX-License-Identifier: Unlicense

use hex::encode_to_slice;
use std::slice;

// ---------------------------------------------------------------------------
// Utilities
// ---------------------------------------------------------------------------

#[inline]
pub fn mix64(mut z: u64) -> u64 {
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
    z ^ (z >> 31)
}

#[inline]
pub fn hex_encode(value: u64, buffer: &mut [u8; 16usize]) -> &str {
    unsafe {
        encode_to_slice(slice::from_raw_parts(&value as *const u64 as *const u8, 8usize), buffer).unwrap();
        str::from_utf8_unchecked(buffer)
    }
}
