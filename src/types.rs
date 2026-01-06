use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;
use chrono::{DateTime, Utc};
use crate::config::constants::TokenSafetyStatus;

/// Token information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenInfo {
    pub address: Pubkey,
    pub name: String,
    pub symbol: String,
    pub description: Option<String>,
    pub image: Option<String>,
    pub metadata_uri: Option<String>,
    pub twitter: Option<String>,
    pub telegram: Option<String>,
    pub website: Option<String>,
    pub creator: Pubkey,
    pub created_at: DateTime<Utc>,
}

/// Bonding curve information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BondingCurveInfo {
    pub address: Pubkey,
    pub token_address: Pubkey,
    pub virtual_sol_reserves: u64,
    pub virtual_token_reserves: u64,
    pub real_sol_reserves: u64,
    pub real_token_reserves: u64,
    pub token_total_supply: u64,
    pub complete: bool,
}

/// Token metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenMetrics {
    pub market_cap: f64,
    pub liquidity: f64,
    pub holders: u32,
    pub volume_24h: f64,
    pub price: f64,
    pub price_change_24h: f64,
}

/// Token analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenAnalysis {
    pub token: TokenInfo,
    pub bonding_curve: BondingCurveInfo,
    pub metrics: TokenMetrics,
    pub safety: TokenSafety,
    pub opportunities: TokenOpportunities,
}

/// Token safety information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenSafety {
    pub status: TokenSafetyStatus,
    pub score: u32,
    pub checks: SafetyChecks,
}

/// Safety checks results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyChecks {
    pub has_lock: bool,
    pub mint_revoked: bool,
    pub is_honeypot: bool,
    pub has_social_links: bool,
    pub creator_verified: bool,
    pub suspicious_creator: bool,
}

/// Token opportunities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenOpportunities {
    pub score: u32,
    pub reasons: Vec<String>,
}

/// Position information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub token_address: Pubkey,
    pub token_symbol: String,
    pub amount: u64,
    pub entry_price: f64,
    pub current_price: f64,
    pub pnl: f64,
    pub pnl_percentage: f64,
    pub opened_at: DateTime<Utc>,
    pub last_updated: DateTime<Utc>,
    pub take_profit_price: Option<f64>,
    pub stop_loss_price: Option<f64>,
    pub trailing_stop_price: Option<f64>,
    pub status: PositionStatus,
}

/// Position status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PositionStatus {
    Open,
    Closed,
    Partial,
}

/// Trade result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeResult {
    pub signature: String,
    pub token_address: Pubkey,
    pub trade_type: TradeType,
    pub amount: u64,
    pub price: f64,
    pub total_value: f64,
    pub fee: f64,
    pub timestamp: DateTime<Utc>,
    pub success: bool,
    pub error: Option<String>,
}

/// Trade type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TradeType {
    Buy,
    Sell,
}

/// Wallet balance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletBalance {
    pub sol: f64,
    pub tokens: std::collections::HashMap<String, u64>,
    pub last_updated: DateTime<Utc>,
}

/// New token event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewTokenEvent {
    pub token_address: Pubkey,
    pub bonding_curve_address: Pubkey,
    pub creator: Pubkey,
    pub timestamp: DateTime<Utc>,
}

/// Buy instruction parameters
#[derive(Debug, Clone)]
pub struct BuyInstruction {
    pub token_address: Pubkey,
    pub bonding_curve_address: Pubkey,
    pub associated_bonding_curve: Pubkey,
    pub amount: u64,  // Amount of tokens to buy
    pub max_sol_cost: u64,  // Maximum SOL to spend in lamports
}

/// Sell instruction parameters
#[derive(Debug, Clone)]
pub struct SellInstruction {
    pub token_address: Pubkey,
    pub bonding_curve_address: Pubkey,
    pub associated_bonding_curve: Pubkey,
    pub user_token_account: Pubkey,
    pub amount: u64,  // Amount of tokens to sell
    pub min_sol_output: u64,  // Minimum SOL to receive
}

/// Safety check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyCheckResult {
    pub passed: bool,
    pub score: u32,
    pub issues: Vec<String>,
    pub recommendations: Vec<String>,
}

/// Wallet information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletInfo {
    pub public_key: Pubkey,
    pub balance: f64,
    pub last_updated: DateTime<Utc>,
}

/// Health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    pub timestamp: DateTime<Utc>,
    pub solana_connection: bool,
    pub monitoring_active: bool,
    pub trading_active: bool,
    pub active_positions: usize,
    pub simulation_mode: bool,
}
