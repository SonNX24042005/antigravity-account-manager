use anyhow::Result;
use serde::{Deserialize, Serialize};

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthTokenResponse {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_in: Option<i64>,
    pub token_type: String,
}

pub struct GoogleOAuth;

impl GoogleOAuth {
    pub fn get_client_id() -> String {
        // Priority 1: environment variable override
        if let Ok(val) = std::env::var("GOOGLE_CLIENT_ID") {
            if !val.is_empty() {
                return val;
            }
        }
        // Priority 2: config file override
        if let Some(val) = Self::load_oauth_config_value("client_id") {
            return val;
        }
        // Priority 3: built-in fallback
        let bytes: &[u8] = &[107, 106, 109, 107, 106, 106, 108, 106, 108, 106, 111, 99, 107, 119, 46, 55, 50, 41, 41, 51, 52, 104, 50, 104, 107, 54, 57, 40, 63, 104, 105, 111, 44, 46, 53, 54, 53, 48, 50, 110, 61, 110, 106, 105, 63, 42, 116, 59, 42, 42, 41, 116, 61, 53, 53, 61, 54, 63, 47, 41, 63, 40, 57, 53, 52, 46, 63, 52, 46, 116, 57, 53, 55];
        bytes.iter().map(|&b| (b ^ 0x5a) as char).collect()
    }

    pub fn get_client_secret() -> String {
        // Priority 1: environment variable override
        if let Ok(val) = std::env::var("GOOGLE_CLIENT_SECRET") {
            if !val.is_empty() {
                return val;
            }
        }
        // Priority 2: config file override
        if let Some(val) = Self::load_oauth_config_value("client_secret") {
            return val;
        }
        // Priority 3: built-in fallback
        let bytes: &[u8] = &[29, 21, 25, 9, 10, 2, 119, 17, 111, 98, 28, 13, 8, 110, 98, 108, 22, 62, 22, 16, 107, 55, 22, 24, 98, 41, 2, 25, 110, 32, 108, 43, 30, 27, 60];
        bytes.iter().map(|&b| (b ^ 0x5a) as char).collect()
    }

    /// Load a single value from the optional OAuth config file.
    fn load_oauth_config_value(key: &str) -> Option<String> {
        let config_path = dirs::home_dir()?
            .join(".antigravity-relay")
            .join("oauth_credentials.json");
        let content = std::fs::read_to_string(config_path).ok()?;
        let json: serde_json::Value = serde_json::from_str(&content).ok()?;
        json.get(key)?
            .as_str()
            .map(|s| s.to_string())
            .filter(|s| !s.is_empty())
    }

    pub fn build_auth_url(redirect_uri: &str, state: &str) -> String {
        format!(
            "https://accounts.google.com/o/oauth2/v2/auth?\
            client_id={}&\
            redirect_uri={}&\
            response_type=code&\
            scope=https://www.googleapis.com/auth/userinfo.email%20https://www.googleapis.com/auth/cloud-platform&\
            access_type=offline&\
            prompt=consent&\
            state={}",
            Self::get_client_id(),
            urlencoding::encode(redirect_uri),
            urlencoding::encode(state)
        )
    }

    #[allow(dead_code)]
    pub async fn refresh_access_token(
        client: &reqwest::Client,
        refresh_token: &str,
    ) -> Result<OAuthTokenResponse> {
        let client_id = Self::get_client_id();
        let client_secret = Self::get_client_secret();
        let params = [
            ("client_id", client_id.as_str()),
            ("client_secret", client_secret.as_str()),
            ("grant_type", "refresh_token"),
            ("refresh_token", refresh_token),
        ];

        let res = client
            .post("https://oauth2.googleapis.com/token")
            .form(&params)
            .send()
            .await?
            .json::<OAuthTokenResponse>()
            .await?;

        Ok(res)
    }
}
