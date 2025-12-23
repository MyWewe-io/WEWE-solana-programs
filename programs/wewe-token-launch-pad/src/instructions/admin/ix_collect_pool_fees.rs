use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::Token,
    token_interface::{TokenAccount, TokenInterface},
};

use crate::{
    const_pda::{self, const_authority::VAULT_BUMP},
    constant::{self, seeds::VAULT_AUTHORITY, treasury},
    errors::ProposalError,
    event::PositionFeeClaimed,
    state::proposal::Proposal,
};

#[derive(Accounts)]
pub struct ClaimPositionFee<'info> {
    /// CHECK: pool authority
    #[account(
        mut,
        address = const_pda::const_authority::POOL_ID,
    )]
    pub pool_authority: UncheckedAccount<'info>,

    #[account(mut)]
    pub payer: Signer<'info>,

    /// CHECK: maker of the propposal
    #[account(constraint = maker.key() == proposal.maker @ ProposalError::NotOwner)]
    pub maker: UncheckedAccount<'info>,

    /// CHECK: owner of the propposal
    #[account(address = treasury::ID)]
    pub wewe_treasury: UncheckedAccount<'info>,

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
        init_if_needed,
        payer = payer,
        associated_token::mint = token_b_mint,
        associated_token::authority = wewe_treasury,
        associated_token::token_program = token_b_program,
    )]
    pub wewe_wsol_account: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        init_if_needed,
        payer = payer,
        associated_token::mint = token_a_mint,
        associated_token::authority = wewe_treasury,
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
    #[account(mut)]
    pub position: UncheckedAccount<'info>,

    /// The user token a account
    #[account(
        mut,
        seeds = [constant::seeds::TOKEN_VAULT, vault_authority.key().as_ref(), token_a_mint.key().as_ref()],
        bump,
        token::mint = token_a_mint,
        token::authority = vault_authority,
        token::token_program = token_a_program,
    )]
    pub token_a_account: Box<InterfaceAccount<'info, TokenAccount>>,

    /// The user token b account
    #[account(
        mut,
        seeds = [constant::seeds::TOKEN_VAULT, vault_authority.key().as_ref(), token_b_mint.key().as_ref()],
        bump,
        token::mint = token_b_mint,
        token::authority = vault_authority,
        token::token_program = token_b_program,
    )]
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
    #[account(address = damm_v2_cpi::ID)]
    pub amm_program: UncheckedAccount<'info>,

    /// CHECK:
    pub event_authority: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

impl<'info> ClaimPositionFee<'info> {
    pub fn handle_claim_position_fee(&mut self) -> Result<()> {
        let vault_authority_seeds: &[&[u8]] = &[VAULT_AUTHORITY, &[VAULT_BUMP]];

        let pre_a = self.token_a_account.amount;
        let pre_b = self.token_b_account.amount;

        damm_v2_cpi::cpi::claim_position_fee(CpiContext::new_with_signer(
            self.amm_program.to_account_info(),
            damm_v2_cpi::cpi::accounts::ClaimPositionFeeCtx {
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
            &[&vault_authority_seeds[..]],
        ))?;

        self.token_a_account.reload()?;
        self.token_b_account.reload()?;

        let claimed_token_a = self.token_a_account.amount.saturating_sub(pre_a);
        let claimed_token_b = self.token_b_account.amount.saturating_sub(pre_b);

        // If nothing was claimed, we’re done
        if claimed_token_a == 0 && claimed_token_b == 0 {
            return Ok(());
        }

        #[inline]
        fn split_even(amount: u64) -> (u64, u64) {
            let half = amount / 2;
            let remainder = amount % 2;
            (half + remainder, half)
        }

        let (treasury_a, maker_a) = split_even(claimed_token_a);
        let (treasury_b, maker_b) = split_even(claimed_token_b);

        if treasury_a > 0 {
            anchor_spl::token::transfer(
                CpiContext::new_with_signer(
                    self.token_a_program.to_account_info(),
                    anchor_spl::token::Transfer {
                        from: self.token_a_account.to_account_info(),
                        to: self.wewe_token_account.to_account_info(),
                        authority: self.vault_authority.to_account_info(),
                    },
                    &[&vault_authority_seeds[..]],
                ),
                treasury_a,
            )?;
        }

        if maker_a > 0 {
            anchor_spl::token::transfer(
                CpiContext::new_with_signer(
                    self.token_a_program.to_account_info(),
                    anchor_spl::token::Transfer {
                        from: self.token_a_account.to_account_info(),
                        to: self.maker_token_account.to_account_info(),
                        authority: self.vault_authority.to_account_info(),
                    },
                    &[&vault_authority_seeds[..]],
                ),
                maker_a,
            )?;
        }

        if treasury_b > 0 {
            anchor_spl::token::transfer(
                CpiContext::new_with_signer(
                    self.token_b_program.to_account_info(),
                    anchor_spl::token::Transfer {
                        from: self.token_b_account.to_account_info(),
                        to: self.wewe_wsol_account.to_account_info(),
                        authority: self.vault_authority.to_account_info(),
                    },
                    &[&vault_authority_seeds[..]],
                ),
                treasury_b,
            )?;
        }

        if maker_b > 0 {
            anchor_spl::token::transfer(
                CpiContext::new_with_signer(
                    self.token_b_program.to_account_info(),
                    anchor_spl::token::Transfer {
                        from: self.token_b_account.to_account_info(),
                        to: self.maker_wsol_account.to_account_info(),
                        authority: self.vault_authority.to_account_info(),
                    },
                    &[&vault_authority_seeds[..]],
                ),
                maker_b,
            )?;
        }

        emit!(PositionFeeClaimed {
            proposal: self.proposal.key(),
            maker: self.maker.key(),
            user: self.payer.key(),
            user_token_amount: claimed_token_a,
            user_wsol_amount: claimed_token_b,
            token_mint: self.token_a_mint.key(),
            wsol_mint: self.token_b_mint.key(),
        });

        Ok(())
    }
}
