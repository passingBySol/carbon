
use super::super::types::*;

use carbon_core::deserialize::{ArrangeAccounts, CarbonDeserialize};
use carbon_proc_macros::CarbonDeserialize;
use carbon_core::borsh;


#[derive(CarbonDeserialize, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq, Clone, Hash)]
#[carbon(discriminator = "0xd70d19bb0b5d228f")]
pub struct UpdateNftList{
    pub mints: Vec<UpdateIndex>,
}

pub struct UpdateNftListInstructionAccounts {
    pub nft_list: solana_sdk::pubkey::Pubkey,
    pub payer: solana_sdk::pubkey::Pubkey,
}

impl ArrangeAccounts for UpdateNftList {
    type ArrangedAccounts = UpdateNftListInstructionAccounts;

    fn arrange_accounts(&self, accounts: Vec<solana_sdk::instruction::AccountMeta>) -> Option<Self::ArrangedAccounts> {
        let nft_list = accounts.get(0)?;
        let payer = accounts.get(1)?;

        Some(UpdateNftListInstructionAccounts {
            nft_list: nft_list.pubkey,
            payer: payer.pubkey,
        })
    }
}