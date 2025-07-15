.PHONY: help install-hooks check fmt clippy test test-coverage build clean prepare-sqlx all

# Default target
all: check test build

help:
	@echo "Available targets:"
	@echo "  make install-hooks  - Install Git pre-commit and pre-push hooks"
	@echo "  make check         - Run all checks (fmt, clippy)"
	@echo "  make fmt           - Format code with cargo fmt"
	@echo "  make clippy        - Run clippy linter"
	@echo "  make test          - Run tests"
	@echo "  make test-coverage - Run tests with coverage report (requires cargo-llvm-cov)"
	@echo "  make build         - Build release binary"
	@echo "  make clean         - Clean build artifacts"
	@echo "  make prepare-sqlx  - Update SQLx offline query cache"
	@echo "  make all           - Run check, test, and build"

install-hooks:
	@echo "Installing Git hooks..."
	@./scripts/install-hooks.sh

check: fmt clippy
	@echo "✅ All checks passed!"

fmt:
	@echo "Formatting code..."
	@cargo fmt --all
	@echo "✅ Code formatted"

clippy:
	@echo "Running clippy..."
	@cargo clippy -- -D warnings
	@echo "✅ Clippy passed"

test:
	@echo "Running tests..."
	@cargo test
	@echo "✅ Tests passed"

test-coverage:
	@echo "Running test coverage..."
	@if command -v cargo-llvm-cov >/dev/null 2>&1; then \
		cargo llvm-cov --all-features --html; \
		coverage=$$(cargo llvm-cov --all-features --summary-only 2>/dev/null | grep TOTAL | awk '{print $$10}' | sed 's/%//'); \
		echo "📊 Test coverage: $$coverage%"; \
		if [ "$${coverage%%.*}" -lt 95 ]; then \
			echo "❌ Coverage is below 95% threshold"; \
			echo "📈 Target: 95%+ coverage following OpenZeppelin Monitor standards"; \
			exit 1; \
		else \
			echo "✅ Coverage meets 95% threshold"; \
		fi; \
		echo "📂 Detailed report: target/llvm-cov/html/index.html"; \
	else \
		echo "❌ cargo-llvm-cov not installed"; \
		echo "   Install with: cargo install cargo-llvm-cov"; \
		exit 1; \
	fi

build:
	@echo "Building release binary..."
	@cargo build --release
	@echo "✅ Build complete"

clean:
	@echo "Cleaning build artifacts..."
	@cargo clean
	@echo "✅ Clean complete"

prepare-sqlx:
	@echo "Preparing SQLx offline cache..."
	@if [ -z "$$DATABASE_URL" ]; then \
		echo "❌ DATABASE_URL environment variable not set"; \
		echo "   Set it to your PostgreSQL connection string:"; \
		echo "   export DATABASE_URL=postgres://user:password@localhost/database"; \
		exit 1; \
	fi
	@cargo sqlx prepare
	@echo "✅ SQLx cache updated"