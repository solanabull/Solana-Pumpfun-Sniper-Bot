use serde::{Deserialize, Serialize};
use std::env;

/// Bot configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BotConfig {
    // Solana Configuration
    pub rpc_url: String,
    pub ws_url: Option<String>,

    // Wallet Configuration
    pub private_key: Option<String>,
    pub main_wallet_private_key: Option<String>,

    // Trading Configuration
    pub buy_amount_sol: f64,
    pub min_liquidity: f64,
    pub max_slippage: f64,
    pub take_profit_percentage: f64,
    pub stop_loss_percentage: f64,
    pub trailing_stop_loss_percentage: f64,

    // Safety Settings
    pub trading_cooldown_ms: u64,
    pub max_loss_per_trade_sol: f64,
    pub max_trades_per_hour: u32,

    // Token Filtering
    pub min_market_cap: f64,
    pub max_market_cap: f64,
    pub min_holders: u32,
    pub max_holders: u32,
    pub require_social_links: bool,
    pub require_creator_verification: bool,

    // Gas Optimization
    pub priority_fee_lamports: u64,
    pub max_priority_fee_lamports: u64,

    // Monitoring
    pub log_level: String,
    pub telegram_bot_token: Option<String>,
    pub telegram_chat_id: Option<String>,

    // Simulation Mode
    pub simulation_mode: bool,
}

impl Default for BotConfig {
    fn default() -> Self {
        Self {
            // Solana Configuration
            rpc_url: "https://api.mainnet-beta.solana.com".to_string(),
            ws_url: None,

            // Wallet Configuration
            private_key: None,
            main_wallet_private_key: None,

            // Trading Configuration
            buy_amount_sol: 0.1,
            min_liquidity: 5.0,
            max_slippage: 25.0,
            take_profit_percentage: 100.0,
            stop_loss_percentage: 30.0,
            trailing_stop_loss_percentage: 10.0,

            // Safety Settings
            trading_cooldown_ms: 5000,
            max_loss_per_trade_sol: 0.5,
            max_trades_per_hour: 10,

            // Token Filtering
            min_market_cap: 1000.0,
            max_market_cap: 50000.0,
            min_holders: 10,
            max_holders: 1000,
            require_social_links: false,
            require_creator_verification: false,

            // Gas Optimization
            priority_fee_lamports: 10000,
            max_priority_fee_lamports: 100000,

            // Monitoring
            log_level: "info".to_string(),
            telegram_bot_token: None,
            telegram_chat_id: None,

            // Simulation Mode
            simulation_mode: true,
        }
    }
}

/// Load configuration from environment variables
pub fn load_config() -> Result<BotConfig, Box<dyn std::error::Error>> {
    // Load .env file if it exists
    dotenv::dotenv().ok();

    let mut config = BotConfig::default();

    // Solana Configuration
    if let Ok(rpc_url) = env::var("RPC_URL") {
        config.rpc_url = rpc_url;
    }
    if let Ok(ws_url) = env::var("WS_URL") {
        config.ws_url = Some(ws_url);
    }

    // Wallet Configuration
    config.private_key = env::var("PRIVATE_KEY").ok();
    config.main_wallet_private_key = env::var("MAIN_WALLET_PRIVATE_KEY").ok();

    // Trading Configuration
    if let Ok(val) = env::var("BUY_AMOUNT_SOL") {
        config.buy_amount_sol = val.parse()?;
    }
    if let Ok(val) = env::var("MIN_LIQUIDITY") {
        config.min_liquidity = val.parse()?;
    }
    if let Ok(val) = env::var("MAX_SLIPPAGE") {
        config.max_slippage = val.parse()?;
    }
    if let Ok(val) = env::var("TAKE_PROFIT_PERCENTAGE") {
        config.take_profit_percentage = val.parse()?;
    }
    if let Ok(val) = env::var("STOP_LOSS_PERCENTAGE") {
        config.stop_loss_percentage = val.parse()?;
    }
    if let Ok(val) = env::var("TRAILING_STOP_LOSS_PERCENTAGE") {
        config.trailing_stop_loss_percentage = val.parse()?;
    }

    // Safety Settings
    if let Ok(val) = env::var("TRADING_COOLDOWN_MS") {
        config.trading_cooldown_ms = val.parse()?;
    }
    if let Ok(val) = env::var("MAX_LOSS_PER_TRADE_SOL") {
        config.max_loss_per_trade_sol = val.parse()?;
    }
    if let Ok(val) = env::var("MAX_TRADES_PER_HOUR") {
        config.max_trades_per_hour = val.parse()?;
    }

    // Token Filtering
    if let Ok(val) = env::var("MIN_MARKET_CAP") {
        config.min_market_cap = val.parse()?;
    }
    if let Ok(val) = env::var("MAX_MARKET_CAP") {
        config.max_market_cap = val.parse()?;
    }
    if let Ok(val) = env::var("MIN_HOLDERS") {
        config.min_holders = val.parse()?;
    }
    if let Ok(val) = env::var("MAX_HOLDERS") {
        config.max_holders = val.parse()?;
    }
    if let Ok(val) = env::var("REQUIRE_SOCIAL_LINKS") {
        config.require_social_links = val.parse()?;
    }
    if let Ok(val) = env::var("REQUIRE_CREATOR_VERIFICATION") {
        config.require_creator_verification = val.parse()?;
    }

    // Gas Optimization
    if let Ok(val) = env::var("PRIORITY_FEE_LAMPORTS") {
        config.priority_fee_lamports = val.parse()?;
    }
    if let Ok(val) = env::var("MAX_PRIORITY_FEE_LAMPORTS") {
        config.max_priority_fee_lamports = val.parse()?;
    }

    // Monitoring
    if let Ok(val) = env::var("LOG_LEVEL") {
        config.log_level = val;
    }
    config.telegram_bot_token = env::var("TELEGRAM_BOT_TOKEN").ok();
    config.telegram_chat_id = env::var("TELEGRAM_CHAT_ID").ok();

    // Simulation Mode
    if let Ok(val) = env::var("SIMULATION_MODE") {
        config.simulation_mode = val.parse()?;
    }

    // Validate configuration
    validate_config(&config)?;

    Ok(config)
}

