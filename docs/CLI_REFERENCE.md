# Tài liệu hướng dẫn sử dụng CLI (`agyr`)

Hệ thống quản lý dịch vụ và bộ chuyển đổi tài khoản Antigravity Relay cung cấp công cụ dòng lệnh **`agyr`** (và `antigravity-relay`) để bạn có thể thao tác ở bất kỳ thư mục nào trên terminal.

---

## 1. Cài đặt

### Cài đặt nhanh qua curl (không cần clone repo, không cần Rust):

```bash
curl -fsSL https://raw.githubusercontent.com/SonNX24042005/antigravity-account-manager/main/install.sh | bash
```

### Cài đặt từ mã nguồn cục bộ:

```bash
./install.sh
```

---

## 2. Danh sách lệnh điều khiển (`agyr`)

| Lệnh | Mô tả |
|---|---|
| `agyr` | Tự động khởi chạy ngầm dịch vụ (nếu chưa chạy) và mở ngay bảng điều khiển web trên trình duyệt mặc định. |
| `agyr autostart` *(hoặc `enable`)* | Kích hoạt tự động chạy cùng hệ điều hành (systemd user service) kể cả khi khởi động lại máy, tự động hồi phục sau 3 giây nếu crash. |
| `agyr start` | Khởi chạy dịch vụ chạy ngầm. |
| `agyr status` | Kiểm tra trạng thái hoạt động của dịch vụ, cổng kết nối, trình quản lý và chế độ tự khởi động. |
| `agyr stop` | Dừng dịch vụ đang chạy. |
| `agyr restart` | Khởi động lại dịch vụ. |
| `agyr update` *(hoặc `upgrade`)* | Tự động tải bản phát hành mới nhất từ GitHub và khởi động lại dịch vụ. |
| `agyr version` *(hoặc `-v`)* | Hiển thị phiên bản hiện tại của chương trình. |
| `agyr disable` | Tắt chế độ tự khởi động cùng hệ thống. |
| `agyr run` | Khởi chạy dịch vụ trực tiếp trên terminal hiện tại (foreground) để theo dõi log chi tiết. |

---

## 3. Danh sách API backend (Cổng 8045)

Máy chủ backend cung cấp các REST API cục bộ để quản lý tài khoản và hạn ngạch:

### Quản lý tài khoản
- `GET /api/accounts`: Lấy danh sách toàn bộ tài khoản trong pool kèm hạn ngạch real-time.
- `POST /api/accounts/add`: Thêm tài khoản mới bằng Access token và Refresh token thủ công.
- `POST /api/accounts/delete`: Xóa tài khoản khỏi bộ nhớ và ổ đĩa (`{"account_id": "..."}`).
- `POST /api/accounts/switch`: Chuyển đổi thủ công sang tài khoản được chọn (`{"account_id": "..."}`).
- `POST /api/accounts/auto-select`: Đánh giá và tự động chuyển sang tài khoản có hạn ngạch Gemini 5h cao nhất.

### Đăng nhập Google OAuth
- `GET /api/accounts/oauth/start`: Lấy URL xác thực Google OAuth.
- `GET /api/accounts/oauth/callback`: Tiếp nhận mã xác thực từ Google, nạp token và tự động điều hướng về bảng điều khiển.

### Giao diện quản trị
- `GET /` hoặc `GET /admin`: Trả về giao diện web quản lý tài khoản (HTML/CSS/JS).

---

## 4. Kiến trúc Zero-Alias (Không cần sửa file shell)

Khi `agyr` đang chạy ngầm:
1. Tác vụ nền (background worker) tự động quét hạn ngạch và làm mới token mỗi 30 giây.
2. Tự động chọn tài khoản có **hạn ngạch Gemini 5h cao nhất** và nạp sẵn vào:
   - **OS Keyring** (D-Bus Secret Service / GNOME Keyring: `service: gemini, user: antigravity`).
   - **Cơ sở dữ liệu SQLite của Antigravity IDE** (`state.vscdb`).
   - Tệp xác thực CLI (`~/.antigravity/auth.json` và `~/.gemini/antigravity-cli/auth.json`).
3. Người dùng mở terminal và gõ lệnh `agy` như bình thường, `agy` sẽ tự động nhận diện và sử dụng ngay tài khoản tốt nhất mà không cần qua bất kỳ lệnh alias hay hook trung gian nào.
