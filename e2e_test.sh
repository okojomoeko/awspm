#!/bin/bash
set -e

# Setup isolated environment
TEST_DIR=$(mktemp -d)
export HOME="$TEST_DIR"
if [ -z "$AWSPM_BIN" ]; then
    export AWSPM_BIN="$(pwd)/target/debug/awspm"
fi

# Define cleanup
cleanup() {
    rm -rf "$TEST_DIR"
    echo "Cleaned up: $TEST_DIR"
}
trap cleanup EXIT

echo "Starting E2E Verification in $TEST_DIR"
echo "Binary: $AWSPM_BIN"

# 1. Setup Dummy AWS Config
mkdir -p "$HOME/.aws"
cat <<EOF > "$HOME/.aws/config"
[profile default]
region = us-east-1
output = json

[profile production-db]
region = eu-central-1
source_profile = default
role_arn = arn:aws:iam::123456789012:role/ProdRole

[profile dev-api]
region = ap-northeast-1
EOF

echo "✅ Created dummy ~/.aws/config"

# 2. Test 'awspm init'
echo "Running 'awspm init'..."
$AWSPM_BIN init
if [ -f "$HOME/.awspm.yaml" ]; then
    echo "✅ 'awspm init' created .awspm.yaml"
else
    echo "❌ 'awspm init' failed to create .awspm.yaml"
    exit 1
fi

# 3. Test 'awspm sync'
echo "Running 'awspm sync'..."
$AWSPM_BIN sync --yes
# Verify profiles are in metadata (simple grep)
if grep -q "production-db" "$HOME/.awspm.yaml"; then
    echo "✅ 'awspm sync' registered 'production-db'"
else
    echo "❌ 'awspm sync' failed to register profiles"
    cat "$HOME/.awspm.yaml"
    exit 1
fi

# 4. Test 'awspm update' (Add Tag)
echo "Running 'awspm update'..."
$AWSPM_BIN update production-db --add-tag "critical" --set-note "Do not delete"
if grep -q "critical" "$HOME/.awspm.yaml" && grep -q "Do not delete" "$HOME/.awspm.yaml"; then
    echo "✅ 'awspm update' modified metadata"
else
    echo "❌ 'awspm update' failed"
    exit 1
fi


# 4.5 Test 'awspm sync --check' and 'env-vars'
echo "Running 'awspm sync --check'..."
$AWSPM_BIN sync --check
# We expect success exit code

echo "Running 'awspm current' with env vars..."
export AWS_ACCESS_KEY_ID=test
if [ "$($AWSPM_BIN current)" = "env-vars" ]; then
    echo "✅ 'awspm current' detected env-vars"
else
    echo "❌ 'awspm current' failed to detect env-vars"
    exit 1
fi
unset AWS_ACCESS_KEY_ID

echo "Running 'awspm list'..."
# Just ensure it runs without error. We can't easily parse visual table output in bash, but exit code 0 is good.
$AWSPM_BIN list > /dev/null
echo "✅ 'awspm list' ran successfully"

# 6. Test 'awspm exec' (Region Override Check)
# Set region override first
$AWSPM_BIN update dev-api --set-region "us-west-2"
echo "Running 'awspm exec' to check env vars..."
OUTPUT=$($AWSPM_BIN exec dev-api -- printenv AWS_REGION)
if [ "$OUTPUT" = "us-west-2" ]; then
    echo "✅ 'awspm exec' injected AWS_REGION=us-west-2"
else
    echo "❌ 'awspm exec' failed region override. Got: '$OUTPUT'"
    exit 1
fi

# 6.5 Test 'awspm exec' handles environment variable pollution
echo "Running 'awspm exec' with polluted environment..."
export AWS_ACCESS_KEY_ID=should_be_removed
export AWS_SECRET_ACCESS_KEY=should_be_removed
# We check if AWS_ACCESS_KEY_ID is *absent* in the child process
EXEC_ENV_OUTPUT=$($AWSPM_BIN exec dev-api -- sh -c "printenv AWS_ACCESS_KEY_ID || echo 'CLEAN'")
if [ "$EXEC_ENV_OUTPUT" = "CLEAN" ]; then
    echo "✅ 'awspm exec' successfully unset AWS_ACCESS_KEY_ID"
