use std::ops::Mul;
use anchor_lang::prelude::*;

use crate::{const_pda::const_authority::VAULT_BUMP, constant::{FEE_TO_DEDUCT, VAULT_AUTHORITY}, event::ProposalRejected, state::proposal::Proposal};

#[derive(Accounts)]
pub struct RejectProposal<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    /// CHECK: vault authority
    #[account(
        mut,
        seeds = [
            VAULT_AUTHORITY.as_ref(),
        ],
        bump,
    )]
    pub vault_authority: UncheckedAccount<'info>,
    #[account(mut)]
    pub proposal: Account<'info, Proposal>,
    pub system_program: Program<'info, System>,
}

impl<'info> RejectProposal<'info> {
    pub fn reject_proposal(&mut self) -> Result<()> {
        self.proposal.is_rejected = true;

        let fee_collected = self.proposal.total_backers.mul(FEE_TO_DEDUCT);

        let signer_seeds: &[&[&[u8]]] = &[&[b"vault_authority", &[VAULT_BUMP]]];

        // Transfer SOL from proposal to backer
        anchor_lang::system_program::transfer(
            CpiContext::new_with_signer(
                self.system_program.to_account_info(),
                anchor_lang::system_program::Transfer {
                    from: self.vault_authority.to_account_info(),
                    to: self.authority.to_account_info(),
                },
                signer_seeds,
            ),
            fee_collected,
        )?;

        emit!(ProposalRejected {
            maker: self.proposal.maker,
            proposal_address: self.proposal.key(),
        });

        Ok(())
    }
}
