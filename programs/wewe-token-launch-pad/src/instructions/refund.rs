use std::ops::Sub;

use anchor_lang::prelude::*;
use anchor_lang::system_program;

use crate::constant::FEE_TO_DEDUCT;
use crate::event::BackerRefunded;
use crate::{
    constant::{SECONDS_TO_DAYS, TOTAL_AMOUNT_TO_RAISE},
    errors::ProposalError,
    state::{backers::Backers, proposer::Proposal},
};

#[derive(Accounts)]
pub struct Refund<'info> {
    pub payer: Signer<'info>,
    pub backer: SystemAccount<'info>,
    pub maker: SystemAccount<'info>,
    #[account(
        mut,
        seeds = [b"proposer", maker.key().as_ref()],
        bump = proposal.bump,
    )]
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

            // Check if the target amount has not been met
            require!(
                TOTAL_AMOUNT_TO_RAISE < self.proposal.current_amount,
                ProposalError::TargetMet
            );
        }

        let refund_amount = self.backer_account.amount.sub(FEE_TO_DEDUCT);

        system_program::transfer(
            CpiContext::new(
                self.system_program.to_account_info(),
                system_program::Transfer {
                    from: self.proposal.to_account_info(),
                    to: self.backer.to_account_info(),
                },
            ),
            refund_amount,
        )?;

        self.proposal.current_amount -= refund_amount;

        emit!(BackerRefunded {
           backer: self.backer.key(),
           amount: refund_amount,
        });

        Ok(())
    }
}

