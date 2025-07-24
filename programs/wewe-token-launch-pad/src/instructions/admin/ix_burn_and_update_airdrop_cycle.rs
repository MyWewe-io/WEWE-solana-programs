use {
    crate::{
        const_pda::const_authority::VAULT_BUMP, constant::{seeds::VAULT_AUTHORITY, TOTAL_AIRDROP_AMOUNT_PER_MILESTONE}, errors::ProposalError, event::TokensBurned, state::proposal::Proposal
    },
    anchor_lang::prelude::*, anchor_spl::token::{Mint, Token, TokenAccount},
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
    pub vault_authority: UncheckedAccount<'info>,

    #[account(mut)]
    pub token_vault: Account<'info, TokenAccount>,

    #[account(mut)]
    pub mint: Account<'info, Mint>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

impl<'info> BurnTokens<'info> {
    pub fn burn_tokens(&mut self, amount: u64) -> Result<()> {
        require!(amount <= TOTAL_AIRDROP_AMOUNT_PER_MILESTONE, ProposalError::AmountTooBig);

        let signer_seeds: &[&[&[u8]]] = &[&[VAULT_AUTHORITY, &[VAULT_BUMP]]];
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
            amount,
        )?;

        self.proposal.current_airdrop_cycle += 1;

        emit!(TokensBurned {
            proposal: self.proposal.key(),
            amount,
            cycle: self.proposal.current_airdrop_cycle,
        });
        
        Ok(())
    }
}

