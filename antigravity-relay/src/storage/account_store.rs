use std::fs;
use std::path::PathBuf;
use crate::models::Account;
use anyhow::{Context, Result};

pub struct AccountStore {
    base_dir: PathBuf,
}

impl AccountStore {
    pub fn new(base_dir: PathBuf) -> Self {
        let _ = fs::create_dir_all(&base_dir);
        Self { base_dir }
    }

    pub fn load_all(&self) -> Result<Vec<Account>> {
        let mut accounts = Vec::new();
        if !self.base_dir.exists() {
            return Ok(accounts);
        }

        for entry in fs::read_dir(&self.base_dir).context("Failed to read accounts dir")? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().map_or(false, |ext| ext == "json") {
                if let Ok(content) = fs::read_to_string(&path) {
                    if let Ok(account) = serde_json::from_str::<Account>(&content) {
                        accounts.push(account);
                    }
                }
            }
        }

        Ok(accounts)
    }

    pub fn save(&self, account: &Account) -> Result<()> {
        fs::create_dir_all(&self.base_dir)?;
        let path = self.base_dir.join(format!("{}.json", account.id));
        let json = serde_json::to_string_pretty(account)?;
        fs::write(&path, json)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = fs::set_permissions(&path, fs::Permissions::from_mode(0o600));
        }
        Ok(())
    }

    #[allow(dead_code)]
    pub fn delete(&self, id: &str) -> Result<()> {
        let path = self.base_dir.join(format!("{}.json", id));
        if path.exists() {
            fs::remove_file(path)?;
        }
        Ok(())
    }
}
