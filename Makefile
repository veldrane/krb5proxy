# Build release binary
build:
	cargo build --release

# Install to /usr/local/bin (can be overridden by PREFIX)
PREFIX ?= /usr/local
install: build
	install -Dm755 target/release/krb5proxy $(PREFIX)/bin/krb5proxy

# Remove build artifacts
clean:
	cargo clean

# Run tests
test:
	cargo test

# Display help
help:
	@echo "Available targets:"
	@echo "  build    - Build the project in release mode"
	@echo "  install  - Install the binary to $(PREFIX)/bin (default /usr/local/bin)"
	@echo "  clean    - Remove build artifacts"
	@echo "  test     - Run tests"
	@echo "  help     - Show this help message"

.PHONY: build install clean test help