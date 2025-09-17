use anchor_lang::prelude::*;

use crate::{errors::ProposalError, event::ProposalRejected, state::proposal::Proposal};

#[derive(Accounts)]
pub struct RejectProposal<'info> {
    pub authority: Signer<'info>,
    #[account(mut)]
    pub proposal: Account<'info, Proposal>,
}

impl<'info> RejectProposal<'info> {
    pub fn handle_reject_proposal(&mut self) -> Result<()> {
        require!(
            !self.proposal.is_pool_launched,
            ProposalError::PoolAlreadyLaunched
        );
        require!(!self.proposal.is_rejected, ProposalError::ProposalRejected);
        self.proposal.is_rejected = true;

        emit!(ProposalRejected {
            maker: self.proposal.maker,
            proposal_address: self.proposal.key(),
            mint_account: self.proposal.mint_account.key(),
        });

        Ok(())
    }
}
