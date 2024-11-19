
 
use carbon_core::{borsh, CarbonDeserialize};

#[derive(CarbonDeserialize, Debug)] 
 

#[carbon(discriminator = "0xc251d9670c130c42")] 
pub struct ReferrerState { 
        pub short_url: solana_sdk::pubkey::Pubkey, 
        pub owner: solana_sdk::pubkey::Pubkey, 
}