
use super::*;
use carbon_core::deserialize::CarbonDeserialize;
use carbon_proc_macros::CarbonDeserialize;
use carbon_core::borsh;


#[derive(CarbonDeserialize, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq, Clone, Hash)]
pub struct ReservationV1 {
    pub address: solana_sdk::pubkey::Pubkey,
    pub spots_remaining: u8,
    pub total_spots: u8,
}
