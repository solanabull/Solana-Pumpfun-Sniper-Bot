use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    system_program,
    compute_budget,
};
use crate::{
    config::BotConfig,
    types::{BuyInstruction, SellInstruction},
    utils::solana_client::SolanaClient,
};

/// Transaction builder for Pump.fun operations
pub struct TransactionBuilder {
    client: std::sync::Arc<SolanaClient>,
    config: std::sync::Arc<BotConfig>,
}

impl TransactionBuilder {
    /// Create a new transaction builder
    pub fn new(
        client: std::sync::Arc<SolanaClient>,
        config: std::sync::Arc<BotConfig>,
    ) -> Self {
        Self { client, config }
    }

    /// Build a buy transaction
    pub async fn build_buy_transaction(
        &self,
        token_address: &Pubkey,
        bonding_curve_address: &Pubkey,
        amount_sol: f64,
        slippage_percentage: f64,
    ) -> Result<solana_sdk::transaction::Transaction, Box<dyn std::error::Error>> {
        // Calculate amounts
        let amount_lamports = (amount_sol * crate::config::constants::LAMPORTS_PER_SOL as f64) as u64;
        let max_sol_cost = ((amount_lamports as f64) * (1.0 + slippage_percentage / 100.0)) as u64;

        // Get associated bonding curve
        let associated_bonding_curve = self.find_associated_token_address(
            bonding_curve_address,
            token_address,
        )?;

        let buy_instruction = BuyInstruction {
            token_address: *token_address,
            bonding_curve_address: *bonding_curve_address,
            associated_bonding_curve,
            amount: amount_lamports,
            max_sol_cost,
        };

        // Get priority fee
        let priority_fee = self.client.get_priority_fee_estimate().await?;

        // Build instructions
        let mut instructions = Vec::new();

        // Add compute budget instructions
        instructions.push(
            compute_budget::ComputeBudgetInstruction::set_compute_unit_price(priority_fee),
        );

        instructions.push(
            compute_budget::ComputeBudgetInstruction::set_compute_unit_limit(200_000),
        );

        // Add buy instruction
        instructions.push(self.create_buy_instruction(&buy_instruction)?);

        // Create transaction
        let mut transaction = solana_sdk::transaction::Transaction::new_with_payer(
            &instructions,
            Some(&self.client.public_key()?),
        );

        Ok(transaction)
    }

    /// Build a sell transaction
    pub async fn build_sell_transaction(
        &self,
        token_address: &Pubkey,
        bonding_curve_address: &Pubkey,
        amount: u64,
        min_sol_output: u64,
    ) -> Result<solana_sdk::transaction::Transaction, Box<dyn std::error::Error>> {
        // Get associated accounts
        let associated_bonding_curve = self.find_associated_token_address(
            bonding_curve_address,
            token_address,
        )?;

        let user_token_account = self.find_associated_token_address(
            &self.client.public_key()?,
            token_address,
        )?;

        let sell_instruction = SellInstruction {
            token_address: *token_address,
            bonding_curve_address: *bonding_curve_address,
            associated_bonding_curve,
            user_token_account,
            amount,
            min_sol_output,
        };

        // Get priority fee
        let priority_fee = self.client.get_priority_fee_estimate().await?;

        // Build instructions
        let mut instructions = Vec::new();

        // Add compute budget instructions
        instructions.push(
            compute_budget::ComputeBudgetInstruction::set_compute_unit_price(priority_fee),
        );

        instructions.push(
            compute_budget::ComputeBudgetInstruction::set_compute_unit_limit(200_000),
        );

        // Add sell instruction
        instructions.push(self.create_sell_instruction(&sell_instruction)?);

        // Create transaction
        let mut transaction = solana_sdk::transaction::Transaction::new_with_payer(
            &instructions,
            Some(&self.client.public_key()?),
        );

        Ok(transaction)
    }

    /// Create buy instruction for Pump.fun
    fn create_buy_instruction(
        &self,
        params: &BuyInstruction,
    ) -> Result<Instruction, Box<dyn std::error::Error>> {
        use crate::config::constants::*;

        // Pump.fun buy instruction accounts (approximate)
        let accounts = vec![
            AccountMeta::new(self.client.public_key()?, true), // User
            AccountMeta::new_readonly(PUMP_FUN_FEE_RECIPIENT, false), // Fee recipient
            AccountMeta::new(params.token_address, false), // Mint
            AccountMeta::new(params.bonding_curve_address, false), // Bonding curve
            AccountMeta::new(params.associated_bonding_curve, false), // Associated bonding curve
            AccountMeta::new_readonly(system_program::id(), false), // System program
            AccountMeta::new_readonly(spl_token::id(), false), // Token program
            AccountMeta::new_readonly(spl_associated_token_account::id(), false), // Associated token program
        ];

        // Instruction data for buy (simplified)
        let mut data = vec![0x00]; // Buy instruction discriminator
        data.extend_from_slice(&params.amount.to_le_bytes());
        data.extend_from_slice(&params.max_sol_cost.to_le_bytes());

        Ok(Instruction {
            program_id: PUMP_FUN_PROGRAM_ID,
            accounts,
            data,
        })
    }

    /// Create sell instruction for Pump.fun
    fn create_sell_instruction(
        &self,
        params: &SellInstruction,
    ) -> Result<Instruction, Box<dyn std::error::Error>> {
        use crate::config::constants::*;

        // Pump.fun sell instruction accounts
        let accounts = vec![
            AccountMeta::new(self.client.public_key()?, true), // User
            AccountMeta::new_readonly(PUMP_FUN_FEE_RECIPIENT, false), // Fee recipient
            AccountMeta::new(params.token_address, false), // Mint
            AccountMeta::new(params.bonding_curve_address, false), // Bonding curve
            AccountMeta::new(params.associated_bonding_curve, false), // Associated bonding curve
            AccountMeta::new(params.user_token_account, false), // User token account
            AccountMeta::new_readonly(system_program::id(), false), // System program
            AccountMeta::new_readonly(spl_token::id(), false), // Token program
            AccountMeta::new_readonly(spl_associated_token_account::id(), false), // Associated token program
        ];

        // Instruction data for sell
        let mut data = vec![0x01]; // Sell instruction discriminator
        data.extend_from_slice(&params.amount.to_le_bytes());
        data.extend_from_slice(&params.min_sol_output.to_le_bytes());

        Ok(Instruction {
            program_id: PUMP_FUN_PROGRAM_ID,
            accounts,
            data,
        })
    }

    /// Find associated token address
    fn find_associated_token_address(
        &self,
        owner: &Pubkey,
        mint: &Pubkey,
    ) -> Result<Pubkey, Box<dyn std::error::Error>> {
        // For now, return a placeholder - would need proper derivation
        // In a real implementation, you'd use spl_associated_token_account::get_associated_token_address
        Ok(Pubkey::new_unique()) // Placeholder
    }
}
