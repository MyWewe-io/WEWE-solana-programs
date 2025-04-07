use anchor_lang::prelude::*;
use anchor_lang::system_program;

use crate::{constant::{SECONDS_TO_DAYS, TOTAL_AMOUNT_TO_RAISE}, errors::ProposalError, state::{backers::Backers, proposer::Proposer}};


#[derive(Accounts)]
pub struct Refund<'info> {
    #[account(mut)]
    pub backer: Signer<'info>,
    pub maker: SystemAccount<'info>,
    #[account(
        mut,
        seeds = [b"proposer", maker.key().as_ref()],
        bump = proposer.bump,
    )]
    pub proposer: Account<'info, Proposer>,
    #[account(
        mut,
        seeds = [b"backer", proposer.key().as_ref(), backer.key().as_ref()],
        bump,
        close = backer,
    )]
    pub backer_account: Account<'info, Backers>,
    pub system_program: Program<'info, System>,
}

impl<'info> Refund<'info> {
    pub fn refund(&mut self) -> Result<()> {

        // Check if the fundraising duration has been reached
        let current_time = Clock::get()?.unix_timestamp;
 
        require!(
            self.proposer.duration >= ((current_time - self.proposer.time_started) / SECONDS_TO_DAYS) as u16,
            ProposalError::BackingNotEnded
        );

        require!(
            TOTAL_AMOUNT_TO_RAISE < self.proposer.current_amount,
            ProposalError::TargetMet
        );

        system_program::transfer(
            CpiContext::new(
                self.system_program.to_account_info(),
                system_program::Transfer {
                    from: self.proposer.to_account_info(),
                    to: self.backer.to_account_info(),
                },
            ),
            self.backer_account.amount,
        )?;
        
        self.proposer.current_amount -= self.backer_account.amount;

        Ok(())
    }
}
