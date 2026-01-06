use std::sync::Arc;
use tokio::time::{self, Duration};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod config;
mod monitors;
mod traders;
mod utils;
mod types;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "solana_pumpfun_sniper=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Starting Solana Pump.fun Sniper Bot (Rust Edition)");

    // Load configuration
    let config = Arc::new(config::load_config()?);
    tracing::info!("Configuration loaded successfully");

    // Create bot instance
    let bot = Arc::new(solana_pumpfun_sniper::PumpFunSniper::new().await?);

    // Start the bot
    bot.start().await?;

    // Set up signal handling for graceful shutdown
    let bot_clone = Arc::clone(&bot);
    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.unwrap();
        tracing::info!("Received shutdown signal");
        if let Err(e) = bot_clone.stop().await {
            tracing::error!("Error during shutdown: {}", e);
        }
        std::process::exit(0);
    });

    // Health check loop
    let mut interval = time::interval(Duration::from_secs(60));
    loop {
        interval.tick().await;
        let status = bot.status().await;
        tracing::info!("Health check: {}", status);
    }
}
