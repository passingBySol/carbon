 
use carbon_core::{borsh, CarbonDeserialize};
use super::super::types::*;

#[derive(CarbonDeserialize, Debug)] 
 

#[carbon(discriminator = "0xef4f0cce7499018c")] 
pub struct ReservationListV1 { 
        pub key: Key, 
        pub master_edition: solana_sdk::pubkey::Pubkey, 
        pub supply_snapshot: Option<u64>, 
        pub reservations: Vec<ReservationV1>, 
}
