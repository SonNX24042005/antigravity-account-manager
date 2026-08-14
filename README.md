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

### 1. Cài đặt nhanh 1 dòng lệnh (không cần clone repo)

Tương tự như `claude code` hay `rustup`, bạn có thể cài đặt ngay lập tức ở bất kỳ máy nào bằng 1 lệnh curl:

```bash
curl -fsSL https://raw.githubusercontent.com/SonNX24042005/antigravity-account-manager/main/install.sh | bash
```

*(Hoặc nếu đã tải mã nguồn về máy, bạn có thể chạy trực tiếp `./install.sh`)*

Lệnh này sẽ tự động tải hoặc biên dịch và cài đặt lệnh điều khiển toàn cục **`agyr`** (và `antigravity-relay`) vào `~/.local/bin/`. Sau khi cài, bạn có thể gõ `agyr` ở bất kỳ thư mục nào trong terminal.

### 2. Các tùy chọn khởi chạy

- **Tự động chạy liên tục cùng hệ thống (Khuyên dùng - Auto-start on boot):**
  ```bash
  agyr autostart
  ```
  *Dịch vụ sẽ tự khởi động ngầm mỗi khi mở máy, tự động hồi phục và bật lại sau 3 giây nếu bị tắt. Muốn dừng hoàn toàn chỉ cần gõ `agyr stop` hoặc `agyr disable`.*

- **Chạy nền thông thường:**
  ```bash
  agyr start
  ```

- **Kiểm tra trạng thái:**
  ```bash
  agyr status
  ```

- **Dừng dịch vụ:**
  ```bash
  agyr stop
  ```

- **Cập nhật lên phiên bản mới nhất:**
  ```bash
  agyr update
  ```

- **Khởi động lại dịch vụ:**
  ```bash
  agyr restart
  ```

Truy cập bảng điều khiển web tại: [http://127.0.0.1:8045](http://127.0.0.1:8045)

### 3. Tự động đồng bộ không cần alias

Khi dịch vụ `agyr` đang chạy, hệ thống sẽ tự động quét hạn ngạch ngầm và duy trì tài khoản có hạn ngạch Gemini 5h cao nhất vào OS Keyring và Antigravity IDE. Bạn chỉ cần chạy lệnh `agy` như bình thường mà **không cần tạo bất kỳ alias hay cấu hình shell nào**.

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
