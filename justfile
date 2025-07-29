# List available recipes
list:
    just --list

# Run all unit and integration tests
test:
    cargo test --all-features

# Generate README.md from library documentation
readme:
    cargo readme --no-badges > README.md
