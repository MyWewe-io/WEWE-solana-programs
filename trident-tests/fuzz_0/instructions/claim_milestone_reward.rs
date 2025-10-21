use crate::fuzz_accounts::FuzzAccounts;
use borsh::{BorshDeserialize, BorshSerialize};
use trident_fuzz::fuzzing::*;

#[derive(TridentInstruction, Default)]
#[program_id("14LLwL8ixmeex2Ab4irrLJe1Nrxwj3N9CuYVq3vnwPbb")]
#[discriminator([12u8, 145u8, 214u8, 209u8, 199u8, 176u8, 156u8, 102u8])]
pub struct ClaimMilestoneRewardInstruction {
    pub accounts: ClaimMilestoneRewardInstructionAccounts,
    pub data: ClaimMilestoneRewardInstructionData,
}

/// Instruction Accounts
#[derive(Debug, Clone, TridentAccounts, Default)]
#[instruction_data(ClaimMilestoneRewardInstructionData)]
#[storage(FuzzAccounts)]
pub struct ClaimMilestoneRewardInstructionAccounts {
    #[account(mut, signer)]
    pub backer: TridentAccount,

    pub proposal: TridentAccount,

    #[account(mut)]
    pub vault_authority: TridentAccount,

    #[account(mut)]
    pub mint_account: TridentAccount,

    #[account(mut)]
    pub token_vault: TridentAccount,

    #[account(mut)]
    pub backer_account: TridentAccount,

    #[account(mut)]
    pub backer_token_account: TridentAccount,

    #[account(address = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA")]
    pub token_program: TridentAccount,

    #[account(address = "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL")]
    pub associated_token_program: TridentAccount,

    #[account(address = "11111111111111111111111111111111")]
    pub system_program: TridentAccount,
}

/// Instruction Data
#[derive(Debug, BorshDeserialize, BorshSerialize, Clone, Default)]
pub struct ClaimMilestoneRewardInstructionData {}

/// Implementation of instruction setters for fuzzing
///
/// Provides methods to:
/// - Set instruction data during fuzzing
/// - Configure instruction accounts during fuzzing
/// - (Optional) Set remaining accounts during fuzzing
///
/// Docs: https://ackee.xyz/trident/docs/latest/start-fuzzing/writting-fuzz-test/
impl InstructionHooks for ClaimMilestoneRewardInstruction {
    type IxAccounts = FuzzAccounts;
}
