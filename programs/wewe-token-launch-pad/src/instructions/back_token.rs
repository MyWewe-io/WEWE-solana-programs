use anchor_lang::{
    prelude::*,
    system_program::{transfer, Transfer},
};

use crate::{
    constant::{AMOUNT_TO_RAISE_PER_USER, ANCHOR_DISCRIMINATOR, MAXIMUM_BACKERS, SECONDS_TO_DAYS, VAULT_AUTHORITY},
    errors::ProposalError,
    event::ProposalBacked,
    state::{backers::Backers, proposal::Proposal},
};

#[derive(Accounts)]
pub struct Contribute<'info> {
    #[account(mut)]
    pub backer: Signer<'info>,
    #[account(mut)]
    pub proposal: Account<'info, Proposal>,
    /// CHECK: vault authority
    #[account(
        mut,
        seeds = [
            VAULT_AUTHORITY.as_ref(),
        ],
        bump,
    )]
    pub vault_authority: UncheckedAccount<'info>,
    #[account(
        init,
        payer = backer,
        seeds = [b"backer", proposal.key().as_ref(), backer.key().as_ref()],
        bump,
        space = ANCHOR_DISCRIMINATOR + Backers::INIT_SPACE,
    )]
    pub backer_account: Account<'info, Backers>,
    pub system_program: Program<'info, System>,
}

impl<'info> Contribute<'info> {
    pub fn deposit_sol(&mut self) -> Result<()> {
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

        require! (
            self.proposal.total_backers < MAXIMUM_BACKERS,
            ProposalError::BackingGoalReached
        );

        let program_id = self.system_program.to_account_info();
        let cpi_context = CpiContext::new(
            program_id,
            Transfer {
                from: self.backer.to_account_info(),
                to: self.vault_authority.to_account_info(),
            },
        );

        transfer(cpi_context, AMOUNT_TO_RAISE_PER_USER)?;

        // Update the proposal and backer accounts with the new amounts
        self.proposal.total_backing += AMOUNT_TO_RAISE_PER_USER;
        self.proposal.total_backers += 1;

        emit!(ProposalBacked {
            backer: self.backer.key(),
            proposal_backed: self.proposal.key(),
        });

        Ok(())
    }
}
