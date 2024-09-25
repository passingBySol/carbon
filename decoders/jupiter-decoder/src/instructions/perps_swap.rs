
use carbon_core::deserialize::{ArrangeAccounts, CarbonDeserialize};
use carbon_proc_macros::CarbonDeserialize;
use crate::types::*;


#[derive(CarbonDeserialize, Debug, serde::Serialize, PartialEq, Eq, Clone, Hash)]
#[carbon(discriminator = "0xbcfa5dc84360c626")]
pub struct PerpsSwap{
}

pub struct PerpsSwapInstructionAccounts {
    pub swap_program: solana_sdk::pubkey::Pubkey,
    pub owner: solana_sdk::pubkey::Pubkey,
    pub funding_account: solana_sdk::pubkey::Pubkey,
    pub receiving_account: solana_sdk::pubkey::Pubkey,
    pub transfer_authority: solana_sdk::pubkey::Pubkey,
    pub perpetuals: solana_sdk::pubkey::Pubkey,
    pub pool: solana_sdk::pubkey::Pubkey,
    pub receiving_custody: solana_sdk::pubkey::Pubkey,
    pub receiving_custody_oracle_account: solana_sdk::pubkey::Pubkey,
    pub receiving_custody_token_account: solana_sdk::pubkey::Pubkey,
    pub dispensing_custody: solana_sdk::pubkey::Pubkey,
    pub dispensing_custody_oracle_account: solana_sdk::pubkey::Pubkey,
    pub dispensing_custody_token_account: solana_sdk::pubkey::Pubkey,
    pub token_program: solana_sdk::pubkey::Pubkey,
    pub event_authority: solana_sdk::pubkey::Pubkey,
    pub program: solana_sdk::pubkey::Pubkey,
}

impl ArrangeAccounts for PerpsSwap {
    type ArrangedAccounts = PerpsSwapInstructionAccounts;

    fn arrange_accounts(&self, accounts: Vec<solana_sdk::pubkey::Pubkey>) -> Option<Self::ArrangedAccounts> {
        let swap_program = accounts.get(0)?;
        let owner = accounts.get(1)?;
        let funding_account = accounts.get(2)?;
        let receiving_account = accounts.get(3)?;
        let transfer_authority = accounts.get(4)?;
        let perpetuals = accounts.get(5)?;
        let pool = accounts.get(6)?;
        let receiving_custody = accounts.get(7)?;
        let receiving_custody_oracle_account = accounts.get(8)?;
        let receiving_custody_token_account = accounts.get(9)?;
        let dispensing_custody = accounts.get(10)?;
        let dispensing_custody_oracle_account = accounts.get(11)?;
        let dispensing_custody_token_account = accounts.get(12)?;
        let token_program = accounts.get(13)?;
        let event_authority = accounts.get(14)?;
        let program = accounts.get(15)?;

        Some(PerpsSwapInstructionAccounts {
            swap_program: *swap_program,
            owner: *owner,
            funding_account: *funding_account,
            receiving_account: *receiving_account,
            transfer_authority: *transfer_authority,
            perpetuals: *perpetuals,
            pool: *pool,
            receiving_custody: *receiving_custody,
            receiving_custody_oracle_account: *receiving_custody_oracle_account,
            receiving_custody_token_account: *receiving_custody_token_account,
            dispensing_custody: *dispensing_custody,
            dispensing_custody_oracle_account: *dispensing_custody_oracle_account,
            dispensing_custody_token_account: *dispensing_custody_token_account,
            token_program: *token_program,
            event_authority: *event_authority,
            program: *program,
        })
    }
}