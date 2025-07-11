use std::ops::Sub;

use anchor_lang::prelude::*;
use anchor_spl::token::Token;

use crate::const_pda::const_authority::VAULT_BUMP;
use crate::constant::{AMOUNT_TO_RAISE_PER_USER, FEE_TO_DEDUCT, SECONDS_TO_DAYS, VAULT_AUTHORITY};
use crate::errors::ProposalError;
use crate::event::BackerRefunded;
use crate::state::{backers::Backers, proposal::Proposal};

#[derive(Accounts)]
pub struct Refund<'info> {
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
        mut,
        seeds = [b"backer", proposal.key().as_ref(), backer.key().as_ref()],
        bump,
        close = backer,
    )]
    pub backer_account: Account<'info, Backers>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

impl<'info> Refund<'info> {
    pub fn refund(&mut self) -> Result<()> {
        if !self.proposal.is_rejected {
            let current_time = Clock::get()?.unix_timestamp;
            require!(
                self.proposal.duration
                    <= ((current_time - self.proposal.time_started) / SECONDS_TO_DAYS) as u16,
                ProposalError::BackingNotEnded
            );
        }

        let refund_amount = AMOUNT_TO_RAISE_PER_USER.sub(FEE_TO_DEDUCT);

        let signer_seeds: &[&[&[u8]]] = &[&[b"vault_authority", &[VAULT_BUMP]]];

        // Transfer SOL from proposal to backer
        anchor_lang::system_program::transfer(
            CpiContext::new_with_signer(
                self.system_program.to_account_info(),
                anchor_lang::system_program::Transfer {
                    from: self.vault_authority.to_account_info(),
                    to: self.backer.to_account_info(),
                },
                signer_seeds,
            ),
            refund_amount,
        )?;

        self.proposal.total_backing -= refund_amount;

        emit!(BackerRefunded {
            backer: self.backer.key(),
            amount: refund_amount,
        });

        Ok(())
    }
}
