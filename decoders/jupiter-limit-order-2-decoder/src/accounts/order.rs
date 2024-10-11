 
use carbon_core::deserialize::CarbonDeserialize; 
use carbon_proc_macros::CarbonDeserialize; 
use carbon_core::borsh;
use super::super::types::*;

#[derive(CarbonDeserialize, Debug)] 
 

#[carbon(discriminator = "0x86addfb94d561c33")] 
pub struct Order { 
        pub maker: solana_sdk::pubkey::Pubkey, 
        pub input_mint: solana_sdk::pubkey::Pubkey, 
        pub output_mint: solana_sdk::pubkey::Pubkey, 
        pub input_token_program: solana_sdk::pubkey::Pubkey, 
        pub output_token_program: solana_sdk::pubkey::Pubkey, 
        pub input_mint_reserve: solana_sdk::pubkey::Pubkey, 
        pub unique_id: u64, 
        pub ori_making_amount: u64, 
        pub ori_taking_amount: u64, 
        pub making_amount: u64, 
        pub taking_amount: u64, 
        pub borrow_making_amount: u64, 
        pub expired_at: Option<i64>, 
        pub fee_bps: u16, 
        pub fee_account: solana_sdk::pubkey::Pubkey, 
        pub created_at: i64, 
        pub updated_at: i64, 
        pub bump: u8, 
}