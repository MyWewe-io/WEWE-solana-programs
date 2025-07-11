use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{Mint, Token, TokenAccount};

#[derive(Accounts)]
pub struct MintSoulboundToUser<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        init_if_needed,
        payer = payer,
        mint::decimals = 0,
        mint::authority = mint_authority,
        mint::freeze_authority = mint_authority,
        seeds = [b"mint_soulbound"],
        bump,
    )]
    pub mint: Account<'info, Mint>,

    /// CHECK: mint authority PDA
    #[account(
        seeds = [b"mint_authority"],
        bump,
    )]
    pub mint_authority: UncheckedAccount<'info>,

    /// CHECK: user
    pub user: UncheckedAccount<'info>,

    #[account(
        init_if_needed,
        payer = payer,
        associated_token::mint = mint,
        associated_token::authority = user,
    )]
    pub user_token_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl<'info> MintSoulboundToUser<'info> {
    pub fn mint_soulbound_to_user(&mut self, bumps: &MintSoulboundToUserBumps) -> Result<()> {
        let mint_authority_seeds: &[&[u8]] = &[b"mint_authority", &[bumps.mint_authority]];

        anchor_spl::token::mint_to(
            CpiContext::new_with_signer(
                self.token_program.to_account_info(),
                anchor_spl::token::MintTo {
                    mint: self.mint.to_account_info(),
                    to: self.user_token_account.to_account_info(),
                    authority: self.mint_authority.to_account_info(),
                },
                &[mint_authority_seeds],
            ),
            1,
        )?;

        // Freeze the user's token account
        anchor_spl::token::freeze_account(CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            anchor_spl::token::FreezeAccount {
                account: self.user_token_account.to_account_info(),
                mint: self.mint.to_account_info(),
                authority: self.mint_authority.to_account_info(),
            },
            &[mint_authority_seeds],
        ))?;

        Ok(())
    }
}
