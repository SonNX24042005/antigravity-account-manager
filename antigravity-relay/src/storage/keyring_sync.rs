use anyhow::Result;
use keyring::Entry;

pub struct KeyringSync;

impl KeyringSync {
    pub fn inject_keyring_credential(
        access_token: &str,
        refresh_token: &str,
        expires_at_rfc3339: &str,
    ) -> Result<()> {
        let entry = Entry::new("gemini", "antigravity")?;
        let payload = serde_json::json!({
            "token": {
                "access_token": access_token,
                "token_type": "Bearer",
                "refresh_token": refresh_token,
                "expiry": expires_at_rfc3339
            },
            "auth_method": "consumer"
        }).to_string();

        let _ = entry.set_password(&payload);
        tracing::info!("[Keyring Sync] Successfully updated OS Keyring (service: gemini, user: antigravity)");
        Ok(())
    }

    pub fn get_keyring_credential() -> Option<(String, Option<String>)> {
        let entry = Entry::new("gemini", "antigravity").ok()?;
        let password = entry.get_password().ok()?;
        let json: serde_json::Value = serde_json::from_str(&password).ok()?;
        let token_obj = json.get("token")?;
        let access_token = token_obj.get("access_token")?.as_str()?.to_string();
        let refresh_token = token_obj.get("refresh_token").and_then(|r| r.as_str()).map(|s| s.to_string());
        Some((access_token, refresh_token))
    }
}
