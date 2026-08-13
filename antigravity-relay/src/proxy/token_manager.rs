use std::sync::Arc;
use tokio::sync::RwLock;
use crate::models::Account;
use crate::storage::{AccountStore, AccountSwitcher};
use anyhow::{anyhow, Result};
use rand::Rng;

#[derive(Clone)]
pub struct TokenManager {
    accounts: Arc<RwLock<Vec<Account>>>,
    store: Arc<AccountStore>,
}

impl TokenManager {
    pub fn new(store: AccountStore) -> Self {
        let store_arc = Arc::new(store);
        let loaded = store_arc.load_all().unwrap_or_default();
        tracing::info!("[TokenManager] Loaded {} accounts into token pool", loaded.len());

        Self {
            accounts: Arc::new(RwLock::new(loaded)),
            store: store_arc,
        }
    }

    pub async fn sync_active_account_from_disk(&self) {
        let mut disk_email = None;
        if let Some(home) = dirs::home_dir() {
            let auth_file = home.join(".antigravity").join("auth.json");
            if auth_file.exists() {
                if let Ok(content) = std::fs::read_to_string(&auth_file) {
                    if let Ok(json_val) = serde_json::from_str::<serde_json::Value>(&content) {
                        disk_email = json_val["email"].as_str().map(|s| s.to_string());
                    }
                }
            }
        }

        let mut list = self.accounts.write().await;
        for acc in list.iter_mut() {
            if let Some(ref email) = disk_email {
                acc.is_active = acc.email == *email;
            }
        }
    }

    pub async fn list_accounts(&self) -> Vec<Account> {
        self.sync_active_account_from_disk().await;
        self.accounts.read().await.clone()
    }

    pub async fn add_account(&self, account: Account) -> Result<()> {
        self.store.save(&account)?;
        let mut list = self.accounts.write().await;
        // Replace existing account with same email if present
        list.retain(|a| a.email != account.email);
        list.push(account.clone());
        tracing::info!("[TokenManager] Added/Updated account to pool");
        
        // Auto-switch to newly added account
        let _ = crate::storage::AccountSwitcher::switch_active_account(&account);
        Ok(())
    }

    pub async fn select_highest_gemini_account(&self) -> Result<Account> {
        self.sync_active_account_from_disk().await;
        let list = self.accounts.read().await.clone();
        if list.is_empty() {
            return Err(anyhow!("No accounts in pool"));
        }

        let best = list.into_iter().max_by(|a, b| {
            let score_a = Self::get_gemini_score(a);
            let score_b = Self::get_gemini_score(b);
            score_a.partial_cmp(&score_b).unwrap_or(std::cmp::Ordering::Equal)
        }).ok_or_else(|| anyhow!("Failed to select best account"))?;

        AccountSwitcher::switch_active_account(&best)?;
        Ok(best)
    }

    fn get_gemini_score(account: &Account) -> f64 {
        for g in &account.quota_groups {
            if g.name.to_lowercase().contains("gemini") {
                for b in &g.buckets {
                    let w = b.window.to_uppercase();
                    if w.contains("5H") || w.contains("FIVE") {
                        return b.remaining_percentage;
                    }
                }
            }
        }
        account.quota_percentage
    }

    pub async fn switch_account(&self, target_id_or_email: &str) -> Result<Account> {
        let mut list = self.accounts.write().await;
        let mut target_account: Option<Account> = None;

        for acc in list.iter_mut() {
            if acc.id == target_id_or_email || acc.email == target_id_or_email {
                acc.is_active = true;
                target_account = Some(acc.clone());
            } else {
                acc.is_active = false;
            }
            let _ = self.store.save(acc);
        }

        if let Some(acc) = target_account {
            crate::storage::AccountSwitcher::switch_active_account(&acc)?;
            Ok(acc)
        } else {
            Err(anyhow!("Account not found: {}", target_id_or_email))
        }
    }

    pub async fn select_best_account(&self) -> Result<Account> {
        let list = self.accounts.read().await;
        let available: Vec<&Account> = list
            .iter()
            .filter(|a| a.is_active && !a.is_rate_limited())
            .collect();

        if available.is_empty() {
            return Err(anyhow!("No active, non-rate-limited accounts available in pool!"));
        }

        if available.len() == 1 {
            return Ok(available[0].clone());
        }

        // Power-of-2-Choices (P2C) selection based on quota percentage
        let mut rng = rand::thread_rng();
        let idx1 = rng.gen_range(0..available.len());
        let mut idx2 = rng.gen_range(0..available.len());
        while idx2 == idx1 && available.len() > 1 {
            idx2 = rng.gen_range(0..available.len());
        }

        let choice1 = available[idx1];
        let choice2 = available[idx2];

        if choice1.quota_percentage >= choice2.quota_percentage {
            Ok(choice1.clone())
        } else {
            Ok(choice2.clone())
        }
    }

    pub async fn mark_rate_limited(&self, email: &str, cooldown_seconds: i64) {
        let mut list = self.accounts.write().await;
        if let Some(account) = list.iter_mut().find(|a| a.email == email) {
            account.set_rate_limit(cooldown_seconds);
            account.quota_percentage = 0.0;
            tracing::warn!(
                "[CircuitBreaker] Account {} rate limited (429/403). Cooldown set for {}s",
                email,
                cooldown_seconds
            );
            let _ = self.store.save(account);
        }
    }

    pub async fn reset_cooldowns(&self) {
        let mut list = self.accounts.write().await;
        for acc in list.iter_mut() {
            acc.rate_limit_until = None;
            acc.is_active = true;
            let _ = self.store.save(acc);
        }
    }

    #[allow(dead_code)]
    pub async fn refresh_quotas(&self, client: &reqwest::Client) {
        let list = self.accounts.read().await.clone();
        for mut account in list {
            let (overall_pct, groups) = crate::proxy::quota::QuotaFetcher::fetch_account_quota_full(client, &account.access_token).await;
            if let Some(pct) = overall_pct {
                account.quota_percentage = pct;
            }
            account.quota_groups = groups.clone();

            let _ = self.store.save(&account);
            let mut write_list = self.accounts.write().await;
            if let Some(acc) = write_list.iter_mut().find(|a| a.id == account.id) {
                if let Some(pct) = overall_pct {
                    acc.quota_percentage = pct;
                }
                acc.quota_groups = groups;
            }
        }
    }
}
