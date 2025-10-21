use crate::fuzz_accounts::FuzzAccounts;
use borsh::{BorshDeserialize, BorshSerialize};
use trident_fuzz::fuzzing::*;

#[derive(TridentInstruction, Default)]
#[program_id("14LLwL8ixmeex2Ab4irrLJe1Nrxwj3N9CuYVq3vnwPbb")]
#[discriminator([19u8, 191u8, 156u8, 122u8, 108u8, 188u8, 171u8, 212u8])]
pub struct EndMilestoneInstruction {
    pub accounts: EndMilestoneInstructionAccounts,
    pub data: EndMilestoneInstructionData,
}

/// Instruction Accounts
#[derive(Debug, Clone, TridentAccounts, Default)]
#[instruction_data(EndMilestoneInstructionData)]
#[storage(FuzzAccounts)]
pub struct EndMilestoneInstructionAccounts {
    #[account(signer)]
    pub authority: TridentAccount,

    #[account(mut)]
    pub proposal: TridentAccount,

    #[account(mut)]
    pub mint: TridentAccount,

    #[account(mut)]
    pub vault_authority: TridentAccount,

    #[account(mut)]
    pub token_vault: TridentAccount,

    #[account(address = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA")]
    pub token_program: TridentAccount,
}

/// Instruction Data
#[derive(Debug, BorshDeserialize, BorshSerialize, Clone, Default)]
pub struct EndMilestoneInstructionData {}

/// Implementation of instruction setters for fuzzing
///
/// Provides methods to:
/// - Set instruction data during fuzzing
/// - Configure instruction accounts during fuzzing
/// - (Optional) Set remaining accounts during fuzzing
///
/// Docs: https://ackee.xyz/trident/docs/latest/start-fuzzing/writting-fuzz-test/
impl InstructionHooks for EndMilestoneInstruction {
    type IxAccounts = FuzzAccounts;
}
