use anchor_lang::prelude::*;
use anchor_spl::token_interface::{TokenAccount, TokenInterface};
use anchor_spl::token::Mint;

use crate::{
    constant::{
        seeds::{TOKEN_VAULT, VAULT_AUTHORITY},
        wsol_pubkey,
        MINT_DECIMALS,
    },
    errors::ProposalError,
    state::proposal::Proposal,
};

#[derive(Accounts)]
pub struct ResetPoolLaunch<'info> {
    pub authority: Signer<'info>,
    
    #[account(mut)]
    pub payer: Signer<'info>,
    
    #[account(mut)]
    pub proposal: Account<'info, Proposal>,
    
    /// CHECK: vault authority
    #[account(
        mut,
        seeds = [VAULT_AUTHORITY.as_ref()],
        bump,
    )]
    pub vault_authority: SystemAccount<'info>,
    
    #[account(
        mut,
        seeds = [TOKEN_VAULT, vault_authority.key().as_ref(), wsol_pubkey::ID.as_ref()],
        bump,
        token::mint = quote_mint,
        token::authority = vault_authority,
    )]
    pub wsol_vault: Box<InterfaceAccount<'info, TokenAccount>>,
    
    /// CHECK: quote token mint (WSOL)
    #[account(
        address = wsol_pubkey::ID @ ProposalError::IncorrectAccount
    )]
    pub quote_mint: UncheckedAccount<'info>,
    
    #[account(
        init,
        payer = payer,
        mint::decimals = MINT_DECIMALS,
        mint::authority = proposal.key(),
        mint::freeze_authority = proposal.key(),
    )]
    pub mint_account: Account<'info, Mint>,

    #[account(
        init,
        seeds = [TOKEN_VAULT, vault_authority.key().as_ref(), mint_account.key().as_ref()],
        payer = payer,
        token::mint = mint_account,
        token::authority = vault_authority,
        token::token_program = token_program,
        bump,
    )]
    pub token_vault: Box<InterfaceAccount<'info, TokenAccount>>,
    
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

impl<'info> ResetPoolLaunch<'info> {
    pub fn handle_reset_pool_launch(&mut self) -> Result<()> {
        
        // Check that pool is currently launched
        require!(
            self.proposal.is_pool_launched,
            ProposalError::PoolNotInitialized
        );
        
        // Check that WSOL vault has enough funds to cover the proposal's total_backing
        require!(
            self.wsol_vault.amount >= self.proposal.total_backing,
            ProposalError::InsufficientFunds
        );
        
        // Reset the pool launch flag
        self.proposal.is_pool_launched = false;

        self.proposal.milestone_active = false;
        
        // Update the mint account (use the initialized mint account's key)
        self.proposal.mint_account = self.mint_account.key();
        
        // Optionally reset launch_timestamp (set to None)
        self.proposal.launch_timestamp = None;
        
        Ok(())
    }
}

