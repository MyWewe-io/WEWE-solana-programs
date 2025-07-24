use std::ops::Sub;

use anchor_lang::prelude::*;
use anchor_spl::token::Token;

use crate::{
    const_pda::const_authority::VAULT_BUMP,
    constant::{seeds::*, AMOUNT_TO_RAISE_PER_USER, FEE_TO_DEDUCT},
    errors::ProposalError,
    event::BackerRefunded,
    state::{backers::Backers, proposal::Proposal},
};

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
        seeds = [BACKER, proposal.key().as_ref(), backer.key().as_ref()],
        bump,
        close = backer,
    )]
    pub backer_account: Account<'info, Backers>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

impl<'info> Refund<'info> {
    pub fn refund(&mut self) -> Result<()> {
        require!(self.proposal.is_rejected, ProposalError::BackingNotEnded);
        let refund_amount = AMOUNT_TO_RAISE_PER_USER.sub(FEE_TO_DEDUCT);

        let signer_seeds: &[&[&[u8]]] = &[&[VAULT_AUTHORITY, &[VAULT_BUMP]]];

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
            backer_account: self.backer_account.key(),
            proposal_address: self.proposal.key(),
        });

        Ok(())
    }
}
