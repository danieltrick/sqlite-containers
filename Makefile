.SHELLFLAGS = -e -c

.PHONY: all bench build check clean clippy codecov doc fmt publish test upgrade

all: clean check build

build:
	cargo build --release --locked

check:
	cargo fmt --all --check
	cargo check --no-default-features
	cargo check --release --all-features --all-targets

clippy: check
	cargo clippy --no-default-features -- -D warnings
	cargo clippy --all-features --all-targets -- -D warnings

fmt:
	cargo fmt --all

upgrade:
	cargo upgrade -i
	cargo update

test:
	cargo test --locked
	cargo test --release --locked

bench:
	cargo bench --bench sqlite_containers-bench -- --quick

doc:
	cargo doc --locked

codecov:
	cargo llvm-cov --locked --lcov --output-path target/codecov-output.info

publish:
	cargo publish --locked

clean:
	cargo clean
