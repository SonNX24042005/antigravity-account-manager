use crate::models::Account;
use crate::proxy::model_detector::{ModelDetector, TargetModelCategory};
use crate::storage::{AccountStore, AccountSwitcher};
use anyhow::{anyhow, Result};
use rand::Rng;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct TokenManager {
    accounts: Arc<RwLock<Vec<Account>>>,
    store: Arc<AccountStore>,
    model_detector: Arc<ModelDetector>,
}

impl TokenManager {
    const MAX_ACCOUNTS: usize = 100;

    pub fn new(store: AccountStore, data_dir: std::path::PathBuf) -> Self {
        let store_arc = Arc::new(store);
        let loaded = store_arc.load_all().unwrap_or_default();
        tracing::info!(
            "[TokenManager] Loaded {} accounts into token pool",
            loaded.len()
        );

        Self {
            accounts: Arc::new(RwLock::new(loaded)),
            store: store_arc,
            model_detector: Arc::new(ModelDetector::new(data_dir)),
        }
    }

    pub fn get_model_detector(&self) -> Arc<ModelDetector> {
        self.model_detector.clone()
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
        let mut list = self.accounts.write().await;
        let existing = list
            .iter()
            .find(|item| item.email == account.email)
            .cloned();
        anyhow::ensure!(
            existing.is_some() || list.len() < Self::MAX_ACCOUNTS,
            "Account limit reached ({})",
            Self::MAX_ACCOUNTS
        );

        self.store.save(&account)?;
        if let Some(previous) = existing {
            if previous.id != account.id {
                self.store.delete(&previous.id)?;
            }
        }
        list.retain(|a| a.email != account.email);
        list.push(account.clone());
        tracing::info!("[TokenManager] Added/Updated account to pool");

        let _ = crate::storage::AccountSwitcher::switch_active_account(&account);
        Ok(())
    }

