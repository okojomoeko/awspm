#!/bin/sh
set -e

# Detect OS and Architecture
OS="$(uname -s)"
ARCH="$(uname -m)"
REPO="okojomoeko/awspm"

case "$OS" in
    Linux)
        case "$ARCH" in
            x86_64) ASSET="awspm-linux-amd64" ;;
            *) echo "Unsupported Linux architecture: $ARCH"; exit 1 ;;
        esac
        ;;
    Darwin)
        case "$ARCH" in
            x86_64) ASSET="awspm-macos-amd64" ;;
            arm64) ASSET="awspm-macos-arm64" ;;
            *) echo "Unsupported macOS architecture: $ARCH"; exit 1 ;;
        esac
        ;;
    *)
        echo "Unsupported OS: $OS"
        exit 1
        ;;
esac

# Determine Version (Latest or specific)
if [ -z "$1" ]; then
    # Note: This API call will fail if the repository is private and no token is provided.
    VERSION_JSON=$(curl -fsSL "https://api.github.com/repos/$REPO/releases/latest") || {
        echo "Error: Could not fetch latest release. If the repository is private, this script requires public access or a token."
        exit 1
    }
    VERSION=$(echo "$VERSION_JSON" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/')
    if [ -z "$VERSION" ]; then
        echo "Error: Could not parse version from GitHub API response."
        exit 1
    fi
else
    VERSION="$1"
fi

URL="https://github.com/$REPO/releases/download/$VERSION/$ASSET"
DEST="/usr/local/bin/awspm"

echo "Downloading awspm $VERSION for $OS $ARCH..."
curl -fsSL -o awspm "$URL"
chmod +x awspm

echo "Installing to $DEST..."
if [ -w "/usr/local/bin" ]; then
    mv awspm "$DEST"
else
    sudo mv awspm "$DEST"
fi

echo "Successfully installed awspm $VERSION to $DEST"
