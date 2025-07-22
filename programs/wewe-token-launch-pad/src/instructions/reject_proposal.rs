use anchor_lang::prelude::*;

use crate::{constant::VAULT_AUTHORITY, errors::ProposalError, event::ProposalRejected, state::proposal::Proposal};

#[derive(Accounts)]
pub struct RejectProposal<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    /// CHECK: vault authority
    #[account(
        mut,
        seeds = [
            VAULT_AUTHORITY.as_ref(),
        ],
        bump,
    )]
    pub vault_authority: UncheckedAccount<'info>,
    #[account(mut)]
    pub proposal: Account<'info, Proposal>,
    pub system_program: Program<'info, System>,
}

impl<'info> RejectProposal<'info> {
    pub fn reject_proposal(&mut self) -> Result<()> {
        require!(!self.proposal.is_pool_launched, ProposalError::PoolAlreadyLaunched);
        require!(!self.proposal.is_rejected, ProposalError::ProposalRejected);
        self.proposal.is_rejected = true;

        emit!(ProposalRejected {
            maker: self.proposal.maker,
            proposal_address: self.proposal.key(),
        });

        Ok(())
    }
}
