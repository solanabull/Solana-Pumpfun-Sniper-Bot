use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    transaction::Transaction,
    system_instruction,
    native_token::LAMPORTS_PER_SOL,
};
use std::sync::Arc;
use crate::config::{BotConfig, constants};

/// Solana client wrapper for the bot
pub struct SolanaClient {
    rpc_client: RpcClient,
    keypair: Option<Keypair>,
    main_keypair: Option<Keypair>,
}

impl SolanaClient {
    /// Create a new Solana client
    pub async fn new(config: &BotConfig) -> Result<Self, Box<dyn std::error::Error>> {
        let commitment_config = CommitmentConfig {
            commitment: constants::DEFAULT_COMMITMENT,
        };

        let rpc_client = RpcClient::new_with_commitment(
            config.rpc_url.clone(),
            commitment_config,
        );

        // Initialize keypairs
        let keypair = if let Some(private_key) = &config.private_key {
            Some(Self::keypair_from_base58(private_key)?)
        } else {
            None
        };

        let main_keypair = if let Some(private_key) = &config.main_wallet_private_key {
            Some(Self::keypair_from_base58(private_key)?)
        } else {
            None
        };

        Ok(Self {
            rpc_client,
            keypair,
            main_keypair,
        })
    }

    /// Get the RPC client
    pub fn rpc_client(&self) -> &RpcClient {
        &self.rpc_client
    }

    /// Get the trading keypair
    pub fn keypair(&self) -> Option<&Keypair> {
        self.keypair.as_ref()
    }

    /// Get the main wallet keypair
    pub fn main_keypair(&self) -> Option<&Keypair> {
        self.main_keypair.as_ref()
    }

    /// Get the public key of the trading wallet
    pub fn public_key(&self) -> Result<Pubkey, Box<dyn std::error::Error>> {
        self.keypair
            .as_ref()
            .map(|kp| kp.pubkey())
            .ok_or_else(|| "No trading wallet configured".into())
    }

    /// Get balance for a public key
    pub async fn get_balance(&self, pubkey: &Pubkey) -> Result<f64, Box<dyn std::error::Error>> {
        let balance = self.rpc_client.get_balance(pubkey)?;
        Ok(balance as f64 / LAMPORTS_PER_SOL as f64)
    }

    /// Get the current balance of the trading wallet
    pub async fn get_wallet_balance(&self) -> Result<f64, Box<dyn std::error::Error>> {
        let pubkey = self.public_key()?;
        self.get_balance(&pubkey).await
    }

    /// Get recent blockhash
    pub async fn get_recent_blockhash(&self) -> Result<String, Box<dyn std::error::Error>> {
        let (blockhash, _) = self.rpc_client.get_recent_blockhash()?;
        Ok(blockhash.to_string())
    }

    /// Send a transaction
    pub async fn send_transaction(
        &self,
        mut transaction: Transaction,
    ) -> Result<String, Box<dyn std::error::Error>> {
        // Sign the transaction if we have a keypair
        if let Some(keypair) = &self.keypair {
            let recent_blockhash = self.rpc_client.get_recent_blockhash()?.0;
            transaction.sign(&[keypair], recent_blockhash);

            // Send the transaction
            let signature = self.rpc_client.send_and_confirm_transaction(&transaction)?;
            Ok(signature.to_string())
        } else {
            Err("No trading wallet configured for signing".into())
        }
    }

    /// Get latest block height
    pub async fn get_latest_block_height(&self) -> Result<u64, Box<dyn std::error::Error>> {
        let block_height = self.rpc_client.get_block_height()?;
        Ok(block_height)
    }

    /// Get priority fee estimate
    pub async fn get_priority_fee_estimate(&self) -> Result<u64, Box<dyn std::error::Error>> {
        // Get recent priority fees
        let fees = self.rpc_client.get_recent_prioritization_fees(&[])?;

        if fees.is_empty() {
            return Ok(10000); // Default fee
        }

        // Calculate average fee
        let total: u64 = fees.iter().map(|fee| fee.prioritization_fee).sum();
        let avg_fee = total / fees.len() as u64;

        Ok(avg_fee.max(10000).min(100000)) // Clamp between min and max
    }

    /// Health check
    pub async fn health_check(&self) -> Result<bool, Box<dyn std::error::Error>> {
        match self.rpc_client.get_version() {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    /// Create keypair from base58 string
    fn keypair_from_base58(private_key: &str) -> Result<Keypair, Box<dyn std::error::Error>> {
        let secret_key = bs58::decode(private_key)
            .into_vec()
            .map_err(|e| format!("Invalid base58 private key: {}", e))?;

        let keypair = Keypair::from_bytes(&secret_key)
            .map_err(|e| format!("Invalid keypair bytes: {}", e))?;

        Ok(keypair)
    }

    /// Transfer SOL between wallets (for refueling)
    pub async fn transfer_sol(
        &self,
        to: &Pubkey,
        amount_lamports: u64,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let from_keypair = self.keypair.as_ref()
            .ok_or("No trading wallet configured")?;

        let instruction = system_instruction::transfer(
            &from_keypair.pubkey(),
            to,
            amount_lamports,
        );

        let mut transaction = Transaction::new_with_payer(
            &[instruction],
            Some(&from_keypair.pubkey()),
        );

        let recent_blockhash = self.rpc_client.get_recent_blockhash()?.0;
        transaction.sign(&[from_keypair], recent_blockhash);

        let signature = self.rpc_client.send_and_confirm_transaction(&transaction)?;
        Ok(signature.to_string())
    }
}
