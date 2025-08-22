use {
    crate::{
        constant::{seeds::BACKER, TOTAL_AIRDROP_AMOUNT_PER_MILESTONE},
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
    /// CHECK: backer wallet used to bind the PDA seeds
    pub backer: UncheckedAccount<'info>,
    #[account(
        mut,
        seeds = [BACKER, proposal.key().as_ref(), backer.key().as_ref()],
        bump
    )]
    pub backer_account: Account<'info, Backers>,
    pub system_program: Program<'info, System>,
}

impl<'info> UpdateBacker<'info> {
    pub fn update_airdrop_amount(&mut self, amount: u64) -> Result<()> {
        require!(
            self.proposal.is_pool_launched,
            ProposalError::BackingNotEnded
        );
        let max_per_user = TOTAL_AIRDROP_AMOUNT_PER_MILESTONE
            .checked_div(self.proposal.total_backers)
            .ok_or(ProposalError::NumericalOverflow)?;
        require!(amount <= max_per_user, ProposalError::AmountTooBig);

        require!(
            self.backer_account.amount_updated_upto_cycle < self.proposal.current_airdrop_cycle,
            ProposalError::AmountAlreadyUpdated
        );

        self.backer_account.claim_amount = self
            .backer_account
            .claim_amount
            .checked_add(amount)
            .ok_or(ProposalError::NumericalOverflow)?;
        self.backer_account.amount_updated_upto_cycle += 1;

        emit!(AirdropClaimUpdated {
            proposal: self.proposal.key(),
            backer: self.backer.key(),
            amount,
            cycle: self.proposal.current_airdrop_cycle,
        });

        Ok(())
    }
}
