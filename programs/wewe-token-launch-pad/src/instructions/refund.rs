use std::ops::Sub;

use anchor_lang::prelude::*;

use crate::constant::FEE_TO_DEDUCT;
use crate::event::BackerRefunded;
use crate::{
    constant::{SECONDS_TO_DAYS, TOTAL_AMOUNT_TO_RAISE},
    errors::ProposalError,
    state::{backers::Backers, proposer::Proposal},
};

#[derive(Accounts)]
#[instruction(_proposal_index: u64)]
pub struct Refund<'info> {
    #[account(mut)]
    /// CHECK: it recievs refund
    pub recepient: AccountInfo<'info>,
    /// CHECK: maker of the proposal
    pub maker: AccountInfo<'info>,
    #[account(
        mut,
        seeds = [b"proposer", maker.key().as_ref(), &_proposal_index.to_le_bytes()],
        bump = proposal.bump,
    )]
    pub proposal: Account<'info, Proposal>,
    #[account(
        mut,
        seeds = [b"backer", proposal.key().as_ref(), recepient.key().as_ref()],
        bump,
        close = recepient,
    )]
    pub backer_account: Account<'info, Backers>,
    pub system_program: Program<'info, System>,
}

impl<'info> Refund<'info> {
    pub fn refund(&mut self, _proposal_index: u64) -> Result<()> {
        // Check if the proposal is not rejected before performing other checks
        if !self.proposal.is_rejected {
            // Check if the fundraising duration has been reached
            let current_time = Clock::get()?.unix_timestamp;
            require!(
                self.proposal.duration
                    <= ((current_time - self.proposal.time_started) / SECONDS_TO_DAYS) as u16,
                ProposalError::BackingNotEnded
            );

            // Check if the target amount has not been met
            require!(
                TOTAL_AMOUNT_TO_RAISE < self.proposal.current_amount,
                ProposalError::TargetMet
            );
        }
        let refund_amount = self.backer_account.amount.sub(FEE_TO_DEDUCT);

        **self.proposal.to_account_info().try_borrow_mut_lamports()? -= refund_amount;
        **self.recepient.try_borrow_mut_lamports()? += refund_amount;
        
        self.proposal.current_amount -= refund_amount;

        emit!(BackerRefunded {
            backer: self.recepient.key(),
            amount: refund_amount,
        });

        Ok(())
    }
}
