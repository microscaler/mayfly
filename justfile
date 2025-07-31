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
    @echo "Available commands: build, test, run-agent, run-cli"

run-agent:
    cargo run -p daemon --bin tinkerbell

# Execute the CLI to check daemon status
run-cli:
    cargo run -p cli --bin tctl -- status

# Run nextest for faster test execution
nextest-test:
    cargo nextest run --workspace --all-targets --fail-fast --retries 1

alias nt := nextest-test
