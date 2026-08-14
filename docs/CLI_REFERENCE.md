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

### Quản lý tài khoản và định tuyến mô hình
- `GET /api/accounts`: Lấy danh sách toàn bộ tài khoản trong pool kèm hạn ngạch real-time.
- `POST /api/accounts/add`: Thêm tài khoản mới bằng Access token và Refresh token thủ công.
- `POST /api/accounts/delete`: Xóa tài khoản khỏi bộ nhớ và ổ đĩa (`{"account_id": "..."}`).
- `POST /api/accounts/switch`: Chuyển đổi thủ công sang tài khoản được chọn (`{"account_id": "..."}`).
- `POST /api/accounts/auto-select`: Tự động nhận diện mô hình vừa dùng (Gemini hoặc Claude/GPT) và chuyển sang tài khoản có hạn ngạch cao nhất của mô hình đó.
- `GET /api/preference`: Lấy trạng thái định tuyến mô hình hiện tại (chế độ, mô hình vừa phát hiện, nguồn phát hiện).
- `POST /api/preference`: Cập nhật chế độ định tuyến (`{"preference": "auto" | "gemini" | "claude_gpt"}`).

### Đăng nhập Google OAuth
- `GET /api/accounts/oauth/start`: Lấy URL xác thực Google OAuth.
- `GET /api/accounts/oauth/callback`: Tiếp nhận mã xác thực từ Google, nạp token và tự động điều hướng về bảng điều khiển.

### Giao diện quản trị
- `GET /` hoặc `GET /admin`: Trả về giao diện web quản lý tài khoản (HTML/CSS/JS).

---

## 4. Kiến trúc tự động chọn tài khoản theo mô hình (Smart Dynamic Model Routing)

Khi `agyr` đang chạy ngầm:
1. **Phát hiện mô hình vừa dùng (3 lớp thông minh):**
   - **Lớp 1 (Transcript Scanner):** Quét các phiên hội thoại gần nhất của Antigravity CLI để phát hiện người dùng vừa tương tác với Gemini hay Claude/GPT.
   - **Lớp 2 (Quota Delta Consumption Tracker):** Tự động so sánh độ tiêu hao hạn ngạch giữa các lần quét để xác định mô hình đang được tiêu thụ.
   - **Lớp 3 (Cấu hình trên giao diện Web):** Cho phép người dùng tùy chọn `Tự động (theo mô hình vừa dùng)`, `Luôn ưu tiên Gemini`, hoặc `Luôn ưu tiên Claude & GPT`.
2. **Tự động chọn tài khoản tối ưu:**
   - Nếu bạn vừa dùng **Gemini** -> Hệ thống chọn tài khoản có **hạn ngạch Gemini 5h cao nhất**.
   - Nếu bạn vừa dùng **Claude / GPT / khác** -> Hệ thống chọn tài khoản có **hạn ngạch Claude & GPT cao nhất**.
3. **Đồng bộ siêu tốc:** Nạp sẵn tài khoản tối ưu vào **OS Keyring** và **Antigravity IDE SQLite DB**, giúp bạn mở terminal gõ `agy` là dùng ngay mà không cần cấu hình thủ công.
