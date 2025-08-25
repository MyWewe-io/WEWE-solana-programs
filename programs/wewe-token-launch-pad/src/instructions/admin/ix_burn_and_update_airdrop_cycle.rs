use {
    crate::{
        const_pda::const_authority::VAULT_BUMP,
        constant::{
            seeds::{TOKEN_VAULT, VAULT_AUTHORITY},
            TOTAL_AIRDROP_AMOUNT_PER_MILESTONE,
        },
        errors::ProposalError,
        event::TokensBurned,
        state::proposal::Proposal,
    },
    anchor_lang::prelude::*,
    anchor_spl::token::{Mint, Token, TokenAccount},
};

#[derive(Accounts)]
pub struct BurnTokens<'info> {
    pub authority: Signer<'info>,
    #[account(mut)]
    pub proposal: Account<'info, Proposal>,

    /// CHECK: vault authority
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

    #[account(
        mut,
        constraint = mint.key() == proposal.mint_account @ ProposalError::IncorrectAccount
    )]
    pub mint: Account<'info, Mint>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

impl<'info> BurnTokens<'info> {
    pub fn burn_tokens(&mut self, amount: u64) -> Result<()> {
        require!(!self.proposal.is_rejected, ProposalError::ProposalRejected);
        require!(
            self.proposal.is_pool_launched,
            ProposalError::BackingNotEnded
        );
        require!(
            amount <= TOTAL_AIRDROP_AMOUNT_PER_MILESTONE,
            ProposalError::AmountTooBig
        );

        let signer_seeds: &[&[&[u8]]] = &[&[VAULT_AUTHORITY, &[VAULT_BUMP]]];

        let pow = 10u64
            .checked_pow(self.mint.decimals as u32)
            .ok_or(ProposalError::NumericalOverflow)?;
        let burn_amount = amount.checked_mul(pow).ok_or(ProposalError::NumericalOverflow)?;

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

        self.proposal.current_airdrop_cycle = self
            .proposal
            .current_airdrop_cycle
            .checked_add(1)
            .ok_or(ProposalError::NumericalOverflow)?;

        emit!(TokensBurned {
            proposal: self.proposal.key(),
            amount,
            cycle: self.proposal.current_airdrop_cycle,
        });

        Ok(())
    }
}
