.SHELLFLAGS = -e -c

.PHONY: all bench build check clean clippy codecov doc fmt test upgrade

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
	CARGO_PROFILE_RELEASE_DEBUG=true RUST_BACKTRACE=1 \
	cargo test --release --tests --locked

bench:
	cargo bench --bench sqlite_set-bench -- --quick

doc:
	cargo doc --locked

codecov:
	cargo llvm-cov --html

clean:
	cargo clean
