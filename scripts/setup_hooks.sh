#!/bin/sh
# scripts/setup_hooks.sh
cp scripts/git-hooks/pre-commit .git/hooks/pre-commit
chmod +x .git/hooks/pre-commit
echo "Git hooks installed successfully."
