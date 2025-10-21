use crate::fuzz_accounts::FuzzAccounts;
use borsh::{BorshDeserialize, BorshSerialize};
use trident_fuzz::fuzzing::*;

#[derive(TridentInstruction, Default)]
#[program_id("14LLwL8ixmeex2Ab4irrLJe1Nrxwj3N9CuYVq3vnwPbb")]
#[discriminator([108u8, 81u8, 78u8, 117u8, 125u8, 155u8, 56u8, 200u8])]
pub struct DepositSolInstruction {
    pub accounts: DepositSolInstructionAccounts,
    pub data: DepositSolInstructionData,
}

/// Instruction Accounts
#[derive(Debug, Clone, TridentAccounts, Default)]
#[instruction_data(DepositSolInstructionData)]
#[storage(FuzzAccounts)]
pub struct DepositSolInstructionAccounts {
    #[account(mut, signer)]
    pub backer: TridentAccount,

    #[account(mut, address = "76U9hvHNUNn7YV5FekSzDHzqnHETsUpDKq4cMj2dMxNi")]
    pub wewe_vault: TridentAccount,

    #[account(mut)]
    pub proposal: TridentAccount,

    #[account(mut)]
    pub vault_authority: TridentAccount,

    #[account(mut, address = "8zNWuPFfiiWVHTxUjHqpZsJRxxjDhyjKvoYTpvz2fnVN")]
    pub mint: TridentAccount,

    pub user_token_account: TridentAccount,

    #[account(mut)]
    pub backer_account: TridentAccount,

    #[account(address = "11111111111111111111111111111111")]
    pub system_program: TridentAccount,

    pub config: TridentAccount,
}

/// Instruction Data
#[derive(Debug, BorshDeserialize, BorshSerialize, Clone, Default)]
pub struct DepositSolInstructionData {}

/// Implementation of instruction setters for fuzzing
///
/// Provides methods to:
/// - Set instruction data during fuzzing
/// - Configure instruction accounts during fuzzing
/// - (Optional) Set remaining accounts during fuzzing
///
/// Docs: https://ackee.xyz/trident/docs/latest/start-fuzzing/writting-fuzz-test/
impl InstructionHooks for DepositSolInstruction {
    type IxAccounts = FuzzAccounts;
}
