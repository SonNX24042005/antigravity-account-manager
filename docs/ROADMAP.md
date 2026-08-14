# Lộ trình phát triển dự án (`antigravity-relay` / `agyr`)

> **Mục tiêu sản phẩm:** Hệ thống quản lý đa tài khoản Google và bộ chuyển đổi tài khoản siêu tốc (1-Click fast account switcher) dành riêng cho **Antigravity CLI (`agy`)** và **Antigravity IDE**. Đảm bảo kết nối trực tiếp 100% đến Google, không gây ô nhiễm biến môi trường, không làm chậm kết nối mạng, cài đặt 1 dòng lệnh qua curl.

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
│  ├── Tự động đồng bộ ngầm tài khoản có quota tốt nhất vào Keyring & IDE│
│  └── Hoàn toàn không cần tạo alias hay sửa đổi shell profile           │
│                                                                        │
│  Giai đoạn 3: Giao diện web tối giản, CLI agyr và phân phối (Xong)     │
│  ├── Bảng điều khiển web hiện đại, tối giản tại http://127.0.0.1:8045  │
│  ├── Trình điều khiển toàn cục 'agyr' (start, autostart, update,...)   │
│  ├── Cài đặt 1 dòng lệnh bằng curl không cần clone repo, không cần Rust│
│  └── CI/CD GitHub Actions tự động đóng gói release binary              │
│                                                                        │
│  Giai đoạn 4: Tính năng mở rộng nâng cao (Lộ trình tương lai)          │
│  └── Biểu tượng khay hệ thống (System tray applet - tùy chọn mở rộng)  │
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

## Giai đoạn 2: Quota real-time và cơ chế Zero-Alias (Hoàn thành 100%)

- [x] **Tra cứu hạn ngạch chi tiết (Google CloudCode PA API)**:
  - Tự động lấy dữ liệu nhóm mô hình: `Gemini Models` và `Claude and GPT models`.
  - Phân tách rõ ràng 2 hạn mức: **Hạn ngạch 5 giờ** và **Hạn ngạch tuần**.
  - Chuẩn hóa phần trăm dạng số nguyên gọn gàng, không có số thập phân.
- [x] **Đếm ngược thời gian reset**:
  - Hiển thị chính xác thời gian còn lại (ví dụ: `Reset: 03:46 (còn 2h 15m)` hoặc `Reset: 23:38 (còn 6 ngày 22h)`).
  - Tự động quy đổi sang múi giờ địa phương và tự động hiển thị phục hồi 100% khi hết chu kỳ 5h.
- [x] **Cơ chế tự động đồng bộ ngầm (Zero-Alias Architecture)**:
  - Background worker tự động quét quota định kỳ mỗi 30 giây.
  - Tự động nạp sẵn tài khoản có **hạn ngạch Gemini 5h cao nhất** vào OS Keyring và Antigravity IDE.
  - Người dùng mở terminal và gõ `agy` như bình thường mà không cần alias hay hook trung gian.

---

## Giai đoạn 3: Giao diện web, CLI toàn cục và CI/CD (Hoàn thành 100%)

- [x] **Bảng điều khiển web tinh tế (Minimalist Tech UI)**:
  - Phong cách dark zinc hiện đại lấy cảm hứng từ Linear/Vercel với điểm nhấn màu xanh công nghệ.
  - Nút thêm tài khoản dạng dropdown thông minh với 2 phương thức rõ ràng: Đăng nhập Google (OAuth) và Nhập token trực tiếp.
  - Luồng OAuth callback tự động làm mới dữ liệu và tự động đóng tab quay về bảng điều khiển.
  - Xóa tài khoản 1-click có xác nhận và tự động failover sang tài khoản còn lại.
- [x] **Bộ công cụ CLI toàn cục (`agyr`)**:
  - `agyr`: Tự động bật daemon nếu chưa chạy và mở ngay bảng điều khiển web trong trình duyệt.
  - `agyr autostart`: Đăng ký systemd user service tự chạy cùng máy tính và tự hồi phục khi crash.
  - `agyr start` / `agyr stop` / `agyr restart` / `agyr status` / `agyr version`.
  - `agyr update`: Tự động kiểm tra và nâng cấp lên phiên bản mới nhất từ GitHub.
- [x] **Phân phối và cài đặt 1 dòng lệnh**:
  - Cài đặt qua `curl -fsSL https://raw.githubusercontent.com/SonNX24042005/antigravity-account-manager/main/install.sh | bash`.
  - Tải trực tiếp pre-built binary từ GitHub Releases mà không cần máy người dùng phải cài đặt Rust hay Cargo.
- [x] **Tự động hóa CI/CD**:
  - GitHub Actions workflow `.github/workflows/release.yml` tự động biên dịch và phát hành bản nhị phân cho Linux và macOS khi gắn tag release.

---

## Giai đoạn 4: Tính năng mở rộng nâng cao (Lộ trình tiếp theo)

- [ ] **Khay hệ thống (System tray applet - tùy chọn mở rộng)**:
  - Biểu tượng nhỏ trên khay hệ thống hiển thị nhanh % quota và chuyển đổi tài khoản nhanh không cần mở trình duyệt.
- [ ] **Hỗ trợ proxy chuyển tiếp thông minh (Multi-account load balancer)**:
  - Tùy chọn proxy cục bộ cho các công cụ của bên thứ 3 nếu muốn gọi API qua định dạng OpenAI tương thích.
