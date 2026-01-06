pub mod config;
pub mod monitors;
pub mod traders;
pub mod utils;
pub mod types;

use std::sync::Arc;
use tokio::sync::RwLock;

/// Main Pump.fun sniper bot structure
pub struct PumpFunSniper {
    config: Arc<config::BotConfig>,
    client: Arc<utils::solana_client::SolanaClient>,
    monitor: Arc<RwLock<Option<monitors::pump_fun_monitor::PumpFunMonitor>>>,
    trader: Arc<traders::trader::Trader>,
}

impl PumpFunSniper {
    /// Create a new instance of the sniper bot
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        // Load configuration
        let config = Arc::new(config::load_config()?);

        // Initialize Solana client
        let client = Arc::new(utils::solana_client::SolanaClient::new(&config).await?);

        // Initialize trader
        let trader = Arc::new(traders::trader::Trader::new(
            Arc::clone(&client),
            Arc::clone(&config),
        ).await?);

        Ok(Self {
            config,
            client,
            monitor: Arc::new(RwLock::new(None)),
            trader,
        })
    }

    /// Start the sniper bot
    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        tracing::info!("Starting Pump.fun sniper bot...");

        // Start the monitor
        let monitor = monitors::pump_fun_monitor::PumpFunMonitor::new(
            Arc::clone(&self.client),
            Arc::clone(&self.config),
        ).await?;

        // Set up token event handler
        let trader = Arc::clone(&self.trader);
        let config = Arc::clone(&self.config);
        monitor.on_new_token(move |event| {
            let trader = Arc::clone(&trader);
            let config = Arc::clone(&config);
            tokio::spawn(async move {
                if let Err(e) = handle_new_token(trader, config, event).await {
                    tracing::error!("Error handling new token: {}", e);
                }
            });
        }).await;

        // Store the monitor
        *self.monitor.write().await = Some(monitor);

        tracing::info!("Pump.fun sniper bot started successfully");
        Ok(())
    }

    /// Stop the sniper bot
    pub async fn stop(&self) -> Result<(), Box<dyn std::error::Error>> {
        tracing::info!("Stopping Pump.fun sniper bot...");

        if let Some(monitor) = self.monitor.write().await.take() {
            monitor.stop().await?;
        }

        self.trader.stop().await?;

        tracing::info!("Pump.fun sniper bot stopped successfully");
        Ok(())
    }

    /// Get bot status
    pub async fn status(&self) -> serde_json::Value {
        serde_json::json!({
            "config": {
                "simulation_mode": self.config.simulation_mode,
                "rpc_url": self.config.rpc_url,
                "buy_amount_sol": self.config.buy_amount_sol,
            },
            "monitoring": {
                "active": self.monitor.read().await.is_some(),
            },
            "trading": self.trader.status().await,
        })
    }
}

/// Handle new token detection
async fn handle_new_token(
    trader: Arc<traders::trader::Trader>,
    config: Arc<config::BotConfig>,
    event: monitors::pump_fun_monitor::NewTokenEvent,
) -> Result<(), Box<dyn std::error::Error>> {
    tracing::info!(
        "Processing new token: {} (creator: {})",
        event.token_address,
        event.creator
    );

    // Analyze the token
    let analysis = utils::token_analyzer::analyze_token(
        &event.token_address,
        &event.bonding_curve_address,
        trader.client(),
    ).await?;

    // Check if token passes filters
    if should_trade_token(&analysis, &config) {
        // Execute trade
        trader.execute_buy(&analysis).await?;
    } else {
        tracing::info!("Token filtered out: {}", event.token_address);
    }

    Ok(())
}

/// Check if token should be traded based on configuration
fn should_trade_token(
    analysis: &utils::token_analyzer::TokenAnalysis,
    config: &config::BotConfig,
) -> bool {
    // Safety score check
    if analysis.safety.score < 60 {
        return false;
    }

    // Market cap check
    if analysis.metrics.market_cap < config.min_market_cap ||
       analysis.metrics.market_cap > config.max_market_cap {
        return false;
    }

    // Liquidity check
    if analysis.metrics.liquidity < config.min_liquidity {
        return false;
    }

    true
}
