use anchor_lang::prelude::*;

use crate::{
    constant::{
        ANCHOR_DISCRIMINATOR, MAX_AMOUNT_TO_RAISE, MIN_AMOUNT_TO_RAISE,
        SECONDS_TO_DAYS,
    },
    state::{backers::Backers, proposer::Proposer},
    errors::ProposalError,
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
        // require!(
        //     (self.backer_account.amount
        //         <= (self.proposer.amount_to_raise * MAX_CONTRIBUTION_PERCENTAGE)
        //             / PERCENTAGE_SCALER)
        //         && (self.backer_account.amount + amount
        //             <= (self.proposer.amount_to_raise * MAX_CONTRIBUTION_PERCENTAGE)
        //                 / PERCENTAGE_SCALER),
        //     ProposalError::MaximumContributionsReached
        // );

        let cpi_transfer_ix = system_instruction::transfer(
            &self.backer.key(),
            &self.proposer.key(),
            amount,
        );

        anchor_lang::solana_program::program::invoke(
            &cpi_transfer_ix,
            &[
                self.backer.to_account_info(),
                self.proposer.to_account_info(),
                self.system_program.to_account_info(),
            ],
        )?;

        // Update the proposer and backer accounts with the new amounts
        self.proposer.current_amount += amount;

        self.backer_account.amount += amount;

        Ok(())
    }
}
