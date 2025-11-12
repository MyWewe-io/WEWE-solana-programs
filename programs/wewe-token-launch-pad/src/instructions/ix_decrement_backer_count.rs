use crate::{
    constant::seeds::BACKER_PROPOSAL_COUNT,
    errors::ProposalError,
    state::{backer_proposal_count::BackerProposalCount, proposal::Proposal},
};
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct DecrementBackerCount<'info> {
    #[account(mut)]
    pub backer: Signer<'info>,

    #[account(
        mut,
        constraint = (proposal.is_pool_launched || proposal.is_rejected) @ ProposalError::BackingNotEnded,
    )]
    pub proposal: Account<'info, Proposal>,

    #[account(
        mut,
        seeds = [BACKER_PROPOSAL_COUNT, backer.key().as_ref()],
        bump,
        constraint = backer_proposal_count.backer == backer.key() @ ProposalError::IncorrectAccount,
    )]
    pub backer_proposal_count: Account<'info, BackerProposalCount>,

    #[account(
        seeds = [crate::constant::seeds::BACKER, proposal.key().as_ref(), backer.key().as_ref()],
        bump,
    )]
    /// CHECK: Verify backer actually backed this proposal
    pub backer_account: UncheckedAccount<'info>,
}

impl<'info> DecrementBackerCount<'info> {
    pub fn handle_decrement(&mut self) -> Result<()> {
        // Verify the backer actually backed this proposal by checking if backer_account exists
        require!(
            !self.backer_account.data_is_empty(),
            ProposalError::IncorrectAccount
        );

        // Decrement the count if it's greater than 0
        if self.backer_proposal_count.active_count > 0 {
            self.backer_proposal_count.active_count = self
                .backer_proposal_count
                .active_count
                .checked_sub(1)
                .ok_or(ProposalError::NumericalOverflow)?;
        }

        Ok(())
    }
}

