use crate::fuzz_accounts::FuzzAccounts;
use borsh::{BorshDeserialize, BorshSerialize};
use trident_fuzz::fuzzing::*;

#[derive(TridentInstruction, Default)]
#[program_id("14LLwL8ixmeex2Ab4irrLJe1Nrxwj3N9CuYVq3vnwPbb")]
#[discriminator([114u8, 162u8, 164u8, 82u8, 191u8, 11u8, 102u8, 25u8])]
pub struct RejectProposalInstruction {
    pub accounts: RejectProposalInstructionAccounts,
    pub data: RejectProposalInstructionData,
}

/// Instruction Accounts
#[derive(Debug, Clone, TridentAccounts, Default)]
#[instruction_data(RejectProposalInstructionData)]
#[storage(FuzzAccounts)]
pub struct RejectProposalInstructionAccounts {
    #[account(signer)]
    pub authority: TridentAccount,

    #[account(mut)]
    pub proposal: TridentAccount,
}

/// Instruction Data
#[derive(Debug, BorshDeserialize, BorshSerialize, Clone, Default)]
pub struct RejectProposalInstructionData {}

/// Implementation of instruction setters for fuzzing
///
/// Provides methods to:
/// - Set instruction data during fuzzing
/// - Configure instruction accounts during fuzzing
/// - (Optional) Set remaining accounts during fuzzing
///
/// Docs: https://ackee.xyz/trident/docs/latest/start-fuzzing/writting-fuzz-test/
impl InstructionHooks for RejectProposalInstruction {
    type IxAccounts = FuzzAccounts;
}
