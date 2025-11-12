use crate::{
    const_pda::const_authority::VAULT_BUMP,
    constant::{seeds::*, FEE_TO_DEDUCT},
    errors::ProposalError,
    event::BackerRefunded,
    state::{backers::Backers, backer_proposal_count::BackerProposalCount, proposal::Proposal,config::Configs},
};
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct Refund<'info> {
    /// CHECK:
    #[account(mut)]
    pub backer: AccountInfo<'info>,

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
    pub vault_authority: SystemAccount<'info>,

    #[account(
        mut,
        seeds = [BACKER, proposal.key().as_ref(), backer.key().as_ref()],
        bump,
        close = backer,
    )]
    pub backer_account: Account<'info, Backers>,

    #[account(
        mut,
        seeds = [BACKER_PROPOSAL_COUNT, backer.key().as_ref()],
        bump,
    )]
    pub backer_proposal_count: Account<'info, BackerProposalCount>,

    pub system_program: Program<'info, System>,
    pub config: Account<'info, Configs>,
}

impl<'info> Refund<'info> {
    pub fn handle_refund(&mut self) -> Result<()> {
        require!(self.proposal.is_rejected, ProposalError::BackingNotEnded);
        let refund_amount = self.config.amount_to_raise_per_user //AMOUNT_TO_RAISE_PER_USER
            .checked_sub(FEE_TO_DEDUCT)
            .ok_or(ProposalError::NumericalOverflow)?;

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

        self.proposal.total_backing = self
            .proposal
            .total_backing
            .checked_sub(refund_amount)
            .ok_or(ProposalError::NumericalOverflow)?;

        // Decrement the backer's active proposal count
        if self.backer_proposal_count.active_count > 0 {
            self.backer_proposal_count.active_count = self
                .backer_proposal_count
                .active_count
                .checked_sub(1)
                .ok_or(ProposalError::NumericalOverflow)?;
        }

        emit!(BackerRefunded {
            backer: self.backer.key(),
            amount: refund_amount,
            backer_account: self.backer_account.key(),
            proposal_address: self.proposal.key(),
        });

        Ok(())
    }
}
