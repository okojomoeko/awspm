#!/usr/bin/env bash
set -e

if [ "$#" -ne 2 ]; then
    echo "Usage: $0 <version> <path_to_homebrew_tap>"
    echo "Example: $0 0.1.2 ~/work/homebrew-awspm"
    exit 1
fi

VERSION="${1#v}" # 'v' prefixがあれば削除 (例: v0.1.2 -> 0.1.2)
TAP_DIR="$2"
FORMULA_FILE="$TAP_DIR/Formula/awspm.rb"

echo "==> Verifying tap directory..."
if [ ! -d "$TAP_DIR" ]; then
    echo "Error: Directory '$TAP_DIR' does not exist."
    exit 1
fi

if [ ! -f "$FORMULA_FILE" ]; then
    echo "Error: Formula file '$FORMULA_FILE' not found."
    echo "Are you sure this is the correct homebrew-awspm tap repository?"
    exit 1
fi

REMOTE_URL=$(git -C "$TAP_DIR" remote get-url origin 2>/dev/null || echo "")
if [[ "$REMOTE_URL" != *"homebrew-awspm"* ]]; then
    echo "Error: The origin remote '$REMOTE_URL' does not seem to point to the correct public repository (homebrew-awspm)."
    exit 1
fi

echo "==> Fetching release assets for v${VERSION} from GitHub..."

# GitHub CLIを用いてリリースされた資産のSHA256 URLを取得し、curlで中身を読み取る
MAC_INTEL_SHA=$(gh release view "v${VERSION}" --json assets --jq '.assets[] | select(.name == "awspm-macos-amd64.tar.gz.sha256") | .url' | xargs curl -sL | awk '{print $1}')
MAC_ARM_SHA=$(gh release view "v${VERSION}" --json assets --jq '.assets[] | select(.name == "awspm-macos-arm64.tar.gz.sha256") | .url' | xargs curl -sL | awk '{print $1}')
LINUX_INTEL_SHA=$(gh release view "v${VERSION}" --json assets --jq '.assets[] | select(.name == "awspm-linux-amd64.tar.gz.sha256") | .url' | xargs curl -sL | awk '{print $1}')

if [ -z "$MAC_INTEL_SHA" ] || [ -z "$MAC_ARM_SHA" ] || [ -z "$LINUX_INTEL_SHA" ]; then
    echo "Error: Failed to fetch some SHA256 checksums."
    echo "Make sure the release v${VERSION} exists on GitHub and the assets are uploaded."
    exit 1
fi

echo "==> Updating Formula with new version and checksums..."

# Rubyを使って確実・安全に置換を行う
ruby -e "
  content = File.read('$FORMULA_FILE')
  
  # versionを更新
  content.gsub!(/version \".*?\"/, %Q{version \"$VERSION\"})
  
  # 各アーキテクチャごとのsha256を更新 (正規表現で対応するOS/CPUブロック内のsha256を置換)
  content.sub!(/(if OS\.mac\? && Hardware::CPU\.intel\?.*?sha256 \")[^\"]+(\")/m) { \$1 + '$MAC_INTEL_SHA' + \$2 }
  content.sub!(/(elsif OS\.mac\? && Hardware::CPU\.arm\?.*?sha256 \")[^\"]+(\")/m) { \$1 + '$MAC_ARM_SHA' + \$2 }
  content.sub!(/(elsif OS\.linux\? && Hardware::CPU\.intel\?.*?sha256 \")[^\"]+(\")/m) { \$1 + '$LINUX_INTEL_SHA' + \$2 }
  
  File.write('$FORMULA_FILE', content)
"

echo "==> Formula updated successfully."
echo ""
echo "Please review the changes below:"
echo "--------------------------------------------------------"
git -C "$TAP_DIR" diff
echo "--------------------------------------------------------"
echo ""
echo "If the changes look correct, you can commit and push them:"
echo "  cd $TAP_DIR"
echo "  git commit -am \"bump awspm to v${VERSION}\""
echo "  git push"
