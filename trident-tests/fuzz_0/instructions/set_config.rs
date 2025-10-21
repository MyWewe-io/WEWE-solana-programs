use crate::fuzz_accounts::FuzzAccounts;
use borsh::{BorshDeserialize, BorshSerialize};
use trident_fuzz::fuzzing::*;

#[derive(TridentInstruction, Default)]
#[program_id("14LLwL8ixmeex2Ab4irrLJe1Nrxwj3N9CuYVq3vnwPbb")]
#[discriminator([108u8, 158u8, 154u8, 175u8, 212u8, 98u8, 52u8, 66u8])]
pub struct SetConfigInstruction {
    pub accounts: SetConfigInstructionAccounts,
    pub data: SetConfigInstructionData,
}

/// Instruction Accounts
#[derive(Debug, Clone, TridentAccounts, Default)]
#[instruction_data(SetConfigInstructionData)]
#[storage(FuzzAccounts)]
pub struct SetConfigInstructionAccounts {
    #[account(mut, signer)]
    pub authority: TridentAccount,

    #[account(mut)]
    pub config: TridentAccount,

    #[account(address = "11111111111111111111111111111111")]
    pub system_program: TridentAccount,
}

/// Instruction Data
#[derive(Debug, BorshDeserialize, BorshSerialize, Clone, Default)]
pub struct SetConfigInstructionData {
    pub amount_to_raise_per_user: u64,

    pub total_mint: u64,

    pub total_pool_tokens: u64,

    pub maker_token_amount: u64,

    pub total_airdrop_amount_per_milestone: u64,

    pub min_backers: u64,
}

/// Implementation of instruction setters for fuzzing
///
/// Provides methods to:
/// - Set instruction data during fuzzing
/// - Configure instruction accounts during fuzzing
/// - (Optional) Set remaining accounts during fuzzing
///
/// Docs: https://ackee.xyz/trident/docs/latest/start-fuzzing/writting-fuzz-test/
impl InstructionHooks for SetConfigInstruction {
    type IxAccounts = FuzzAccounts;
}
