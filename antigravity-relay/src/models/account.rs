use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use crate::device::DeviceProfile;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuotaBucketInfo {
    pub window: String,
    pub remaining_percentage: f64,
    pub reset_time: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuotaGroupInfo {
    pub name: String,
    pub buckets: Vec<QuotaBucketInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub id: String,
    pub email: String,
    pub access_token: String,
    pub refresh_token: String,
    pub expires_at: DateTime<Utc>,
    pub custom_label: Option<String>,
    pub device_profile: DeviceProfile,
    pub quota_percentage: f64,
    #[serde(default)]
    pub quota_groups: Vec<QuotaGroupInfo>,
    pub is_active: bool,
    pub rate_limit_until: Option<DateTime<Utc>>,
}

impl Account {
    pub fn new(email: String, access_token: String, refresh_token: String, expires_in_seconds: i64) -> Self {
        let id = uuid::Uuid::new_v4().to_string();
        let expires_at = Utc::now() + chrono::Duration::seconds(expires_in_seconds);
        let device_profile = DeviceProfile::generate();

        Self {
            id,
            email,
            access_token,
            refresh_token,
            expires_at,
            custom_label: None,
            device_profile,
            quota_percentage: 100.0,
            quota_groups: Vec::new(),
            is_active: true,
            rate_limit_until: None,
        }
    }

    #[allow(dead_code)]
    pub fn is_token_expired(&self) -> bool {
        Utc::now() >= self.expires_at - chrono::Duration::minutes(5)
    }

    pub fn is_rate_limited(&self) -> bool {
        if let Some(until) = self.rate_limit_until {
            Utc::now() < until
        } else {
            false
        }
    }

    pub fn set_rate_limit(&mut self, duration_seconds: i64) {
        self.rate_limit_until = Some(Utc::now() + chrono::Duration::seconds(duration_seconds));
    }
}
