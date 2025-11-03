use crate::{
    const_pda::const_authority::VAULT_BUMP,
    constant::{seeds::*, treasury, FEE_TO_DEDUCT},
    errors::ProposalError,
    event::BackerRefunded,
    state::{backers::Backers, backer_proposal_count::BackerProposalCount, proposal::Proposal, config::Configs},
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

    /// CHECK: WEWE treasury account
    #[account(
        mut,
        address = treasury::ID,
    )]
    pub wewe_treasury: UncheckedAccount<'info>,

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
        
        // Calculate what was actually deposited (amount minus the fee that was deducted)
        // This matches what was added to total_backing in ix_back_token.rs
        let deposited_amount = self.config.amount_to_raise_per_user
            .checked_sub(FEE_TO_DEDUCT)
            .ok_or(ProposalError::NumericalOverflow)?;
        
        // Get refund fee basis points with validation
        // If uninitialized (likely 0 or garbage), default to 0 to avoid overflow
        let fee_bps = if self.config.refund_fee_basis_points > 10000 {
            0u16 // Safety: if value is unreasonable, default to 0
        } else {
            self.config.refund_fee_basis_points
        };
        
        // Calculate refund amount and fee
        // Fee is calculated as a percentage (from config) of the refund amount
        // Formula: refund_amount + fee = deposited_amount
        //          fee = refund_amount * fee_bps / 10000
        //          refund_amount = deposited_amount * 10000 / (10000 + fee_bps)
        const BASIS_POINTS: u128 = 10000;
        let fee_bps_u128 = fee_bps as u128;
        let denominator = BASIS_POINTS
            .checked_add(fee_bps_u128)
            .ok_or(ProposalError::NumericalOverflow)?;
        
        let refund_amount_u128 = (deposited_amount as u128)
            .checked_mul(BASIS_POINTS)
            .and_then(|n| n.checked_div(denominator))
            .ok_or(ProposalError::NumericalOverflow)?;
        
        let refund_amount: u64 = refund_amount_u128
            .try_into()
            .map_err(|_| ProposalError::NumericalOverflow)?;
        
        // Calculate fee as percentage of refund amount (using fee_bps from config)
        let wewe_fee_u128 = (refund_amount_u128)
            .checked_mul(fee_bps_u128)
            .and_then(|n| n.checked_div(BASIS_POINTS))
            .ok_or(ProposalError::NumericalOverflow)?;
        
        let wewe_fee_to_collect: u64 = wewe_fee_u128
            .try_into()
            .map_err(|_| ProposalError::NumericalOverflow)?;
        
        // Verify the math: refund + fee should equal deposited_amount (within rounding)
        let total = refund_amount
            .checked_add(wewe_fee_to_collect)
            .ok_or(ProposalError::NumericalOverflow)?;
        require!(
            total <= deposited_amount && deposited_amount.saturating_sub(total) <= 1,
            ProposalError::NumericalOverflow
        );

        let signer_seeds: &[&[&[u8]]] = &[&[VAULT_AUTHORITY, &[VAULT_BUMP]]];

        // Transfer WEWE fee to treasury
        if wewe_fee_to_collect > 0 {
            anchor_lang::system_program::transfer(
                CpiContext::new_with_signer(
                    self.system_program.to_account_info(),
                    anchor_lang::system_program::Transfer {
                        from: self.vault_authority.to_account_info(),
                        to: self.wewe_treasury.to_account_info(),
                    },
                    signer_seeds,
                ),
                wewe_fee_to_collect,
            )?;
        }

        // Transfer remaining SOL to backer
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

        // Update total_backing to reflect the deposited amount being removed
        // Note: total_backing tracks deposited_amount (amount_to_raise_per_user - FEE_TO_DEDUCT)
        let total_removed = deposited_amount;
        let old_total_backing = self.proposal.total_backing;
        
        // Verify we have enough backing before subtracting
        require!(
            old_total_backing >= total_removed,
            ProposalError::NumericalOverflow
        );
        
        self.proposal.total_backing = old_total_backing
            .checked_sub(total_removed)
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
            backer_account: self.backer_account.key(),
            proposal_address: self.proposal.key(),
            refund_amount,
            wewe_fee: wewe_fee_to_collect,
        });

        Ok(())
    }
}
