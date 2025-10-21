use crate::fuzz_accounts::FuzzAccounts;
use borsh::{BorshDeserialize, BorshSerialize};
use trident_fuzz::fuzzing::*;

#[derive(TridentInstruction, Default)]
#[program_id("14LLwL8ixmeex2Ab4irrLJe1Nrxwj3N9CuYVq3vnwPbb")]
#[discriminator([132u8, 116u8, 68u8, 174u8, 216u8, 160u8, 198u8, 22u8])]
pub struct CreateProposalInstruction {
    pub accounts: CreateProposalInstructionAccounts,
    pub data: CreateProposalInstructionData,
}

/// Instruction Accounts
#[derive(Debug, Clone, TridentAccounts, Default)]
#[instruction_data(CreateProposalInstructionData)]
#[storage(FuzzAccounts)]
pub struct CreateProposalInstructionAccounts {
    #[account(mut, signer)]
    pub payer: TridentAccount,

    #[account(mut, signer)]
    pub maker: TridentAccount,

    #[account(mut)]
    pub maker_account: TridentAccount,

    #[account(mut)]
    pub proposal: TridentAccount,

    #[account(mut)]
    pub vault_authority: TridentAccount,

    #[account(mut, signer)]
    pub mint_account: TridentAccount,

    #[account(mut)]
    pub metadata_account: TridentAccount,

    #[account(mut)]
    pub token_vault: TridentAccount,

    #[account(mut, address = "8zNWuPFfiiWVHTxUjHqpZsJRxxjDhyjKvoYTpvz2fnVN")]
    pub mint: TridentAccount,

    pub user_token_account: TridentAccount,

    #[account(address = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA")]
    pub token_program: TridentAccount,

    #[account(address = "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s")]
    pub token_metadata_program: TridentAccount,

    #[account(address = "11111111111111111111111111111111")]
    pub system_program: TridentAccount,

    #[account(address = "SysvarRent111111111111111111111111111111111")]
    pub rent: TridentAccount,

    pub config: TridentAccount,
}

/// Instruction Data
#[derive(Debug, BorshDeserialize, BorshSerialize, Clone, Default)]
pub struct CreateProposalInstructionData {
    pub token_name: String,

    pub token_symbol: String,

    pub token_uri: String,
}

/// Implementation of instruction setters for fuzzing
///
/// Provides methods to:
/// - Set instruction data during fuzzing
/// - Configure instruction accounts during fuzzing
/// - (Optional) Set remaining accounts during fuzzing
///
/// Docs: https://ackee.xyz/trident/docs/latest/start-fuzzing/writting-fuzz-test/
impl InstructionHooks for CreateProposalInstruction {
    type IxAccounts = FuzzAccounts;
}
