#!/bin/sh
set -eu

REPO="jhughes-dev/Minecraft-Mod-Starter"
INSTALL_DIR="$HOME/.local/bin"
BINARY_NAME="mcmod"

main() {
    os="$(uname -s)"
    arch="$(uname -m)"

    case "$os" in
        Linux)  platform="linux" ;;
        Darwin) platform="macos" ;;
        *)
            echo "Error: Unsupported OS: $os" >&2
            exit 1
            ;;
    esac

    case "$arch" in
        x86_64|amd64)  arch="x86_64" ;;
        aarch64|arm64) arch="aarch64" ;;
        *)
            echo "Error: Unsupported architecture: $arch" >&2
            exit 1
            ;;
    esac

    asset="${BINARY_NAME}-${platform}-${arch}"

    echo "Detecting platform... ${platform}/${arch}"

    # Fetch latest release tag
    tag="$(curl -sSf "https://api.github.com/repos/${REPO}/releases/latest" \
        | grep '"tag_name"' \
        | sed -E 's/.*"tag_name": *"([^"]+)".*/\1/')"

    if [ -z "$tag" ]; then
        echo "Error: Could not determine latest release" >&2
        exit 1
    fi

    echo "Latest release: ${tag}"

    download_url="https://github.com/${REPO}/releases/download/${tag}/${asset}"

    echo "Downloading ${asset}..."

    mkdir -p "$INSTALL_DIR"

    curl -sSfL "$download_url" -o "${INSTALL_DIR}/${BINARY_NAME}"
    chmod +x "${INSTALL_DIR}/${BINARY_NAME}"

    echo "Installed ${BINARY_NAME} to ${INSTALL_DIR}/${BINARY_NAME}"

    # Check if INSTALL_DIR is on PATH
    case ":$PATH:" in
        *":${INSTALL_DIR}:"*)
            echo "Run 'mcmod --help' to get started."
            ;;
        *)
            echo ""
            echo "WARNING: ${INSTALL_DIR} is not on your PATH."
            echo "Add it by appending this to your shell profile (~/.bashrc, ~/.zshrc, etc.):"
            echo ""
            echo "  export PATH=\"${INSTALL_DIR}:\$PATH\""
            echo ""
            echo "Then restart your shell or run: source ~/.bashrc"
            ;;
    esac
}

main
