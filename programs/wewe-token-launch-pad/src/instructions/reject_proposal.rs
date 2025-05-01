use anchor_lang::prelude::*;

use crate::{event::ProposalRejected, state::proposer::Proposal};

#[derive(Accounts)]
#[instruction(_proposal_index: u64)]
pub struct RejectProposal<'info> {
    #[account(mut)]
    pub authority: Signer<'info>, // Creator of the proposal
    #[account(
        mut,
        seeds = [b"proposer".as_ref(), proposal.maker.as_ref(), &_proposal_index.to_le_bytes()],
        bump = proposal.bump,
    )]
    pub proposal: Account<'info, Proposal>,
    pub system_program: Program<'info, System>,
}

impl<'info> RejectProposal<'info> {
    pub fn reject_proposal(
        &mut self,
        _proposal_index: u64,
    ) -> Result<()> {
        self.proposal.is_rejected = true;

        emit!(ProposalRejected {
            maker: self.proposal.maker,
            proposal_address: self.proposal.key(),
        });

        Ok(())
    }
}
