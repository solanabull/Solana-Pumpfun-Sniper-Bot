use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::RwLock;
use chrono::Utc;
use crate::{
    config::BotConfig,
    types::{TokenAnalysis, TradeResult, TradeType, Position, PositionStatus},
    utils::{solana_client::SolanaClient, transaction_builder::TransactionBuilder},
};

/// Trading bot for executing buy/sell orders
pub struct Trader {
    client: Arc<SolanaClient>,
    config: Arc<BotConfig>,
    transaction_builder: Arc<TransactionBuilder>,
    positions: Arc<RwLock<HashMap<String, Position>>>,
    is_buying: Arc<RwLock<bool>>,
    is_selling: Arc<RwLock<bool>>,
    last_buy_time: Arc<RwLock<u64>>,
    daily_trades: Arc<RwLock<u32>>,
    last_reset_date: Arc<RwLock<String>>,
}

impl Trader {
    /// Create a new trader
    pub async fn new(
        client: Arc<SolanaClient>,
        config: Arc<BotConfig>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let transaction_builder = Arc::new(TransactionBuilder::new(
            Arc::clone(&client),
            Arc::clone(&config),
        ));

        Ok(Self {
            client,
            config,
            transaction_builder,
            positions: Arc::new(RwLock::new(HashMap::new())),
            is_buying: Arc::new(RwLock::new(false)),
            is_selling: Arc::new(RwLock::new(false)),
            last_buy_time: Arc::new(RwLock::new(0)),
            daily_trades: Arc::new(RwLock::new(0)),
            last_reset_date: Arc::new(RwLock::new(Utc::now().format("%Y-%m-%d").to_string())),
        })
    }

    /// Get client reference
    pub fn client(&self) -> &Arc<SolanaClient> {
        &self.client
    }

    /// Execute a buy order
    pub async fn execute_buy(&self, analysis: &TokenAnalysis) -> Result<(), Box<dyn std::error::Error>> {
        // Check if buying is allowed
        if !self.can_buy().await {
            tracing::warn!("Buy blocked by safety limits");
            return Ok(());
        }

        // Check simulation mode
        if self.config.simulation_mode {
            return self.simulate_buy(analysis).await;
        }

        // Check balance
        let balance = self.client.get_wallet_balance().await?;
        if balance < self.config.buy_amount_sol + 0.01 {
            tracing::warn!("Insufficient balance for buy: {} SOL", balance);
            return Ok(());
        }

        tracing::info!(
            "Executing buy for {}: {} SOL",
            analysis.token.symbol,
            self.config.buy_amount_sol
        );

        *self.is_buying.write().await = true;

        // Build transaction
        let transaction = self.transaction_builder.build_buy_transaction(
            &analysis.token.address,
            &analysis.bonding_curve.address,
            self.config.buy_amount_sol,
            self.config.max_slippage,
        ).await?;

        // Send transaction
        match self.client.send_transaction(transaction).await {
            Ok(signature) => {
                // Update tracking
                self.update_buy_tracking().await;

                // Create position
                self.create_position(analysis, signature).await;

                tracing::info!(
                    "Buy executed successfully: {} - {}",
                    analysis.token.symbol,
                    signature
                );

                Ok(())
            }
            Err(e) => {
                tracing::error!("Buy execution failed: {}", e);
                Ok(())
            }
        }
    }

    /// Execute a sell order
    pub async fn execute_sell(
        &self,
        position: &Position,
        percentage: f64,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if *self.is_selling.read().await {
            tracing::warn!("Sell already in progress");
            return Ok(());
        }

        if self.config.simulation_mode {
            return self.simulate_sell(position, percentage).await;
        }

        let amount_to_sell = ((position.amount as f64) * percentage / 100.0) as u64;
        let estimated_value = (amount_to_sell as f64) * position.current_price;
        let min_sol_output = ((estimated_value * (1.0 - self.config.max_slippage / 100.0)) * 1_000_000_000.0) as u64;

        tracing::info!(
            "Executing sell for {}: {}% ({} tokens)",
            position.token_symbol,
            percentage,
            amount_to_sell
        );

        *self.is_selling.write().await = true;

        // Build transaction
        let transaction = self.transaction_builder.build_sell_transaction(
            &position.token_address,
            &solana_sdk::pubkey::Pubkey::new_unique(), // Would need actual bonding curve
            amount_to_sell,
            min_sol_output,
        ).await?;

        // Send transaction
        match self.client.send_transaction(transaction).await {
            Ok(signature) => {
                // Update position
                self.update_position_after_sell(position, amount_to_sell).await;

                tracing::info!(
                    "Sell executed successfully: {} - {}",
                    position.token_symbol,
                    signature
                );

                Ok(())
            }
            Err(e) => {
                tracing::error!("Sell execution failed: {}", e);
                Ok(())
            }
        }
    }

    /// Check automated sells for take-profit/stop-loss
    pub async fn check_automated_sells(&self) -> Result<(), Box<dyn std::error::Error>> {
        let positions: Vec<Position> = self.positions.read().await.values().cloned().collect();

        for position in positions {
            // Update position price (simplified)
            self.update_position_price(&position).await?;

            // Check take profit
            if self.should_take_profit(&position) {
                self.execute_sell(&position, 100.0).await?;
            }
            // Check stop loss
            else if self.should_stop_loss(&position) {
                self.execute_sell(&position, 100.0).await?;
            }
        }

        Ok(())
    }

