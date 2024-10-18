

use carbon_core::deserialize::{ArrangeAccounts, CarbonDeserialize};
use carbon_proc_macros::CarbonDeserialize;
use carbon_core::borsh;


#[derive(CarbonDeserialize, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq, Clone, Hash)]
#[carbon(discriminator = "0xf326c64cac407f18")]
pub struct CreateNftList{
    pub collection_name: String,
}

pub struct CreateNftListInstructionAccounts {
    pub nft_list: solana_sdk::pubkey::Pubkey,
    pub payer: solana_sdk::pubkey::Pubkey,
}

impl ArrangeAccounts for CreateNftList {
    type ArrangedAccounts = CreateNftListInstructionAccounts;

    fn arrange_accounts(&self, accounts: Vec<solana_sdk::instruction::AccountMeta>) -> Option<Self::ArrangedAccounts> {
        let nft_list = accounts.get(0)?;
        let payer = accounts.get(1)?;

        Some(CreateNftListInstructionAccounts {
            nft_list: nft_list.pubkey,
            payer: payer.pubkey,
        })
    }
}