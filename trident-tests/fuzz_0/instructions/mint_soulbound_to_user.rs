use crate::fuzz_accounts::FuzzAccounts;
use borsh::{BorshDeserialize, BorshSerialize};
use trident_fuzz::fuzzing::*;

#[derive(TridentInstruction, Default)]
#[program_id("14LLwL8ixmeex2Ab4irrLJe1Nrxwj3N9CuYVq3vnwPbb")]
#[discriminator([16u8, 56u8, 32u8, 67u8, 14u8, 4u8, 200u8, 194u8])]
pub struct MintSoulboundToUserInstruction {
    pub accounts: MintSoulboundToUserInstructionAccounts,
    pub data: MintSoulboundToUserInstructionData,
}

/// Instruction Accounts
#[derive(Debug, Clone, TridentAccounts, Default)]
#[instruction_data(MintSoulboundToUserInstructionData)]
#[storage(FuzzAccounts)]
pub struct MintSoulboundToUserInstructionAccounts {
    #[account(mut, signer)]
    pub payer: TridentAccount,

    #[account(mut)]
    pub mint: TridentAccount,

    pub mint_authority: TridentAccount,

    pub user: TridentAccount,

    #[account(mut)]
    pub user_token_account: TridentAccount,

    #[account(address = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA")]
    pub token_program: TridentAccount,

    #[account(address = "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL")]
    pub associated_token_program: TridentAccount,

    #[account(address = "11111111111111111111111111111111")]
    pub system_program: TridentAccount,
}

/// Instruction Data
#[derive(Debug, BorshDeserialize, BorshSerialize, Clone, Default)]
pub struct MintSoulboundToUserInstructionData {}

/// Implementation of instruction setters for fuzzing
///
/// Provides methods to:
/// - Set instruction data during fuzzing
/// - Configure instruction accounts during fuzzing
/// - (Optional) Set remaining accounts during fuzzing
///
/// Docs: https://ackee.xyz/trident/docs/latest/start-fuzzing/writting-fuzz-test/
impl InstructionHooks for MintSoulboundToUserInstruction {
    type IxAccounts = FuzzAccounts;
}
