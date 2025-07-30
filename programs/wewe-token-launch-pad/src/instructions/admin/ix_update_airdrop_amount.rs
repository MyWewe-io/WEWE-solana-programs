use {
    crate::{
        constant::TOTAL_AIRDROP_AMOUNT_PER_MILESTONE,
        errors::ProposalError,
        event::AirdropClaimUpdated,
        state::{backers::Backers, proposal::Proposal},
    },
    anchor_lang::prelude::*,
};

#[derive(Accounts)]
pub struct UpdateBacker<'info> {
    pub authority: Signer<'info>,
    pub proposal: Account<'info, Proposal>,
    #[account(mut)]
    pub backer_account: Account<'info, Backers>,
    pub system_program: Program<'info, System>,
}

impl<'info> UpdateBacker<'info> {
    pub fn update_airdrop_amount(&mut self, amount: u64) -> Result<()> {
        require!(self.proposal.is_pool_launched, ProposalError::BackingNotEnded);
        let max_per_user = TOTAL_AIRDROP_AMOUNT_PER_MILESTONE
            .checked_div(self.proposal.total_backers)
            .ok_or(ProposalError::NumericalOverflow)?;
        require!(
            amount <= max_per_user,
            ProposalError::AmountTooBig
        );

        require!(
            self.backer_account.amount_updated_upto_cycle < self.proposal.current_airdrop_cycle,
            ProposalError::AmountAlreadyUpdated
        );

        self.backer_account.claim_amount += amount;
        self.backer_account.amount_updated_upto_cycle += 1;

        emit!(AirdropClaimUpdated {
            proposal: self.proposal.key(),
            backer: self.backer_account.key(),
            amount,
            cycle: self.proposal.current_airdrop_cycle,
        });

        Ok(())
    }
}
