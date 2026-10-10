// Benchmark Utilities
// This file is part of the 'SQLite-based containers for Rust' project (sqlite-containers)
// SPDX-License-Identifier: Unlicense

use hex::encode_to_slice;
use std::slice;

// ---------------------------------------------------------------------------
// Utilities
// ---------------------------------------------------------------------------

pub struct HexEncoder([u8; 16usize]);

impl HexEncoder {
    #[inline]
    pub const fn default() -> Self {
        Self([0u8; 16usize])
    }

    #[inline]
    pub fn encode(&mut self, value: u64) -> &str {
        unsafe {
            encode_to_slice(slice::from_raw_parts(&value as *const u64 as *const u8, 8usize), &mut self.0).unwrap();
            str::from_utf8_unchecked(&self.0)
        }
    }
}

#[inline]
pub fn mix64(mut z: u64) -> u64 {
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
    z ^ (z >> 31)
}
