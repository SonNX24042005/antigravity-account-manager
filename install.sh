#!/usr/bin/env bash
set -e

# ==============================================================================
#  Antigravity Relay Installer (agyr)
# ==============================================================================

BOLD='\033[1m'
BLUE='\033[0;34m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${BOLD}${BLUE}=====================================================${NC}"
echo -e "${BOLD}${BLUE}   Cai dat Antigravity Relay Manager (agyr)          ${NC}"
echo -e "${BOLD}${BLUE}=====================================================${NC}"

INSTALL_DIR="$HOME/.local/bin"
mkdir -p "$INSTALL_DIR"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RELAY_DIR="$SCRIPT_DIR/antigravity-relay"

# 1. Check if building from local source
if [ -d "$RELAY_DIR" ]; then
    echo -e "${YELLOW}[build] Dang bien dich tu ma nguon du an...${NC}"
    cd "$RELAY_DIR"
    cargo build --release -j 2
    BIN_SRC="$RELAY_DIR/target/release/antigravity-relay"
elif [ -f "$SCRIPT_DIR/target/release/antigravity-relay" ]; then
    BIN_SRC="$SCRIPT_DIR/target/release/antigravity-relay"
else
    echo -e "${RED}[error] Khong tim thay ma nguon hoac file nhi phan phat hanh.${NC}"
    exit 1
fi

# 2. Copy binary to ~/.local/bin
echo -e "${YELLOW}[install] Dang cai dat file thuc thi vao $INSTALL_DIR...${NC}"
cp -f "$BIN_SRC" "$INSTALL_DIR/antigravity-relay"
chmod +x "$INSTALL_DIR/antigravity-relay"

# 3. Create short alias symlink 'agyr'
ln -sf "$INSTALL_DIR/antigravity-relay" "$INSTALL_DIR/agyr"

# 4. Check PATH in shell profiles
if [[ ":$PATH:" != *":$HOME/.local/bin:"* ]]; then
    echo -e "${YELLOW}[config] Dang bo sung ~/.local/bin vao PATH trong ~/.bashrc...${NC}"
    echo 'export PATH="$HOME/.local/bin:$PATH"' >> "$HOME/.bashrc"
fi

echo -e "\n${BOLD}${GREEN}Cai dat thanh cong lenh 'agyr' (Antigravity Relay)!${NC}\n"
echo -e "${BOLD}Cac lenh dieu khien nhanh:${NC}"
echo -e "  ${BLUE}agyr${NC}              Tu dong mo bang dieu khien web va chay dich vu"
echo -e "  ${BLUE}agyr start${NC}        Khoi chay dich vu chay ngam ngay lap tuc"
echo -e "  ${BLUE}agyr autostart${NC}    Tu dong chay lien tuc cung may tinh (ke ca restart may)"
echo -e "  ${BLUE}agyr stop${NC}         Dung dich vu"
echo -e "  ${BLUE}agyr restart${NC}      Khoi dong lai dich vu"
echo -e "  ${BLUE}agyr status${NC}       Kiem tra trang thai hoat dong"
echo -e "  ${BLUE}agyr disable${NC}      Tat tu khoi dong cung may"
echo ""
echo -e "Bang dieu khien: ${BOLD}http://127.0.0.1:8045${NC}"
