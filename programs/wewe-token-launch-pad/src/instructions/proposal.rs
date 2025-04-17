use anchor_lang::prelude::*;

use crate::{
    constant::ANCHOR_DISCRIMINATOR, state::proposer::Proposal,
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
        space = ANCHOR_DISCRIMINATOR + Proposal::INIT_SPACE, // Allocate space
    )]
    pub proposal: Account<'info, Proposal>,
    pub system_program: Program<'info, System>, // Needed for SOL transfers
}

impl<'info> CreateProposal<'info> {
    pub fn create_proposal(&mut self, duration: u16, backing_goal: u64, bumps: &CreateProposalBumps) -> Result<()> {
    
        self.proposal.set_inner(Proposal {
            maker: self.maker.key(),
            current_amount: 0,
            time_started: Clock::get()?.unix_timestamp,
            duration,
            bump: bumps.proposal,
            backing_goal,
        });

        emit!(ProposalCreated {
            maker: self.maker.key(),
            proposal_address: self.proposal.key(),
            start_time: Clock::get()?.unix_timestamp,
            duration,
        });
        
        Ok(())
    }
}