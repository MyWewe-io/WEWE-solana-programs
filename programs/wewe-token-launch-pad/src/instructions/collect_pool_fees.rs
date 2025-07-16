use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};

use crate::{const_pda::const_authority::VAULT_BUMP, constant::{POOL_AUTHORITY_PREFIX, VAULT_AUTHORITY}, state::proposal::Proposal};

#[event_cpi]
#[derive(Accounts)]
pub struct ClaimPositionFee<'info> {
    /// CHECK: pool authority
    #[account(
        seeds = [
            POOL_AUTHORITY_PREFIX.as_ref(),
        ],
        bump,
    )]
    pub pool_authority: UncheckedAccount<'info>,

    #[account(mut)]
    pub proposal: Account<'info, Proposal>,
    
    #[account(
        mut,
        seeds = [
            VAULT_AUTHORITY.as_ref(),
        ],
        bump,
    )]
    pub vault_authority: UncheckedAccount<'info>,

    pub pool: UncheckedAccount<'info>,

    pub position: UncheckedAccount<'info>,

    /// The user token a account
    #[account(mut)]
    pub token_a_account: Box<InterfaceAccount<'info, TokenAccount>>,

    /// The user token b account
    #[account(mut)]
    pub token_b_account: Box<InterfaceAccount<'info, TokenAccount>>,

    /// The vault token account for input token
    #[account(mut, token::token_program = token_a_program, token::mint = token_a_mint)]
    pub token_a_vault: Box<InterfaceAccount<'info, TokenAccount>>,

    /// The vault token account for output token
    #[account(mut, token::token_program = token_b_program, token::mint = token_b_mint)]
    pub token_b_vault: Box<InterfaceAccount<'info, TokenAccount>>,

    pub token_a_mint: Box<InterfaceAccount<'info, Mint>>,

    pub token_b_mint: Box<InterfaceAccount<'info, Mint>>,

    pub position_nft_account: UncheckedAccount<'info>,
    
    pub token_a_program: Interface<'info, TokenInterface>,

    pub token_b_program: Interface<'info, TokenInterface>,

    #[account(address = cp_amm::ID)]
    pub amm_program: UncheckedAccount<'info>,
}

impl<'info> ClaimPositionFee<'info> {
    pub fn claim_position_fee(&self) -> Result<()> {
        let pool_authority_seeds: &[&[u8]] = &[b"vault_authority", &[VAULT_BUMP]];

        cp_amm::cpi::claim_position_fee(CpiContext::new_with_signer(
            self.amm_program.to_account_info(),
            cp_amm::cpi::accounts::ClaimPositionFeeCtx {
                pool_authority: self.pool_authority.to_account_info(),
                pool: self.pool.to_account_info(),
                position: self.position.to_account_info(),
                token_a_account: self.token_a_account.to_account_info(),
                token_b_account: self.token_b_account.to_account_info(),
                token_a_vault: self.token_a_vault.to_account_info(),
                token_b_vault: self.token_b_vault.to_account_info(),
                token_a_mint: self.token_a_mint.to_account_info(),
                token_b_mint: self.token_b_mint.to_account_info(),
                position_nft_account: self.position_nft_account.to_account_info(),
                owner: self.vault_authority.to_account_info(),
                token_a_program: self.token_a_program.to_account_info(),
                token_b_program: self.token_b_program.to_account_info(),
                event_authority: self.event_authority.to_account_info(),
                program: self.program.to_account_info(),
            },
            &[&pool_authority_seeds[..]],
        ))?;
        Ok(())
    }
}
