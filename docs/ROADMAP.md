# Lộ trình phát triển dự án (`antigravity-relay`)

> **Mục tiêu sản phẩm:** Hệ thống quản lý đa tài khoản Google và bộ chuyển đổi tài khoản siêu tốc (1-Click fast account switcher) dành riêng cho **Antigravity CLI (`agy`)** và **Antigravity IDE**. Đảm bảo kết nối trực tiếp 100% đến Google, không gây ô nhiễm biến môi trường, không làm chậm kết nối mạng.

```
┌────────────────────────────────────────────────────────────────────────┐
│                        Tổng quan hệ thống hiện tại                     │
├────────────────────────────────────────────────────────────────────────┤
│                                                                        │
│  Giai đoạn 1: Quản lý pool tài khoản và cơ chế đồng bộ ngầm (Xong)     │
│  ├── Lõi daemon Rust Axum siêu nhẹ (cổng 8045)                         │
│  ├── Đồng bộ OS Keyring (Linux Secret Service / GNOME Keyring)         │
│  ├── Đồng bộ Protobuf vào SQLite DB của Antigravity IDE (state.vscdb)  │
│  └── Cô lập phần cứng (Hardware fingerprint) cho từng tài khoản        │
│                                                                        │
│  Giai đoạn 2: Quota real-time và tự động chọn tài khoản (Xong)         │
│  ├── Tra cứu hạn ngạch Google PA (Gemini 5h & tuần, Claude/GPT)        │
│  ├── Hiển thị thời gian đếm ngược reset chính xác theo giờ địa phương  │
│  ├── Tự động chọn tài khoản có quota Gemini 5h cao nhất khi gõ agy     │
│  └── Đồng bộ 2 chiều (Tự bắt token khi login lại trong agy)            │
│                                                                        │
│  Giai đoạn 3: Giao diện người dùng tối giản và phân phối (Hiện tại)    │
│  ├── Giao diện web dashboard phong cách tối giản (Zinc dark theme)     │
│  ├── Đóng gói nhị phân release tối ưu kích thước                       │
│  └── Mở rộng script cài đặt tự động 1 dòng lệnh                        │
│                                                                        │
└────────────────────────────────────────────────────────────────────────┘
```

---

## Giai đoạn 1: Quản lý pool tài khoản và cơ chế đồng bộ (Hoàn thành 100%)

- [x] **Lõi Rust daemon siêu nhẹ (`antigravity-relay`)**:
  - Khởi chạy nền với server Axum tối giản tại `http://127.0.0.1:8045`.
  - Không chèn proxy, không sửa đổi file `.bashrc`/`.zshrc`, đảm bảo `agy` luôn kết nối trực tiếp đến Google với tốc độ tối đa.
- [x] **Tích hợp OS Keyring (`KeyringSync`)**:
  - Tương thích trực tiếp với thư viện `go-keyring` của `agy` CLI trên Linux (D-Bus Secret Service / GNOME Keyring).
  - Tự động cập nhật `service: "gemini"`, `username: "antigravity"` khi chuyển đổi tài khoản.
- [x] **Tích hợp cơ sở dữ liệu Antigravity IDE (`IdeDbSync`)**:
  - Mã hóa nhị phân Protobuf `OAuthTokenInfo` cho `antigravityUnifiedStateSync.oauthToken`.
  - Tự động nạp token vào `~/.config/Antigravity IDE/User/globalStorage/state.vscdb`.
- [x] **Cô lập định danh phần cứng (`Fingerprint`)**:
  - Tạo và gán `machine_id`, `mac_machine_id`, `dev_device_id`, `sqm_id` độc lập cho từng tài khoản để chống liên đới khi dùng nhiều tài khoản.
- [x] **Lưu trữ cục bộ chuẩn hóa**:
  - Quản lý dữ liệu an toàn tại `~/.antigravity-relay/accounts/*.json` và được bảo vệ tuyệt đối trong `.gitignore`.

---

## Giai đoạn 2: Quota real-time và tự động chọn tài khoản (Hoàn thành 100%)

- [x] **Tra cứu hạn ngạch chi tiết (Google CloudCode PA API)**:
  - Tự động lấy dữ liệu nhóm mô hình: `Gemini Models` và `Claude and GPT models`.
  - Phân tách rõ ràng 2 hạn mức: **Hạn ngạch 5 giờ** và **Hạn ngạch tuần**.
  - Chuẩn hóa phần trăm dạng số nguyên gọn gàng, không có số thập phân.
- [x] **Đếm ngược thời gian reset**:
  - Hiển thị chính xác thời gian còn lại (ví dụ: `Reset: 03:46 (còn 2h 15m)` hoặc `Reset: 23:38 (còn 6 ngày 22h)`).
  - Tự động quy đổi sang múi giờ địa phương.
- [x] **Tự động chọn tài khoản tối ưu khi dùng `agy`**:
  - Khi gõ lệnh `agy` trong terminal, hệ thống tự động so sánh và chuyển sang tài khoản có **hạn ngạch 5 giờ nhóm Gemini Models cao nhất** trước khi khởi chạy CLI (thao tác ngầm < 2ms).
  - Nếu relay không chạy, `agy` vẫn thực thi trực tiếp bình thường không gián đoạn.
- [x] **Đồng bộ 2 chiều (Two-way token sync)**:
  - Nếu người dùng đăng xuất và đăng nhập lại bên trong `agy`, relay server sẽ tự động phát hiện token mới và lưu vào cơ sở dữ liệu relay.

---

## Giai đoạn 3: Giao diện dashboard và trải nghiệm người dùng (Đang hoàn thiện)

- [x] **Giao diện web dashboard tối giản (Minimalist dark UI)**:
  - Phong cách thiết kế đơn sắc tối giản (Zinc dark theme) lấy cảm hứng từ Linear và Vercel.
  - Loại bỏ hoàn toàn hiệu ứng màu mè và chuẩn hóa câu chữ tiếng Việt đúng ngữ pháp.
  - Thao tác nhanh: Chuyển tài khoản 1-click, đăng nhập Google OAuth mới, thêm token trực tiếp.
- [ ] **Mở rộng script cài đặt 1 dòng lệnh**:
  - Script tự động tải binary release và thiết lập alias wrapper cho `agy`.
  - Hỗ trợ Linux và macOS.
- [ ] **Khay hệ thống (System tray - tùy chọn mở rộng)**:
  - Biểu tượng nhỏ trên taskbar hiển thị nhanh % quota và đổi tài khoản nhanh không cần mở trình duyệt.
