use crate::config::Config;
use crate::models::account::QuotaGroupInfo;
use crate::models::{
    Account, ChatCompletionRequest, ChatCompletionResponse, ChatCompletionResponseChoice,
    ChatCompletionResponseUsage, ChatMessage,
};
use crate::oauth::GoogleOAuth;
use crate::proxy::mappers::Mappers;
use crate::proxy::token_manager::TokenManager;
use axum::{
    body::Body,
    extract::{Query, State},
    http::{header, HeaderMap, HeaderName, HeaderValue, StatusCode},
    middleware,
    response::{Html, IntoResponse, Json, Response},
    routing::{any, get, post},
    Router,
};
use base64::Engine;
use futures_util::StreamExt;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use subtle::ConstantTimeEq;
use tokio::sync::{Mutex, Semaphore};
use tower_http::cors::{AllowOrigin, CorsLayer};

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub token_manager: TokenManager,
    pub http_client: reqwest::Client,
    oauth_flows: Arc<Mutex<HashMap<String, OAuthFlow>>>,
    tunnel_limit: Arc<Semaphore>,
    refresh_gate: Arc<Semaphore>,
    browser_bootstraps: Arc<Mutex<HashMap<[u8; 32], Instant>>>,
    browser_sessions: Arc<Mutex<HashMap<[u8; 32], Instant>>>,
}

struct OAuthFlow {
    redirect_uri: String,
    code_verifier: String,
    expires_at: Instant,
}

#[derive(Serialize)]
struct PublicAccount {
    id: String,
    email: String,
    expires_at: chrono::DateTime<chrono::Utc>,
    custom_label: Option<String>,
    quota_percentage: f64,
    quota_groups: Vec<QuotaGroupInfo>,
    is_active: bool,
    rate_limit_until: Option<chrono::DateTime<chrono::Utc>>,
}

impl From<&Account> for PublicAccount {
    fn from(account: &Account) -> Self {
        Self {
            id: account.id.clone(),
            email: account.email.clone(),
            expires_at: account.expires_at,
            custom_label: account.custom_label.clone(),
            quota_percentage: account.quota_percentage,
            quota_groups: account.quota_groups.clone(),
            is_active: account.is_active,
            rate_limit_until: account.rate_limit_until,
        }
    }
}

pub struct Server;

/// Authentication middleware: validates Bearer token on protected routes.
/// Public routes (admin UI, OAuth callback) are excluded.
async fn require_auth(
    State(state): State<AppState>,
    req: axum::extract::Request,
    next: middleware::Next,
) -> Response {
    let path = req.uri().path().to_string();

    // Public routes that don't require authentication
    if path == "/"
        || path == "/admin"
        || path == "/api/accounts/oauth/callback"
        || path == "/api/session/exchange"
    {
        return next.run(req).await;
    }

    let bearer_is_valid = req
        .headers()
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|auth| auth.strip_prefix("Bearer "))
        .map(|token| constant_time_token_matches(token, &state.config.master_key))
        .unwrap_or(false);
    let session_is_valid = if bearer_is_valid {
        false
    } else {
        validate_browser_session(req.headers(), &state).await
    };

    if bearer_is_valid || session_is_valid {
        next.run(req).await
    } else {
        (
            StatusCode::UNAUTHORIZED,
            Json(json!({
                "error": "Invalid or missing API key",
                "hint": "Include header: Authorization: Bearer <your-master-key>"
            })),
        )
            .into_response()
    }
}

async fn validate_browser_session(headers: &HeaderMap, state: &AppState) -> bool {
    let Some(token) = cookie_value(headers, "agyr_session") else {
        return false;
    };
    let fingerprint = token_fingerprint(&token);
    let now = Instant::now();
    let mut sessions = state.browser_sessions.lock().await;
    sessions.retain(|_, expires_at| *expires_at > now);
    sessions.contains_key(&fingerprint)
}

fn cookie_value(headers: &HeaderMap, name: &str) -> Option<String> {
    headers
        .get(header::COOKIE)?
        .to_str()
        .ok()?
        .split(';')
        .filter_map(|part| part.trim().split_once('='))
        .find_map(|(key, value)| (key == name).then(|| value.to_string()))
}

fn token_fingerprint(token: &str) -> [u8; 32] {
    Sha256::digest(token.as_bytes()).into()
}

fn constant_time_token_matches(provided: &str, expected: &str) -> bool {
    let provided_hash = Sha256::digest(provided.as_bytes());
    let expected_hash = Sha256::digest(expected.as_bytes());
    bool::from(provided_hash.ct_eq(&expected_hash))
}

impl Server {
    pub async fn run(config: Config, token_manager: TokenManager) -> anyhow::Result<()> {
        let addr: SocketAddr = format!("{}:{}", config.host, config.port).parse()?;
        let http_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(120))
            .build()?;