else
    echo "❌ 'awspm exec' leaked AWS_ACCESS_KEY_ID. Got: '$EXEC_ENV_OUTPUT'"
    exit 1
fi
unset AWS_ACCESS_KEY_ID
unset AWS_SECRET_ACCESS_KEY

# 7. Test 'awspm pin' (.envrc creation)
mkdir -p "$TEST_DIR/my-project"
cd "$TEST_DIR/my-project"
echo "Running 'awspm pin'..."
$AWSPM_BIN pin dev-api
if [ -f ".envrc" ] && grep -q "export AWS_PROFILE=dev-api" ".envrc"; then
    echo "✅ 'awspm pin' created .envrc with correct content"
else
    echo "❌ 'awspm pin' failed"
    exit 1
fi

# 8. Test 'current'
echo "Running 'awspm current'..."
# Mock AWS_PROFILE
export AWS_PROFILE=production-db
NAME_OUTPUT=$($AWSPM_BIN current)
if [ "$NAME_OUTPUT" = "production-db" ]; then
    echo "✅ 'awspm current' returned correct profile"
else
    echo "❌ 'awspm current' failed. Got: '$NAME_OUTPUT'"
    exit 1
fi
unset AWS_PROFILE

# 9. Test 'completion'
echo "Running 'awspm completion'..."
$AWSPM_BIN completion zsh > /dev/null
echo "✅ 'awspm completion zsh' generated output"

# 10. Test 'list' options (--short, query)
echo "Running 'awspm list --short'..."
$AWSPM_BIN list --short > /dev/null
echo "✅ 'awspm list --short' ran successfully"

echo "Running 'awspm list --query'..."
$AWSPM_BIN list --query "prod" > /dev/null
echo "✅ 'awspm list --query' ran successfully"

# 11. Test 'update' --add-alias
echo "Running 'awspm update --add-alias'..."
$AWSPM_BIN update production-db --add-alias "db-prod"
if grep -q "db-prod" "$HOME/.awspm.yaml"; then
    echo "✅ 'awspm update --add-alias' modified metadata"
else
    echo "❌ 'awspm update --add-alias' failed"
    exit 1
fi

# 12. Test Global Flag --verbose
echo "Running with --verbose..."
$AWSPM_BIN --verbose list > /dev/null
echo "✅ '--verbose' flag accepted"

# 13. Security Test: Sensitive Profile Protection
echo "Running Security Test: Sensitive Profile Protection..."
# Pipe input to simulate non-interactive execution.
# It SHOULD fail because non-interactive execution on sensitive profiles is forbidden by default.
if echo "n" | $AWSPM_BIN exec production-db -- echo "DANGEROUS" > /dev/null 2>&1; then
    echo "❌ 'awspm exec' executed on sensitive profile in non-interactive mode (Security Hole)"
    exit 1
else
    echo "✅ 'awspm exec' properly refused execution in non-interactive mode"
fi

# 14. Feature Test: Alias Execution (Non-Sensitive)
echo "Running 'awspm exec' via alias..."
# We use a non-sensitive profile for this test to avoid the TTY requirement.
# Let's add an alias to 'dev-api' (which is not sensitive)
$AWSPM_BIN update dev-api --add-alias "api-dev"
OUTPUT=$($AWSPM_BIN exec api-dev -- echo "ALIAS_WORKED")
if echo "$OUTPUT" | grep -q "ALIAS_WORKED"; then
    echo "✅ 'awspm exec' worked with alias 'api-dev'"
else
    echo "❌ 'awspm exec' failed with alias"
    exit 1
fi

# 15. Error Handling: Non-existent Profile
echo "Running Error Handling Test..."
if $AWSPM_BIN exec ghost-profile -- echo "Should not run" 2>/dev/null; then
    echo "❌ 'awspm exec' succeeded on non-existent profile"
    exit 1
else
    echo "✅ 'awspm exec' failed on non-existent profile as expected"
fi


# 16. Feature Test: Pin Idempotency / Overwrite
echo "Running 'awspm pin' overwrite test..."
# Create a conflict
echo "export OLD_VAR=1" > .envrc
# Run pin again (assume overwrite or pipe yes if needed)
echo "y" | $AWSPM_BIN pin dev-api > /dev/null
if grep -q "export AWS_PROFILE=dev-api" ".envrc"; then
    echo "✅ 'awspm pin' successfully updated .envrc"