    /// Simulate a buy for testing
    async fn simulate_buy(&self, analysis: &TokenAnalysis) -> Result<(), Box<dyn std::error::Error>> {
        tracing::info!(
            "[SIMULATION] Buy executed for {}: {} SOL",
            analysis.token.symbol,
            self.config.buy_amount_sol
        );

        self.update_buy_tracking().await;
        self.create_position(analysis, "sim_".to_string() + &Utc::now().timestamp().to_string()).await;

        Ok(())
    }

    /// Simulate a sell for testing
    async fn simulate_sell(&self, position: &Position, percentage: f64) -> Result<(), Box<dyn std::error::Error>> {
        tracing::info!(
            "[SIMULATION] Sell executed for {}: {}%",
            position.token_symbol,
            percentage
        );

        let amount_to_sell = ((position.amount as f64) * percentage / 100.0) as u64;
        self.update_position_after_sell(position, amount_to_sell).await;

        Ok(())
    }

    /// Check if buying is allowed
    async fn can_buy(&self) -> bool {
        // Check cooldown
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        let last_buy = *self.last_buy_time.read().await;
        if now - last_buy < self.config.trading_cooldown_ms {
            return false;
        }

        // Check daily trade limit
        self.reset_daily_trades_if_needed().await;
        if *self.daily_trades.read().await >= self.config.max_trades_per_hour * 24 {
            return false;
        }

        // Check if another buy is in progress
        if *self.is_buying.read().await {
            return false;
        }

        true
    }

    /// Update buy tracking
    async fn update_buy_tracking(&self) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        *self.last_buy_time.write().await = now;
        *self.daily_trades.write().await += 1;
    }

    /// Reset daily trades if needed
    async fn reset_daily_trades_if_needed(&self) {
        let today = Utc::now().format("%Y-%m-%d").to_string();
        if today != *self.last_reset_date.read().await {
            *self.daily_trades.write().await = 0;
            *self.last_reset_date.write().await = today;
        }
    }

    /// Create a new position after successful buy
    async fn create_position(&self, analysis: &TokenAnalysis, signature: String) {
        let position = Position {
            token_address: analysis.token.address,
            token_symbol: analysis.token.symbol.clone(),
            amount: (self.config.buy_amount_sol * 1_000_000.0) as u64, // Approximate
            entry_price: analysis.metrics.price,
            current_price: analysis.metrics.price,
            pnl: 0.0,
            pnl_percentage: 0.0,
            opened_at: Utc::now(),
            last_updated: Utc::now(),
            take_profit_price: Some(analysis.metrics.price * (1.0 + self.config.take_profit_percentage / 100.0)),
            stop_loss_price: Some(analysis.metrics.price * (1.0 - self.config.stop_loss_percentage / 100.0)),
            trailing_stop_price: None,
            status: PositionStatus::Open,
        };

        self.positions.write().await.insert(
            position.token_address.to_string(),
            position
        );
    }

    /// Update position after sell
    async fn update_position_after_sell(&self, position: &Position, amount_sold: u64) {
        let mut positions = self.positions.write().await;
        if let Some(pos) = positions.get_mut(&position.token_address.to_string()) {
            pos.amount -= amount_sold;
            if pos.amount == 0 {
                pos.status = PositionStatus::Closed;
            } else {
                pos.status = PositionStatus::Partial;
            }
            pos.last_updated = Utc::now();
        }
    }

    /// Update position price (simplified)
    async fn update_position_price(&self, position: &Position) -> Result<(), Box<dyn std::error::Error>> {
        // In a real implementation, you'd fetch current price from the blockchain
        // For now, simulate small price movements
        let price_change = (rand::random::<f64>() - 0.5) * 0.1; // -5% to +5%
        let new_price = position.current_price * (1.0 + price_change);

        let mut positions = self.positions.write().await;
        if let Some(pos) = positions.get_mut(&position.token_address.to_string()) {
            pos.current_price = new_price;
            pos.pnl = (new_price - pos.entry_price) * pos.amount as f64;
            pos.pnl_percentage = ((new_price - pos.entry_price) / pos.entry_price) * 100.0;
            pos.last_updated = Utc::now();
        }

        Ok(())
    }

    /// Check if position should take profit
    fn should_take_profit(&self, position: &Position) -> bool {
        if let Some(tp_price) = position.take_profit_price {
            return position.current_price >= tp_price;
        }
        false
    }

    /// Check if position should stop loss
    fn should_stop_loss(&self, position: &Position) -> bool {
        if let Some(sl_price) = position.stop_loss_price {
            return position.current_price <= sl_price;
        }
        false
    }

    /// Stop the trader
    pub async fn stop(&self) -> Result<(), Box<dyn std::error::Error>> {
        *self.is_buying.write().await = false;
        *self.is_selling.write().await = false;
        tracing::info!("Trader stopped");
        Ok(())
    }

    /// Get trader status
    pub async fn status(&self) -> serde_json::Value {
        let positions_count = self.positions.read().await.len();
        let is_buying = *self.is_buying.read().await;
        let is_selling = *self.is_selling.read().await;

        serde_json::json!({
            "is_buying": is_buying,
            "is_selling": is_selling,
            "active_positions": positions_count,
            "daily_trades": *self.daily_trades.read().await,
        })
    }
}
