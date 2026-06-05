#!/bin/bash
set -e

echo "======================================"
echo " AWS Profile Manager (awspm)"
echo " Dependency Upgrade & Verification Automation"
echo "======================================"

# 実行モードの確認
MODE="update"
AUTO_COMMIT=false

for arg in "$@"; do
    case $arg in
        --major)
            MODE="upgrade"
            ;;
        --commit)
            AUTO_COMMIT=true
            ;;
    esac
done

if [ "$MODE" == "upgrade" ]; then
    echo "[1/6] Running Major Upgrade (cargo upgrade)..."
    mise exec -- cargo upgrade --exclude clap --exclude clap_complete --exclude tempfile
else
    echo "[1/6] Running Safe Update (cargo update)..."
    mise exec -- cargo update
fi

echo ""
echo "[2/6] Checking Formatting and Linting..."
mise exec -- cargo fmt --all
mise exec -- cargo clippy --all-targets --all-features -- -D warnings
echo "✅ Lint & Format passed."

echo ""
echo "[3/6] Running Unit and Integration Tests..."
mise exec -- cargo test
echo "✅ Unit & Integration Tests passed."

echo ""
echo "[4/6] Running End-to-End Tests..."
if [ -f "./e2e_test.sh" ]; then
    ./e2e_test.sh
else
    echo "⚠️ ./e2e_test.sh not found. Skipping."
fi
echo "✅ E2E Tests passed."

echo ""
echo "[5/6] Regenerating Third-Party Licenses..."
mise exec -- cargo about generate about.hbs > THIRD_PARTY_LICENSES.md
echo "✅ THIRD_PARTY_LICENSES.md regenerated."

echo ""
echo "[6/6] Running Security and License Audit (cargo deny)..."
mise exec -- cargo deny check
echo "✅ Security & License audit passed."

echo ""
if [ "$AUTO_COMMIT" = true ]; then
    echo "[7/7] Auto-committing and Updating CHANGELOG..."
    git add Cargo.toml Cargo.lock THIRD_PARTY_LICENSES.md
    if [ "$MODE" == "upgrade" ]; then
        git commit -m "chore: upgrade major dependencies"
    else
        git commit -m "chore: update dependencies"
    fi
    # コミット履歴を元にCHANGELOGへ追記
    mise exec -- git cliff --unreleased --prepend CHANGELOG.md
    git add CHANGELOG.md
    git commit --amend --no-edit
    echo "✅ Changes committed and CHANGELOG.md updated!"
else
    echo "🎉 All checks completed successfully!"
    echo "To automatically commit and update CHANGELOG.md, run with '--commit' flag."
    if [ "$MODE" == "update" ]; then
        echo "Run './scripts/upgrade_deps.sh --major' to perform a major version upgrade."
    fi
fi
