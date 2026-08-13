# 🗺️ LỘ TRÌNH PHÁT TRIỂN DỰ ÁN (`antigravity-relay`)

> **Mục tiêu sản phẩm:** Hệ thống quản lý đa tài khoản Google & Bộ chuyển đổi tài khoản siêu tốc (1-Click Fast Account Switcher) dành riêng cho **Antigravity CLI (`agy`)** và **Antigravity IDE**. Đảm bảo kết nối trực tiếp 100% đến Google, không gây ô nhiễm biến môi trường, không làm chậm kết nối mạng.

```
┌────────────────────────────────────────────────────────────────────────┐
│                        TỔNG QUAN HỆ THỐNG HIỆN TẠI                     │
├────────────────────────────────────────────────────────────────────────┤
│                                                                        │
│  GIAI ĐOẠN 1: Quản Lý Pool Tài Khoản & Cơ Chế Đồng Bộ Ngầm [HOÀN THÀNH] │
│  ├── Lõi daemon Rust Axum siêu nhẹ (Port 8045)                         │
│  ├── Đồng bộ OS Keyring (Linux Secret Service / GNOME Keyring)         │
│  ├── Đồng bộ Protobuf vào SQLite DB của Antigravity IDE (state.vscdb)  │
│  └── Cô lập phần cứng (Hardware Fingerprint) cho từng tài khoản        │
│                                                                        │
│  GIAI ĐOẠN 2: Quota Real-Time & Tự Động Chọn Tài Khoản [HOÀN THÀNH]    │
│  ├── Tra cứu hạn ngạch Google PA (Gemini 5h & Tuần, Claude/GPT)       │
│  ├── Hiển thị thời gian đếm ngược Reset chính xác theo giờ địa phương  │
│  ├── Tự động chọn tài khoản có Quota Gemini 5h cao nhất khi gõ agy     │
│  └── Đồng bộ 2 chiều (Auto-Capture Token khi login lại trong agy)      │
│                                                                        │
│  GIAI ĐOẠN 3: Giao Diện Người Dùng Tối Giản & Phân Phối [HIỆN TẠI]     │
│  ├── Giao diện Web Dashboard phong cách tối giản (Zinc Dark Theme)     │
│  ├── Đóng gói nhị phân Release tối ưu kích thước                       │
│  └── Mở rộng script cài đặt tự động 1 dòng lệnh                        │
│                                                                        │
└────────────────────────────────────────────────────────────────────────┘
```

---

## 📌 GIAI ĐOẠN 1: Quản Lý Pool Tài Khoản & Cơ Chế Đồng Bộ *(HOÀN THÀNH 100%)*

- [x] **Lõi Rust Daemon Siêu Nhẹ (`antigravity-relay`)**:
  - Khởi chạy nền với Server Axum tối giản tại `http://127.0.0.1:8045`.
  - Không chèn Proxy, không sửa đổi file `.bashrc`/`.zshrc`, đảm bảo `agy` luôn kết nối trực tiếp đến Google với tốc độ tối đa.
- [x] **Tích hợp OS Keyring (`KeyringSync`)**:
  - Tương thích trực tiếp với thư viện `go-keyring` của `agy` CLI trên Linux (D-Bus Secret Service / GNOME Keyring).
  - Tự động cập nhật `service: "gemini"`, `username: "antigravity"` khi chuyển đổi tài khoản.
- [x] **Tích hợp Cơ sở Dữ liệu Antigravity IDE (`IdeDbSync`)**:
  - Mã hóa nhị phân Protobuf `OAuthTokenInfo` cho `antigravityUnifiedStateSync.oauthToken`.
  - Tự động nạp Token vào `~/.config/Antigravity IDE/User/globalStorage/state.vscdb`.
- [x] **Cô lập Định danh Phần cứng (`Fingerprint`)**:
  - Tạo và gán `machine_id`, `mac_machine_id`, `dev_device_id`, `sqm_id` độc lập cho từng tài khoản để chống liên đới khi dùng nhiều tài khoản.
- [x] **Lưu trữ Cục bộ Chuẩn hóa**:
  - Quản lý dữ liệu an toàn tại `~/.antigravity-relay/accounts/*.json` và được bảo vệ tuyệt đối trong `.gitignore`.

---

## 📌 GIAI ĐOẠN 2: Quota Real-Time & Tự Động Chọn Tài Khoản *(HOÀN THÀNH 100%)*

- [x] **Tra Cứu Hạn Ngạch Chi Tiết (Google CloudCode PA API)**:
  - Tự động lấy dữ liệu nhóm mô hình: `Gemini Models` và `Claude and GPT models`.
  - Phân tách rõ ràng 2 hạn mức: **Hạn ngạch 5 Giờ (5-Hour)** và **Hạn ngạch Hàng Tuần (Weekly)**.
  - Chuẩn hóa phần trăm dạng số nguyên gọn gàng, không có số thập phân.
- [x] **Đếm Ngược Thời Gian Reset**:
  - Hiển thị chính xác thời gian còn lại (ví dụ: `Reset: 03:46 (còn 2h 15m)` hoặc `Reset: 23:38 (còn 6 ngày 22h)`).
  - Tự động quy đổi sang múi giờ địa phương.
- [x] **Tự Động Chọn Tài Khoản Tối Ưu Khi Dùng `agy`**:
  - Khi gõ lệnh `agy` trong Terminal, hệ thống tự động so sánh và chuyển sang tài khoản có **Hạn ngạch 5 Giờ nhóm Gemini Models cao nhất** trước khi khởi chạy CLI (thao tác ngầm < 2ms).
  - Nếu Relay không chạy, `agy` vẫn thực thi trực tiếp bình thường không gián đoạn.
- [x] **Đồng Bộ 2 Chiều (Two-Way Token Sync)**:
  - Nếu người dùng đăng xuất và đăng nhập lại bên trong `agy`, Relay Server sẽ tự động phát hiện Token mới và lưu vào cơ sở dữ liệu Relay.

---

## 📌 GIAI ĐOẠN 3: Giao Diện Dashboard & Trải Nghiệm Người Dùng *(ĐANG HOÀN THIỆN)*

- [x] **Giao Diện Web Dashboard Tối Giản (Minimalist Dark UI)**:
  - Phong cách thiết kế đơn sắc tối giản (Zinc Dark Theme) lấy cảm hứng từ Linear và Vercel.
  - Loại bỏ hoàn toàn hiệu ứng màu mè và chuẩn hóa câu chữ tiếng Việt đúng ngữ pháp.
  - Thao tác nhanh: Chuyển tài khoản 1-click, Đăng nhập Google OAuth mới, Thêm token trực tiếp.
- [ ] **Mở Rộng Script Cài Đặt 1 Dòng Lệnh**:
  - Script tự động tải binary release và thiết lập alias wrapper cho `agy`.
  - Hỗ trợ Linux và macOS.
- [ ] **Khay Hệ Thống (System Tray - Tùy Chọn Mở Rộng)**:
  - Biểu tượng nhỏ trên taskbar hiển thị nhanh % Quota và đổi tài khoản nhanh không cần mở trình duyệt.
