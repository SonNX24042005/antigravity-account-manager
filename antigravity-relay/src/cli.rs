use anyhow::{Context, Result};
use sha2::{Digest, Sha256};
use std::fs;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::path::PathBuf;
use std::process::{Command, Stdio};

pub struct Cli;

impl Cli {
    pub fn handle_args(args: &[String]) -> Option<Result<()>> {
        if args.len() < 2 {
            return Some(Self::open_dashboard());
        }

        let cmd = args[1].to_lowercase();
        match cmd.as_str() {
            "run" | "daemon" => None,
            "open" | "ui" | "web" => Some(Self::open_dashboard()),
            "start" => Some(Self::start_service()),
            "stop" => Some(Self::stop_service()),
            "restart" => Some(Self::restart_service()),
            "status" => Some(Self::status_service()),
            "enable" | "autostart" => Some(Self::enable_autostart()),
            "disable" => Some(Self::disable_autostart()),
            "install" => Some(Self::install_binary()),
            "update" | "upgrade" => Some(Self::update_binary()),
            "version" | "-v" | "--version" => {
                println!("agyr v1.0.0 (Antigravity Relay Manager)");
                Some(Ok(()))
            }
            "help" | "--help" | "-h" => {
                Self::print_help();
                Some(Ok(()))
            }
            unknown => {
                println!("Lệnh không hợp lệ: '{}'\n", unknown);
                Self::print_help();
                Some(Ok(()))
            }
        }
    }

    fn open_dashboard() -> Result<()> {
        let port = crate::config::Config::default().port;
        if !Self::is_relay_service_running(port) {
            anyhow::ensure!(
                !Self::is_port_in_use(port),
                "Cổng {} đang bị một dịch vụ không xác định chiếm dụng",
                port
            );
            println!("[agyr] Dịch vụ chưa chạy, đang tự động khởi động nền...");
            Self::start_service()?;
            std::thread::sleep(std::time::Duration::from_millis(600));
        }
        anyhow::ensure!(
            Self::is_relay_service_running(port),
            "Không thể xác minh dịch vụ Antigravity Relay trên cổng {}",
            port
        );

        let bootstrap_token = Self::create_browser_bootstrap(port)?;
        let url = format!("http://127.0.0.1:{port}/#bootstrap={bootstrap_token}");
        println!("[agyr] Đang mở bảng điều khiển tại http://127.0.0.1:{port}");

        #[cfg(target_os = "linux")]
        {
            let _ = Command::new("xdg-open")
                .arg(url)
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn();
        }

        #[cfg(target_os = "macos")]
        {
            let _ = Command::new("open")
                .arg(url)
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn();
        }

        #[cfg(target_os = "windows")]
        {
            let _ = Command::new("cmd")
                .args(["/c", "start", url])
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn();
        }

        Ok(())
    }