/// Validate configuration
fn validate_config(config: &BotConfig) -> Result<(), Box<dyn std::error::Error>> {
    if !config.simulation_mode && config.private_key.is_none() {
        return Err("PRIVATE_KEY is required when not in simulation mode".into());
    }

    if config.rpc_url.is_empty() {
        return Err("RPC_URL is required".into());
    }

    if config.buy_amount_sol <= 0.0 {
        return Err("BUY_AMOUNT_SOL must be greater than 0".into());
    }

    Ok(())
}

/// Pump.fun program constants
pub mod constants {
    use solana_sdk::pubkey::Pubkey;

    // Pump.fun Program ID
    pub const PUMP_FUN_PROGRAM_ID: Pubkey = solana_sdk::pubkey!("6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P");

    // System Program ID
    pub const SYSTEM_PROGRAM_ID: Pubkey = solana_sdk::pubkey!("11111111111111111111111111111112");

    // Associated Token Program ID
    pub const ASSOCIATED_TOKEN_PROGRAM_ID: Pubkey = solana_sdk::pubkey!("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL");

    // Token Program ID
    pub const TOKEN_PROGRAM_ID: Pubkey = solana_sdk::pubkey!("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");

    // Rent Program ID
    pub const RENT_PROGRAM_ID: Pubkey = solana_sdk::pubkey!("SysvarRent111111111111111111111111111111111");

    // Pump.fun Fee Recipient
    pub const PUMP_FUN_FEE_RECIPIENT: Pubkey = solana_sdk::pubkey!("CebN5WGQ4jvEPvsVU4EoHEpgzq1VV7AbicfhtW4xC9iM");

    // Bonding curve seed
    pub const BONDING_CURVE_SEED: &str = "bonding-curve";

    // Metadata seed
    pub const METADATA_SEED: &str = "metadata";

    // Default commitment
    pub const DEFAULT_COMMITMENT: solana_sdk::commitment_config::CommitmentLevel =
        solana_sdk::commitment_config::CommitmentLevel::Confirmed;

    // Time constants (in milliseconds)
    pub const ONE_MINUTE_MS: u64 = 60 * 1000;
    pub const ONE_HOUR_MS: u64 = 60 * ONE_MINUTE_MS;
    pub const ONE_DAY_MS: u64 = 24 * ONE_HOUR_MS;

    // Solana constants
    pub const SOL_DECIMALS: u32 = 9;
    pub const LAMPORTS_PER_SOL: u64 = 1_000_000_000;
}

/// Transaction types for logging
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransactionType {
    Buy,
    Sell,
    Transfer,
}

/// Token safety status
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TokenSafetyStatus {
    Safe,
    Suspicious,
    Dangerous,
}

/// Trading status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TradingStatus {
    Active,
    Paused,
    Stopped,
}
