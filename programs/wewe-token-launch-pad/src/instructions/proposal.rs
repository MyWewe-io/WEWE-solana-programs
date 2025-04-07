use anchor_lang::prelude::*;

use crate::{
    constant::ANCHOR_DISCRIMINATOR, state::proposer::Proposer,
    event::ProposalCreated
};

#[derive(Accounts)]
pub struct CreateProposal<'info> {
    #[account(mut)]
    pub maker: Signer<'info>, // Creator of the proposal
    #[account(
        init,
        payer = maker,
        seeds = [b"proposer", maker.key().as_ref()],
        bump,
        space = ANCHOR_DISCRIMINATOR + Proposer::INIT_SPACE, // Allocate space
    )]
    pub proposer: Account<'info, Proposer>,
    pub system_program: Program<'info, System>, // Needed for SOL transfers
}

impl<'info> CreateProposal<'info> {
    pub fn create_proposal(&mut self, duration: u16, bumps: &CreateProposalBumps) -> Result<()> {
    
        self.proposer.set_inner(Proposer {
            maker: self.maker.key(),
            current_amount: 0,
            time_started: Clock::get()?.unix_timestamp,
            duration,
            bump: bumps.proposer,
        });

        emit!(ProposalCreated {
            maker: self.maker.key(),
            start_time: Clock::get()?.unix_timestamp,
            duration,
        });
        
        Ok(())
    }
}