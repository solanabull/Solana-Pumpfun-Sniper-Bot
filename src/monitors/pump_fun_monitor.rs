use futures_util::{SinkExt, StreamExt};
use solana_client::rpc_config::{RpcTransactionLogsConfig, RpcTransactionLogsFilter};
use solana_sdk::commitment_config::CommitmentConfig;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use serde_json::json;
use crate::{
    config::{BotConfig, constants::*},
    types::NewTokenEvent,
    utils::solana_client::SolanaClient,
};

/// Pump.fun token launch monitor
pub struct PumpFunMonitor {
    client: Arc<SolanaClient>,
    config: Arc<BotConfig>,
    event_sender: mpsc::UnboundedSender<NewTokenEvent>,
    event_receiver: Arc<RwLock<Option<mpsc::UnboundedReceiver<NewTokenEvent>>>>,
    is_monitoring: Arc<RwLock<bool>>,
}

impl PumpFunMonitor {
    /// Create a new Pump.fun monitor
    pub fn new(
        client: Arc<SolanaClient>,
        config: Arc<BotConfig>,
    ) -> Self {
        let (event_sender, event_receiver) = mpsc::unbounded_channel();

        Self {
            client,
            config,
            event_sender,
            event_receiver: Arc::new(RwLock::new(Some(event_receiver))),
            is_monitoring: Arc::new(RwLock::new(false)),
        }
    }

    /// Start monitoring for new token launches
    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        if *self.is_monitoring.read().await {
            tracing::info!("Pump.fun monitor is already running");
            return Ok(());
        }

        *self.is_monitoring.write().await = true;

        tracing::info!("Starting Pump.fun token launch monitor...");

        // Start WebSocket monitoring
        self.start_websocket_monitoring().await?;

        tracing::info!("Pump.fun monitor started successfully");
        Ok(())
    }

    /// Stop monitoring
    pub async fn stop(&self) -> Result<(), Box<dyn std::error::Error>> {
        if !*self.is_monitoring.read().await {
            tracing::info!("Pump.fun monitor is not running");
            return Ok(());
        }

        *self.is_monitoring.write().await = false;

        // Close the event receiver
        if let Some(receiver) = self.event_receiver.write().await.take() {
            drop(receiver);
        }

        tracing::info!("Pump.fun monitor stopped successfully");
        Ok(())
    }

    /// Register callback for new token events
    pub async fn on_new_token<F>(&self, callback: F)
    where
        F: Fn(NewTokenEvent) + Send + Sync + 'static,
    {
        let mut receiver = self.event_receiver.write().await.take().unwrap();

        tokio::spawn(async move {
            while let Some(event) = receiver.recv().await {
                callback(event);
            }
        });
    }

    /// Start WebSocket monitoring for program logs
    async fn start_websocket_monitoring(&self) -> Result<(), Box<dyn std::error::Error>> {
        let ws_url = self.config.ws_url.as_ref()
            .ok_or("WebSocket URL not configured")?;

        let (ws_stream, _) = connect_async(ws_url).await?;
        let (mut write, mut read) = ws_stream.split();

        // Subscribe to program logs
        let subscribe_message = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "logsSubscribe",
            "params": [
                RpcTransactionLogsFilter::Mentions(vec![PUMP_FUN_PROGRAM_ID.to_string()]),
                RpcTransactionLogsConfig {
                    commitment: Some(CommitmentConfig {
                        commitment: DEFAULT_COMMITMENT,
                    }),
                }
            ]
        });

        write.send(Message::Text(subscribe_message.to_string())).await?;

        // Handle incoming messages
        let event_sender = self.event_sender.clone();
        let is_monitoring = Arc::clone(&self.is_monitoring);

        tokio::spawn(async move {
            while let Some(message) = read.next().await {
                if !*is_monitoring.read().await {
                    break;
                }

                match message {
                    Ok(Message::Text(text)) => {
                        if let Err(e) = Self::handle_websocket_message(&text, &event_sender).await {
                            tracing::error!("Error handling WebSocket message: {}", e);
                        }
                    }
                    Ok(Message::Close(_)) => {
                        tracing::info!("WebSocket connection closed");
                        break;
                    }
                    Err(e) => {
                        tracing::error!("WebSocket error: {}", e);
                        break;
                    }
                    _ => {}
                }
            }
        });

        Ok(())
    }

    /// Handle WebSocket message
    async fn handle_websocket_message(
        text: &str,
        event_sender: &mpsc::UnboundedSender<NewTokenEvent>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let message: serde_json::Value = serde_json::from_str(text)?;

        // Check if this is a logs notification
        if let Some(params) = message.get("params") {
            if let Some(result) = params.get("result") {
                if let Some(logs) = Self::extract_logs_from_notification(result) {
                    if let Some(token_event) = Self::parse_token_creation(logs).await {
                        if event_sender.send(token_event).is_err() {
                            tracing::error!("Failed to send token event - channel closed");
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Extract logs from notification
    fn extract_logs_from_notification(result: &serde_json::Value) -> Option<&serde_json::Value> {
        result.get("value").and_then(|v| v.get("logs"))
    }

    /// Parse token creation from transaction logs
    async fn parse_token_creation(logs: &serde_json::Value) -> Option<NewTokenEvent> {
        if let Some(logs_array) = logs.as_array() {
            // Look for Pump.fun specific log patterns
            let has_create_log = logs_array.iter().any(|log| {
                log.as_str()
                    .map(|s| s.contains("Create") || s.contains("create"))
                    .unwrap_or(false)
            });

            if has_create_log {
                // In a real implementation, you'd parse the transaction to get token details
                // For now, return a placeholder event
                Some(NewTokenEvent {
                    token_address: solana_sdk::pubkey::Pubkey::new_unique(),
                    bonding_curve_address: solana_sdk::pubkey::Pubkey::new_unique(),
                    creator: solana_sdk::pubkey::Pubkey::new_unique(),
                    timestamp: chrono::Utc::now(),
                })
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Get monitor status
    pub async fn status(&self) -> serde_json::Value {
        json!({
            "is_monitoring": *self.is_monitoring.read().await,
            "program_id": PUMP_FUN_PROGRAM_ID.to_string(),
        })
    }
}
