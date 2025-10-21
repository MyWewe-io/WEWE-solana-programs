use crate::fuzz_accounts::FuzzAccounts;
use borsh::{BorshDeserialize, BorshSerialize};
use trident_fuzz::fuzzing::*;

#[derive(TridentInstruction, Default)]
#[program_id("14LLwL8ixmeex2Ab4irrLJe1Nrxwj3N9CuYVq3vnwPbb")]
#[discriminator([233u8, 146u8, 209u8, 142u8, 207u8, 104u8, 64u8, 188u8])]
pub struct CreatePoolInstruction {
    pub accounts: CreatePoolInstructionAccounts,
    pub data: CreatePoolInstructionData,
}

/// Instruction Accounts
#[derive(Debug, Clone, TridentAccounts, Default)]
#[instruction_data(CreatePoolInstructionData)]
#[storage(FuzzAccounts)]
pub struct CreatePoolInstructionAccounts {
    #[account(mut)]
    pub proposal: TridentAccount,

    #[account(mut)]
    pub vault_authority: TridentAccount,

    #[account(mut)]
    pub token_vault: TridentAccount,

    #[account(mut)]
    pub wsol_vault: TridentAccount,

    pub maker: TridentAccount,

    #[account(mut)]
    pub maker_token_account: TridentAccount,

    #[account(mut, address = "HLnpSz9h2S4hiLQ43rnSD9XkcUThA7B8hQMKmDaiTLcC")]
    pub pool_authority: TridentAccount,

    pub pool_config: TridentAccount,

    #[account(mut)]
    pub pool: TridentAccount,

    #[account(mut, signer)]
    pub position_nft_mint: TridentAccount,

    pub damm_pool_authority: TridentAccount,

    #[account(mut)]
    pub position_nft_account: TridentAccount,

    #[account(mut)]
    pub position: TridentAccount,

    #[account(address = "cpamdpZCGKUy5JxQXB4dcpGPiikHawvSWAd6mEn1sGG")]
    pub amm_program: TridentAccount,

    #[account(mut)]
    pub base_mint: TridentAccount,

    #[account(mut)]
    pub quote_mint: TridentAccount,

    #[account(mut)]
    pub token_a_vault: TridentAccount,

    #[account(mut)]
    pub token_b_vault: TridentAccount,

    #[account(mut, signer)]
    pub payer: TridentAccount,

    pub token_base_program: TridentAccount,

    pub token_quote_program: TridentAccount,

    pub token_2022_program: TridentAccount,

    pub damm_event_authority: TridentAccount,

    #[account(address = "11111111111111111111111111111111")]
    pub system_program: TridentAccount,

    #[account(address = "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL")]
    pub associated_token_program: TridentAccount,

    pub config: TridentAccount,
}

/// Instruction Data
#[derive(Debug, BorshDeserialize, BorshSerialize, Clone, Default)]
pub struct CreatePoolInstructionData {
    pub sqrt_price: u128,
}

/// Implementation of instruction setters for fuzzing
///
/// Provides methods to:
/// - Set instruction data during fuzzing
/// - Configure instruction accounts during fuzzing
/// - (Optional) Set remaining accounts during fuzzing
///
/// Docs: https://ackee.xyz/trident/docs/latest/start-fuzzing/writting-fuzz-test/
impl InstructionHooks for CreatePoolInstruction {
    type IxAccounts = FuzzAccounts;
}
