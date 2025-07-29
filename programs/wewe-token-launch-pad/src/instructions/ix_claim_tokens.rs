use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount, Transfer},
};
use crate::{
    const_pda::const_authority::VAULT_BUMP, constant::seeds::{BACKER, TOKEN_VAULT, VAULT_AUTHORITY}, errors::ProposalError, event::AirdropClaimed, state::{backers::Backers, proposal::Proposal}
};

#[derive(Accounts)]
pub struct Claim<'info> {
    #[account(mut)]
    pub backer: Signer<'info>,
    pub maker: SystemAccount<'info>,
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

    #[account(mut)]
    pub mint_account: Account<'info, Mint>,

    #[account(
        mut,
        seeds = [TOKEN_VAULT, vault_authority.key().as_ref(), mint_account.key().as_ref()],
        token::mint = mint_account,
        token::authority = vault_authority,
        bump,
    )]
    pub token_vault: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [BACKER, proposal.key().as_ref(), backer.key().as_ref()],
        bump,
    )]
    pub backer_account: Account<'info, Backers>,

    #[account(
        init_if_needed,
        payer = backer,
        associated_token::mint = mint_account,
        associated_token::authority = backer,
    )]
    pub backer_token_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl<'info> Claim<'info> {
    pub fn claim(&mut self) -> Result<()> {
        require!(self.proposal.is_pool_launched, ProposalError::TargetNotMet);

        let signer_seeds: &[&[&[u8]]] = &[&[VAULT_AUTHORITY, &[VAULT_BUMP]]];

        let claim_amount = self.backer_account.claim_amount;
        anchor_spl::token::transfer(
            CpiContext::new_with_signer(
                self.token_program.to_account_info(),
                Transfer {
                    from: self.token_vault.to_account_info(),
                    to: self.backer_token_account.to_account_info(),
                    authority: self.vault_authority.to_account_info(),
                },
                signer_seeds,
            ),
            claim_amount * 10u64.pow(9 as u32),
        )?;

        // set claim amount to zero, for succesive airdrops
        self.backer_account.claim_amount = 0;

        emit!(AirdropClaimed {
            proposal_address: self.proposal.key(),
            backer: self.backer.key(),
            backer_account: self.backer_account.key(),
            mint_account: self.mint_account.key(),
            vault_account: self.token_vault.key(),
            recipient_account: self.backer_token_account.key(),
            amount: claim_amount,
        });

        Ok(())
    }
}
