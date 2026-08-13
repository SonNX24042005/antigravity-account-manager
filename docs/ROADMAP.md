# 🗺️ ROADMAP PHÁT TRIỂN SẢN PHẨM (`antigravity-relay`)

> **Mục tiêu sản phẩm:** Trạm Proxy Daemon siêu nhẹ chạy ngầm trong Terminal, tự động điều hướng và đảo tài khoản Google (Gemini) trong suốt khi dính limit `429`, phục vụ cho **Antigravity CLI (`agy`)** và **Antigravity IDE**.

```
┌────────────────────────────────────────────────────────────────────────┐
│                        TỔNG QUAN LỘ TRÌNH 3 GIAI ĐOẠN                  │
├────────────────────────────────────────────────────────────────────────┤
│                                                                        │
│  GIAI ĐOẠN 1: Tối Ưu Lõi Rust & Chuẩn Hóa Terminal Daemon [HOÀN THÀNH]  │
│  ├── Clean Rewrite 100% dự án Rust antigravity-relay (Binary 9.1MB)    │
│  └── Khởi chạy Server Axum ngầm (Port 8045) với log màu Terminal       │
│                                                                        │
│  GIAI ĐOẠN 2: Tích Hợp Vận Hành Cho agy CLI & Antigravity IDE [HOÀN THÀNH]│
│  ├── Đăng nhập đa tài khoản Google qua OAuth Browser Flow              │
│  ├── Đồng bộ Proxy vào agy CLI & SQLite DB của Antigravity IDE         │
│  └── Transparent Auto-Failover & Circuit Breaker (Đổi acc ngầm 100%)   │
│                                                                        │
│  GIAI ĐOẠN 3: Phát Triển Giao Diện GUI (Mở Rộng Tương Lai)             │
│  ├── Thiết kế Giao diện Desktop tối giản 2 trang (Tài khoản & Proxy)   │
│  └── Đóng gói cài đặt 1 dòng lệnh (install.sh / install.ps1)           │
│                                                                        │
└────────────────────────────────────────────────────────────────────────┘
```

---

## 📌 GIAI ĐOẠN 1: Tối Ưu Lõi Rust & Chuẩn Hóa Terminal Daemon *(HOÀN THÀNH 100%)*

- [x] **Clean Rewrite 100%**: Tạo dự án Rust mới `antigravity-relay/` tại root repository, loại bỏ hoàn toàn code rác và GUI cũ.
- [x] **Dọn dẹp module thừa**: Xóa các module Python scripts, update_checker, user_token, droid_sync, cloudflared, caveman/rtk cleaners.
- [x] **Cấu hình Terminal Headless Server**:
  - Khởi chạy Server Axum ngầm trên cổng `http://127.0.0.1:8045`.
  - Hệ thống log Terminal màu trực quan (`tracing`) theo dõi luồng request, auto-failover, quota status.
- [x] **Thuật toán P2C & Circuit Breaker**: Tự động chọn tài khoản có Quota cao nhất và tự động khóa tạm tài khoản bị limit `429/403`.
- [x] **Tự động đồng bộ CLI**: Tự động tiêm cấu hình Proxy vào `~/.antigravity/config.json`.
- [x] **Tiêm SQLite DB IDE**: Module `ide_db.rs` tự động tiêm `access_token` + `machine_id` vào Antigravity IDE.
- [x] **Giả lập Device Profile**: Module `device/fingerprint.rs` tạo định danh phần ứng (`machine_id`, `sqm_id`) chống ban acc.
- [x] **Bảo vệ Git**: Cấu hình tệp `.gitignore` bỏ qua dữ liệu token cá nhân và file build binary.

---

## 📌 GIAI ĐOẠN 2: Tích Hợp Vận Hành Cho `agy` CLI & Antigravity IDE *(HOÀN THÀNH 100%)*

### 2.1 Quản lý Pool Đa Tài Khoản
- [x] Đăng nhập đa tài khoản Google cá nhân qua OAuth Browser URL (`/api/accounts/oauth/start`).
- [x] Dữ liệu tài khoản lưu trữ dưới dạng tệp JSON chuẩn hóa tại `~/.antigravity-relay/accounts/`.
- [x] Định danh phần cứng giả lập (`machine_id`, `sqm_id`) cố định cho từng tài khoản chống ban.

### 2.2 Tự động Đồng bộ Proxy
- [x] **Dành cho `agy` CLI**: Module `cli_sync.rs` tự động tiêm Proxy `http://127.0.0.1:8045` vào file config `~/.antigravity/config.json` của `agy`.
- [x] **Dành cho Antigravity IDE**: Module `ide_db.rs` tiêm `access_token` + `machine_id` thẳng vào SQLite DB của IDE.

### 2.3 Transparent Auto-Failover Engine
- [x] Khi chat trong `agy` CLI mà tài khoản dính limit `429` / `403`:
  - [x] Proxy tự động kích hoạt Circuit Breaker khóa tạm tài khoản lỗi.
  - [x] Proxy tự động chọn tài khoản lành lặn tiếp theo (P2C algorithm).
  - [x] Proxy tự đóng gói ngữ cảnh chat cũ gửi sang tài khoản mới.
  - [x] Phía `agy` CLI nhận câu trả lời liên tục ngầm (Transparent) mà không bị văng lỗi hay dừng session.

---

## 📌 GIAI ĐOẠN 3: Phát Triển Giao Diện GUI *(Dự kiến mở rộng tương lai)*

### 3.1 Xây dựng Giao diện Desktop
- [ ] Chọn Tech Stack nhẹ: **Tauri 2 + Svelte 5** (hoặc **React + TailwindCSS**).
- [ ] Xây dựng **2 màn hình chính**:
  1. *Trang Tài Khoản*: Danh sách tài khoản, % Quota live, nút Đăng nhập tài khoản mới.
  2. *Trang Proxy*: Bật/tắt Proxy, xem Cổng 8045, xem log hoạt động.

### 3.2 Đóng gói & Phân phối
- [ ] Viết script cài đặt 1 dòng lệnh gọn nhẹ:
  - Linux/macOS: `curl -fsSL ... | bash`
  - Windows: `powershell ...`
