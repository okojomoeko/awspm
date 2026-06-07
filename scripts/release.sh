#!/usr/bin/env bash
set -e

if [ "$#" -ne 1 ]; then
    echo "Usage: $0 <version>"
    echo "Example: $0 0.1.2"
    exit 1
fi

VERSION="${1#v}" # 'v' prefixがあれば削除

echo "==> Bumping version to ${VERSION} in Cargo.toml..."
# macOSとLinuxの両方で動くようにsedを使用
sed -i.bak -e "s/^version = \".*\"/version = \"${VERSION}\"/" Cargo.toml
rm -f Cargo.toml.bak

echo "==> Updating CHANGELOG.md..."
if command -v git-cliff &> /dev/null; then
    git-cliff --tag "v${VERSION}" -o CHANGELOG.md
else
    echo "Warning: git-cliff is not installed. Skipping CHANGELOG update."
fi

echo "==> Committing and tagging v${VERSION}..."
git add Cargo.toml CHANGELOG.md
git commit -m "chore: release v${VERSION}"
git tag "v${VERSION}"

echo "==> Pushing to origin..."
git push origin main
git push origin "v${VERSION}"

echo ""
echo "================================================================"
echo "Release v${VERSION} triggered successfully!"
echo "GitHub Actions is now building the binaries and creating the release."
echo "Wait for the Actions to finish, then run:"
echo "  ./scripts/update_brew.sh ${VERSION} ~/work/homebrew-awspm"
echo "================================================================"
