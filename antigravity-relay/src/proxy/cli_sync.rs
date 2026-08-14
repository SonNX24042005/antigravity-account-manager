use crate::storage::secure_file;
use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;

pub struct CliSync;

impl CliSync {
    pub fn get_agy_config_paths() -> Vec<PathBuf> {
        let home = match dirs::home_dir() {
            Some(h) => h,
            None => return vec![],
        };
        let ant_dir = home.join(".antigravity");
        vec![ant_dir.join("config.json"), ant_dir.join("settings.json")]
    }

    pub fn sync_agy_proxy(proxy_url: &str, master_key: &str) -> Result<()> {
        let parsed_url = reqwest::Url::parse(proxy_url).context("Invalid proxy URL")?;
        anyhow::ensure!(parsed_url.scheme() == "http", "Proxy URL must use HTTP");
        anyhow::ensure!(
            matches!(parsed_url.host_str(), Some("127.0.0.1") | Some("localhost")),
            "Proxy URL must target loopback"
        );
        anyhow::ensure!(
            parsed_url.username().is_empty() && parsed_url.password().is_none(),
            "Proxy URL must not contain credentials"
        );

        let paths = Self::get_agy_config_paths();
        let base_url = format!("{}/v1", proxy_url);

        for path in paths {
            let mut json: serde_json::Value = if path.exists() {
                let content = fs::read_to_string(&path).unwrap_or_default();
                serde_json::from_str(&content).unwrap_or_else(|_| serde_json::json!({}))
            } else {
                serde_json::json!({})
            };

            if let Some(parent) = path.parent() {
                let _ = fs::create_dir_all(parent);
            }

            if json.as_object().is_none() {
                json = serde_json::json!({});
            }
            json["proxy"] = serde_json::Value::String(proxy_url.to_string());
            json["api_key"] = serde_json::Value::String(master_key.to_string());
            json["base_url"] = serde_json::Value::String(base_url.clone());
            json["GOOGLE_GEMINI_BASE_URL"] = serde_json::Value::String(base_url.clone());

            // Force agy CLI security auth mode to gemini-api-key
            let sec = json
                .as_object_mut()
                .unwrap()
                .entry("security")
                .or_insert(serde_json::json!({}));
            if sec.as_object().is_none() {
                *sec = serde_json::json!({});
            }
            let auth = sec
                .as_object_mut()
                .unwrap()
                .entry("auth")
                .or_insert(serde_json::json!({}));
            if auth.as_object().is_none() {
                *auth = serde_json::json!({});
            }
            if let Some(auth_obj) = auth.as_object_mut() {
                auth_obj.insert(
                    "selectedType".to_string(),
                    serde_json::Value::String("gemini-api-key".to_string()),
                );
            }

            let json_str = serde_json::to_string_pretty(&json)?;
            let mode = secure_file::existing_mode_or(&path, 0o600);
            secure_file::atomic_write(&path, json_str.as_bytes(), mode)
                .context("Failed to write agy CLI config")?;
            tracing::info!(
                "[CLI Sync] Successfully injected proxy ({}) into agy CLI config at {:?}",
                proxy_url,
                path
            );
        }

        Self::sync_shell_profile(proxy_url);
        Ok(())
    }

    fn sync_shell_profile(proxy_url: &str) {
        let home = match dirs::home_dir() {
            Some(h) => h,
            None => return,
        };

        let profiles = [home.join(".bashrc"), home.join(".zshrc")];
        let export_lines = format!(
            "\n# Antigravity Relay Proxy Config\nexport GOOGLE_GEMINI_BASE_URL={}/v1\nexport ANTIGRAVITY_BASE_URL={}/v1\n# End Antigravity Relay Proxy Config\n",
            proxy_url, proxy_url
        );

        for profile in profiles {
            if profile.exists() {
                if let Ok(content) = fs::read_to_string(&profile) {
                    if !content.contains("Antigravity Relay Proxy Config") {
                        let mut new_content = content;
                        new_content.push_str(&export_lines);
                        let mode = secure_file::existing_mode_or(&profile, 0o600);
                        if let Err(error) = secure_file::backup(&profile) {
                            tracing::warn!(
                                "[CLI Sync] Refusing to update {:?} without a backup: {}",
                                profile,
                                error
                            );
                            continue;
                        }
                        if let Err(error) =
                            secure_file::atomic_write(&profile, new_content.as_bytes(), mode)
                        {
                            tracing::warn!("[CLI Sync] Failed to update {:?}: {}", profile, error);
                        }
                    }
                }
            }
        }
    }
}
