mod config;
mod device;
mod models;
mod oauth;
mod proxy;
mod storage;

use config::Config;
use proxy::{Server, TokenManager};
use storage::AccountStore;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. Initialize colorful terminal logging subscriber
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info,antigravity_relay=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer().with_thread_ids(false))
        .init();

    tracing::info!("=====================================================");
    tracing::info!("   🚀 Antigravity Relay Daemon Engine v1.0.0");
    tracing::info!("=====================================================");

    // 2. Load configuration & create data directories
    let config = Config::default();
    config.ensure_directories()?;
    tracing::info!("[Config] Data directory: {:?}", config.data_dir);

    // 3. Initialize Account Storage & Token Pool
    let store = AccountStore::new(config.accounts_dir());
    let token_manager = TokenManager::new(store);

    // 4. Start Axum Web Server & Account Manager
    Server::run(config, token_manager).await?;

    Ok(())
}
