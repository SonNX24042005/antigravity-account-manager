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
echo -e "${BOLD}${BLUE}   🚀 Cài đặt Antigravity Relay Manager (agyr)       ${NC}"
echo -e "${BOLD}${BLUE}=====================================================${NC}"

INSTALL_DIR="$HOME/.local/bin"
mkdir -p "$INSTALL_DIR"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RELAY_DIR="$SCRIPT_DIR/antigravity-relay"

# 1. Check if building from local source
if [ -d "$RELAY_DIR" ]; then
    echo -e "${YELLOW}⚙️  Đang biên dịch từ mã nguồn dự án...${NC}"
    cd "$RELAY_DIR"
    cargo build --release -j 2
    BIN_SRC="$RELAY_DIR/target/release/antigravity-relay"
elif [ -f "$SCRIPT_DIR/target/release/antigravity-relay" ]; then
    BIN_SRC="$SCRIPT_DIR/target/release/antigravity-relay"
else
    echo -e "${RED}❌ Không tìm thấy mã nguồn hoặc file nhị phân phát hành.${NC}"
    exit 1
fi

# 2. Copy binary to ~/.local/bin
echo -e "${YELLOW}📦 Đang cài đặt file thực thi vào $INSTALL_DIR...${NC}"
cp -f "$BIN_SRC" "$INSTALL_DIR/antigravity-relay"
chmod +x "$INSTALL_DIR/antigravity-relay"

# 3. Create short alias symlink 'agyr'
ln -sf "$INSTALL_DIR/antigravity-relay" "$INSTALL_DIR/agyr"

# 4. Check PATH in shell profiles
if [[ ":$PATH:" != *":$HOME/.local/bin:"* ]]; then
    echo -e "${YELLOW}📝 Đang bổ sung ~/.local/bin vào PATH trong ~/.bashrc...${NC}"
    echo 'export PATH="$HOME/.local/bin:$PATH"' >> "$HOME/.bashrc"
fi

echo -e "\n${BOLD}${GREEN}🎉 Cài đặt thành công lệnh 'agyr' (Antigravity Relay)!${NC}\n"
echo -e "${BOLD}Các lệnh điều khiển nhanh:${NC}"
echo -e "  ${BLUE}agyr start${NC}        Khởi chạy dịch vụ chạy ngầm ngay lập tức"
echo -e "  ${BLUE}agyr autostart${NC}    Tự động chạy liên tục cùng máy tính (kể cả restart máy)"
echo -e "  ${BLUE}agyr stop${NC}         Dừng dịch vụ"
echo -e "  ${BLUE}agyr restart${NC}      Khởi động lại dịch vụ"
echo -e "  ${BLUE}agyr status${NC}       Kiểm tra trạng thái hoạt động"
echo -e "  ${BLUE}agyr disable${NC}      Tắt tự khởi động cùng máy"
echo ""
echo -e "💡 Giao diện quản lý tài khoản: ${BOLD}http://127.0.0.1:8045${NC}"
echo -e "💡 Bạn chỉ cần gõ lệnh ${BOLD}agy${NC} như bình thường, hệ thống tự nạp tài khoản tốt nhất."
