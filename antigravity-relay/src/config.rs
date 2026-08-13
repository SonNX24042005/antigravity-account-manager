use std::path::PathBuf;
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

        Self {
            host: "127.0.0.1".to_string(),
            port: 8045,
            master_key: "sk-antigravity-local-key".to_string(),
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
}
