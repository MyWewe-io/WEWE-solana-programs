use std::ops::Sub;

use anchor_lang::prelude::*;

use crate::constant::{AMOUNT_TO_RAISE_PER_USER, FEE_TO_DEDUCT};
use crate::event::BackerRefunded;
use crate::{
    constant::SECONDS_TO_DAYS,
    errors::ProposalError,
    state::{backers::Backers, proposer::Proposal},
};

#[derive(Accounts)]
pub struct Refund<'info> {
    #[account(mut)]
    /// CHECK: it recievs refund
    pub backer: AccountInfo<'info>,
    #[account(mut)]
    pub proposal: Account<'info, Proposal>,
    #[account(
        mut,
        seeds = [b"backer", proposal.key().as_ref(), backer.key().as_ref()],
        bump,
        close = backer,
    )]
    pub backer_account: Account<'info, Backers>,
    pub system_program: Program<'info, System>,
}

impl<'info> Refund<'info> {
    pub fn refund(&mut self) -> Result<()> {
        // Check if the proposal is not rejected before performing other checks
        if !self.proposal.is_rejected {
            // Check if the fundraising duration has been reached
            let current_time = Clock::get()?.unix_timestamp;
            require!(
                self.proposal.duration
                    <= ((current_time - self.proposal.time_started) / SECONDS_TO_DAYS) as u16,
                ProposalError::BackingNotEnded
            );
        }
        let refund_amount = AMOUNT_TO_RAISE_PER_USER.sub(FEE_TO_DEDUCT);

        **self.proposal.to_account_info().try_borrow_mut_lamports()? -= refund_amount;
        **self.backer.try_borrow_mut_lamports()? += refund_amount;

        self.proposal.current_amount -= refund_amount;

        emit!(BackerRefunded {
            backer: self.backer.key(),
            amount: refund_amount,
        });

        Ok(())
    }
}
