use crate::{
    constant::{seeds::BACKER, TOTAL_AIRDROP_AMOUNT_PER_MILESTONE},
    errors::ProposalError,
    event::BackerMilestoneSettled,
    state::{backers::Backers, proposal::Proposal},
};
use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, TokenAccount};

#[derive(Accounts)]
pub struct SnapshotBacker<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(mut)]
    pub proposal: Account<'info, Proposal>,

    /// CHECK: wallet being processed
    pub backer: UncheckedAccount<'info>,

    #[account(
        mut,
        seeds = [BACKER, proposal.key().as_ref(), backer.key().as_ref()],
        bump,
    )]
    pub backer_account: Account<'info, Backers>,

    #[account(constraint = mint_account.key() == proposal.mint_account @ ProposalError::NotOwner)]
    pub mint_account: Account<'info, Mint>,

    #[account(
        constraint = backer_token_account.mint  == mint_account.key() @ ProposalError::NotOwner,
        constraint = backer_token_account.owner == backer.key()      @ ProposalError::NotOwner,
    )]
    pub backer_token_account: Account<'info, TokenAccount>,
}

impl<'info> SnapshotBacker<'info> {
    pub fn handle_snapshot(&mut self) -> Result<()> {
        require!(self.proposal.is_pool_launched, ProposalError::TargetNotMet);
        require!(
            self.proposal.milestone_active,
            ProposalError::NoMilestoneActive
        );

        let cur = self.proposal.current_airdrop_cycle;
        require!(
            self.backer_account.settle_cycle >= cur,
            ProposalError::AmountAlreadyUpdated
        );
        let per = TOTAL_AIRDROP_AMOUNT_PER_MILESTONE
            .checked_div(self.proposal.total_backers)
            .ok_or(ProposalError::NumericalOverflow)?;

        let pending_claim = self.backer_account.settle_cycle.saturating_sub(self.backer_account.claimed_upto);
        let expected_units = per;
        let expected_base = expected_units.saturating_mul(self.backer_account.settle_cycle as u64);
        let mut actual_base = self.backer_token_account.amount;
        if self.backer_account.claimed_upto < self.backer_account.settle_cycle {
            actual_base = actual_base.saturating_mul(pending_claim as u64);
        }

        let bp = (actual_base.saturating_mul(10_000)).saturating_div(expected_base);
        let pct = tier_pct_from_bp(bp);

        let alloc_units = (per as u128)
            .saturating_mul(pct as u128)
            .checked_div(100)
            .ok_or(ProposalError::NumericalOverflow)? as u64;

        if alloc_units > 0 {
            self.backer_account.claim_amount = self
                .backer_account
                .claim_amount
                .checked_add(alloc_units)
                .ok_or(ProposalError::NumericalOverflow)?;
            self.proposal.milestone_units_assigned = self
                .proposal
                .milestone_units_assigned
                .checked_add(alloc_units)
                .ok_or(ProposalError::NumericalOverflow)?;
        }

        self.backer_account.settle_cycle = cur;
        self.proposal.milestone_backers_weighted = self
            .proposal
            .milestone_backers_weighted
            .checked_add(1)
            .ok_or(ProposalError::NumericalOverflow)?;

        emit!(BackerMilestoneSettled {
            proposal: self.proposal.key(),
            backer: self.backer.key(),
            cycle: cur,
            alloc_units,
        });

        Ok(())
    }
}

fn tier_pct_from_bp(bp: u64) -> u8 {
    if bp >= 10_000 {
        100
    } else if bp >= 7_000 {
        75
    } else if bp >= 5_000 {
        50
    } else if bp >= 2_500 {
        25
    } else {
        0
    }
}
