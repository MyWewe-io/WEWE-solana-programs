use crate::{
    const_pda::const_authority::VAULT_BUMP,
    constant::seeds::{TOKEN_VAULT, VAULT_AUTHORITY},
    errors::ProposalError,
    event::MilestoneEnded,
    state::proposal::Proposal,
};
use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};

#[derive(Accounts)]
pub struct EndMilestone<'info> {
    pub authority: Signer<'info>,
    #[account(mut)]
    pub proposal: Account<'info, Proposal>,

    #[account(
        mut,
        constraint = mint.key() == proposal.mint_account @ ProposalError::IncorrectAccount
    )]
    pub mint: Account<'info, Mint>,

    #[account(
        mut,
        seeds = [VAULT_AUTHORITY.as_ref()],
        bump,
    )]
    pub vault_authority: SystemAccount<'info>,

    #[account(
        mut,
        seeds = [TOKEN_VAULT, vault_authority.key().as_ref(), mint.key().as_ref()],
        bump,
        token::mint = mint,
        token::authority = vault_authority,
    )]
    pub token_vault: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

impl<'info> EndMilestone<'info> {
    pub fn handle_end_milestone(&mut self) -> Result<()> {
        require!(self.proposal.is_pool_launched, ProposalError::TargetNotMet);
        require!(!self.proposal.is_rejected, ProposalError::ProposalRejected);
        require!(
            self.proposal.milestone_active,
            ProposalError::NoMilestoneActive
        );
        require!(
            self.proposal.milestone_backers_weighted == self.proposal.total_backers,
            ProposalError::AllBackerScoreNotUpdated
        );

        let signer_seeds: &[&[&[u8]]] = &[&[VAULT_AUTHORITY, &[VAULT_BUMP]]];
        let pow = 10u64
            .checked_pow(self.mint.decimals as u32)
            .ok_or(ProposalError::NumericalOverflow)?;

        // Calculate burn amount using reputation-based formula:
        // (NUM_HOLDERS * 100) - SUM(reputation_scores) = burn amount (in base units)
        let num_holders = self.proposal.milestone_backers_weighted;
        let max_possible_reputation = num_holders
            .checked_mul(100)
            .ok_or(ProposalError::NumericalOverflow)?;
        let burn_amount_base = max_possible_reputation
            .saturating_sub(self.proposal.milestone_reputation_sum);
        
        // Convert from base units to token units (with decimals)
        let burn_amount = burn_amount_base
            .checked_mul(pow)
            .ok_or(ProposalError::NumericalOverflow)?;

        anchor_spl::token::burn(
            CpiContext::new_with_signer(
                self.token_program.to_account_info(),
                anchor_spl::token::Burn {
                    mint: self.mint.to_account_info(),
                    from: self.token_vault.to_account_info(),
                    authority: self.vault_authority.to_account_info(),
                },
                signer_seeds,
            ),
            burn_amount,
        )?;

        self.proposal.milestone_active = false;
        self.proposal.milestone_reputation_sum = 0;
        self.proposal.current_airdrop_cycle = self.proposal.current_airdrop_cycle.checked_add(1)
        .ok_or(ProposalError::NumericalOverflow)?;

        emit!(MilestoneEnded {
            proposal: self.proposal.key(),
            cycle: self.proposal.current_airdrop_cycle,
            burned_units: burn_amount
        });
        Ok(())
    }
}