        let state = AppState {
            config: config.clone(),
            token_manager,
            http_client,
            oauth_flows: Arc::new(Mutex::new(HashMap::new())),
            tunnel_limit: Arc::new(Semaphore::new(16)),
            refresh_gate: Arc::new(Semaphore::new(1)),
            browser_bootstraps: Arc::new(Mutex::new(HashMap::new())),
            browser_sessions: Arc::new(Mutex::new(HashMap::new())),
        };
        let allowed_origins = [
            format!("http://127.0.0.1:{}", config.port).parse::<HeaderValue>()?,
            format!("http://localhost:{}", config.port).parse::<HeaderValue>()?,
        ];

        let app = Router::new()
            .route("/", get(handle_admin_ui))
            .route("/admin", get(handle_admin_ui))
            .route("/v1/chat/completions", post(handle_chat_completions))
            .route("/v1/messages", post(handle_chat_completions))
            .route("/api/accounts", get(handle_list_accounts))
            .route("/api/health", get(handle_health))
            .route(
                "/api/session/bootstrap",
                post(handle_create_browser_bootstrap),
            )
            .route(
                "/api/session/exchange",
                post(handle_exchange_browser_session),
            )
            .route("/api/accounts/add", post(handle_add_account))
            .route("/api/accounts/delete", post(handle_delete_account))
            .route("/api/accounts/switch", post(handle_switch_account))
            .route(
                "/api/accounts/auto-select",
                post(handle_auto_select_highest_gemini),
            )
            .route("/api/accounts/reset", post(handle_reset_cooldowns))
            .route("/api/accounts/oauth/start", get(handle_oauth_start))
            .route("/api/accounts/oauth/callback", get(handle_oauth_callback))
            .route(
                "/api/preference",
                get(handle_get_preference).post(handle_set_preference),
            )
            .route("/v1internal/*path", any(handle_passthrough_forwarding))
            .fallback(any(handle_passthrough_forwarding))
            .layer(middleware::from_fn_with_state(state.clone(), require_auth))
            .layer(
                CorsLayer::new()
                    .allow_origin(AllowOrigin::list(allowed_origins))
                    .allow_methods(tower_http::cors::Any)
                    .allow_headers(tower_http::cors::Any),
            )
            .with_state(state.clone());

        // Background quota auto-refresher & auto-synchronizer (runs on startup and every 30s)
        let tm_bg = state.token_manager.clone();
        let client_bg = state.http_client.clone();
        let refresh_gate_bg = state.refresh_gate.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(30));
            loop {
                interval.tick().await;
                let Ok(_permit) = refresh_gate_bg.clone().acquire_owned().await else {
                    return;
                };
                tm_bg.refresh_quotas(&client_bg).await;
                // Automatically keep the best quota account for active model synchronized into OS Keyring and IDE DB
                let _ = tm_bg.select_best_account_for_active_model().await;
            }
        });

        tracing::info!("Antigravity Relay Server running on http://{}", addr);
        let listener = tokio::net::TcpListener::bind(addr).await?;
        axum::serve(listener, app).await?;
        Ok(())
    }
}

async fn handle_admin_ui() -> Response {
    let nonce = random_urlsafe(24);
    let html = crate::proxy::ui::get_admin_ui_html()
        .replace("{{CSP_NONCE}}", &nonce)
        .replace("{{ADMIN_CSS}}", include_str!("admin.css"));
    secure_html_response(html, &nonce)
}

async fn handle_list_accounts(State(state): State<AppState>) -> impl IntoResponse {
    let accounts = state.token_manager.list_accounts().await;
    let public_accounts: Vec<PublicAccount> = accounts.iter().map(PublicAccount::from).collect();
    (StatusCode::OK, Json(public_accounts))
}

async fn handle_health() -> impl IntoResponse {
    (
        StatusCode::OK,
        [("x-antigravity-relay", "1")],
        Json(json!({ "service": "antigravity-relay" })),
    )
}

async fn handle_create_browser_bootstrap(State(state): State<AppState>) -> impl IntoResponse {
    let token = random_urlsafe(32);
    let now = Instant::now();
    let mut bootstraps = state.browser_bootstraps.lock().await;
    bootstraps.retain(|_, expires_at| *expires_at > now);
    if bootstraps.len() >= 32 {
        return (
            StatusCode::TOO_MANY_REQUESTS,
            Json(json!({ "error": "Too many pending browser sessions" })),
        )
            .into_response();
    }
    bootstraps.insert(token_fingerprint(&token), now + Duration::from_secs(90));
    (
        StatusCode::OK,
        [(header::CACHE_CONTROL, "no-store")],
        Json(json!({ "bootstrap_token": token, "expires_in": 90 })),
    )
        .into_response()
}

