use crate::fuzz_accounts::FuzzAccounts;
use borsh::{BorshDeserialize, BorshSerialize};
use trident_fuzz::fuzzing::*;

#[derive(TridentInstruction, Default)]
#[program_id("14LLwL8ixmeex2Ab4irrLJe1Nrxwj3N9CuYVq3vnwPbb")]
#[discriminator([103u8, 154u8, 128u8, 192u8, 211u8, 17u8, 132u8, 49u8])]
pub struct SnapshotBackerAmountInstruction {
    pub accounts: SnapshotBackerAmountInstructionAccounts,
    pub data: SnapshotBackerAmountInstructionData,
}

/// Instruction Accounts
#[derive(Debug, Clone, TridentAccounts, Default)]
#[instruction_data(SnapshotBackerAmountInstructionData)]
#[storage(FuzzAccounts)]
pub struct SnapshotBackerAmountInstructionAccounts {
    #[account(mut, signer)]
    pub authority: TridentAccount,

    #[account(mut)]
    pub proposal: TridentAccount,

    pub backer: TridentAccount,

    #[account(mut)]
    pub backer_account: TridentAccount,

    pub mint_account: TridentAccount,

    pub backer_token_account: TridentAccount,

    pub config: TridentAccount,
}

/// Instruction Data
#[derive(Debug, BorshDeserialize, BorshSerialize, Clone, Default)]
pub struct SnapshotBackerAmountInstructionData {}

/// Implementation of instruction setters for fuzzing
///
/// Provides methods to:
/// - Set instruction data during fuzzing
/// - Configure instruction accounts during fuzzing
/// - (Optional) Set remaining accounts during fuzzing
///
/// Docs: https://ackee.xyz/trident/docs/latest/start-fuzzing/writting-fuzz-test/
impl InstructionHooks for SnapshotBackerAmountInstruction {
    type IxAccounts = FuzzAccounts;
}
