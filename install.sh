#!/usr/bin/env bash
set -e

# ==============================================================================
#  Antigravity Relay 1-Line Installer (agyr)
#  Usage: curl -fsSL https://raw.githubusercontent.com/SonNX24042005/antigravity-account-manager/main/install.sh | bash
# ==============================================================================

REPO="SonNX24042005/antigravity-account-manager"
INSTALL_DIR="$HOME/.local/bin"
mkdir -p "$INSTALL_DIR"

OS="$(uname -s | tr '[:upper:]' '[:lower:]')"
ARCH="$(uname -m)"

case "$ARCH" in
    x86_64|amd64)
        TARGET_ARCH="x86_64"
        ;;
    aarch64|arm64)
        TARGET_ARCH="aarch64"
        ;;
    *)
        TARGET_ARCH="$ARCH"
        ;;
esac

echo "====================================================="
echo "   Cai dat Antigravity Relay Manager (agyr)"
echo "====================================================="

# 1. Check if running locally inside repo
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}" 2>/dev/null)" && pwd || true)"
if [ -n "$SCRIPT_DIR" ] && [ -d "$SCRIPT_DIR/antigravity-relay" ]; then
    echo "[build] Phat hien ma nguon cuc bo, dang bien dich..."
    (cd "$SCRIPT_DIR/antigravity-relay" && cargo build --release -j 2)
    cp -f "$SCRIPT_DIR/antigravity-relay/target/release/antigravity-relay" "$INSTALL_DIR/antigravity-relay"
else
    # 2. Remote curl execution: Try downloading prebuilt binary from GitHub Release first
    DOWNLOAD_SUCCESS=false
    RELEASE_URL="https://github.com/${REPO}/releases/latest/download/antigravity-relay-${OS}-${TARGET_ARCH}.tar.gz"
    
    echo "[download] Dang tai ban phat hanh tu GitHub..."
    TMP_DIR="$(mktemp -d)"
    if curl -fsSL "$RELEASE_URL" -o "$TMP_DIR/antigravity-relay.tar.gz" 2>/dev/null; then
        tar -xzf "$TMP_DIR/antigravity-relay.tar.gz" -C "$TMP_DIR"
        if [ -f "$TMP_DIR/antigravity-relay" ]; then
            cp -f "$TMP_DIR/antigravity-relay" "$INSTALL_DIR/antigravity-relay"
            DOWNLOAD_SUCCESS=true
        fi
    fi
    rm -rf "$TMP_DIR"

    # 3. Fallback: Build from git repository if prebuilt binary is not available yet
    if [ "$DOWNLOAD_SUCCESS" != "true" ]; then
        if command -v cargo >/dev/null 2>&1 && command -v git >/dev/null 2>&1; then
            echo "[build] Khong tim thay pre-built binary, dang bien dich tu GitHub repo..."
            TMP_SRC="$(mktemp -d)"
            git clone --depth 1 "https://github.com/${REPO}.git" "$TMP_SRC/repo"
            (cd "$TMP_SRC/repo/antigravity-relay" && cargo build --release -j 2)
            cp -f "$TMP_SRC/repo/antigravity-relay/target/release/antigravity-relay" "$INSTALL_DIR/antigravity-relay"
            rm -rf "$TMP_SRC"
        else
            echo "[error] Khong the tai ban phat hanh va may chua cai dat 'cargo' / 'git'."
            echo "        Vui long cai dat Rust (https://rustup.rs) hoac clone repo de build."
            exit 1
        fi
    fi
fi

chmod +x "$INSTALL_DIR/antigravity-relay"
ln -sf "$INSTALL_DIR/antigravity-relay" "$INSTALL_DIR/agyr"

# Ensure PATH
if [[ ":$PATH:" != *":$HOME/.local/bin:"* ]]; then
    echo '[config] Dang bo sung ~/.local/bin vao PATH trong ~/.bashrc...'
    echo 'export PATH="$HOME/.local/bin:$PATH"' >> "$HOME/.bashrc"
    if [ -f "$HOME/.zshrc" ]; then
        echo 'export PATH="$HOME/.local/bin:$PATH"' >> "$HOME/.zshrc"
    fi
fi

echo ""
echo "Cai dat thanh cong lenh 'agyr'!"
echo ""
echo "Cac lenh su dung:"
echo "  agyr              Tu dong mo bang dieu khien web va chay dich vu"
echo "  agyr start        Khoi chay dich vu chay ngam"
echo "  agyr autostart    Tu dong chay cung he thong (ke ca khi restart may)"
echo "  agyr stop         Dung dich vu"
echo "  agyr restart      Khoi dong lai dich vu"
echo "  agyr status       Kiem tra trang thai hoat dong"
echo "  agyr disable      Tat tu khoi dong cung may"
echo ""
echo "Bang dieu khien: http://127.0.0.1:8045"