#[derive(Deserialize)]
struct BrowserSessionExchange {
    bootstrap_token: String,
}

async fn handle_exchange_browser_session(
    State(state): State<AppState>,
    Json(payload): Json<BrowserSessionExchange>,
) -> impl IntoResponse {
    if payload.bootstrap_token.len() > 128 || payload.bootstrap_token.is_empty() {
        return (StatusCode::BAD_REQUEST, "Invalid browser bootstrap token").into_response();
    }

    let now = Instant::now();
    let fingerprint = token_fingerprint(&payload.bootstrap_token);
    let is_valid = {
        let mut bootstraps = state.browser_bootstraps.lock().await;
        bootstraps.retain(|_, expires_at| *expires_at > now);
        bootstraps.remove(&fingerprint).is_some()
    };
    if !is_valid {
        return (
            StatusCode::UNAUTHORIZED,
            "Invalid or expired browser bootstrap token",
        )
            .into_response();
    }

    let session_token = random_urlsafe(32);
    let mut sessions = state.browser_sessions.lock().await;
    sessions.retain(|_, expires_at| *expires_at > now);
    if sessions.len() >= 128 {
        if let Some(oldest) = sessions
            .iter()
            .min_by_key(|(_, expires_at)| **expires_at)
            .map(|(key, _)| *key)
        {
            sessions.remove(&oldest);
        }
    }
    sessions.insert(
        token_fingerprint(&session_token),
        now + Duration::from_secs(12 * 60 * 60),
    );
    drop(sessions);

    let cookie = format!(
        "agyr_session={}; HttpOnly; SameSite=Strict; Path=/; Max-Age=43200",
        session_token
    );
    let mut response = (
        StatusCode::OK,
        [(header::CACHE_CONTROL, "no-store")],
        Json(json!({ "status": "ok" })),
    )
        .into_response();
    response.headers_mut().insert(
        header::SET_COOKIE,
        HeaderValue::from_str(&cookie).expect("generated session cookie is valid"),
    );
    response
}

async fn handle_reset_cooldowns(State(state): State<AppState>) -> impl IntoResponse {
    state.token_manager.reset_cooldowns().await;
    tracing::info!("[TokenManager] Cooldowns reset for all accounts in pool");
    (
        StatusCode::OK,
        Json(json!({ "status": "ok", "message": "All account cooldowns reset" })),
    )
}

#[derive(Deserialize)]
struct DeleteAccountRequest {
    account_id: String,
}

async fn handle_delete_account(
    State(state): State<AppState>,
    Json(payload): Json<DeleteAccountRequest>,
) -> impl IntoResponse {
    match state
        .token_manager
        .delete_account(&payload.account_id)
        .await
    {
        Ok(email) => (
            StatusCode::OK,
            Json(json!({
                "status": "ok",
                "message": format!("Đã xóa tài khoản {}", email)
            })),
        )
            .into_response(),
        Err(err) => (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": err.to_string() })),
        )
            .into_response(),
    }
}

#[derive(Deserialize)]
struct SwitchAccountRequest {
    account_id: String,
}

async fn handle_switch_account(
    State(state): State<AppState>,
    Json(payload): Json<SwitchAccountRequest>,
) -> impl IntoResponse {
    match state
        .token_manager
        .switch_account(&payload.account_id)
        .await
    {
        Ok(acc) => (
            StatusCode::OK,
            Json(json!({
                "status": "ok",
                "message": format!("Switched active account to {}", acc.email),
                "account": PublicAccount::from(&acc)
            })),
        )
            .into_response(),
        Err(err) => (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": err.to_string() })),
        )
            .into_response(),
    }
}

async fn handle_auto_select_highest_gemini(State(state): State<AppState>) -> impl IntoResponse {
    match state.token_manager.select_best_account_for_active_model().await {
        Ok((acc, cat)) => (
            StatusCode::OK,
            Json(json!({
                "status": "ok",
                "category": cat.display_name(),
                "account": acc.email,
                "message": format!("Tự động chuyển sang tài khoản {} có hạn ngạch {} cao nhất", acc.email, cat.display_name()),
                "data": PublicAccount::from(&acc)
            })),
        )
            .into_response(),
        Err(err) => (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": err.to_string() })),
        )
            .into_response(),
    }
}

#[derive(Deserialize)]
struct SetPreferenceRequest {
    preference: crate::proxy::model_detector::RoutingPreference,
}

async fn handle_get_preference(State(state): State<AppState>) -> impl IntoResponse {
    let info = state.token_manager.get_model_detector().get_state();
    (StatusCode::OK, Json(info))
}

