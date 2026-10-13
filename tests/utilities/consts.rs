// Test Constants
// This file is part of the 'SQLite-based containers for Rust' project (sqlite-containers)
// SPDX-License-Identifier: Unlicense

use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(any(debug_assertions, target_pointer_width = "16", target_pointer_width = "32"))] {
        pub const MAX_ITEMS: u64 = 125_000u64;
    } else {
        pub const MAX_ITEMS: u64 = match option_env!["GITHUB_ACTIONS"] {
            Some(value) if is_true(value) => 1_000_000u64,
            _ => 25_000_000u64,
        };

        #[inline(always)]
        const fn is_true(str: &str) -> bool {
            let bytes = str.trim_ascii().as_bytes();
            (bytes.len() == 4usize)
                && ((bytes[0] == b't') || (bytes[0] == b'T'))
                && ((bytes[1] == b'r') || (bytes[1] == b'R'))
                && ((bytes[2] == b'u') || (bytes[2] == b'U'))
                && ((bytes[3] == b'e') || (bytes[3] == b'E'))
        }
    }
}