    fn get_installed_bin_path() -> PathBuf {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".local")
            .join("bin")
            .join("antigravity-relay")
    }

    fn get_service_file_path() -> Option<PathBuf> {
        let home = dirs::home_dir()?;
        Some(
            home.join(".config")
                .join("systemd")
                .join("user")
                .join("antigravity-relay.service"),
        )
    }

    fn is_port_in_use(port: u16) -> bool {
        std::net::TcpListener::bind(format!("127.0.0.1:{}", port)).is_err()
    }

    fn is_relay_service_running(port: u16) -> bool {
        let Ok(response) = Self::relay_http_request(port, "GET", "/api/health") else {
            return false;
        };
        response.starts_with("HTTP/1.1 200")
            && response.contains("application/json")
            && response
                .to_ascii_lowercase()
                .contains("x-antigravity-relay: 1")
    }

    fn create_browser_bootstrap(port: u16) -> Result<String> {
        let response = Self::relay_http_request(port, "POST", "/api/session/bootstrap")?;
        Self::parse_browser_bootstrap_response(&response)
    }

    fn parse_browser_bootstrap_response(response: &str) -> Result<String> {
        anyhow::ensure!(
            response.starts_with("HTTP/1.1 200"),
            "Daemon từ chối tạo phiên trình duyệt"
        );
        let body = response
            .split_once("\r\n\r\n")
            .map(|(_, body)| body)
            .context("Phản hồi tạo phiên trình duyệt không hợp lệ")?;
        let json: serde_json::Value = serde_json::from_str(body)?;
        let token = json["bootstrap_token"]
            .as_str()
            .context("Phản hồi thiếu bootstrap token")?;
        anyhow::ensure!(
            !token.is_empty()
                && token.len() <= 128
                && token
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_'),
            "Bootstrap token không hợp lệ"
        );
        Ok(token.to_string())
    }

    fn relay_http_request(port: u16, method: &str, path: &str) -> Result<String> {
        let address = SocketAddr::from(([127, 0, 0, 1], port));
        let mut stream =
            TcpStream::connect_timeout(&address, std::time::Duration::from_millis(800))?;
        let _ = stream.set_read_timeout(Some(std::time::Duration::from_millis(500)));
        let _ = stream.set_write_timeout(Some(std::time::Duration::from_millis(500)));
        let key = crate::config::Config::default().master_key;
        anyhow::ensure!(
            key.bytes().all(|byte| byte.is_ascii_graphic()),
            "Master key chứa ký tự không hợp lệ"
        );
        anyhow::ensure!(
            matches!(method, "GET" | "POST")
                && path.starts_with('/')
                && !path.contains(['\r', '\n']),
            "Yêu cầu nội bộ không hợp lệ"
        );
        let request = format!(
            "{method} {path} HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nAuthorization: Bearer {key}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
        );
        stream.write_all(request.as_bytes())?;

        let mut response = Vec::new();
        stream.take(64 * 1024).read_to_end(&mut response)?;
        Ok(String::from_utf8(response).context("Phản hồi daemon không phải UTF-8")?)
    }

    fn is_systemd_service_active() -> bool {
        let output = Command::new("systemctl")
            .args(["--user", "is-active", "antigravity-relay.service"])
            .stdin(Stdio::null())
            .stderr(Stdio::null())
            .output();
        if let Ok(out) = output {
            let status = String::from_utf8_lossy(&out.stdout).trim().to_string();
            status == "active"
        } else {
            false
        }
    }

    fn is_systemd_service_enabled() -> bool {
        let output = Command::new("systemctl")
            .args(["--user", "is-enabled", "antigravity-relay.service"])
            .stdin(Stdio::null())
            .stderr(Stdio::null())
            .output();
        if let Ok(out) = output {
            let status = String::from_utf8_lossy(&out.stdout).trim().to_string();
            status == "enabled"
        } else {
            false
        }
    }

    fn start_service() -> Result<()> {
        let port = crate::config::Config::default().port;
        if Self::is_port_in_use(port) {
            anyhow::ensure!(
                Self::is_relay_service_running(port),
                "Cổng {} đang bị một dịch vụ không xác định chiếm dụng",
                port
            );
            println!("[agyr] Dịch vụ Antigravity Relay đang chạy tại http://127.0.0.1:{port}");
            return Ok(());
        }

        if Self::is_systemd_service_enabled() {
            println!("[agyr] Đang khởi chạy dịch vụ qua systemd...");
            let _ = Command::new("systemctl")
                .args(["--user", "start", "antigravity-relay.service"])
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status();
        } else {
            let exe = std::env::current_exe()?;
            println!("[agyr] Đang khởi chạy tiến trình nền...");
            Command::new(exe)
                .arg("run")
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
                .context("Không thể khởi chạy tiến trình nền")?;
        }

        std::thread::sleep(std::time::Duration::from_millis(600));

        if Self::is_relay_service_running(port) {
            println!("[agyr] Dịch vụ đã khởi chạy thành công tại http://127.0.0.1:{port}");
        } else if Self::is_port_in_use(port) {
            anyhow::bail!(
                "Cổng {} đang phản hồi nhưng không phải Antigravity Relay",
                port
            );
        } else {
            println!(
                "[agyr] Đã gửi lệnh khởi chạy. Vui lòng kiểm tra lại bằng lệnh 'agyr status'."
            );
        }

        Ok(())
    }

    fn stop_service() -> Result<()> {
        println!("[agyr] Đang dừng dịch vụ Antigravity Relay...");
        let port = crate::config::Config::default().port;
        let was_relay_running = Self::is_relay_service_running(port);

        // Stop systemd unit if active
        let _ = Command::new("systemctl")
            .args(["--user", "stop", "antigravity-relay.service"])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();

        // Stop only a previously authenticated relay process left outside systemd.
        if was_relay_running && Self::is_relay_service_running(port) {
            let _ = Command::new("fuser")
                .args(["-k", &format!("{port}/tcp")])
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status();
        }

        std::thread::sleep(std::time::Duration::from_millis(300));
        println!("[agyr] Đã dừng dịch vụ Antigravity Relay.");
        Ok(())
    }

    fn restart_service() -> Result<()> {
        Self::stop_service()?;
        std::thread::sleep(std::time::Duration::from_millis(500));
        Self::start_service()?;
        Ok(())
    }

    fn status_service() -> Result<()> {
        println!("=====================================================");
        println!("   Trạng thái dịch vụ Antigravity Relay (agyr)");
        println!("=====================================================");

        let port = crate::config::Config::default().port;
        let is_running = Self::is_relay_service_running(port);
        let has_port_conflict = !is_running && Self::is_port_in_use(port);
        let is_sysd_active = Self::is_systemd_service_active();
        let is_sysd_enabled = Self::is_systemd_service_enabled();

        if is_running {
            println!("* Trạng thái:           Đang hoạt động (Running)");
            println!("* Địa chỉ máy chủ:      http://127.0.0.1:{port}");
            if is_sysd_active {
                println!("* Trình quản lý:        systemd (user service)");
            } else {
                println!("* Trình quản lý:        background daemon process");
            }
        } else {
            println!("* Trạng thái:           Đã dừng (Stopped)");
            if has_port_conflict {
                println!("* Cảnh báo:             Cổng {port} đang do dịch vụ khác sử dụng");
            }
        }

        println!(
            "* Tự khởi động (boot):  {}",
            if is_sysd_enabled {
                "Đã bật (Auto-start on boot)"
            } else {
                "Đang tắt"
            }
        );
        println!("=====================================================");

        if is_running {
            println!("Giao diện quản lý: http://127.0.0.1:{port}");
            println!("Gõ 'agy' trong terminal để tự động dùng tài khoản có quota tốt nhất.");
        } else {
            println!("Dùng 'agyr' hoặc 'agyr start' để chạy dịch vụ và mở giao diện.");
            println!("Dùng 'agyr autostart' để tự động chạy liên tục kể cả khi restart máy.");
        }

        Ok(())
    }

    fn enable_autostart() -> Result<()> {
        let service_path = match Self::get_service_file_path() {
            Some(p) => p,
            None => return Err(anyhow::anyhow!("Không tìm thấy thư mục home")),
        };

        if let Some(parent) = service_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Ensure binary is installed in ~/.local/bin/antigravity-relay
        let bin_path = Self::get_installed_bin_path();
        if !bin_path.exists() {
            let current_exe = std::env::current_exe()?;
            if let Some(bin_parent) = bin_path.parent() {
                fs::create_dir_all(bin_parent)?;
            }
            fs::copy(&current_exe, &bin_path)?;
            let _ = Command::new("chmod")
                .args(["+x", bin_path.to_str().unwrap_or_default()])
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status();
        }

        let bin_arg = Self::quote_systemd_exec_path(&bin_path)?;

        let service_content = format!(
            "[Unit]\n\
            Description=Antigravity Relay Account Manager Daemon\n\
            After=network.target\n\n\
            [Service]\n\
            Type=simple\n\
            ExecStart={} run\n\
            Restart=always\n\
            RestartSec=3\n\
            Environment=RUST_LOG=info\n\n\
            [Install]\n\
            WantedBy=default.target\n",
            bin_arg
        );

        crate::storage::secure_file::atomic_write(
            &service_path,
            service_content.as_bytes(),
            0o600,
        )?;
        println!("[agyr] Đã tạo service file tại {:?}", service_path);

        // Reload systemd & enable
        let _ = Command::new("systemctl")
            .args(["--user", "daemon-reload"])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();

        let status = Command::new("systemctl")
            .args(["--user", "enable", "--now", "antigravity-relay.service"])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()?;

        if status.success() {
            println!("[agyr] Đã kích hoạt chế độ tự động chạy cùng hệ thống (Auto-start on boot).");
            println!("       - Tự động chạy nền liên tục kể cả khi khởi động lại máy tính.");
            println!("       - Tự động hồi phục và bật lại sau 3 giây nếu bị dừng.");
            println!("       - Dùng lệnh 'agyr stop' hoặc 'agyr disable' khi muốn dừng.");
        } else {
            println!("[agyr] Có lỗi khi kích hoạt systemd service.");
        }

        Ok(())
    }

    fn quote_systemd_exec_path(path: &PathBuf) -> Result<String> {
        let value = path
            .to_str()
            .context("Đường dẫn binary không phải UTF-8 hợp lệ")?;
        anyhow::ensure!(
            !value.chars().any(char::is_control),
            "Đường dẫn binary chứa ký tự điều khiển không an toàn"
        );
        let escaped = value
            .replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('%', "%%")
            .replace('$', "$$");
        Ok(format!("\"{}\"", escaped))
    }

    fn disable_autostart() -> Result<()> {
        println!("[agyr] Đang tắt chế độ tự động chạy cùng hệ thống...");
        let _ = Command::new("systemctl")
            .args(["--user", "disable", "--now", "antigravity-relay.service"])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();

        if let Some(service_path) = Self::get_service_file_path() {
            if service_path.exists() {
                let _ = fs::remove_file(service_path);
            }
        }

        let _ = Command::new("systemctl")
            .args(["--user", "daemon-reload"])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();

        println!("[agyr] Đã tắt chế độ tự khởi động cùng hệ thống.");
        Ok(())
    }

    fn install_binary() -> Result<()> {
        let current_exe = std::env::current_exe()?;
        let target_bin = Self::get_installed_bin_path();

        if let Some(parent) = target_bin.parent() {
            fs::create_dir_all(parent)?;
        }

        if current_exe != target_bin {
            fs::copy(&current_exe, &target_bin)?;
            let _ = Command::new("chmod")
                .args(["+x", target_bin.to_str().unwrap_or_default()])
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status();
        }

        // Create short symlink 'agyr'
        if let Some(parent) = target_bin.parent() {
            let symlink_path = parent.join("agyr");
            let _ = fs::remove_file(&symlink_path);
            #[cfg(unix)]
            {
                use std::os::unix::fs::symlink;
                let _ = symlink(&target_bin, &symlink_path);
            }
        }

        println!(
            "[agyr] Đã cài đặt lệnh 'agyr' và 'antigravity-relay' vào {:?}",
            target_bin
        );
        println!("       Bạn có thể dùng lệnh 'agyr' ở bất kỳ đâu trong terminal.");
        Ok(())
    }

    fn update_binary() -> Result<()> {
        println!("[agyr] Đang kiểm tra và tải bản cập nhật mới nhất từ GitHub...");

        let port = crate::config::Config::default().port;
        let was_running = Self::is_relay_service_running(port);

        Self::download_verified_release()?;
        println!("[agyr] Cập nhật phiên bản mới nhất thành công!");
        if was_running {
            println!("[agyr] Đang khởi động lại dịch vụ với phiên bản mới...");
            Self::restart_service()?;
        }

        Ok(())
    }

    fn download_verified_release() -> Result<()> {
        let os = match std::env::consts::OS {
            "macos" => "darwin",
            value => value,
        };
        let arch = match std::env::consts::ARCH {
            "x86_64" => "x86_64",
            "aarch64" => "aarch64",
            value => value,
        };
        let asset_name = format!("antigravity-relay-{}-{}.tar.gz", os, arch);
        let asset_url = format!(
            "https://github.com/SonNX24042005/antigravity-account-manager/releases/latest/download/{}",
            asset_name
        );
        let checksum_url = format!("{}.sha256", asset_url);
        let temp_dir = std::env::temp_dir().join(format!("agyr-update-{}", uuid::Uuid::new_v4()));
        fs::create_dir(&temp_dir).context("Không thể tạo thư mục cập nhật tạm")?;

        let result = (|| -> Result<()> {
            let archive = temp_dir.join(&asset_name);
            let checksum_file = temp_dir.join(format!("{}.sha256", asset_name));
            Self::download_file(&asset_url, &archive)?;
            Self::download_file(&checksum_url, &checksum_file)?;

            let checksum_text = fs::read_to_string(&checksum_file)
                .context("Không thể đọc checksum của bản phát hành")?;
            let expected = Self::parse_sha256(&checksum_text)?;
            let actual = Self::sha256_file(&archive)?;
            anyhow::ensure!(
                actual.eq_ignore_ascii_case(&expected),
                "Checksum SHA-256 của bản cập nhật không khớp"
            );

            let list_output = Command::new("tar")
                .args(["-tzf"])
                .arg(&archive)
                .output()
                .context("Không thể kiểm tra nội dung gói cập nhật")?;
            anyhow::ensure!(
                list_output.status.success(),
                "Gói cập nhật không phải tar.gz hợp lệ"
            );
            let listing = String::from_utf8_lossy(&list_output.stdout);
            let entries: Vec<&str> = listing
                .lines()
                .filter(|line| !line.trim().is_empty())
                .collect();
            anyhow::ensure!(
                entries.len() == 1
                    && matches!(entries[0], "antigravity-relay" | "./antigravity-relay"),
                "Gói cập nhật chứa đường dẫn không mong đợi"
            );

            let status = Command::new("tar")
                .args(["-xzf"])
                .arg(&archive)
                .arg("-C")
                .arg(&temp_dir)
                .status()
                .context("Không thể giải nén bản cập nhật")?;
            anyhow::ensure!(status.success(), "Không thể giải nén bản cập nhật");

            let extracted = temp_dir.join("antigravity-relay");
            let metadata = fs::symlink_metadata(&extracted)
                .context("Gói cập nhật thiếu file antigravity-relay")?;
            anyhow::ensure!(
                metadata.file_type().is_file(),
                "Binary cập nhật không phải file thường"
            );
            let binary = fs::read(&extracted).context("Không thể đọc binary cập nhật")?;
            crate::storage::secure_file::atomic_write(
                &Self::get_installed_bin_path(),
                &binary,
                0o755,
            )?;
            Ok(())
        })();

        let _ = fs::remove_dir_all(&temp_dir);
        result
    }

    fn download_file(url: &str, destination: &PathBuf) -> Result<()> {
        let status = Command::new("curl")
            .args([
                "--proto",
                "=https",
                "--tlsv1.2",
                "-fsSL",
                "--retry",
                "3",
                "--max-time",
                "120",
            ])
            .arg(url)
            .arg("-o")
            .arg(destination)
            .status()
            .with_context(|| format!("Không thể tải {}", url))?;
        anyhow::ensure!(status.success(), "Tải bản cập nhật thất bại");
        Ok(())
    }

    fn parse_sha256(value: &str) -> Result<String> {
        let checksum = value
            .split_whitespace()
            .next()
            .ok_or_else(|| anyhow::anyhow!("File checksum trống"))?;
        anyhow::ensure!(
            checksum.len() == 64 && checksum.bytes().all(|byte| byte.is_ascii_hexdigit()),
            "Checksum SHA-256 không hợp lệ"
        );
        Ok(checksum.to_ascii_lowercase())
    }

    fn sha256_file(path: &PathBuf) -> Result<String> {
        let mut file = fs::File::open(path).context("Không thể mở gói cập nhật")?;
        let mut hasher = Sha256::new();
        let mut buffer = [0_u8; 64 * 1024];
        loop {
            let read = file.read(&mut buffer)?;
            if read == 0 {
                break;
            }
            hasher.update(&buffer[..read]);
        }
        Ok(format!("{:x}", hasher.finalize()))
    }

    fn print_help() {
        println!("Antigravity Relay CLI Manager (agyr)");
        println!();
        println!("Cách sử dụng:");
        println!("  agyr              Tự động bật dịch vụ (nếu chưa chạy) và mở giao diện web");
        println!("  agyr update       Cập nhật agyr lên phiên bản mới nhất từ GitHub");
        println!("  agyr start        Khởi chạy dịch vụ chạy ngầm");
        println!("  agyr autostart    Bật tự động chạy liên tục cùng hệ thống (kể cả restart máy)");
        println!("  agyr stop         Dừng dịch vụ đang chạy");
        println!("  agyr restart      Khởi động lại dịch vụ");
        println!("  agyr status       Xem trạng thái hoạt động của dịch vụ");
        println!("  agyr version      Xem phiên bản hiện tại");
        println!("  agyr disable      Tắt chế độ tự khởi động cùng máy");
        println!("  agyr install      Cài đặt lệnh agyr vào ~/.local/bin");
        println!("  agyr run          Chạy trực tiếp trên terminal hiện tại (foreground)");
        println!();
    }
}

#[cfg(test)]
mod tests {
    use super::Cli;
    use std::path::PathBuf;

    #[test]
    fn accepts_only_well_formed_sha256() {
        let valid = "a".repeat(64);
        assert_eq!(
            Cli::parse_sha256(&format!("{}  release.tar.gz", valid)).unwrap(),
            valid
        );
        assert!(Cli::parse_sha256("not-a-checksum").is_err());
    }

    #[test]
    fn quotes_systemd_exec_paths() {
        let quoted = Cli::quote_systemd_exec_path(&PathBuf::from("/tmp/a b/%x$y")).unwrap();
        assert_eq!(quoted, "\"/tmp/a b/%%x$$y\"");
        assert!(Cli::quote_systemd_exec_path(&PathBuf::from("/tmp/a\nInjected=true")).is_err());
    }

    #[test]
    fn parses_browser_bootstrap_response() {
        let response = "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\n\r\n{\"bootstrap_token\":\"safe_token-123\"}";
        assert_eq!(
            Cli::parse_browser_bootstrap_response(response).unwrap(),
            "safe_token-123"
        );
        assert!(
            Cli::parse_browser_bootstrap_response("HTTP/1.1 401 Unauthorized\r\n\r\n").is_err()
        );
    }
}
