#!/bin/bash
# Install Git hooks for the project

set -e

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
PROJECT_ROOT="$( cd "$SCRIPT_DIR/.." && pwd )"

echo "Installing Git hooks for stellar-monitor-tenant-isolation..."
echo ""

# Check if pre-commit is installed
if command -v pre-commit &> /dev/null; then
    echo "✅ pre-commit is installed"
    echo "Installing hooks using pre-commit framework..."
    
    cd "$PROJECT_ROOT"
    pre-commit install --install-hooks -t pre-commit -t pre-push -t commit-msg
    
    echo ""
    echo "✅ Pre-commit hooks installed successfully!"
    echo ""
    echo "Hooks managed by pre-commit framework:"
    echo "  - rustfmt: Format Rust code"
    echo "  - clippy: Lint Rust code"
    echo "  - cargo-test: Run tests"
    echo "  - check-json/toml/yaml: Validate file formats"
    echo "  - detect-private-key: Prevent committing secrets"
    echo "  - typos: Check for typos"
    echo "  - conventional-pre-commit: Enforce commit message format"
    echo ""
    echo "To run hooks manually:"
    echo "  pre-commit run --all-files"
    echo ""
    echo "To update hooks:"
    echo "  pre-commit autoupdate"
else
    echo "⚠️  pre-commit is not installed"
    echo "Installing basic Git hooks..."
    
    HOOKS_DIR=".git/hooks"
    mkdir -p "$PROJECT_ROOT/$HOOKS_DIR"
    
    # Create basic pre-commit hook
    cat > "$PROJECT_ROOT/$HOOKS_DIR/pre-commit" << 'EOF'
#!/bin/bash
# Basic pre-commit hook for stellar-monitor-tenant-isolation

set -e

echo "Running pre-commit checks..."

# 1. Check formatting
echo "Checking code formatting with cargo fmt..."
if ! cargo fmt --all -- --check; then
    echo "❌ Code is not formatted. Run 'cargo fmt' to fix."
    exit 1
fi
echo "✅ Code formatting check passed"

# 2. Run clippy
echo "Running clippy linter..."
if ! cargo clippy -- -D warnings; then
    echo "❌ Clippy found issues. Fix them before committing."
    exit 1
fi
echo "✅ Clippy check passed"

# 3. Run tests
echo "Running tests..."
if ! cargo test --lib; then
    echo "❌ Tests failed. Fix them before committing."
    exit 1
fi
echo "✅ Tests passed"

echo "✅ All pre-commit checks passed!"
EOF

    # Create basic pre-push hook
    cat > "$PROJECT_ROOT/$HOOKS_DIR/pre-push" << 'EOF'
#!/bin/bash
# Basic pre-push hook for stellar-monitor-tenant-isolation

set -e

echo "Running pre-push checks..."

# Run full test suite
echo "Running full test suite..."
if ! cargo test; then
    echo "❌ Tests failed. Fix them before pushing."
    exit 1
fi

# Build in release mode
echo "Building in release mode..."
if ! cargo build --release; then
    echo "❌ Release build failed. Fix it before pushing."
    exit 1
fi

echo "✅ All pre-push checks passed!"
EOF

    chmod +x "$PROJECT_ROOT/$HOOKS_DIR/pre-commit"
    chmod +x "$PROJECT_ROOT/$HOOKS_DIR/pre-push"
    
    echo ""
    echo "✅ Basic Git hooks installed!"
    echo ""
    echo "For better hook management, install pre-commit:"
    echo "  pip install pre-commit"
    echo "  ./scripts/install-hooks.sh"
fi

echo ""
echo "To skip hooks temporarily, use --no-verify flag:"
echo "  git commit --no-verify"
echo "  git push --no-verify"