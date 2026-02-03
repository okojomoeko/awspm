#!/bin/bash
set -e

echo "🔍 Running fmt check..."
cargo fmt --all -- --check

echo "📎 Running clippy..."
cargo clippy --all-targets --all-features -- -D warnings

echo "🧪 Running unit tests..."
cargo test

if [ -f "./e2e_test.sh" ]; then
    echo "🚢 Running End-to-End tests..."
    ./e2e_test.sh
else
    echo "⚠️ e2e_test.sh not found, skipping."
fi

echo "✅ All checks passed!"
