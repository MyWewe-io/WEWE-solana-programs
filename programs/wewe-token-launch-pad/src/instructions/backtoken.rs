use anchor_lang::{prelude::*, system_program::{transfer, Transfer}};

use crate::{
    constant::{
        ANCHOR_DISCRIMINATOR, MAX_AMOUNT_TO_RAISE, MIN_AMOUNT_TO_RAISE,
        SECONDS_TO_DAYS,
    }, errors::ProposalError, event::ProposalBacked, state::{backers::Backers, proposer::Proposer},
};

#[derive(Accounts)]
pub struct Contribute<'info> {
    #[account(mut)]
    pub backer: Signer<'info>,
    #[account(
        mut,
        seeds = [b"proposer".as_ref(), proposer.maker.as_ref()],
        bump = proposer.bump,
    )]
    pub proposer: Account<'info, Proposer>,
    #[account(
        init_if_needed,
        payer = backer,
        seeds = [b"backer", proposer.key().as_ref(), backer.key().as_ref()],
        bump,
        space = ANCHOR_DISCRIMINATOR + Backers::INIT_SPACE,
    )]
    pub backer_account: Account<'info, Backers>,
    pub system_program: Program<'info, System>,
}

impl<'info> Contribute<'info> {
    pub fn deposit_sol(&mut self, amount: u64) -> Result<()> {
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
            self.proposer.duration
                <= ((current_time - self.proposer.time_started) / SECONDS_TO_DAYS) as u16,
            ProposalError::BackingEnded
        );

        // Check if the maximum contributions per backer have been reached
        require!(
            (self.backer_account.amount
                <= MAX_AMOUNT_TO_RAISE)
                && (self.backer_account.amount + amount
                    <= MAX_AMOUNT_TO_RAISE),
            ProposalError::MaximumContributionsReached
        );

        let program_id = self.system_program.to_account_info();
        let cpi_context = CpiContext::new(
            program_id,
            Transfer {
                from: self.backer.to_account_info(),
                to: self.proposer.to_account_info(),
            },
        );

        transfer(cpi_context, amount)?;

        // Update the proposer and backer accounts with the new amounts
        self.proposer.current_amount += amount;

        self.backer_account.amount += amount;

        emit!(ProposalBacked { backer: self.backer.key(), proposal_backed: self.proposer.key(), amount });

        Ok(())
    }
}