else
    echo "❌ 'awspm pin' failed to update .envrc"
    exit 1
fi

# 17. Feature Test: Alias Conflict (feat/safety-conflicts)
echo "Running 'awspm update' conflict check..."
# 'dev-api' exists. Try to add 'dev-api' as an alias to 'production-db'. Should fail.
if $AWSPM_BIN update production-db --add-alias "dev-api" > /dev/null 2>&1; then
    echo "❌ 'awspm update' failed to detect alias conflict"
    exit 1
else
    echo "✅ 'awspm update' correctly rejected conflicting alias"
fi

# 18. Feature Test: Env Command (feat/env-command)
echo "Running 'awspm env' test..."
ENV_OUT=$($AWSPM_BIN env dev-api)
if echo "$ENV_OUT" | grep -q "export AWS_PROFILE=dev-api"; then
    echo "✅ 'awspm env' output correct profile export"
else
    echo "❌ 'awspm env' failed. Got: $ENV_OUT"
    exit 1
fi

# 19. Feature Test: CI Flags --preserve-env (feat/ci-flags)
echo "Running 'awspm exec --preserve-env'..."
export PRESERVE_ME="important_value"
# Without flag (default): Env should be cleared (we can't easily test *clearing* of arbitrary vars,
# but we know AWS vars are cleared. Let's rely on unit tests for clearing.
# Here we test that if we PASS the flag, it IS preserved.)
# Wait, actually `exec` logic only clears `AWS_*` vars involved in credentials.
# Arbitrary vars like PRESERVE_ME are always preserved by default in Unix shells unless explicitly unset?
# No, `Command::new` inherits env by default.
# Ah, `awspm` *manually* clears AWS vars.
# The `--preserve-env` flag in `awspm` specifically controls whether `AWS_ACCESS_KEY_ID` etc are cleared.
export AWS_ACCESS_KEY_ID="do_not_leak"
# With flag: Should be PRESERVED (leaked/kept)
RES=$($AWSPM_BIN exec dev-api --preserve-env -- sh -c "printenv AWS_ACCESS_KEY_ID")
if [ "$RES" = "do_not_leak" ]; then
    echo "✅ 'awspm exec --preserve-env' preserved AWS vars"
else
    echo "❌ 'awspm exec --preserve-env' failed. Expected 'do_not_leak', got '$RES'"
    exit 1
fi
unset AWS_ACCESS_KEY_ID
unset PRESERVE_ME

# 20. Feature Test: CI Flags --yes (feat/ci-flags)
echo "Running 'awspm exec --yes' on sensitive profile..."
# 'production-db' matches 'production' keyword (default).
# Normal exec fails (tested in step 13).
# With --yes, it should succeed.
if $AWSPM_BIN exec production-db --yes -- echo "ALLOWED" > /dev/null; then
    echo "✅ 'awspm exec --yes' bypassed confirmation"
else
    echo "❌ 'awspm exec --yes' failed to bypass confirmation"
    exit 1
fi

# 21. Feature Test: Configurable Guardrails (feat/guardrails)
echo "Running 'awspm' Configurable Guardrails test..."
# Create config with custom keyword
cat <<EOF > "$HOME/.awspm.yaml"
config:
  sensitive_keywords: ["secret-corp"]
profiles: {}
EOF
# We need to re-register profiles because we overwrote metadata file
$AWSPM_BIN sync --yes > /dev/null

# Now 'production-db' should NOT be sensitive (default 'production' keyword overridden)
if echo "n" | $AWSPM_BIN exec production-db -- echo "SAFE_NOW" > /dev/null; then
    echo "✅ Custom policy respected (default keyword ignored)"
else
    echo "❌ Custom policy failed (still treating default as sensitive)"
    exit 1
fi

# Add a TAG containing the NEW keyword to 'dev-api'
$AWSPM_BIN update dev-api --add-tag "secret-corp" > /dev/null

# Now 'dev-api' should be sensitive due to tag
if echo "n" | $AWSPM_BIN exec dev-api -- echo "SHOULD_FAIL" > /dev/null 2>&1; then
    echo "❌ Custom policy failed (new keyword not enforced via tag)"
    exit 1
else
    echo "✅ Custom policy enforced (new keyword caught via tag)"
fi

echo "🎉 All E2E tests passed successfully!"
