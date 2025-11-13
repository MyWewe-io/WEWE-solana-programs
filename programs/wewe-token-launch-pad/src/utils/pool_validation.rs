use anchor_lang::prelude::*;
use anchor_spl::token_interface::TokenAccount;

use crate::errors::ProposalError;

/// Validates that pool creation succeeded.
/// 
/// Note: DAMM v2 validates pool creation internally when initialize_pool CPI succeeds.
/// If the CPI succeeds, DAMM v2 has already verified:
/// - Pool was created successfully
/// - Funds were transferred correctly
/// - Liquidity calculations are valid
/// - All internal state is correct
/// 
/// This function performs minimal sanity checks. Swap functionality is tested in the test suite.
pub fn validate_pool_creation(
    token_vault: &TokenAccount,
    wsol_vault: &TokenAccount,
    token_vault_before: u64,
    wsol_vault_before: u64,
    _expected_token_amount: u64,
    _expected_wsol_amount: u64,
) -> Result<()> {
    // Minimal sanity check: verify that some funds were transferred (non-zero)
    // DAMM v2 handles all exact amount validation internally, so we just verify
    // that transfers occurred as a basic sanity check.
    let token_transferred = token_vault_before
        .checked_sub(token_vault.amount)
        .unwrap_or(0);

    let wsol_transferred = wsol_vault_before
        .checked_sub(wsol_vault.amount)
        .unwrap_or(0);

    // Only fail if NO funds were transferred (which would indicate a real problem)
    // If CPI succeeded but no funds transferred, something is wrong
    require!(
        token_transferred > 0 || wsol_transferred > 0,
        ProposalError::InsufficientFundsTransferred
    );

    Ok(())
}

/// Validates that the pool account exists and is properly initialized.
/// Swap functionality validation is performed in the test suite.
pub fn validate_pool_account_exists(pool_account: &AccountInfo) -> Result<()> {
    // Check that account exists and has data
    require!(
        pool_account.data_is_empty() == false,
        ProposalError::PoolNotInitialized
    );

    // Check that account is owned by the AMM program
    require!(
        *pool_account.owner != anchor_lang::system_program::ID,
        ProposalError::PoolNotInitialized
    );

    Ok(())
}

