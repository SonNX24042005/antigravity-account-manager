# Báo cáo kiểm toán bảo mật dự án Antigravity Relay

Báo cáo này tổng hợp kết quả quét và phân tích mã nguồn toàn bộ dự án `antigravity-relay` bằng CodeGraph nhằm xác định các lỗ hổng bảo mật và rủi ro tiềm ẩn.

---

## 1. Tổng quan kiểm toán

- **Dự án**: Antigravity Account Manager (`antigravity-relay`)
- **Ngôn ngữ & nền tảng**: Rust (Axum, Tokio, Reqwest, Rusqlite)
- **Mục tiêu đánh giá**: Cơ chế xác thực, quản lý token, phân quyền, giao diện quản trị, lưu trữ cục bộ, ủy quyền OAuth và luồng chuyển tiếp proxy.

---

## 2. Bảng tổng hợp các lỗ hổng bảo mật

| Lỗ hổng / Rủi ro | Vị trí phát hiện | Mức độ | Tại sao nguy hiểm | Cách khắc phục |
| :--- | :--- | :--- | :--- | :--- |
| **Lộ master API key công khai qua giao diện quản trị** | [`server.rs:L38-L40`](file:///mnt/181EC3061EC2DBBE/DT/Code/PJ/antigravity-account-manager/antigravity-relay/src/proxy/server.rs#L38-L40)<br>[`ui.rs:L179`](file:///mnt/181EC3061EC2DBBE/DT/Code/PJ/antigravity-account-manager/antigravity-relay/src/proxy/ui.rs#L179) | **Cao (High)** | • Trang web quản trị (`/` và `/admin`) không yêu cầu mật khẩu hay token để truy cập.<br>• Mã nguồn máy chủ thay thế và chèn trực tiếp chuỗi khóa bí mật `master_key` vào mã JavaScript của giao diện.<br>• Bất kỳ ai mở được giao diện (trình duyệt nội bộ, máy khác trong mạng LAN nếu mở cổng, hoặc qua các kịch bản tấn công trên máy) đều đọc được toàn bộ `master_key`, từ đó chiếm toàn quyền quản trị và sử dụng tài khoản. | • Loại bỏ hoàn toàn việc nhúng tĩnh khóa bí mật vào nội dung mã HTML/JS trả về.<br>• Chuyển sang cơ chế xác thực phiên (session cookie với cờ `HttpOnly` và `SameSite=Strict`) hoặc yêu cầu người dùng nhập khóa trực tiếp trên giao diện ở phiên làm việc đầu tiên và lưu tạm trong `sessionStorage`. |
| **Lỗ hổng path traversal khi xóa tài khoản** | [`account_store.rs:L51-L57`](file:///mnt/181EC3061EC2DBBE/DT/Code/PJ/antigravity-account-manager/antigravity-relay/src/storage/account_store.rs#L51-L57) | **Cao (High)** | • Hàm xóa tài khoản nối trực tiếp chuỗi ID từ yêu cầu HTTP vào đường dẫn file mà không lọc ký tự đặc biệt.<br>• Kẻ tấn công có thể truyền ID chứa các ký tự điều hướng thư mục như `../../config`.<br>• Dẫn đến việc hệ thống có thể bị lợi dụng để xóa nhầm hoặc xóa có chủ đích các file `.json` quan trọng khác của người dùng trên máy tính. | • Thêm bộ lọc kiểm tra nghiêm ngặt định dạng ID (chỉ chấp nhận ký tự an toàn như UUID hoặc chữ số).<br>• Cấm hoàn toàn các ký tự `/`, `\`, `..` trong tham số ID hoặc kiểm tra đường dẫn sau khi chuẩn hóa phải luôn nằm bên trong thư mục lưu trữ tài khoản. |
| **Lỗ hổng OAuth CSRF do thiếu kiểm tra tham số state** | [`server.rs:L285-L306`](file:///mnt/181EC3061EC2DBBE/DT/Code/PJ/antigravity-account-manager/antigravity-relay/src/proxy/server.rs#L285-L306) | **Trung bình (Medium)** | • Khi tạo đường dẫn đăng nhập Google OAuth, hệ thống sinh một chuỗi `state` ngẫu nhiên nhưng không lưu lại trên máy chủ.<br>• Khi Google phản hồi về endpoint callback, hệ thống bỏ qua việc kiểm tra tham số `state` và tiến hành đổi mã lấy token ngay.<br>• Kẻ tấn công có thể lừa trình duyệt của nạn nhân truy cập vào link callback chứa mã ủy quyền của kẻ tấn công, khiến máy chủ của nạn nhân liên kết với tài khoản lạ. | • Lưu trữ chuỗi `state` vào bộ nhớ đệm của máy chủ kèm theo thời gian hết hạn ngắn (khoảng 5 đến 10 phút).<br>• Khi tiếp nhận callback, bắt buộc phải kiểm tra tham số `state` gửi về có trùng khớp với kho lưu trữ hay không; nếu không khớp hoặc hết hạn thì từ chối xử lý ngay. |
| **Nguy cơ DOM-based XSS trong bảng điều khiển** | [`ui.rs:L285-L390`](file:///mnt/181EC3061EC2DBBE/DT/Code/PJ/antigravity-account-manager/antigravity-relay/src/proxy/ui.rs#L285-L390) | **Trung bình (Medium)** | • Các trường dữ liệu tài khoản (như email, ID, tên nhóm hạn ngạch) được nối trực tiếp vào chuỗi HTML và gán vào `innerHTML`, cũng như đặt trong các sự kiện inline `onclick`.<br>• Nếu có dữ liệu chứa mã kịch bản độc hại (ví dụ qua API thêm tài khoản hoặc chỉnh sửa file cấu hình), trình duyệt sẽ thực thi mã JavaScript đó trong ngữ cảnh trang quản trị, có thể đánh cắp token hoặc thao túng máy chủ. | • Áp dụng cơ chế mã hóa các ký tự đặc biệt HTML (HTML entity encoding) cho mọi dữ liệu động trước khi hiển thị.<br>• Tránh dùng các thuộc tính sự kiện inline dạng chuỗi mà chuyển sang gắn sự kiện qua phương thức lắng nghe sự kiện an toàn của DOM (`addEventListener`). |
| **Cập nhật phần mềm không kiểm tra tính toàn vẹn** | [`cli.rs:L366-L390`](file:///mnt/181EC3061EC2DBBE/DT/Code/PJ/antigravity-account-manager/antigravity-relay/src/cli.rs#L366-L390)<br>[`install.sh:L45-L53`](file:///mnt/181EC3061EC2DBBE/DT/Code/PJ/antigravity-account-manager/install.sh#L45-L53) | **Trung bình (Medium)** | • Lệnh cập nhật tải trực tiếp script bash qua mạng bằng `curl | bash` và cài đặt file nhị phân mới mà không xác minh mã băm (checksum) hay chữ ký số.<br>• Nếu mạng bị can thiệp (man-in-the-middle) hoặc kho lưu trữ bị tấn công, nhị phân chứa mã độc có thể được tải về và thực thi tự động trên máy người dùng. | • Đính kèm file danh sách mã băm chính thức (như `SHA256SUMS`) cho mỗi bản phát hành trên GitHub.<br>• Bổ sung bước kiểm tra mã băm SHA256 của file tải về trước khi tiến hành giải nén và thay thế file thực thi trên hệ thống. |
| **Rủi ro timing attack khi kiểm tra API key** | [`server.rs:L42-L51`](file:///mnt/181EC3061EC2DBBE/DT/Code/PJ/antigravity-account-manager/antigravity-relay/src/proxy/server.rs#L42-L51) | **Thấp (Low)** | • Sử dụng phép so sánh chuỗi thông thường (`==`) để kiểm tra `master_key`.<br>• Phép so sánh này sẽ trả về sai ngay khi gặp ký tự đầu tiên không khớp, tạo ra sự chênh lệch nhỏ về thời gian phản hồi.<br>• Kẻ tấn công có thể đo thời gian phản hồi liên tục để suy đoán dần từng ký tự của khóa bí mật. | • Sử dụng hàm so sánh với thời gian cố định (constant-time comparison) để đảm bảo thời gian so sánh luôn đồng nhất dù token đúng hay sai bao nhiêu ký tự. |
| **Ghi đè file shell profile không đảm bảo tính nguyên tử** | [`account_switcher.rs:L95-L117`](file:///mnt/181EC3061EC2DBBE/DT/Code/PJ/antigravity-account-manager/antigravity-relay/src/storage/account_switcher.rs#L95-L117)<br>[`cli_sync.rs:L71-L94`](file:///mnt/181EC3061EC2DBBE/DT/Code/PJ/antigravity-account-manager/antigravity-relay/src/proxy/cli_sync.rs#L71-L94) | **Thấp (Low)** | • Quá trình đồng bộ đọc và ghi đè trực tiếp lên các file cấu hình khởi động shell (`.bashrc`, `.zshrc`).<br>• Nếu hệ thống gặp sự cố mất điện hoặc tiến trình bị buộc dừng đúng lúc đang ghi, file cấu hình của người dùng có thể bị lỗi, trắng dữ liệu hoặc hỏng môi trường dòng lệnh. | • Thực hiện ghi nội dung mới ra một file tạm trước, sau đó dùng thao tác đổi tên nguyên tử (atomic rename) để thay thế file cũ.<br>• Luôn tạo bản sao lưu dự phòng trước khi can thiệp vào các file cấu hình của hệ thống. |

---

## 3. Phân tích chi tiết từng lỗ hổng

### 3.1. Lộ master API key công khai qua giao diện quản trị
- **Vị trí**: [`antigravity-relay/src/proxy/server.rs:L38-L40`](file:///mnt/181EC3061EC2DBBE/DT/Code/PJ/antigravity-account-manager/antigravity-relay/src/proxy/server.rs#L38-L40), [`antigravity-relay/src/proxy/ui.rs:L179`](file:///mnt/181EC3061EC2DBBE/DT/Code/PJ/antigravity-account-manager/antigravity-relay/src/proxy/ui.rs#L179)
- **Mức độ**: Cao (High)
- **Nguyên nhân và cơ chế**:
  - Middleware `require_auth` miễn trừ kiểm tra xác thực cho các đường dẫn `/` và `/admin`.
  - Hàm `handle_admin_ui` thực hiện chèn trực tiếp chuỗi khóa bí mật `state.config.master_key` vào mã HTML giao diện trả về cho trình duyệt.
  - Phía trình duyệt lưu biến `const API_KEY = '{{MASTER_KEY}}'` ở dạng công khai.
- **Rủi ro**:
  Bất kỳ kết nối nào mở được giao diện đều trích xuất được `master_key` đầy đủ. Khi kẻ tấn công sở hữu khóa này, mọi endpoint nội bộ và proxy đều bị truy cập trái phép mà không có rào cản nào.
- **Hướng khắc phục**:
  Tách biệt luồng xác thực giao diện quản trị, không render khóa tĩnh vào mã nguồn HTML. Yêu cầu nhập khóa ở lần đầu hoặc sử dụng cơ chế session cookie có bảo vệ.

---

### 3.2. Lỗ hổng path traversal khi xóa tài khoản
- **Vị trí**: [`antigravity-relay/src/storage/account_store.rs:L51-L57`](file:///mnt/181EC3061EC2DBBE/DT/Code/PJ/antigravity-account-manager/antigravity-relay/src/storage/account_store.rs#L51-L57)
- **Mức độ**: Cao (High)
- **Nguyên nhân và cơ chế**:
  - Thao tác xóa file nhận chuỗi `id` và thực hiện ghép đường dẫn `self.base_dir.join(format!("{}.json", id))`.
  - Không có bất kỳ bước kiểm tra hoặc làm sạch ký tự điều hướng (`../`).
- **Rủi ro**:
  Kẻ tấn công có thể truyền tham số `account_id` dạng `../../config` để xóa các file cấu hình `.json` khác trong thư mục cá nhân của người dùng.
- **Hướng khắc phục**:
  Xác thực nghiêm ngặt giá trị `id` chỉ chứa ký tự chữ, số và dấu gạch nối (định dạng UUID chuẩn). Từ chối mọi chuỗi có chứa ký tự phân cách thư mục `/`, `\` hoặc tiền tố `..`.

---

### 3.3. Lỗ hổng OAuth CSRF do thiếu kiểm tra tham số state
- **Vị trí**: [`antigravity-relay/src/proxy/server.rs:L285-L306`](file:///mnt/181EC3061EC2DBBE/DT/Code/PJ/antigravity-account-manager/antigravity-relay/src/proxy/server.rs#L285-L306)
- **Mức độ**: Trung bình (Medium)
- **Nguyên nhân và cơ chế**:
  - Khi bắt đầu phiên đăng nhập Google OAuth, một chuỗi `state` ngẫu nhiên được sinh ra nhưng không được lưu lại trong bộ nhớ của daemon.
  - Endpoint callback tiếp nhận phản hồi từ Google nhưng struct `OAuthCallbackQuery` không định nghĩa trường `state`, dẫn đến việc bỏ qua hoàn toàn bước kiểm tra tính hợp lệ của phiên đăng nhập.
- **Rủi ro**:
  Kẻ tấn công có thể lợi dụng sơ hở để thực hiện tấn công OAuth CSRF, lừa người dùng liên kết tài khoản Google của kẻ tấn công vào daemon hoặc chiếm đoạt phiên đăng nhập.
- **Hướng khắc phục**:
  Lưu trữ chuỗi `state` trong một tập hợp bộ nhớ tạm thời có kèm thời gian sống (TTL). Trong endpoint callback, bắt buộc phải đối chiếu `state` và xóa ngay sau khi xác nhận thành công.

---

### 3.4. Nguy cơ DOM-based XSS trong bảng điều khiển
- **Vị trí**: [`antigravity-relay/src/proxy/ui.rs:L285-L390`](file:///mnt/181EC3061EC2DBBE/DT/Code/PJ/antigravity-account-manager/antigravity-relay/src/proxy/ui.rs#L285-L390)
- **Mức độ**: Trung bình (Medium)
- **Nguyên nhân và cơ chế**:
  - Dữ liệu tài khoản nhận từ API (`email`, `id`, `name`) được nối trực tiếp vào chuỗi template literal và gán thẳng vào `innerHTML`, cũng như đặt trong các sự kiện inline `onclick`.
- **Rủi ro**:
  Nếu xuất hiện dữ liệu chứa ký tự đặc biệt hoặc payload script độc hại (ví dụ qua API thêm tài khoản), mã JavaScript tùy ý có thể được thực thi trong trình duyệt, dẫn đến nguy cơ đánh cắp token hoặc thao túng dịch vụ.
- **Hướng khắc phục**:
  Áp dụng hàm escape HTML trước khi render dữ liệu động vào DOM hoặc chuyển sang sử dụng các phương thức DOM an toàn như `textContent` và `addEventListener`.

---

### 3.5. Cập nhật phần mềm không kiểm tra tính toàn vẹn
- **Vị trí**: [`antigravity-relay/src/cli.rs:L366-L390`](file:///mnt/181EC3061EC2DBBE/DT/Code/PJ/antigravity-account-manager/antigravity-relay/src/cli.rs#L366-L390), [`install.sh:L45-L53`](file:///mnt/181EC3061EC2DBBE/DT/Code/PJ/antigravity-account-manager/install.sh#L45-L53)
- **Mức độ**: Trung bình (Medium)
- **Nguyên nhân và cơ chế**:
  - Lệnh `agyr update` tải script bash trực tiếp từ GitHub và thực thi mà không có cơ chế xác minh tính toàn vẹn.
  - Script cài đặt tải file nhị phân nén và giải nén trực tiếp vào `~/.local/bin` mà không đối chiếu mã băm SHA256 hay chữ ký số.
- **Rủi ro**:
  Nguy cơ bị tấn công chuỗi cung ứng (supply chain) hoặc giả mạo đường truyền nếu DNS/mạng bị xâm nhập.
- **Hướng khắc phục**:
  Cung cấp file danh sách mã băm chính thức cho mỗi bản phát hành và bổ sung bước kiểm tra `sha256sum` tự động trước khi thay thế file nhị phân.

---

### 3.6. Rủi ro timing attack khi kiểm tra API key
- **Vị trí**: [`antigravity-relay/src/proxy/server.rs:L42-L51`](file:///mnt/181EC3061EC2DBBE/DT/Code/PJ/antigravity-account-manager/antigravity-relay/src/proxy/server.rs#L42-L51)
- **Mức độ**: Thấp (Low)
- **Nguyên nhân và cơ chế**:
  - Sử dụng toán tử `==` để so sánh chuỗi Bearer token.
  - Phép so sánh trả về kết quả sai ngay khi gặp ký tự đầu tiên không trùng khớp, dẫn đến sự khác biệt về thời gian phản hồi giữa các chuỗi token khác nhau.
- **Rủi ro**:
  Kẻ tấn công có thể đo đạc thời gian phản hồi liên tục qua mạng để suy đoán dần từng ký tự của khóa bí mật.
- **Hướng khắc phục**:
  Sử dụng hàm so sánh thời gian cố định (constant-time comparison) để thời gian kiểm tra luôn đồng nhất.

---

### 3.7. Ghi đè file shell profile không đảm bảo tính nguyên tử
- **Vị trí**: [`antigravity-relay/src/storage/account_switcher.rs:L95-L117`](file:///mnt/181EC3061EC2DBBE/DT/Code/PJ/antigravity-account-manager/antigravity-relay/src/storage/account_switcher.rs#L95-L117), [`antigravity-relay/src/proxy/cli_sync.rs:L71-L94`](file:///mnt/181EC3061EC2DBBE/DT/Code/PJ/antigravity-account-manager/antigravity-relay/src/proxy/cli_sync.rs#L71-L94)
- **Mức độ**: Thấp (Low)
- **Nguyên nhân và cơ chế**:
  - Ghi đè trực tiếp nội dung mới lên `.bashrc` và `.zshrc` mà không dùng file tạm hoặc tạo bản sao lưu dự phòng.
- **Rủi ro**:
  Nếu tiến trình bị ngắt đột ngột giữa lúc đang ghi, file cấu hình shell của người dùng có thể bị lỗi cú pháp hoặc mất dữ liệu.
- **Hướng khắc phục**:
  Ghi nội dung ra file tạm trước rồi thực hiện đổi tên nguyên tử (atomic rename), đồng thời tạo bản backup an toàn trước khi chỉnh sửa.
