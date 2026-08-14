use std::fs;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use anyhow::{Context, Result};

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
        if !Self::is_port_in_use(8045) {
            println!("[agyr] Dịch vụ chưa chạy, đang tự động khởi động nền...");
            Self::start_service()?;
            std::thread::sleep(std::time::Duration::from_millis(600));
        }

        let url = "http://127.0.0.1:8045";
        println!("[agyr] Đang mở bảng điều khiển tại: {}", url);

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
        Some(home.join(".config").join("systemd").join("user").join("antigravity-relay.service"))
    }

    fn is_port_in_use(port: u16) -> bool {
        std::net::TcpListener::bind(format!("127.0.0.1:{}", port)).is_err()
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
        if Self::is_port_in_use(8045) {
            println!("[agyr] Dịch vụ Antigravity Relay đang chạy tại http://127.0.0.1:8045");
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

        if Self::is_port_in_use(8045) {
            println!("[agyr] Dịch vụ đã khởi chạy thành công tại http://127.0.0.1:8045");
        } else {
            println!("[agyr] Đã gửi lệnh khởi chạy. Vui lòng kiểm tra lại bằng lệnh 'agyr status'.");
        }

        Ok(())
    }

    fn stop_service() -> Result<()> {
        println!("[agyr] Đang dừng dịch vụ Antigravity Relay...");

        // Stop systemd unit if active
        let _ = Command::new("systemctl")
            .args(["--user", "stop", "antigravity-relay.service"])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();

        // Kill any processes on port 8045
        let _ = Command::new("fuser")
            .args(["-k", "8045/tcp"])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();

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

        let is_running = Self::is_port_in_use(8045);
        let is_sysd_active = Self::is_systemd_service_active();
        let is_sysd_enabled = Self::is_systemd_service_enabled();

        if is_running {
            println!("* Trạng thái:           Đang hoạt động (Running)");
            println!("* Địa chỉ máy chủ:      http://127.0.0.1:8045");
            if is_sysd_active {
                println!("* Trình quản lý:        systemd (user service)");
            } else {
                println!("* Trình quản lý:        background daemon process");
            }
        } else {
            println!("* Trạng thái:           Đã dừng (Stopped)");
        }

        println!("* Tự khởi động (boot):  {}", if is_sysd_enabled { "Đã bật (Auto-start on boot)" } else { "Đang tắt" });
        println!("=====================================================");

        if is_running {
            println!("Giao diện quản lý: http://127.0.0.1:8045");
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

        let bin_str = bin_path.to_str().unwrap_or("/home/samer/.local/bin/antigravity-relay");

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
            bin_str
        );

        fs::write(&service_path, service_content)?;
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

        println!("[agyr] Đã cài đặt lệnh 'agyr' và 'antigravity-relay' vào {:?}", target_bin);
        println!("       Bạn có thể dùng lệnh 'agyr' ở bất kỳ đâu trong terminal.");
        Ok(())
    }

    fn print_help() {
        println!("Antigravity Relay CLI Manager (agyr)");
        println!();
        println!("Cách sử dụng:");
        println!("  agyr              Tự động bật dịch vụ (nếu chưa chạy) và mở giao diện web");
        println!("  agyr start        Khởi chạy dịch vụ chạy ngầm");
        println!("  agyr autostart    Bật tự động chạy liên tục cùng hệ thống (kể cả restart máy)");
        println!("  agyr stop         Dừng dịch vụ đang chạy");
        println!("  agyr restart      Khởi động lại dịch vụ");
        println!("  agyr status       Xem trạng thái hoạt động của dịch vụ");
        println!("  agyr disable      Tắt chế độ tự khởi động cùng máy");
        println!("  agyr install      Cài đặt lệnh agyr vào ~/.local/bin");
        println!("  agyr run          Chạy trực tiếp trên terminal hiện tại (foreground)");
        println!();
    }
}
