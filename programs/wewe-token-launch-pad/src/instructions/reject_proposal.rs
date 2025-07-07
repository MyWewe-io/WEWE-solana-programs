use anchor_lang::prelude::*;

use crate::{event::ProposalRejected, state::proposal::Proposal};

#[derive(Accounts)]
pub struct RejectProposal<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(mut)]
    pub proposal: Account<'info, Proposal>,
    pub system_program: Program<'info, System>,
}

impl<'info> RejectProposal<'info> {
    pub fn reject_proposal(&mut self) -> Result<()> {
        self.proposal.is_rejected = true;

        emit!(ProposalRejected {
            maker: self.proposal.maker,
            proposal_address: self.proposal.key(),
        });

        Ok(())
    }
}