async fn handle_set_preference(
    State(state): State<AppState>,
    Json(payload): Json<SetPreferenceRequest>,
) -> impl IntoResponse {
    let detector = state.token_manager.get_model_detector();
    let updated = detector.set_preference(payload.preference);
    let _ = state
        .token_manager
        .select_best_account_for_active_model()
        .await;
    (StatusCode::OK, Json(updated))
}

#[derive(Deserialize)]
struct AddAccountRequest {
    email: String,
    access_token: String,
    refresh_token: Option<String>,
    expires_in: Option<i64>,
}

async fn handle_add_account(
    State(state): State<AppState>,
    Json(payload): Json<AddAccountRequest>,
) -> impl IntoResponse {
    if let Err(error) = validate_account_input(&payload) {
        return (StatusCode::BAD_REQUEST, Json(json!({ "error": error }))).into_response();
    }

    let refresh_token = payload.refresh_token.unwrap_or_default();
    let expires_in = payload.expires_in.unwrap_or(3600).clamp(60, 604_800);
    let account = Account::new(
        payload.email.trim().to_string(),
        payload.access_token,
        refresh_token,
        expires_in,
    );

    if let Err(e) = state.token_manager.add_account(account.clone()).await {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": e.to_string() })),
        )
            .into_response();
    }

    spawn_quota_refresh(&state);

    (
        StatusCode::OK,
        Json(json!({ "status": "success", "account": PublicAccount::from(&account) })),
    )
        .into_response()
}

async fn handle_oauth_start(State(state): State<AppState>) -> impl IntoResponse {
    let redirect_uri = format!(
        "http://127.0.0.1:{}/api/accounts/oauth/callback",
        state.config.port
    );
    let oauth_state = random_urlsafe(32);
    let code_verifier = random_urlsafe(32);
    let code_challenge = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .encode(Sha256::digest(code_verifier.as_bytes()));

    {
        let mut flows = state.oauth_flows.lock().await;
        let now = Instant::now();
        flows.retain(|_, flow| flow.expires_at > now);
        if flows.len() >= 128 {
            return (
                StatusCode::TOO_MANY_REQUESTS,
                Json(json!({ "error": "Too many pending OAuth flows" })),
            )
                .into_response();
        }
        flows.insert(
            oauth_state.clone(),
            OAuthFlow {
                redirect_uri: redirect_uri.clone(),
                code_verifier,
                expires_at: now + Duration::from_secs(600),
            },
        );
    }

    let auth_url = GoogleOAuth::build_auth_url(&redirect_uri, &oauth_state, &code_challenge);
    (
        StatusCode::OK,
        Json(json!({ "auth_url": auth_url, "redirect_uri": redirect_uri })),
    )
        .into_response()
}

#[derive(Deserialize)]
struct OAuthCallbackQuery {
    code: Option<String>,
    error: Option<String>,
    state: Option<String>,
}

