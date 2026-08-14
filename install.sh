#!/usr/bin/env bash
set -euo pipefail

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
    # 2. Download a release artifact and its checksum. Fail closed if either is absent.
    RELEASE_URL="https://github.com/${REPO}/releases/latest/download/antigravity-relay-${OS}-${TARGET_ARCH}.tar.gz"
    CHECKSUM_URL="${RELEASE_URL}.sha256"
    
    echo "[download] Dang tai ban phat hanh tu GitHub..."
    TMP_DIR="$(mktemp -d)"
    ARCHIVE="$TMP_DIR/antigravity-relay.tar.gz"
    CHECKSUM_FILE="$TMP_DIR/antigravity-relay.tar.gz.sha256"
    cleanup() {
        rm -rf "$TMP_DIR"
    }
    trap cleanup EXIT

    curl --proto '=https' --tlsv1.2 -fsSL --retry 3 --max-time 120 "$RELEASE_URL" -o "$ARCHIVE"
    curl --proto '=https' --tlsv1.2 -fsSL --retry 3 --max-time 120 "$CHECKSUM_URL" -o "$CHECKSUM_FILE"

    EXPECTED_SHA256="$(awk 'NR == 1 { print $1 }' "$CHECKSUM_FILE")"
    if [[ ! "$EXPECTED_SHA256" =~ ^[[:xdigit:]]{64}$ ]]; then
        echo "[error] File checksum SHA-256 khong hop le."
        exit 1
    fi

    if command -v sha256sum >/dev/null 2>&1; then
        ACTUAL_SHA256="$(sha256sum "$ARCHIVE" | awk '{ print $1 }')"
    elif command -v shasum >/dev/null 2>&1; then
        ACTUAL_SHA256="$(shasum -a 256 "$ARCHIVE" | awk '{ print $1 }')"
    else
        echo "[error] Can sha256sum hoac shasum de xac minh ban phat hanh."
        exit 1
    fi

    if [ "${ACTUAL_SHA256,,}" != "${EXPECTED_SHA256,,}" ]; then
        echo "[error] Checksum SHA-256 khong khop. Da huy cai dat."
        exit 1
    fi

    ARCHIVE_ENTRIES="$(tar -tzf "$ARCHIVE")"
    ENTRY_COUNT="$(printf '%s\n' "$ARCHIVE_ENTRIES" | sed '/^[[:space:]]*$/d' | wc -l)"
    ONLY_ENTRY="$(printf '%s\n' "$ARCHIVE_ENTRIES" | sed '/^[[:space:]]*$/d;s#^\./##')"
    if [ "$ENTRY_COUNT" -ne 1 ] || [ "$ONLY_ENTRY" != "antigravity-relay" ]; then
        echo "[error] Goi cap nhat chua duong dan khong mong doi."
        exit 1
    fi

    tar -xzf "$ARCHIVE" -C "$TMP_DIR"
    if [ ! -f "$TMP_DIR/antigravity-relay" ] || [ -L "$TMP_DIR/antigravity-relay" ]; then
        echo "[error] Binary trong goi cap nhat khong hop le."
        exit 1
    fi
    cp -f "$TMP_DIR/antigravity-relay" "$INSTALL_DIR/antigravity-relay"
    cleanup
    trap - EXIT
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
echo "  agyr update       Cap nhat lenh agyr len phien ban moi nhat tu GitHub"
echo "  agyr start        Khoi chay dich vu chay ngam"
echo "  agyr autostart    Tu dong chay cung he thong (ke ca khi restart may)"
echo "  agyr stop         Dung dich vu"
echo "  agyr restart      Khoi dong lai dich vu"
echo "  agyr status       Kiem tra trang thai hoat dong"
echo "  agyr version      Xem phien ban hien tai"
echo "  agyr disable      Tat tu khoi dong cung may"
echo ""
echo "Bang dieu khien: http://127.0.0.1:8045"
echo ""
echo "[start] Dang khoi dong va mo bang dieu khien..."
if ! "$INSTALL_DIR/agyr"; then
    echo "[warning] Khong the tu dong mo trinh duyet. Hay chay lenh 'agyr' de thu lai."
fi
