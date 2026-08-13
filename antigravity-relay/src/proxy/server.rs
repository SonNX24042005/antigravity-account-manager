use std::net::SocketAddr;
use axum::{
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    response::{Html, IntoResponse, Json},
    routing::{any, get, post},
    Router,
};
use serde::Deserialize;
use serde_json::json;
use crate::config::Config;
use crate::models::{Account, ChatCompletionRequest, ChatCompletionResponse, ChatCompletionResponseChoice, ChatCompletionResponseUsage, ChatMessage};
use crate::oauth::GoogleOAuth;
use crate::proxy::mappers::Mappers;
use crate::proxy::token_manager::TokenManager;
use tower_http::cors::CorsLayer;

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub token_manager: TokenManager,
    pub http_client: reqwest::Client,
}

pub struct Server;

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
        };

        let app = Router::new()
            .route("/", get(handle_admin_ui))
            .route("/admin", get(handle_admin_ui))
            .route("/v1/chat/completions", post(handle_chat_completions))
            .route("/v1/messages", post(handle_chat_completions))
            .route("/api/accounts", get(handle_list_accounts))
            .route("/api/accounts/add", post(handle_add_account))
            .route("/api/accounts/switch", post(handle_switch_account))
            .route("/api/accounts/auto-select", post(handle_auto_select_highest_gemini))
            .route("/api/accounts/reset", post(handle_reset_cooldowns))
            .route("/api/accounts/oauth/start", get(handle_oauth_start))
            .route("/api/accounts/oauth/callback", get(handle_oauth_callback))
            .route("/v1internal/*path", any(handle_passthrough_forwarding))
            .fallback(any(handle_passthrough_forwarding))
            .layer(CorsLayer::permissive())
            .with_state(state);

        tracing::info!("🚀 Antigravity Relay Server running on http://{}", addr);
        let listener = tokio::net::TcpListener::bind(addr).await?;
        axum::serve(listener, app).await?;
        Ok(())
    }
}

async fn handle_admin_ui() -> impl IntoResponse {
    Html(crate::proxy::ui::get_admin_ui_html())
}

async fn handle_list_accounts(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let accounts = state.token_manager.list_accounts().await;
    (StatusCode::OK, Json(accounts))
}

async fn handle_reset_cooldowns(
    State(state): State<AppState>,
) -> impl IntoResponse {
    state.token_manager.reset_cooldowns().await;
    tracing::info!("[TokenManager] Cooldowns reset for all accounts in pool");
    (StatusCode::OK, Json(json!({ "status": "ok", "message": "All account cooldowns reset" })))
}

#[derive(Deserialize)]
struct SwitchAccountRequest {
    account_id: String,
}