async fn handle_oauth_callback(
    State(state): State<AppState>,
    Query(query): Query<OAuthCallbackQuery>,
) -> impl IntoResponse {
    let oauth_state = match query.state.as_deref() {
        Some(value) if !value.is_empty() => value,
        _ => return (StatusCode::BAD_REQUEST, "Missing OAuth state").into_response(),
    };
    let flow = {
        let mut flows = state.oauth_flows.lock().await;
        let now = Instant::now();
        flows.retain(|_, flow| flow.expires_at > now);
        flows.remove(oauth_state)
    };
    let flow = match flow {
        Some(flow) => flow,
        None => return (StatusCode::BAD_REQUEST, "Invalid or expired OAuth state").into_response(),
    };

    if let Some(err) = query.error {
        return (StatusCode::BAD_REQUEST, format!("OAuth Error: {}", err)).into_response();
    }

    let code = match query.code {
        Some(c) => c,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                "Missing code parameter".to_string(),
            )
                .into_response()
        }
    };

    let client_id = GoogleOAuth::get_client_id();
    let mut params = vec![
        ("client_id".to_string(), client_id),
        ("code".to_string(), code),
        ("grant_type".to_string(), "authorization_code".to_string()),
        ("redirect_uri".to_string(), flow.redirect_uri),
        ("code_verifier".to_string(), flow.code_verifier),
    ];
    if let Some(client_secret) = GoogleOAuth::get_client_secret() {
        params.push(("client_secret".to_string(), client_secret));
    }

    let token_res = state
        .http_client
        .post("https://oauth2.googleapis.com/token")
        .form(&params)
        .send()
        .await;

    match token_res {
        Ok(res) if res.status().is_success() => {
            let token_data = match read_limited_json(res, 1024 * 1024).await {
                Ok(data) => data,
                Err(error) => {
                    tracing::warn!("[OAuth] Invalid token response: {}", error);
                    return (StatusCode::BAD_GATEWAY, "Invalid OAuth token response")
                        .into_response();
                }
            };
            let access_token = token_data["access_token"]
                .as_str()
                .unwrap_or("")
                .to_string();
            let refresh_token = token_data["refresh_token"]
                .as_str()
                .unwrap_or("")
                .to_string();
            let expires_in = token_data["expires_in"].as_i64().unwrap_or(3600);
            if access_token.is_empty() || access_token.len() > 16 * 1024 {
                return (
                    StatusCode::BAD_GATEWAY,
                    "OAuth response did not contain a valid access token",
                )
                    .into_response();
            }

            // Fetch user info email
            let userinfo: serde_json::Value = match state
                .http_client
                .get("https://www.googleapis.com/oauth2/v2/userinfo")
                .bearer_auth(&access_token)
                .send()
                .await
            {
                Ok(resp) if resp.status().is_success() => read_limited_json(resp, 1024 * 1024)
                    .await
                    .unwrap_or_default(),
                Err(_) => serde_json::json!({}),
                _ => serde_json::json!({}),
            };

            let email = match userinfo["email"].as_str() {
                Some(value) if is_valid_email(value) => value.to_string(),
                _ => {
                    return (
                        StatusCode::BAD_GATEWAY,
                        "OAuth user info did not contain a valid email",
                    )
                        .into_response()
                }
            };
            let account = Account::new(
                email.clone(),
                access_token.clone(),
                refresh_token,
                expires_in,
            );

            // Save to pool and auto-switch
            if let Err(error) = state.token_manager.add_account(account).await {
                tracing::error!("[OAuth] Failed to persist account: {}", error);
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Failed to save OAuth account",
                )
                    .into_response();
            }

            spawn_quota_refresh(&state);

            let nonce = random_urlsafe(24);
            let safe_email = escape_html(&email);
            let success_html = format!(
                r#"<!DOCTYPE html>
<html lang="vi" class="dark">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>Đăng nhập thành công</title>
  <meta http-equiv="refresh" content="1.5;url=/">
  <style nonce="{nonce}">
    body {{ background: #0b0c10; color: #e4e4e7; min-height: 100vh; display: grid; place-items: center; font-family: sans-serif; }}
    main {{ background: #121626; border: 1px solid #3b82f666; border-radius: 1rem; padding: 1.5rem; width: min(22rem, 90vw); text-align: center; }}
    a {{ color: white; background: #2563eb; padding: .6rem 1rem; border-radius: .5rem; display: block; text-decoration: none; }}
  </style>
</head>
<body>
  <main>
    <h2>Đăng nhập thành công</h2>
    <p>{safe_email}</p>
    <p>Đang tự động chuyển hướng về bảng điều khiển...</p>
    <a href="/">Quay về bảng điều khiển ngay</a>
  </main>
  <script nonce="{nonce}">
    if (window.opener && !window.opener.closed) {{
      try {{
        window.opener.fetchAccounts();
      }} catch (e) {{}}
      setTimeout(() => {{
        window.close();
      }}, 1200);
    }} else {{
      setTimeout(() => {{
        window.location.href = '/';
      }}, 1200);
    }}
  </script>
</body>
</html>"#
            );

            secure_html_response(success_html, &nonce)
        }
        _ => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to exchange OAuth code for tokens",
        )
            .into_response(),
    }
}

fn validate_account_input(payload: &AddAccountRequest) -> Result<(), &'static str> {
    if !is_valid_email(payload.email.trim()) {
        return Err("Email is invalid or too long");
    }
    if payload.access_token.is_empty()
        || payload.access_token.len() > 16 * 1024
        || !payload
            .access_token
            .chars()
            .all(|character| character.is_ascii_graphic())
    {
        return Err("Access token is invalid or too long");
    }
    if let Some(refresh_token) = payload.refresh_token.as_deref() {
        if refresh_token.len() > 16 * 1024
            || !refresh_token
                .chars()
                .all(|character| character.is_ascii_graphic())
        {
            return Err("Refresh token is invalid or too long");
        }
    }
    Ok(())
}

fn is_valid_email(email: &str) -> bool {
    if email.is_empty()
        || email.len() > 320
        || email
            .chars()
            .any(|character| character.is_control() || character.is_whitespace())
    {
        return false;
    }
    let Some((local, domain)) = email.split_once('@') else {
        return false;
    };
    !local.is_empty() && domain.contains('.') && !domain.starts_with('.') && !domain.ends_with('.')
}

fn spawn_quota_refresh(state: &AppState) {
    let Ok(permit) = state.refresh_gate.clone().try_acquire_owned() else {
        tracing::debug!("[TokenManager] Quota refresh already running; skipped duplicate task");
        return;
    };
    let token_manager = state.token_manager.clone();
    let client = state.http_client.clone();
    tokio::spawn(async move {
        let _permit = permit;
        token_manager.refresh_quotas(&client).await;
    });
}

fn random_urlsafe(byte_count: usize) -> String {
    let mut bytes = vec![0_u8; byte_count];
    rand::thread_rng().fill_bytes(&mut bytes);
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes)
}

fn escape_html(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

fn secure_html_response(html: String, nonce: &str) -> Response {
    let mut response = Html(html).into_response();
    let csp = format!(
        "default-src 'none'; script-src 'nonce-{nonce}'; style-src 'nonce-{nonce}'; img-src 'self' data:; connect-src 'self'; frame-ancestors 'none'; base-uri 'none'; form-action 'none'"
    );
    let headers = response.headers_mut();
    headers.insert(
        HeaderName::from_static("content-security-policy"),
        HeaderValue::from_str(&csp).expect("generated CSP is valid"),
    );
    headers.insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    headers.insert(
        HeaderName::from_static("x-content-type-options"),
        HeaderValue::from_static("nosniff"),
    );
    headers.insert(
        HeaderName::from_static("referrer-policy"),
        HeaderValue::from_static("no-referrer"),
    );
    headers.insert(
        HeaderName::from_static("x-frame-options"),
        HeaderValue::from_static("DENY"),
    );
    headers.insert(
        HeaderName::from_static("cross-origin-opener-policy"),
        HeaderValue::from_static("same-origin-allow-popups"),
    );
    headers.insert(
        HeaderName::from_static("cross-origin-resource-policy"),
        HeaderValue::from_static("same-origin"),
    );
    response
}

async fn read_limited_json(
    response: reqwest::Response,
    max_bytes: usize,
) -> anyhow::Result<serde_json::Value> {
    let body = read_limited_body(response, max_bytes).await?;
    Ok(serde_json::from_slice(&body)?)
}

async fn read_limited_body(
    response: reqwest::Response,
    max_bytes: usize,
) -> anyhow::Result<Vec<u8>> {
    if response
        .content_length()
        .is_some_and(|length| length > max_bytes as u64)
    {
        anyhow::bail!("Response body exceeds {} bytes", max_bytes);
    }

    let mut body = Vec::new();
    let mut stream = response.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        if body.len().saturating_add(chunk.len()) > max_bytes {
            anyhow::bail!("Response body exceeds {} bytes", max_bytes);
        }
        body.extend_from_slice(&chunk);
    }
    Ok(body)
}

async fn handle_chat_completions(
    State(state): State<AppState>,
    _headers: HeaderMap,
    Json(req): Json<ChatCompletionRequest>,
) -> impl IntoResponse {
    let mut retries = 0;
    let max_retries = 3;

    while retries < max_retries {
        retries += 1;

        // 1. Select best account via P2C algorithm
        let account = match state.token_manager.select_best_account().await {
            Ok(acc) => acc,
            Err(err) => {
                return (
                    StatusCode::SERVICE_UNAVAILABLE,
                    Json(json!({ "error": format!("No available account: {}", err) })),
                )
                    .into_response();
            }
        };

        tracing::info!(
            "[Proxy] Attempt {}/{} -> Routing request to Gemini via account: {}",
            retries,
            max_retries,
            account.email
        );

        // 2. Map request to CloudCode PA format
        let cloudcode_payload = Mappers::openai_to_cloudcode(&req);
        let upstream_url = "https://cloudcode-pa.googleapis.com/v1internal:generateContent";

        // 3. Forward request to upstream with pool account OAuth Bearer token
        let res = state
            .http_client
            .post(upstream_url)
            .header("Host", "cloudcode-pa.googleapis.com")
            .header("Authorization", format!("Bearer {}", account.access_token))
            .header("User-Agent", "Antigravity/1.0.0")
            .header("Content-Type", "application/json")
            .json(&cloudcode_payload)
            .send()
            .await;

        match res {
            Ok(response) => {
                let status = response.status();
                if status.is_success() {
                    let gemini_res = match read_limited_json(response, 10 * 1024 * 1024).await {
                        Ok(value) => value,
                        Err(error) => {
                            tracing::warn!(
                                "[Proxy] Invalid or oversized upstream response: {}",
                                error
                            );
                            return (
                                StatusCode::BAD_GATEWAY,
                                Json(json!({ "error": "Invalid or oversized upstream response" })),
                            )
                                .into_response();
                        }
                    };
                    let text = gemini_res["candidates"][0]["content"]["parts"][0]["text"]
                        .as_str()
                        .or_else(|| {
                            gemini_res["response"]["candidates"][0]["content"]["parts"][0]["text"]
                                .as_str()
                        })
                        .unwrap_or("No text generated")
                        .to_string();

                    let openai_res = ChatCompletionResponse {
                        id: format!("chatcmpl-{}", uuid::Uuid::new_v4()),
                        object: "chat.completion".to_string(),
                        created: chrono::Utc::now().timestamp(),
                        model: req.model.clone(),
                        choices: vec![ChatCompletionResponseChoice {
                            index: 0,
                            message: ChatMessage {
                                role: "assistant".to_string(),
                                content: Some(json!(text)),
                                name: None,
                            },
                            finish_reason: Some("stop".to_string()),
                        }],
                        usage: ChatCompletionResponseUsage {
                            prompt_tokens: 100,
                            completion_tokens: 200,
                            total_tokens: 300,
                        },
                    };

                    tracing::info!("[Proxy] Success 200 OK via account: {}", account.email);
                    return (StatusCode::OK, Json(openai_res)).into_response();
                } else if status == StatusCode::TOO_MANY_REQUESTS
                    || status == StatusCode::FORBIDDEN
                    || status == StatusCode::UNAUTHORIZED
                {
                    tracing::warn!(
                        "[Auto-Failover] Account {} returned {} (Rate Limited/Quota Exceeded)! Circuit breaker activated.",
                        account.email, status
                    );
                    // Mark rate limited in TokenManager (cooldown 300s = 5m)
                    state
                        .token_manager
                        .mark_rate_limited(&account.email, 300)
                        .await;
                    // Loop continues to retry seamlessly on next healthy account in pool!
                    continue;
                } else {
                    let err_text = read_limited_body(response, 64 * 1024)
                        .await
                        .map(|body| String::from_utf8_lossy(&body).into_owned())
                        .unwrap_or_else(|_| "Upstream error body was too large".to_string());
                    return (
                        status,
                        Json(json!({ "error": format!("Upstream error: {}", err_text) })),
                    )
                        .into_response();
                }
            }
            Err(err) => {
                tracing::error!("[Proxy] Request error: {}", err);
                state
                    .token_manager
                    .mark_rate_limited(&account.email, 60)
                    .await;
                continue;
            }
        }
    }

    (
        StatusCode::SERVICE_UNAVAILABLE,
        Json(json!({ "error": "All accounts in pool hit rate limit or failed" })),
    )
        .into_response()
}

async fn handle_passthrough_forwarding(
    State(state): State<AppState>,
    req: axum::extract::Request,
) -> impl IntoResponse {
    let method = req.method().clone();
    let path = req
        .uri()
        .path_and_query()
        .map(|pq| pq.as_str().to_string())
        .unwrap_or_default();

    if method == axum::http::Method::CONNECT {
        let authority = req
            .uri()
            .authority()
            .map(|a| a.to_string())
            .unwrap_or_else(|| "daily-cloudcode-pa.googleapis.com:443".to_string());

        let target = match normalize_google_tunnel_target(&authority) {
            Some(target) => target,
            None => {
                tracing::warn!(
                    "[Tunnel] Blocked unauthorized CONNECT destination: {}",
                    authority
                );
                return (
                    StatusCode::FORBIDDEN,
                    Json(json!({ "error": "Forbidden: Only Google APIs destinations on port 443 are permitted for tunneling" })),
                )
                    .into_response();
            }
        };
        let permit = match state.tunnel_limit.clone().try_acquire_owned() {
            Ok(permit) => permit,
            Err(_) => {
                return (
                    StatusCode::SERVICE_UNAVAILABLE,
                    Json(json!({ "error": "Too many active tunnels" })),
                )
                    .into_response();
            }
        };

        tokio::spawn(async move {
            let _permit = permit;
            match hyper::upgrade::on(req).await {
                Ok(upgraded) => {
                    let mut upgraded = hyper_util::rt::TokioIo::new(upgraded);
                    let connected = tokio::time::timeout(
                        Duration::from_secs(10),
                        tokio::net::TcpStream::connect(&target),
                    )
                    .await;
                    if let Ok(Ok(mut target_stream)) = connected {
                        let _ = tokio::time::timeout(
                            Duration::from_secs(300),
                            tokio::io::copy_bidirectional(&mut upgraded, &mut target_stream),
                        )
                        .await;
                    }
                }
                Err(err) => {
                    tracing::warn!("[Tunnel] Upgrade error: {}", err);
                }
            }
        });

        return StatusCode::OK.into_response();
    }

    // H3 Fix: Validate path to prevent path traversal and arbitrary SSRF injection
    let raw_path = req.uri().path();
    if !raw_path.starts_with("/v1") && !raw_path.starts_with("/v1internal") {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Invalid passthrough path. Only /v1* and /v1internal* are permitted." })),
        )
            .into_response();
    }

    let body_bytes = match axum::body::to_bytes(req.into_body(), 10 * 1024 * 1024).await {
        Ok(bytes) => bytes,
        Err(_) => {
            return (StatusCode::PAYLOAD_TOO_LARGE, "Request body exceeds 10 MiB").into_response()
        }
    };

    // Pick best account from pool
    let account = match state.token_manager.select_best_account().await {
        Ok(acc) => acc,
        Err(_) => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(json!({ "error": "No available account in pool" })),
            )
                .into_response();
        }
    };

    let target_url = format!("https://cloudcode-pa.googleapis.com{}", path);
    tracing::info!(
        "[Passthrough Forwarder] Forwarding {} {} via account: {}",
        method,
        path,
        account.email
    );

    let res = state
        .http_client
        .request(method, &target_url)
        .header("Host", "cloudcode-pa.googleapis.com")
        .header("Authorization", format!("Bearer {}", account.access_token))
        .header("Content-Type", "application/json")
        .header("User-Agent", "Antigravity/1.0.0")
        .body(body_bytes)
        .send()
        .await;

    match res {
        Ok(response) => {
            let status = response.status();
            let headers = response.headers().clone();
            let mut resp = Response::new(Body::from_stream(response.bytes_stream()));
            *resp.status_mut() = status;
            for (name, value) in &headers {
                if !is_hop_by_hop_header(name) {
                    resp.headers_mut().append(name, value.clone());
                }
            }
            resp
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

fn normalize_google_tunnel_target(authority: &str) -> Option<String> {
    let authority = authority.parse::<axum::http::uri::Authority>().ok()?;
    let host = authority.host().trim_end_matches('.').to_ascii_lowercase();
    let port = authority.port_u16().unwrap_or(443);
    if port != 443 || !(host == "googleapis.com" || host.ends_with(".googleapis.com")) {
        return None;
    }
    Some(format!("{}:443", host))
}

fn is_hop_by_hop_header(name: &HeaderName) -> bool {
    matches!(
        name.as_str(),
        "connection"
            | "keep-alive"
            | "proxy-authenticate"
            | "proxy-authorization"
            | "te"
            | "trailer"
            | "transfer-encoding"
            | "upgrade"
    )
}

#[cfg(test)]
mod tests {
    use super::{
        constant_time_token_matches, cookie_value, escape_html, is_valid_email,
        normalize_google_tunnel_target, token_fingerprint, PublicAccount,
    };
    use crate::models::Account;
    use axum::http::{header, HeaderMap, HeaderValue};

    #[test]
    fn compares_api_tokens_without_plaintext_equality() {
        assert!(constant_time_token_matches("secret-value", "secret-value"));
        assert!(!constant_time_token_matches("secret-value", "other-value"));
    }

    #[test]
    fn extracts_only_the_named_session_cookie() {
        let mut headers = HeaderMap::new();
        headers.insert(
            header::COOKIE,
            HeaderValue::from_static("other=value; agyr_session=session-token; last=1"),
        );
        assert_eq!(
            cookie_value(&headers, "agyr_session"),
            Some("session-token".to_string())
        );
        assert!(cookie_value(&headers, "missing").is_none());
        assert_eq!(token_fingerprint("same"), token_fingerprint("same"));
        assert_ne!(token_fingerprint("same"), token_fingerprint("different"));
    }

    #[test]
    fn validates_oauth_emails() {
        assert!(is_valid_email("user@example.com"));
        assert!(!is_valid_email("not-an-email"));
        assert!(!is_valid_email("user @example.com"));
    }

    #[test]
    fn escapes_dynamic_html() {
        assert_eq!(
            escape_html("<script>'x'</script>"),
            "&lt;script&gt;&#39;x&#39;&lt;/script&gt;"
        );
    }

    #[test]
    fn tunnel_allows_only_googleapis_on_tls_port() {
        assert_eq!(
            normalize_google_tunnel_target("cloudcode-pa.googleapis.com:443"),
            Some("cloudcode-pa.googleapis.com:443".to_string())
        );
        assert!(normalize_google_tunnel_target("cloudcode-pa.googleapis.com:22").is_none());
        assert!(normalize_google_tunnel_target("googleapis.com.attacker.example:443").is_none());
    }

    #[test]
    fn public_account_never_serializes_tokens() {
        let account = Account::new(
            "user@example.com".to_string(),
            "access-secret".to_string(),
            "refresh-secret".to_string(),
            3600,
        );
        let serialized = serde_json::to_string(&PublicAccount::from(&account)).unwrap();
        assert!(!serialized.contains("access-secret"));
        assert!(!serialized.contains("refresh-secret"));
        assert!(!serialized.contains("access_token"));
        assert!(!serialized.contains("refresh_token"));
    }
}
