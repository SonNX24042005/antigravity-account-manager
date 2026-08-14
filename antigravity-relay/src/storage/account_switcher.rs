use std::fs;
use anyhow::Result;
use crate::models::Account;
use crate::storage::ide_db::IdeDbSync;
use crate::storage::keyring_sync::KeyringSync;

pub struct AccountSwitcher;

impl AccountSwitcher {
    pub fn switch_active_account(account: &Account) -> Result<()> {
        // 1. Inject into OS Keyring (service=gemini, user=antigravity) used primarily by agy CLI
        let expires_at_rfc3339 = account.expires_at.to_rfc3339();
        let _ = KeyringSync::inject_keyring_credential(
            &account.access_token,
            &account.refresh_token,
            &expires_at_rfc3339,
        );

        // 2. Inject into all Antigravity IDE and Code SQLite DBs (state.vscdb)
        for db_path in IdeDbSync::find_antigravity_ide_db_paths() {
            let _ = IdeDbSync::inject_credential(
                &db_path,
                &account.email,
                &account.access_token,
                &account.refresh_token,
                account.expires_at.timestamp(),
                &account.device_profile.machine_id,
            );
        }

        // 3. Inject into native agy CLI auth credentials stores
        if let Some(home) = dirs::home_dir() {
            let auth_data = serde_json::json!({
                "email": account.email,
                "access_token": account.access_token,
                "refresh_token": account.refresh_token,
                "expires_at": account.expires_at,
                "device_profile": account.device_profile
            });
            let auth_str = serde_json::to_string_pretty(&auth_data)?;

            // Locations for ~/.antigravity
            let ant_dir = home.join(".antigravity");
            let _ = fs::create_dir_all(&ant_dir);
            for name in ["auth.json", "credentials.json"] {
                let p = ant_dir.join(name);
                let _ = fs::write(&p, &auth_str);
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    let _ = fs::set_permissions(&p, fs::Permissions::from_mode(0o600));
                }
            }

            // Locations for ~/.gemini/antigravity-cli
            let gemini_cli_dir = home.join(".gemini").join("antigravity-cli");
            let _ = fs::create_dir_all(&gemini_cli_dir);
            for name in ["auth.json", "credentials.json"] {
                let p = gemini_cli_dir.join(name);
                let _ = fs::write(&p, &auth_str);
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    let _ = fs::set_permissions(&p, fs::Permissions::from_mode(0o600));
                }
            }

            // Clean proxy settings from config.json and settings.json so agy runs natively
            let config_file = ant_dir.join("config.json");
            let settings_file = ant_dir.join("settings.json");

            for file in [config_file, settings_file] {
                if file.exists() {
                    let mut json: serde_json::Value = serde_json::from_str(&fs::read_to_string(&file).unwrap_or_default()).unwrap_or_else(|_| serde_json::json!({}));
                    if let Some(obj) = json.as_object_mut() {
                        obj.remove("proxy");
                        obj.remove("http_proxy");
                        obj.remove("https_proxy");
                        obj.remove("base_url");
                        obj.remove("GOOGLE_GEMINI_BASE_URL");
                        obj.remove("security");
                    }
                    let _ = fs::write(&file, serde_json::to_string_pretty(&json)?);
                }
            }

            // Remove proxy env vars from shell profiles
            Self::clean_shell_profiles();
        }

        tracing::info!("[AccountSwitcher] Switched active account to: {}", account.email);
        Ok(())
    }

    fn clean_shell_profiles() {
        let home = match dirs::home_dir() {
            Some(h) => h,
            None => return,
        };

        for profile_path in [home.join(".bashrc"), home.join(".zshrc")] {
            if profile_path.exists() {
                if let Ok(content) = fs::read_to_string(&profile_path) {
                    let new_lines: Vec<&str> = content
                        .lines()
                        .filter(|line| {
                            !line.contains("GOOGLE_GEMINI_BASE_URL")
                                && !line.contains("ANTIGRAVITY_BASE_URL")
                                && !line.contains("Antigravity Relay Proxy Config")
                                && !(line.contains("8045") && (line.contains("HTTP_PROXY") || line.contains("HTTPS_PROXY")))
                        })
                        .collect();
                    let _ = fs::write(profile_path, new_lines.join("\n"));
                }
            }
        }
    }
}
