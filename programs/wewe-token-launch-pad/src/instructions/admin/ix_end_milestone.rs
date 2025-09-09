use crate::{errors::ProposalError, event::MilestoneStarted, state::proposal::Proposal};
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct EndMilestone<'info> {
    pub authority: Signer<'info>,
    #[account(mut)]
    pub proposal: Account<'info, Proposal>,
}

impl<'info> EndMilestone<'info> {
    pub fn handle_end_milestone(&mut self) -> Result<()> {
        require!(self.proposal.is_pool_launched, ProposalError::TargetNotMet);
        require!(!self.proposal.is_rejected, ProposalError::ProposalRejected);
        require!(
            self.proposal.milestone_active,
            ProposalError::NoMilestoneActive
        );
        require!(
            self.proposal.milestone_backers_weighted == self.proposal.total_backers,
            ProposalError::AllBackerScoreNotUpdated
        );

        //TODO: burn tokens that goes uncliamed for each milestone

        self.proposal.milestone_active = false;

        emit!(MilestoneStarted {
            proposal: self.proposal.key(),
            token_mint: self.proposal.mint_account.key(),
            cycle: self.proposal.current_airdrop_cycle,
        });
        Ok(())
    }
}
