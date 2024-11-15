use carbon_core::{borsh, CarbonDeserialize};
#[derive(
    CarbonDeserialize, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq, Clone, Hash,
)]
#[carbon(discriminator = "0xf0200a98082d573a")]
pub struct UpdateConfigAccount {
    pub param: u8,
    pub owner: solana_sdk::pubkey::Pubkey,
}

pub struct UpdateConfigAccountInstructionAccounts {
    pub admin: solana_sdk::pubkey::Pubkey,
    pub amm_config: solana_sdk::pubkey::Pubkey,
}

impl carbon_core::deserialize::ArrangeAccounts for UpdateConfigAccount {
    type ArrangedAccounts = UpdateConfigAccountInstructionAccounts;

    fn arrange_accounts(
        &self,
        accounts: Vec<solana_sdk::instruction::AccountMeta>,
    ) -> Option<Self::ArrangedAccounts> {
        let admin = accounts.get(0)?;
        let amm_config = accounts.get(1)?;

        Some(UpdateConfigAccountInstructionAccounts {
            admin: admin.pubkey,
            amm_config: amm_config.pubkey,
        })
    }
}
