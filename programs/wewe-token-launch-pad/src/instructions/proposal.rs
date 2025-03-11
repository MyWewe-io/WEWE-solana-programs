use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken, 
    token::{
        Mint, 
        Token, 
        TokenAccount
    }
};

use crate::{
    constant::MIN_AMOUNT_TO_RAISE, errors::ProposalError, state::proposer::Proposer
};

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub maker: Signer<'info>,
    pub proposer: Account<'info, Proposer>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

impl<'info> Initialize<'info> {
    pub fn create_proposal(&mut self, amount: u64, duration: u16, bumps: &InitializeBumps, protocol: &str) -> Result<()> {

        // Check if the amount to raise meets the minimum amount required
        require!(
            amount >= MIN_AMOUNT_TO_RAISE,
            ProposalError::InvalidAmount
        );

        // Initialize the fundraiser account
        self.proposer.set_inner(Proposer {
            proposer: self.maker.key(),
            time_started: Clock::get()?.unix_timestamp,
            duration,
            protocol,
            bump: bumps.proposer
        });
        
        Ok(())
    }
}