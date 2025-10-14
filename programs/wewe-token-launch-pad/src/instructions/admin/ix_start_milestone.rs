use crate::{errors::ProposalError, event::MilestoneStarted, state::proposal::Proposal};
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct InitialiseMilestone<'info> {
    pub authority: Signer<'info>,
    #[account(mut)]
    pub proposal: Account<'info, Proposal>,
}

impl<'info> InitialiseMilestone<'info> {
    pub fn handle_initialise_milestone(&mut self) -> Result<()> {
        require!(!self.proposal.is_rejected, ProposalError::ProposalRejected);
        require!(self.proposal.is_pool_launched, ProposalError::TargetNotMet);
        require!(
            !self.proposal.milestone_active,
            ProposalError::NoMilestoneActive
        );

        self.proposal.milestone_active = true;
        self.proposal.milestone_backers_weighted = 0;

        emit!(MilestoneStarted {
            proposal: self.proposal.key(),
            token_mint: self.proposal.mint_account.key(),
            cycle: self.proposal.current_airdrop_cycle,
        });
        Ok(())
    }
}
