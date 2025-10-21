use crate::fuzz_accounts::FuzzAccounts;
use borsh::{BorshDeserialize, BorshSerialize};
use trident_fuzz::fuzzing::*;

#[derive(TridentInstruction, Default)]
#[program_id("14LLwL8ixmeex2Ab4irrLJe1Nrxwj3N9CuYVq3vnwPbb")]
#[discriminator([123u8, 195u8, 22u8, 57u8, 180u8, 127u8, 174u8, 154u8])]
pub struct InitialiseMilestoneInstruction {
    pub accounts: InitialiseMilestoneInstructionAccounts,
    pub data: InitialiseMilestoneInstructionData,
}

/// Instruction Accounts
#[derive(Debug, Clone, TridentAccounts, Default)]
#[instruction_data(InitialiseMilestoneInstructionData)]
#[storage(FuzzAccounts)]
pub struct InitialiseMilestoneInstructionAccounts {
    #[account(signer)]
    pub authority: TridentAccount,

    #[account(mut)]
    pub proposal: TridentAccount,
}

/// Instruction Data
#[derive(Debug, BorshDeserialize, BorshSerialize, Clone, Default)]
pub struct InitialiseMilestoneInstructionData {}

/// Implementation of instruction setters for fuzzing
///
/// Provides methods to:
/// - Set instruction data during fuzzing
/// - Configure instruction accounts during fuzzing
/// - (Optional) Set remaining accounts during fuzzing
///
/// Docs: https://ackee.xyz/trident/docs/latest/start-fuzzing/writting-fuzz-test/
impl InstructionHooks for InitialiseMilestoneInstruction {
    type IxAccounts = FuzzAccounts;
}
