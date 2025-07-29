# List available recipes
list:
    just --list

# Run all unit and integration tests
test:
    cargo test --all-features
