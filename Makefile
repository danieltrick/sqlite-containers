# This file is part of the 'SQLite-based containers for Rust' project (sqlite-containers)
# SPDX-License-Identifier: Unlicense

EXAMPLES := $(addprefix run-,$(basename $(notdir $(wildcard examples/*.rs))))

.PHONY: all bench build check clean clippy codecov doc fmt publish run test upgrade $(EXAMPLES)

all: clean clippy build test doc

check:
	cargo fmt --all --check
	cargo check
	cargo check --release --all-targets

clippy: check
	cargo clippy --no-default-features -- -D warnings
	cargo clippy --all-features --all-targets -- -D warnings

fmt:
	cargo fmt --all

build:
	cargo build
	cargo build --release

test:
	cargo test --locked
	cargo test --locked --release
	cargo test --locked --release --features with_rowid

run: $(EXAMPLES)
$(EXAMPLES):
	cargo run --locked --release --example $(patsubst run-%,%,$@)

bench:
	cargo bench --bench sqlite_containers-bench -- --quick

doc:
	cargo doc --locked

upgrade:
	cargo upgrade -i
	cargo update

codecov:
	cargo llvm-cov --locked --lcov --output-path target/codecov-output.info

publish:
	cargo publish --locked

clean:
	cargo clean
