use trident_fuzz::fuzzing::*;

/// FuzzAccounts contains all available accounts
///
/// You can create your own accounts by adding new fields to the struct.
///
/// Docs: https://ackee.xyz/trident/docs/latest/trident-api-macro/trident-types/fuzz-accounts/
#[derive(Default)]
pub struct FuzzAccounts {
    pub wewe_wsol_account: AccountsStorage,

    pub maker_wsol_account: AccountsStorage,

    pub pool_authority: AccountsStorage,

    pub backer_token_account: AccountsStorage,

    pub pool_config: AccountsStorage,

    pub wewe_vault: AccountsStorage,

    pub authority: AccountsStorage,

    pub damm_pool_authority: AccountsStorage,

    pub base_mint: AccountsStorage,

    pub backer_account: AccountsStorage,

    pub token_b_program: AccountsStorage,

    pub position_nft_mint: AccountsStorage,

    pub token_quote_program: AccountsStorage,

    pub wsol_vault: AccountsStorage,

    pub config: AccountsStorage,

    pub system_program: AccountsStorage,

    pub token_a_program: AccountsStorage,

    pub token_vault: AccountsStorage,

    pub user: AccountsStorage,

    pub amm_program: AccountsStorage,

    pub token_a_vault: AccountsStorage,

    pub mint: AccountsStorage,

    pub maker: AccountsStorage,

    pub maker_account: AccountsStorage,

    pub associated_token_program: AccountsStorage,

    pub mint_account: AccountsStorage,

    pub token_a_mint: AccountsStorage,

    pub position: AccountsStorage,

    pub payer: AccountsStorage,

    pub wewe_token_account: AccountsStorage,

    pub token_2022_program: AccountsStorage,

    pub proposal: AccountsStorage,

    pub mint_authority: AccountsStorage,

    pub token_a_account: AccountsStorage,

    pub vault_authority: AccountsStorage,

    pub event_authority: AccountsStorage,

    pub token_base_program: AccountsStorage,

    pub maker_token_account: AccountsStorage,

    pub metadata_account: AccountsStorage,

    pub backer: AccountsStorage,

    pub pool: AccountsStorage,

    pub token_metadata_program: AccountsStorage,

    pub quote_mint: AccountsStorage,

    pub damm_event_authority: AccountsStorage,

    pub rent: AccountsStorage,

    pub wewe_treasury: AccountsStorage,

    pub user_token_account: AccountsStorage,

    pub token_program: AccountsStorage,

    pub token_b_mint: AccountsStorage,

    pub position_nft_account: AccountsStorage,

    pub token_b_vault: AccountsStorage,

    pub token_b_account: AccountsStorage,
}
