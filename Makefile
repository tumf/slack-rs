# Makefile for slack-rs

.PHONY: build help install release test clean fmt lint check setup pre-commit-hooks bump-patch bump-minor bump-major index

# Default target - build debug version
build:
	@echo "Building debug version..."
	cargo build

# Help message
help:
	@echo "Available targets:"
	@echo "  make (default)         - Build debug version"
	@echo "  make build             - Build debug version"
	@echo "  make install           - Install the binary to ~/.cargo/bin"
	@echo "  make release           - Build optimized release version"
	@echo "  make test              - Run all tests"
	@echo "  make clean             - Clean build artifacts"
	@echo "  make fmt               - Format code with rustfmt"
	@echo "  make lint              - Run clippy linter"
	@echo "  make check             - Run fmt, lint, and test"
	@echo "  make index             - Build Serena symbol index (.serena/cache)"
	@echo "  make setup             - Setup development environment"
	@echo "  make pre-commit-hooks  - Install git pre-commit hooks"
	@echo "  make bump-patch        - Bump patch version (0.1.0 -> 0.1.1)"
	@echo "  make bump-minor        - Bump minor version (0.1.0 -> 0.2.0)"
	@echo "  make bump-major        - Bump major version (0.1.0 -> 1.0.0)"

# Install binary to ~/.cargo/bin
install:
	@echo "Installing slack-rs..."
	cargo install --path .
	@echo "Installation complete. Binary installed to ~/.cargo/bin/slack-rs"

# Build release version
release:
	@echo "Building release version..."
	cargo build --release
	@echo "Release binary: target/release/slack-rs"

# Run tests
test:
	@echo "Running tests..."
	cargo test --verbose

# Clean build artifacts
clean:
	@echo "Cleaning build artifacts..."
	cargo clean

# Format code
fmt:
	@echo "Formatting code..."
	cargo fmt

# Run linter
lint:
	@echo "Running clippy..."
	cargo clippy -- -D warnings

# Run all checks (format, lint, test)
check: fmt lint test
	@echo "All checks passed!"

# Build Serena symbol index under .serena/cache
index:
	@echo "Indexing project for Serena..."
	@command -v serena >/dev/null 2>&1 || (echo "serena CLI not found. Install it first (e.g. 'uv tool install serena')." && exit 1)
	serena project index . --log-level INFO
	@echo "Serena index complete."

# Setup development environment
setup: pre-commit-hooks
	@echo "Setting up development environment..."
	@command -v rustfmt >/dev/null 2>&1 || rustup component add rustfmt
	@command -v clippy >/dev/null 2>&1 || rustup component add clippy
	@command -v cargo-bump >/dev/null 2>&1 || cargo install cargo-bump
	@echo "Development environment setup complete!"

# Install pre-commit hooks
pre-commit-hooks:
	@echo "Installing pre-commit hooks..."
	@mkdir -p .git/hooks
	@printf '%s\n' \
		'#!/bin/bash' \
		'set -e' \
		'' \
		'echo "Running pre-commit checks..."' \
		'' \
		'# Check formatting' \
		'echo "Checking code formatting..."' \
		'if ! cargo fmt -- --check; then' \
		'    echo "❌ Code formatting check failed. Run '\''cargo fmt'\'' to fix."' \
		'    exit 1' \
		'fi' \
		'' \
		'# Run clippy' \
		'echo "Running clippy..."' \
		'if ! cargo clippy -- -D warnings; then' \
		'    echo "❌ Clippy check failed. Fix the warnings above."' \
		'    exit 1' \
		'fi' \
		'' \
		'# Run tests' \
		'echo "Running tests..."' \
		'if ! cargo test --quiet; then' \
		'    echo "❌ Tests failed. Fix the failing tests."' \
		'    exit 1' \
		'fi' \
		'' \
		'echo "✅ All pre-commit checks passed!"' \
		> .git/hooks/pre-commit
	@chmod +x .git/hooks/pre-commit
	@echo "Pre-commit hooks installed successfully!"

# Bump patch version (0.1.0 -> 0.1.1) and create git tag
bump-patch:
	@echo "Bumping patch version..."
	@cargo bump patch -g
	@echo "Patch version bumped and tagged successfully"

# Bump minor version (0.1.0 -> 0.2.0) and create git tag
bump-minor:
	@echo "Bumping minor version..."
	@cargo bump minor -g
	@echo "Minor version bumped and tagged successfully"

# Bump major version (0.1.0 -> 1.0.0) and create git tag
bump-major:
	@echo "Bumping major version..."
	@cargo bump major -g
	@echo "Major version bumped and tagged successfully"
