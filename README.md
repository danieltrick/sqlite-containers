[![MSRV](https://img.shields.io/badge/MSRV-1.88.0-orange.svg)](https://www.rust-lang.org/) [![License: Unlicense](https://img.shields.io/badge/license-Unlicense-blue.svg)](https://unlicense.org/)


SQLite-based containers for Rust
================================

**`sqlite-containers`** provides [*hash set*](https://doc.rust-lang.org/std/collections/struct.HashSet.html) and [*hash map*](https://doc.rust-lang.org/std/collections/struct.HashMap.html) implementations backed by in-memory **SQLite** databases.

Leveraging the power of the SQLite engine, these containers can efficiently store billions of elements.


Quick Start
-----------

This example demonstrates how to use **`SQLiteSet`**:

```rust
fn main() {
    let mut sqlite_set = SQLiteSet::new().expect("Failed to create SQLiteSet!");
    let mut tx = sqlite_set.transaction().expect("Failed to init transaction!");

    println!("Inserting elements, please wait...");

    for n in 0..100_000_000u64 {
        tx.insert(&format!("{:016X}", mix64(n))).expect("Insertion failed!");
    }

    tx.commit();
    println!("{} element(s) succesfully inserted.", sqlite_set.len().unwrap());
}
```

Acknowledgement
---------------

This project is based on the [**SQLite**](https://www.sqlite.org/) library and the [**Rusqlite**](https://crates.io/crates/rusqlite) wrapper for Rust &#128571;


All of the code and documentation in SQLite has been dedicated to the [public domain](https://sqlite.org/copyright.html) by the authors.

Rusqlite is available under the MIT license. See the [LICENSE](https://github.com/rusqlite/rusqlite/blob/master/LICENSE) file for more info.


License
-------

The “SQLite-based Containers for Rust” project is released under the [Unlicense](https://unlicense.org/).

```
This is free and unencumbered software released into the public domain.

Anyone is free to copy, modify, publish, use, compile, sell, or
distribute this software, either in source code form or as a compiled
binary, for any purpose, commercial or non-commercial, and by any
means.

In jurisdictions that recognize copyright laws, the author or authors
of this software dedicate any and all copyright interest in the
software to the public domain. We make this dedication for the benefit
of the public at large and to the detriment of our heirs and
successors. We intend this dedication to be an overt act of
relinquishment in perpetuity of all present and future rights to this
software under copyright law.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT.
IN NO EVENT SHALL THE AUTHORS BE LIABLE FOR ANY CLAIM, DAMAGES OR
OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE,
ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR
OTHER DEALINGS IN THE SOFTWARE.

For more information, please refer to <https://unlicense.org>
```

∎