    /// Automatically selects and switches to the account with the highest quota for the currently active model category
    pub async fn select_best_account_for_active_model(
        &self,
    ) -> Result<(Account, TargetModelCategory)> {
        self.sync_active_account_from_disk().await;
        let list = self.accounts.read().await.clone();
        if list.is_empty() {
            return Err(anyhow!("No accounts in pool"));
        }

        let target_category = self.model_detector.get_effective_category();

        let eligible: Vec<Account> = list
            .into_iter()
            .filter(|a| {
                !a.is_rate_limited()
                    && a.has_available_weekly_quota_for_category(target_category)
                    && a.get_effective_quota_for_category(target_category) > 0.0
            })
            .collect();

        if eligible.is_empty() {
            return Err(anyhow!(
                "No eligible accounts available with remaining weekly quota for {}",
                target_category.display_name()
            ));
        }

        let best = eligible
            .into_iter()
            .max_by(|a, b| {
                let score_a = a.get_effective_quota_for_category(target_category);
                let score_b = b.get_effective_quota_for_category(target_category);

                score_a
                    .partial_cmp(&score_b)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .ok_or_else(|| anyhow!("Failed to select best account"))?;

        AccountSwitcher::switch_active_account(&best)?;
        tracing::info!(
            "[TokenManager] Auto-selected account {} for category {:?} (score: {:.1}%)",
            best.email,
            target_category,
            best.get_effective_quota_for_category(target_category)
        );

        Ok((best, target_category))
    }

    #[allow(dead_code)]
    pub async fn select_highest_gemini_account(&self) -> Result<Account> {
        let (acc, _) = self.select_best_account_for_active_model().await?;
        Ok(acc)
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
            self.store.save(acc)?;
        }

        if let Some(acc) = target_account {
            crate::storage::AccountSwitcher::switch_active_account(&acc)?;
            Ok(acc)
        } else {
            Err(anyhow!("Account not found: {}", target_id_or_email))
        }
    }

    pub async fn delete_account(&self, target_id_or_email: &str) -> Result<String> {
        let removed_account = {
            let mut list = self.accounts.write().await;
            let pos = list
                .iter()
                .position(|a| a.id == target_id_or_email || a.email == target_id_or_email);
            if let Some(index) = pos {
                let removed = list[index].clone();
                self.store.delete(&removed.id)?;
                list.remove(index);
                tracing::info!("[TokenManager] Deleted account: {}", removed.email);
                Some(removed)
            } else {
                None
            }
        };

        if let Some(removed) = removed_account {
            if removed.is_active {
                let _ = self.select_best_account_for_active_model().await;
            }
            Ok(removed.email)
        } else {
            Err(anyhow!("Account not found: {}", target_id_or_email))
        }
    }

    pub async fn select_best_account(&self) -> Result<Account> {
        let list = self.accounts.read().await;
        let target_category = self.model_detector.get_effective_category();
        let available: Vec<&Account> = list
            .iter()
            .filter(|a| {
                a.is_active
                    && !a.is_rate_limited()
                    && a.has_available_weekly_quota_for_category(target_category)
                    && a.get_effective_quota_for_category(target_category) > 0.0
            })
            .collect();

        if available.is_empty() {
            let fallback: Vec<&Account> = list
                .iter()
                .filter(|a| {
                    !a.is_rate_limited()
                        && a.has_available_weekly_quota_for_category(target_category)
                        && a.get_effective_quota_for_category(target_category) > 0.0
                })
                .collect();

            if fallback.is_empty() {
                return Err(anyhow!(
                    "No active, non-rate-limited accounts with remaining weekly quota available in pool!"
                ));
            }

            let best = fallback
                .into_iter()
                .max_by(|a, b| {
                    let score_a = a.get_effective_quota_for_category(target_category);
                    let score_b = b.get_effective_quota_for_category(target_category);
                    score_a
                        .partial_cmp(&score_b)
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
                .unwrap();
            return Ok(best.clone());
        }

        if available.len() == 1 {
            return Ok(available[0].clone());
        }

        let mut rng = rand::thread_rng();
        let idx1 = rng.gen_range(0..available.len());
        let mut idx2 = rng.gen_range(0..available.len());
        while idx2 == idx1 && available.len() > 1 {
            idx2 = rng.gen_range(0..available.len());
        }

        let choice1 = available[idx1];
        let choice2 = available[idx2];

        let score1 = choice1.get_effective_quota_for_category(target_category);
        let score2 = choice2.get_effective_quota_for_category(target_category);

        if score1 >= score2 {
            Ok(choice1.clone())
        } else {
            Ok(choice2.clone())
        }
    }

    #[allow(dead_code)]
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

    pub async fn refresh_quotas(&self, client: &reqwest::Client) {
        let list = self.accounts.read().await.clone();
        for mut account in list {
            // 1. Auto-refresh OAuth token if expired
            let mut token_refreshed = false;
            if account.is_token_expired() && !account.refresh_token.is_empty() {
                match crate::oauth::GoogleOAuth::refresh_access_token(client, &account.refresh_token).await {
                    Ok(token_resp) => {
                        account.access_token = token_resp.access_token;
                        if let Some(new_refresh) = token_resp.refresh_token {
                            account.refresh_token = new_refresh;
                        }
                        let expires_in = token_resp.expires_in.unwrap_or(3600);
                        account.expires_at = chrono::Utc::now() + chrono::Duration::seconds(expires_in);
                        token_refreshed = true;
                        tracing::info!(
                            "[TokenManager] Successfully refreshed token for {}",
                            account.email
                        );
                    }
                    Err(error) => {
                        tracing::warn!(
                            "[TokenManager] Failed to refresh token for {}: {}",
                            account.email,
                            error
                        );
                    }
                }
            }

            // 2. Fetch latest quota from Google CloudCode PA API
            let (overall_pct, groups) =
                crate::proxy::quota::QuotaFetcher::fetch_account_quota_full(
                    client,
                    &account.access_token,
                )
                .await;

            // If quota fetch failed and token was not already refreshed, attempt a fallback token refresh
            if overall_pct.is_none() && groups.is_empty() && !token_refreshed && !account.refresh_token.is_empty() {
                if let Ok(token_resp) = crate::oauth::GoogleOAuth::refresh_access_token(client, &account.refresh_token).await {
                    account.access_token = token_resp.access_token;
                    if let Some(new_refresh) = token_resp.refresh_token {
                        account.refresh_token = new_refresh;
                    }
                    let expires_in = token_resp.expires_in.unwrap_or(3600);
                    account.expires_at = chrono::Utc::now() + chrono::Duration::seconds(expires_in);
                    token_refreshed = true;
                    tracing::info!(
                        "[TokenManager] Fallback token refresh succeeded for {}",
                        account.email
                    );
                    let (retry_pct, retry_groups) = crate::proxy::quota::QuotaFetcher::fetch_account_quota_full(
                        client,
                        &account.access_token,
                    ).await;
                    if let Some(pct) = retry_pct {
                        account.quota_percentage = pct;
                    }
                    if !retry_groups.is_empty() {
                        account.quota_groups = retry_groups;
                    }
                }
            } else {
                if let Some(pct) = overall_pct {
                    account.quota_percentage = pct;
                }
                if !groups.is_empty() {
                    account.quota_groups = groups.clone();
                }
            }

            // Record quota delta for intelligent usage detection
            self.model_detector.record_quota_delta(
                &account.id,
                account.get_gemini_5h_quota(),
                account.get_claude_gpt_quota(),
            );

            // 3. Save to disk and update memory
            let _ = self.store.save(&account);
            let mut write_list = self.accounts.write().await;
            if let Some(acc) = write_list.iter_mut().find(|a| a.id == account.id) {
                acc.access_token = account.access_token.clone();
                acc.refresh_token = account.refresh_token.clone();
                acc.expires_at = account.expires_at;
                if let Some(pct) = overall_pct {
                    acc.quota_percentage = pct;
                }
                if !groups.is_empty() {
                    acc.quota_groups = groups;
                }
            }

            // 4. If token was refreshed and account is currently active, sync to Keyring and IDE DB
            if token_refreshed && account.is_active {
                let _ = crate::storage::AccountSwitcher::switch_active_account(&account);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::account::{QuotaBucketInfo, QuotaGroupInfo};

    #[tokio::test]
    async fn test_select_best_account_skips_exhausted_weekly_quota() {
        let dir = std::env::temp_dir().join(format!("agyr-tm-test-{}", uuid::Uuid::new_v4()));
        let store = AccountStore::new(dir.clone());
        let tm = TokenManager::new(store, dir.clone());

        // Account 1: 100% 5h quota, but 0% weekly quota
        let mut acc1 = Account::new(
            "acc1@example.com".to_string(),
            "tok1".to_string(),
            "ref1".to_string(),
            3600,
        );
        acc1.quota_groups = vec![QuotaGroupInfo {
            name: "Gemini 2.5 Flash / Pro".to_string(),
            buckets: vec![
                QuotaBucketInfo {
                    window: "FIVE_HOUR".to_string(),
                    remaining_percentage: 100.0,
                    reset_time: None,
                },
                QuotaBucketInfo {
                    window: "WEEKLY".to_string(),
                    remaining_percentage: 0.0,
                    reset_time: None,
                },
            ],
        }];

        // Account 2: 70% 5h quota, 50% weekly quota
        let mut acc2 = Account::new(
            "acc2@example.com".to_string(),
            "tok2".to_string(),
            "ref2".to_string(),
            3600,
        );
        acc2.quota_groups = vec![QuotaGroupInfo {
            name: "Gemini 2.5 Flash / Pro".to_string(),
            buckets: vec![
                QuotaBucketInfo {
                    window: "FIVE_HOUR".to_string(),
                    remaining_percentage: 70.0,
                    reset_time: None,
                },
                QuotaBucketInfo {
                    window: "WEEKLY".to_string(),
                    remaining_percentage: 50.0,
                    reset_time: None,
                },
            ],
        }];

        tm.add_account(acc1).await.unwrap();
        tm.add_account(acc2).await.unwrap();

        let (best, _category) = tm.select_best_account_for_active_model().await.unwrap();
        assert_eq!(best.email, "acc2@example.com");

        let _ = std::fs::remove_dir_all(dir);
    }

    #[tokio::test]
    async fn test_select_best_account_fails_when_all_weekly_quotas_exhausted() {
        let dir = std::env::temp_dir().join(format!("agyr-tm-test-{}", uuid::Uuid::new_v4()));
        let store = AccountStore::new(dir.clone());
        let tm = TokenManager::new(store, dir.clone());

        let mut acc = Account::new(
            "acc@example.com".to_string(),
            "tok".to_string(),
            "ref".to_string(),
            3600,
        );
        acc.quota_groups = vec![QuotaGroupInfo {
            name: "Gemini 2.5 Flash / Pro".to_string(),
            buckets: vec![
                QuotaBucketInfo {
                    window: "FIVE_HOUR".to_string(),
                    remaining_percentage: 100.0,
                    reset_time: None,
                },
                QuotaBucketInfo {
                    window: "WEEKLY".to_string(),
                    remaining_percentage: 0.0,
                    reset_time: None,
                },
            ],
        }];

        tm.add_account(acc).await.unwrap();

        let res = tm.select_best_account_for_active_model().await;
        assert!(res.is_err());

        let _ = std::fs::remove_dir_all(dir);
    }
}
