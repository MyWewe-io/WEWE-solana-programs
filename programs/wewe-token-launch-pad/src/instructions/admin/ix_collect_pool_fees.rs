use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{close_account, CloseAccount, Token},
    token_interface::{TokenAccount, TokenInterface},
};

use crate::{
    const_pda::{self, const_authority::VAULT_BUMP},
    constant::{seeds::VAULT_AUTHORITY, seeds::TOKEN_VAULT, treasury},
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

    /// CHECK:
    pub token_a_mint: UncheckedAccount<'info>,

    /// CHECK:
    pub token_b_mint: UncheckedAccount<'info>,

    /// WSOL account - can be owned by treasury or vault_authority (we'll use as temp account)
    #[account(
        init_if_needed,
        payer = payer,
        associated_token::mint = token_b_mint,
        associated_token::authority = wewe_treasury,
    )]
    pub wewe_wsol_account: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        init_if_needed,
        payer = payer,
        associated_token::mint = token_a_mint,
        associated_token::authority = wewe_treasury,
    )]
    pub wewe_token_account: Box<InterfaceAccount<'info, TokenAccount>>,

    /// WSOL account - can be owned by maker or vault_authority (we'll use as temp account)
    #[account(
        init_if_needed,
        payer = payer,
        associated_token::authority = maker,
        associated_token::mint = token_b_mint,
    )]
    pub maker_wsol_account: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        init_if_needed,
        payer = payer,
        associated_token::mint = token_a_mint,
        associated_token::authority = maker,
    )]
    pub maker_token_account: Box<InterfaceAccount<'info, TokenAccount>>,

    /// CHECK: pool address
    pub pool: UncheckedAccount<'info>,

    /// CHECK: position address
    #[account(mut)]
    pub position: UncheckedAccount<'info>,

    /// The user token a account - vault for token A
    #[account(
        mut,
        seeds = [TOKEN_VAULT, vault_authority.key().as_ref(), token_a_mint.key().as_ref()],
        bump,
    )]
    pub token_a_account: Box<InterfaceAccount<'info, TokenAccount>>,

    /// The user token b account - vault for token B  
    #[account(
        mut,
        seeds = [TOKEN_VAULT, vault_authority.key().as_ref(), token_b_mint.key().as_ref()],
        bump,
    )]
    pub token_b_account: Box<InterfaceAccount<'info, TokenAccount>>,

    /// The vault token account for input token
    #[account(mut)]
    pub token_a_vault: Box<InterfaceAccount<'info, TokenAccount>>,

    /// The vault token account for output token
    #[account(mut)]
    pub token_b_vault: Box<InterfaceAccount<'info, TokenAccount>>,

    /// CHECK:
    pub position_nft_account: UncheckedAccount<'info>,

    /// CHECK: Temporary WSOL account for treasury unwrapping (PDA derived, owned by vault_authority)
    /// PDA: [b"temp_wsol", vault_authority, proposal, b"treasury"]
    #[account(mut)]
    pub treasury_temp_wsol: UncheckedAccount<'info>,

    /// CHECK: Temporary WSOL account for maker unwrapping (PDA derived, owned by vault_authority)
    /// PDA: [b"temp_wsol", vault_authority, proposal, b"maker"]
    #[account(mut)]
    pub maker_temp_wsol: UncheckedAccount<'info>,

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
        // Access control: The maker account is already validated in Accounts struct
        // Anyone can call this function as long as they provide the correct maker account
        // The maker constraint ensures only the correct maker can be specified
        
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

        // Unwrap WSOL (token_b) to SOL before transferring
        // We use PDA-derived temporary accounts that are passed in but validated programmatically
        if treasury_b > 0 {
            // Validate treasury_temp_wsol is the correct PDA
            let token_b_program_key = self.token_b_program.key();
            let (expected_treasury_pda, _treasury_bump) = Pubkey::find_program_address(
                &[
                    b"temp_wsol",
                    self.vault_authority.key().as_ref(),
                    self.proposal.key().as_ref(),
                    b"treasury",
                ],
                &token_b_program_key,
            );
            require!(
                self.treasury_temp_wsol.key() == expected_treasury_pda,
                ProposalError::IncorrectAccount
            );

            // Transfer WSOL to temporary PDA account
            anchor_spl::token::transfer(
                CpiContext::new_with_signer(
                    self.token_b_program.to_account_info(),
                    anchor_spl::token::Transfer {
                        from: self.token_b_account.to_account_info(),
                        to: self.treasury_temp_wsol.to_account_info(),
                        authority: self.vault_authority.to_account_info(),
                    },
                    &[&vault_authority_seeds[..]],
                ),
                treasury_b,
            )?;

            // Close the temporary WSOL account to unwrap it to SOL
            // The SOL (lamports) will be sent to the account owner (vault_authority)
            close_account(
                CpiContext::new_with_signer(
                    self.token_b_program.to_account_info(),
                    CloseAccount {
                        account: self.treasury_temp_wsol.to_account_info(),
                        destination: self.vault_authority.to_account_info(),
                        authority: self.vault_authority.to_account_info(),
                    },
                    &[&vault_authority_seeds[..]],
                ),
            )?;

            // Transfer SOL from vault_authority to treasury
            anchor_lang::system_program::transfer(
                CpiContext::new_with_signer(
                    self.system_program.to_account_info(),
                    anchor_lang::system_program::Transfer {
                        from: self.vault_authority.to_account_info(),
                        to: self.wewe_treasury.to_account_info(),
                    },
                    &[&vault_authority_seeds[..]],
                ),
                treasury_b,
            )?;
        }

        if maker_b > 0 {
            // Validate maker_temp_wsol is the correct PDA
            let token_b_program_key = self.token_b_program.key();
            let (expected_maker_pda, _maker_bump) = Pubkey::find_program_address(
                &[
                    b"temp_wsol",
                    self.vault_authority.key().as_ref(),
                    self.proposal.key().as_ref(),
                    b"maker",
                ],
                &token_b_program_key,
            );
            require!(
                self.maker_temp_wsol.key() == expected_maker_pda,
                ProposalError::IncorrectAccount
            );

            // Transfer WSOL to temporary PDA account
            anchor_spl::token::transfer(
                CpiContext::new_with_signer(
                    self.token_b_program.to_account_info(),
                    anchor_spl::token::Transfer {
                        from: self.token_b_account.to_account_info(),
                        to: self.maker_temp_wsol.to_account_info(),
                        authority: self.vault_authority.to_account_info(),
                    },
                    &[&vault_authority_seeds[..]],
                ),
                maker_b,
            )?;

            // Close the temporary WSOL account to unwrap it to SOL
            close_account(
                CpiContext::new_with_signer(
                    self.token_b_program.to_account_info(),
                    CloseAccount {
                        account: self.maker_temp_wsol.to_account_info(),
                        destination: self.vault_authority.to_account_info(),
                        authority: self.vault_authority.to_account_info(),
                    },
                    &[&vault_authority_seeds[..]],
                ),
            )?;

            // Transfer SOL from vault_authority to maker
            anchor_lang::system_program::transfer(
                CpiContext::new_with_signer(
                    self.system_program.to_account_info(),
                    anchor_lang::system_program::Transfer {
                        from: self.vault_authority.to_account_info(),
                        to: self.maker.to_account_info(),
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
