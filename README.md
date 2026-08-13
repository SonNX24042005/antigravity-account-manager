# Antigravity Account Manager

Hệ thống quản trị đa tài khoản Google và bộ chuyển đổi tài khoản siêu tốc (1-Click fast account switcher) dành cho **Antigravity CLI (`agy`)** và **Antigravity IDE**.

---

## Tính năng chính

- **Chuyển tài khoản siêu tốc 1-click**: Tự động đồng bộ sang OS Keyring (GNOME Keyring / Linux Secret Service) và cơ sở dữ liệu SQLite của Antigravity IDE (`state.vscdb`) trong chưa đầy 2ms.
- **Không gây ô nhiễm môi trường**: Chạy 100% độc lập, không chèn biến môi trường hay chỉnh sửa file shell (`.bashrc`, `.zshrc`), kết nối trực tiếp đến Google với tốc độ tối đa.
- **Tra cứu hạn ngạch real-time**: Theo dõi hạn ngạch 5 giờ và hàng tuần của các nhóm mô hình (Gemini Models, Claude & GPT) kèm thời gian đếm ngược reset chính xác.
- **Tự động chọn tài khoản tốt nhất**: Tự động đánh giá và chuyển sang tài khoản có hạn ngạch 5 giờ Gemini cao nhất mỗi khi chạy lệnh `agy`.
- **Giao diện web tối giản**: Bảng điều khiển gọn gàng, tinh tế theo phong cách dark mode tối giản tại `http://127.0.0.1:8045`.

---

## Cài đặt và sử dụng

### 1. Biên dịch và khởi chạy

```bash
cd antigravity-relay
cargo build --release
./target/release/antigravity-relay
```

Truy cập bảng điều khiển tại: [http://127.0.0.1:8045](http://127.0.0.1:8045)

### 2. Tự động chuyển tài khoản với `agy` CLI

Tạo alias hoặc script wrapper cho `agy` để tự động chọn tài khoản có hạn ngạch Gemini 5h cao nhất:

```bash
# Thêm vào ~/.bashrc hoặc ~/.zshrc nếu muốn
alias agy='curl -s -X POST http://127.0.0.1:8045/api/accounts/auto-select --connect-timeout 0.2 --max-time 0.5 >/dev/null 2>&1 || true; agy-bin'
```

---

## Cấu trúc thư mục

```
├── antigravity-relay/          # Mã nguồn Rust backend và daemon
│   ├── src/
│   │   ├── storage/            # Quản lý tài khoản, OS Keyring và IDE SQLite
│   │   ├── proxy/              # Server Axum, quản lý token, tra cứu quota và UI
│   │   ├── oauth/              # Luồng đăng nhập Google OAuth
│   │   └── device/             # Định danh phần cứng độc lập
│   └── Cargo.toml
├── docs/                       # Tài liệu dự án và lộ trình phát triển
│   ├── ROADMAP.md
│   ├── commands.md
│   └── tags.md
└── .gitignore
```
