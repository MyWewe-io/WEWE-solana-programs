use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{self, Token, Transfer as TokenTransfer},
    token_interface::{TokenAccount, TokenInterface},
};

use crate::{
    const_pda::const_authority::VAULT_BUMP,
    constant::{
        seeds::{POOL_AUTHORITY_PREFIX, VAULT_AUTHORITY},
        treasury,
    },
    state::proposal::Proposal,
};

#[derive(Accounts)]
pub struct ClaimPositionFee<'info> {
    /// CHECK: pool authority
    #[account(
        seeds = [
            POOL_AUTHORITY_PREFIX.as_ref(),
        ],
        bump,
    )]
    pub pool_authority: UncheckedAccount<'info>,

    #[account(mut)]
    pub payer: Signer<'info>,

    /// CHECK: maker of the propposal
    pub maker: UncheckedAccount<'info>,

    /// CHECK: owner of the propposal
    #[account(address = treasury::ID)]
    pub wewe_vault: UncheckedAccount<'info>,

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
        init_if_needed,
        payer = payer,
        associated_token::mint = token_b_mint,
        associated_token::authority = wewe_vault,
        associated_token::token_program = token_b_program,
    )]
    pub wewe_wsol_account: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        init_if_needed,
        payer = payer,
        associated_token::mint = token_a_mint,
        associated_token::authority = wewe_vault,
        associated_token::token_program = token_a_program,
    )]
    pub wewe_token_account: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        init_if_needed,
        payer = payer,
        associated_token::authority = maker,
        associated_token::mint = token_b_mint,
        associated_token::token_program = token_b_program,
    )]
    pub maker_wsol_account: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        mut,
        associated_token::mint = token_a_mint,
        associated_token::authority = proposal.maker,
        associated_token::token_program = token_a_program,
    )]
    pub maker_token_account: Box<InterfaceAccount<'info, TokenAccount>>,

    /// CHECK: pool address
    pub pool: UncheckedAccount<'info>,

    /// CHECK: position address
    pub position: UncheckedAccount<'info>,

    /// The user token a account
    #[account(mut)]
    pub token_a_account: Box<InterfaceAccount<'info, TokenAccount>>,

    /// The user token b account
    #[account(mut)]
    pub token_b_account: Box<InterfaceAccount<'info, TokenAccount>>,

    /// The vault token account for input token
    #[account(mut, token::token_program = token_a_program, token::mint = token_a_mint)]
    pub token_a_vault: Box<InterfaceAccount<'info, TokenAccount>>,

    /// The vault token account for output token
    #[account(mut, token::token_program = token_b_program, token::mint = token_b_mint)]
    pub token_b_vault: Box<InterfaceAccount<'info, TokenAccount>>,

    /// CHECK:
    pub token_a_mint: UncheckedAccount<'info>,

    /// CHECK:
    pub token_b_mint: UncheckedAccount<'info>,

    /// CHECK:
    pub position_nft_account: UncheckedAccount<'info>,

    pub token_a_program: Interface<'info, TokenInterface>,

    pub token_b_program: Interface<'info, TokenInterface>,

    /// CHECK: amm program address
    #[account(address = cp_amm::ID)]
    pub amm_program: UncheckedAccount<'info>,

    /// CHECK:
    pub event_authority: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

impl<'info> ClaimPositionFee<'info> {
    pub fn claim_position_fee(&self, user_wsol_amount: u64, user_token_amount: u64) -> Result<()> {
        let pool_authority_seeds: &[&[u8]] = &[VAULT_AUTHORITY, &[VAULT_BUMP]];

        cp_amm::cpi::claim_position_fee(CpiContext::new_with_signer(
            self.amm_program.to_account_info(),
            cp_amm::cpi::accounts::ClaimPositionFeeCtx {
                pool_authority: self.pool_authority.to_account_info(),
                pool: self.pool.to_account_info(),
                position: self.position.to_account_info(),
                token_a_account: self.token_a_account.to_account_info(),
                token_b_account: self.token_b_account.to_account_info(),
                token_a_vault: self.token_a_vault.to_account_info(),
                token_b_vault: self.token_b_vault.to_account_info(),
                token_a_mint: self.token_a_mint.to_account_info(),
                token_b_mint: self.token_b_mint.to_account_info(),
                position_nft_account: self.position_nft_account.to_account_info(),
                owner: self.vault_authority.to_account_info(),
                token_a_program: self.token_a_program.to_account_info(),
                token_b_program: self.token_b_program.to_account_info(),
                event_authority: self.event_authority.to_account_info(),
                program: self.amm_program.to_account_info(),
            },
            &[&pool_authority_seeds[..]],
        ))?;

        let signer_seeds: &[&[&[u8]]] = &[&[VAULT_AUTHORITY, &[VAULT_BUMP]]];

        // transfer tokens to user
        anchor_spl::token::transfer(
            CpiContext::new_with_signer(
                self.token_a_program.to_account_info(),
                TokenTransfer {
                    from: self.token_a_account.to_account_info(),
                    to: self.maker_token_account.to_account_info(),
                    authority: self.vault_authority.to_account_info(),
                },
                signer_seeds,
            ),
            user_token_amount * 10u64.pow(9 as u32),
        )?;

        anchor_spl::token::transfer(
            CpiContext::new_with_signer(
                self.token_a_program.to_account_info(),
                TokenTransfer {
                    from: self.token_a_account.to_account_info(),
                    to: self.wewe_token_account.to_account_info(),
                    authority: self.vault_authority.to_account_info(),
                },
                signer_seeds,
            ),
            user_token_amount * 10u64.pow(9 as u32),
        )?;

        token::transfer(
            CpiContext::new(
                self.token_b_program.to_account_info(),
                TokenTransfer {
                    from: self.token_b_account.to_account_info(),
                    to: self.wewe_wsol_account.to_account_info(),
                    authority: self.payer.to_account_info(), // user is signer
                },
            ),
            user_wsol_amount,
        )?;

        token::transfer(
            CpiContext::new(
                self.token_b_program.to_account_info(),
                TokenTransfer {
                    from: self.token_b_account.to_account_info(),
                    to: self.maker_wsol_account.to_account_info(),
                    authority: self.vault_authority.to_account_info(),
                },
            ),
            user_wsol_amount,
        )?;

        Ok(())
    }
}