async fn handle_switch_account(
    State(state): State<AppState>,
    Json(payload): Json<SwitchAccountRequest>,
) -> impl IntoResponse {
    match state.token_manager.switch_account(&payload.account_id).await {
        Ok(acc) => (
            StatusCode::OK,
            Json(json!({
                "status": "ok",
                "message": format!("Switched active account to {}", acc.email),
                "account": acc
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

async fn handle_auto_select_highest_gemini(
    State(state): State<AppState>,
) -> impl IntoResponse {
    match state.token_manager.select_highest_gemini_account().await {
        Ok(acc) => (
            StatusCode::OK,
            Json(json!({
                "status": "ok",
                "message": format!("Auto-selected best Gemini account: {}", acc.email),
                "account": acc
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
    let refresh_token = payload.refresh_token.unwrap_or_default();
    let expires_in = payload.expires_in.unwrap_or(3600);
    let account = Account::new(payload.email.clone(), payload.access_token.clone(), refresh_token, expires_in);

    if let Err(e) = state.token_manager.add_account(account.clone()).await {
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() }))).into_response();
    }

    (StatusCode::OK, Json(json!({ "status": "success", "account": account }))).into_response()
}

#[derive(Deserialize)]
struct OAuthStartQuery {
    redirect_uri: Option<String>,
}

async fn handle_oauth_start(
    State(state): State<AppState>,
    Query(query): Query<OAuthStartQuery>,
) -> impl IntoResponse {
    let redirect_uri = query
        .redirect_uri
        .unwrap_or_else(|| format!("http://{}:{}/api/accounts/oauth/callback", state.config.host, state.config.port));
    
    let auth_url = GoogleOAuth::build_auth_url(&redirect_uri, "state123");
    Json(json!({ "auth_url": auth_url, "redirect_uri": redirect_uri }))
}

#[derive(Deserialize)]
struct OAuthCallbackQuery {
    code: Option<String>,
    error: Option<String>,
}

async fn handle_oauth_callback(
    State(state): State<AppState>,
    Query(query): Query<OAuthCallbackQuery>,
) -> impl IntoResponse {
    if let Some(err) = query.error {
        return (StatusCode::BAD_REQUEST, format!("OAuth Error: {}", err)).into_response();
    }

    let code = match query.code {
        Some(c) => c,
        None => return (StatusCode::BAD_REQUEST, "Missing code parameter".to_string()).into_response(),
    };

    let redirect_uri = format!("http://{}:{}/api/accounts/oauth/callback", state.config.host, state.config.port);
    let client_id = GoogleOAuth::get_client_id();
    let client_secret = GoogleOAuth::get_client_secret();
    let params = [
        ("client_id", client_id.as_str()),
        ("client_secret", client_secret.as_str()),
        ("code", code.as_str()),
        ("grant_type", "authorization_code"),
        ("redirect_uri", redirect_uri.as_str()),
    ];

    let token_res = state
        .http_client
        .post("https://oauth2.googleapis.com/token")
        .form(&params)
        .send()
        .await;

    match token_res {
        Ok(res) if res.status().is_success() => {
            let token_data: serde_json::Value = res.json().await.unwrap_or_default();
            let access_token = token_data["access_token"].as_str().unwrap_or("").to_string();
            let refresh_token = token_data["refresh_token"].as_str().unwrap_or("").to_string();
            let expires_in = token_data["expires_in"].as_i64().unwrap_or(3600);

            // Fetch user info email
            let userinfo: serde_json::Value = match state
                .http_client
                .get("https://www.googleapis.com/oauth2/v2/userinfo")
                .bearer_auth(&access_token)
                .send()
                .await
            {
                Ok(resp) => resp.json().await.unwrap_or_default(),
                Err(_) => serde_json::json!({}),
            };

            let email = userinfo["email"].as_str().unwrap_or("unknown@google.com").to_string();
            let account = Account::new(email.clone(), access_token.clone(), refresh_token, expires_in);

            // Save to pool and auto-switch
            let _ = state.token_manager.add_account(account.clone()).await;

            (StatusCode::OK, format!("🎉 Successfully logged in account: {}! You can close this tab now.", email)).into_response()
        }
        _ => (StatusCode::INTERNAL_SERVER_ERROR, "Failed to exchange OAuth code for tokens").into_response(),
    }
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
                    let gemini_res: serde_json::Value = response.json().await.unwrap_or_default();
                    let text = gemini_res["candidates"][0]["content"]["parts"][0]["text"]
                        .as_str()
                        .or_else(|| gemini_res["response"]["candidates"][0]["content"]["parts"][0]["text"].as_str())
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

                    tracing::info!("[Proxy] ✅ Success 200 OK via account: {}", account.email);
                    return (StatusCode::OK, Json(openai_res)).into_response();
                } else if status == StatusCode::TOO_MANY_REQUESTS || status == StatusCode::FORBIDDEN || status == StatusCode::UNAUTHORIZED {
                    tracing::warn!(
                        "[Auto-Failover] Account {} returned {} (Rate Limited/Quota Exceeded)! Circuit breaker activated.",
                        account.email, status
                    );
                    // Mark rate limited in TokenManager (cooldown 300s = 5m)
                    state.token_manager.mark_rate_limited(&account.email, 300).await;
                    // Loop continues to retry seamlessly on next healthy account in pool!
                    continue;
                } else {
                    let err_text = response.text().await.unwrap_or_default();
                    return (
                        status,
                        Json(json!({ "error": format!("Upstream error: {}", err_text) })),
                    )
                        .into_response();
                }
            }
            Err(err) => {
                tracing::error!("[Proxy] Request error: {}", err);
                state.token_manager.mark_rate_limited(&account.email, 60).await;
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
    let path = req.uri().path_and_query().map(|pq| pq.as_str().to_string()).unwrap_or_default();

    if method == axum::http::Method::CONNECT {
        let host = req
            .uri()
            .authority()
            .map(|a| a.to_string())
            .unwrap_or_else(|| "daily-cloudcode-pa.googleapis.com:443".to_string());

        tokio::spawn(async move {
            match hyper::upgrade::on(req).await {
                Ok(upgraded) => {
                    let mut upgraded = hyper_util::rt::TokioIo::new(upgraded);
                    if let Ok(mut target_stream) = tokio::net::TcpStream::connect(&host).await {
                        let _ = tokio::io::copy_bidirectional(&mut upgraded, &mut target_stream).await;
                    }
                }
                Err(err) => {
                    tracing::warn!("[Tunnel] Upgrade error: {}", err);
                }
            }
        });

        return StatusCode::OK.into_response();
    }
    let body_bytes = match axum::body::to_bytes(req.into_body(), 10 * 1024 * 1024).await {
        Ok(bytes) => bytes,
        Err(_) => return (StatusCode::BAD_REQUEST, "Failed to read body").into_response(),
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
        method, path, account.email
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
            let body = response.bytes().await.unwrap_or_default();

            let mut resp = (status, body).into_response();
            *resp.headers_mut() = headers;
            resp
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}
