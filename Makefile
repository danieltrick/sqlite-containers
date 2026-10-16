# This file is part of the 'SQLite-based containers for Rust' project (sqlite-containers)
# SPDX-License-Identifier: Unlicense

EXAMPLES := $(addprefix run-,$(basename $(notdir $(wildcard examples/*.rs))))

.PHONY: all audit bench build check clean clippy codecov doc fmt publish run test upgrade $(EXAMPLES)

all: clean clippy build test doc

check:
	cargo fmt --all --check
	cargo check --locked --all-targets
	cargo check --locked --all-targets --all-features

clippy: check
	cargo clippy --locked --all-targets -- -D warnings
	cargo clippy --locked --all-targets --all-features -- -D warnings

fmt:
	cargo fmt --all

build:
	cargo build --locked
	cargo build --locked --release

test:
	cargo nextest run --locked
	cargo nextest run --locked --all-features
	cargo nextest run --locked --release
	cargo nextest run --locked --release --all-features

run: $(EXAMPLES)
$(EXAMPLES):
	cargo run --locked --release --example $(patsubst run-%,%,$@)

bench:
	cargo bench --locked --bench sqlite_containers-bench -- --quick

doc:
	cargo doc --locked

upgrade:
	cargo upgrade -i
	cargo update

audit:
	cargo audit -Dwarnings

codecov:
	cargo llvm-cov --locked --lcov --output-path target/codecov-output-1.info
	cargo llvm-cov --locked --all-features --lcov --output-path target/codecov-output-2.info

publish:
	cargo publish --locked

clean:
	cargo clean
