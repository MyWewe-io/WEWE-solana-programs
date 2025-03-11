use anchor_lang::prelude::*;
use anchor_spl::token::{transfer, Mint, Token, TokenAccount, Transfer};

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
    pub contributor: Signer<'info>,
    #[account(
        mut,
        seeds = [b"proposer".as_ref(), proposer.proposer.as_ref()],
        bump = proposer.bump,
    )]
    pub proposer: Account<'info, Proposer>,
    #[account(
        init_if_needed,
        payer = contributor,
        seeds = [b"contributor", proposer.key().as_ref(), contributor.key().as_ref()],
        bump,
        space = ANCHOR_DISCRIMINATOR + Backers::INIT_SPACE,
    )]
    pub contributor_account: Account<'info, Backers>,
    #[account(
        mut,
    )]
    pub token_program: Program<'info, Token>,
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

        // Check if the maximum contributions per contributor have been reached
        // require!(
        //     (self.contributor_account.amount
        //         <= (self.proposer.amount_to_raise * MAX_CONTRIBUTION_PERCENTAGE)
        //             / PERCENTAGE_SCALER)
        //         && (self.contributor_account.amount + amount
        //             <= (self.proposer.amount_to_raise * MAX_CONTRIBUTION_PERCENTAGE)
        //                 / PERCENTAGE_SCALER),
        //     ProposalError::MaximumContributionsReached
        // );

        // Transfer the funds to the vault
        // CPI to the token program to transfer the funds
        let cpi_program = self.token_program.to_account_info();

        // Transfer the funds from the contributor to the vault
        let cpi_accounts = Transfer {
            from: self.contributor_ata.to_account_info(),
            to: self.vault.to_account_info(),
            authority: self.contributor.to_account_info(),
        };

        // Crete a CPI context
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

        // Transfer the funds from the contributor to the vault
        transfer(cpi_ctx, amount)?;

        // Update the proposer and contributor accounts with the new amounts
        self.proposer.current_amount += amount;

        self.contributor_account.amount += amount;

        Ok(())
    }
}
