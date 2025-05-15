use anchor_lang::{
    prelude::*,
    system_program::{transfer, Transfer},
};

use crate::{
    constant::{ANCHOR_DISCRIMINATOR, MAX_AMOUNT_TO_RAISE, MIN_AMOUNT_TO_RAISE, SECONDS_TO_DAYS},
    errors::ProposalError,
    event::ProposalBacked,
    state::{backers::Backers, proposer::Proposal},
};

#[derive(Accounts)]
#[instruction(_proposal_index: u64)]
pub struct Contribute<'info> {
    #[account(mut)]
    pub backer: Signer<'info>,
    #[account(
        mut,
        seeds = [b"proposer", proposal.maker.as_ref(), &_proposal_index.to_le_bytes()],
        bump = proposal.bump,
    )]
    pub proposal: Account<'info, Proposal>,
    #[account(
        init_if_needed,
        payer = backer,
        seeds = [b"backer", proposal.key().as_ref(), backer.key().as_ref()],
        bump,
        space = ANCHOR_DISCRIMINATOR + Backers::INIT_SPACE,
    )]
    pub backer_account: Account<'info, Backers>,
    pub system_program: Program<'info, System>,
}

impl<'info> Contribute<'info> {
    pub fn deposit_sol(&mut self, _proposal_index: u64, amount: u64) -> Result<()> {
        // Check if the amount to contribute meets the minimum amount required
        require!(
            amount >= MIN_AMOUNT_TO_RAISE,
            ProposalError::ContributionTooSmall
        );

        // Check if the amount to contribute is less than the maximum allowed contribution
        require!(
            amount <= MAX_AMOUNT_TO_RAISE,
            ProposalError::ContributionTooBig
        );

        // Check if the fundraising duration has been reached
        let current_time = Clock::get()?.unix_timestamp;
        require!(
            self.proposal.duration
                >= ((current_time - self.proposal.time_started) / SECONDS_TO_DAYS) as u16,
            ProposalError::BackingEnded
        );

        require!(
            self.proposal.is_rejected == false,
            ProposalError::ProposalRejected
        );

        // Check if the maximum contributions per backer have been reached
        require!(
            (self.backer_account.amount <= MAX_AMOUNT_TO_RAISE)
                && (self.backer_account.amount + amount <= MAX_AMOUNT_TO_RAISE),
            ProposalError::MaximumContributionsReached
        );

        let program_id = self.system_program.to_account_info();
        let cpi_context = CpiContext::new(
            program_id,
            Transfer {
                from: self.backer.to_account_info(),
                to: self.proposal.to_account_info(),
            },
        );

        transfer(cpi_context, amount)?;

        // Update the proposal and backer accounts with the new amounts
        self.proposal.current_amount += amount;

        self.backer_account.amount += amount;

        emit!(ProposalBacked {
            backer: self.backer.key(),
            proposal_backed: self.proposal.key(),
            amount
        });

        Ok(())
    }
}
