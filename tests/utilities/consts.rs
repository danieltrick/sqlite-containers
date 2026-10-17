// Test Constants
// This file is part of the 'SQLite-based containers for Rust' project (sqlite-containers)
// SPDX-License-Identifier: Unlicense

use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(any(debug_assertions, target_pointer_width = "16", target_pointer_width = "32"))] {
        pub const MAX_ITEMS: u64 = 125_000u64;
    } else {
        pub const MAX_ITEMS: u64 = match option_env!("GITHUB_ACTIONS") {
            Some(value) if is_true(value) => match option_env!("RUNNER_OS") {
                Some(value) if is_macos(value) => 250_000u64,
                _ => 1_000_000u64,
            },
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

        #[inline(always)]
        const fn is_macos(str: &str) -> bool {
            let bytes = str.trim_ascii().as_bytes();
            (bytes.len() == 5usize)
                && ((bytes[0] == b'm') || (bytes[0] == b'M'))
                && ((bytes[1] == b'a') || (bytes[1] == b'A'))
                && ((bytes[2] == b'c') || (bytes[2] == b'C'))
                && ((bytes[3] == b'o') || (bytes[3] == b'O'))
                && ((bytes[4] == b's') || (bytes[4] == b'S'))
        }
    }
}

#[test]
fn test_consts() {
    cfg_if! {
        if #[cfg(any(debug_assertions, target_pointer_width = "16", target_pointer_width = "32"))] {
            assert_eq!(MAX_ITEMS, 125_000u64);
        } else {
            if std::env::var("GITHUB_ACTIONS").is_ok_and(|str| str.eq_ignore_ascii_case("true")) {
                if std::env::var("RUNNER_OS").is_ok_and(|str| str.eq_ignore_ascii_case("macos")) {
                    assert_eq!(MAX_ITEMS, 250_000u64);
                } else {
                    assert_eq!(MAX_ITEMS, 1_000_000u64);
                }
            } else {
                assert_eq!(MAX_ITEMS, 25_000_000u64);
            }
        }
    }
}
