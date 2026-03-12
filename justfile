# Top-level build of all crates
build:
    cargo build --workspace

# Build in release mode
build-release:
    cargo build --workspace --release

# Run all tests
test:
    cargo test --workspace

# Build only docsbookgen (used for mdBook + RustDoc)
build-docsbookgen:
    cargo build --package docsbookgen

# Build documentation for all crates (excluding private/no-docs)
docs:
    cargo doc --workspace --no-deps --all-features

# Check workspace without building artifacts
check:
    cargo check --workspace

# Clean workspace
clean:
    cargo clean

# Build mdBook structure using docsbookgen
docsbookgen:
    cargo run --package docsbookgen -- build ./docs/mdbook/

# Serve the mdBook locally
serve-docs:
    mdbook serve ./docs/mdbook/ --open

default:
    @just --list

# Run the Mayfly scheduler daemon
run-daemon:
    cargo run -p daemon --bin mayfly

alias run-agent := run-daemon

# Test only the scheduler crate
test-scheduler:
    cargo test -p scheduler

# Test only the daemon crate
test-daemon:
    cargo test -p daemon

# Run tests with nextest (faster, parallel execution)
nextest-test:
    cargo nextest run --workspace --all-targets

alias nt := nextest-test

# Install pre-commit hooks (fmt + clippy). Requires: pip install pre-commit (or brew install pre-commit)
install-hooks:
    pre-commit install

# Run pre-commit hooks on all files (without committing)
pre-commit:
    pre-commit run --all-files

# Run tests with coverage and print summary (requires: cargo install cargo-llvm-cov, Rust nightly with llvm-tools)
coverage:
    cargo llvm-cov test --workspace --no-fail-fast
    cargo llvm-cov report

# Generate HTML coverage report in ./coverage/ (run after coverage or alone)
coverage-report:
    cargo llvm-cov report --html --output-dir coverage
    @echo "Open coverage/index.html in a browser"
