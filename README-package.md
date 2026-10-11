SQLite-based containers for Rust
================================

**`sqlite-containers`** provides [*map*](https://en.wikipedia.org/wiki/Associative_array) and [*set*](https://en.wikipedia.org/wiki/Set_(abstract_data_type)) implementations that are backed by **SQLite** [in-memory](https://www.sqlite.org/inmemorydb.html) databases. Intended as drop-in replacements for [`HashMap`](std::collections::HashMap) and [`HashSet`](std::collections::HashSet). Leveraging the power of the SQLite engine, `sqlite-containers` efficiently store billions of elements &#x1F60E;


Quick Start
-----------

This example demonstrates how to use **`SQLiteSet`**:

```rust
use sqlite_containers::SQLiteSet;
type SQLiteStringSet = SQLiteSet<String>;

let mut sqlite_set = SQLiteStringSet::new().unwrap();
let mut tx = sqlite_set.transaction().unwrap();

println!("Inserting elements, please wait...");

for n in 0..1_000_000u64 {
    tx.insert(&format!("{:016X}", n)).unwrap();
}

tx.commit();
println!("{} elements.", sqlite_set.len().unwrap());
```


Acknowledgement
---------------

This project is based on [**SQLite**](https://www.sqlite.org/) and the [**Rusqlite**](https://crates.io/crates/rusqlite) wrapper &#128571;

SQLite’s code and documentation are dedicated to the [public domain](https://sqlite.org/copyright.html).

Rusqlite is available under the MIT license.


License
-------

“SQLite-based Containers for Rust” is published under the [Unlicense](<https://unlicense.org/>).

∎