#!/bin/bash
set -e

# Setup isolated environment
TEST_DIR=$(mktemp -d)
export HOME="$TEST_DIR"
export AWSPM_BIN="$(pwd)/target/debug/awspm"

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

echo "🎉 All E2E tests passed successfully!"
