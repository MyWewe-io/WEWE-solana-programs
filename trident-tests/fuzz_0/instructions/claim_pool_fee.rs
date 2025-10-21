use crate::fuzz_accounts::FuzzAccounts;
use borsh::{BorshDeserialize, BorshSerialize};
use trident_fuzz::fuzzing::*;

#[derive(TridentInstruction, Default)]
#[program_id("14LLwL8ixmeex2Ab4irrLJe1Nrxwj3N9CuYVq3vnwPbb")]
#[discriminator([201u8, 205u8, 15u8, 168u8, 196u8, 41u8, 123u8, 175u8])]
pub struct ClaimPoolFeeInstruction {
    pub accounts: ClaimPoolFeeInstructionAccounts,
    pub data: ClaimPoolFeeInstructionData,
}

/// Instruction Accounts
#[derive(Debug, Clone, TridentAccounts, Default)]
#[instruction_data(ClaimPoolFeeInstructionData)]
#[storage(FuzzAccounts)]
pub struct ClaimPoolFeeInstructionAccounts {
    #[account(mut, address = "HLnpSz9h2S4hiLQ43rnSD9XkcUThA7B8hQMKmDaiTLcC")]
    pub pool_authority: TridentAccount,

    #[account(mut, signer)]
    pub payer: TridentAccount,

    pub maker: TridentAccount,

    #[account(address = "76U9hvHNUNn7YV5FekSzDHzqnHETsUpDKq4cMj2dMxNi")]
    pub wewe_treasury: TridentAccount,

    #[account(mut)]
    pub proposal: TridentAccount,

    #[account(mut)]
    pub vault_authority: TridentAccount,

    #[account(mut)]
    pub wewe_wsol_account: TridentAccount,

    #[account(mut)]
    pub wewe_token_account: TridentAccount,

    #[account(mut)]
    pub maker_wsol_account: TridentAccount,

    #[account(mut)]
    pub maker_token_account: TridentAccount,

    pub pool: TridentAccount,

    #[account(mut)]
    pub position: TridentAccount,

    #[account(mut)]
    pub token_a_account: TridentAccount,

    #[account(mut)]
    pub token_b_account: TridentAccount,

    #[account(mut)]
    pub token_a_vault: TridentAccount,

    #[account(mut)]
    pub token_b_vault: TridentAccount,

    pub token_a_mint: TridentAccount,

    pub token_b_mint: TridentAccount,

    pub position_nft_account: TridentAccount,

    pub token_a_program: TridentAccount,

    pub token_b_program: TridentAccount,

    #[account(address = "cpamdpZCGKUy5JxQXB4dcpGPiikHawvSWAd6mEn1sGG")]
    pub amm_program: TridentAccount,

    pub event_authority: TridentAccount,

    #[account(address = "11111111111111111111111111111111")]
    pub system_program: TridentAccount,

    #[account(address = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA")]
    pub token_program: TridentAccount,

    #[account(address = "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL")]
    pub associated_token_program: TridentAccount,
}

/// Instruction Data
#[derive(Debug, BorshDeserialize, BorshSerialize, Clone, Default)]
pub struct ClaimPoolFeeInstructionData {}

/// Implementation of instruction setters for fuzzing
///
/// Provides methods to:
/// - Set instruction data during fuzzing
/// - Configure instruction accounts during fuzzing
/// - (Optional) Set remaining accounts during fuzzing
///
/// Docs: https://ackee.xyz/trident/docs/latest/start-fuzzing/writting-fuzz-test/
impl InstructionHooks for ClaimPoolFeeInstruction {
    type IxAccounts = FuzzAccounts;
}
