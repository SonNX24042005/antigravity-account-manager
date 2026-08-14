use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub master_key: String,
    pub data_dir: PathBuf,
}

impl Default for Config {
    fn default() -> Self {
        let data_dir = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".antigravity-relay");

        let master_key = Self::load_or_generate_master_key(&data_dir);

        Self {
            host: "127.0.0.1".to_string(),
            port: 8045,
            master_key,
            data_dir,
        }
    }
}

impl Config {
    pub fn accounts_dir(&self) -> PathBuf {
        self.data_dir.join("accounts")
    }

    pub fn ensure_directories(&self) -> anyhow::Result<()> {
        std::fs::create_dir_all(self.accounts_dir())?;
        Ok(())
    }

    /// Load master API key from env var or persisted file. Generates a new random key on first run.
    fn load_or_generate_master_key(data_dir: &Path) -> String {
        // Priority 1: environment variable override
        if let Ok(key) = std::env::var("ANTIGRAVITY_MASTER_KEY") {
            if !key.is_empty() {
                return key;
            }
        }

        // Priority 2: persisted key file from a previous run
        let key_file = data_dir.join("master.key");
        if let Ok(content) = std::fs::read_to_string(&key_file) {
            let key = content.trim().to_string();
            if !key.is_empty() {
                return key;
            }
        }

        // Priority 3: generate new random key and persist it with restricted permissions
        let key = format!("sk-agyr-{}", uuid::Uuid::new_v4().to_string().replace('-', ""));
        let _ = std::fs::create_dir_all(data_dir);
        let _ = std::fs::write(&key_file, &key);
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = std::fs::set_permissions(&key_file, std::fs::Permissions::from_mode(0o600));
        }
        key
    }
}
