use anchor_spl::{
    token_2022::{
        initialize_mint2,
        mint_to,
        spl_token_2022::{extension::ExtensionType, state::Mint as SplMint},
        InitializeMint2, MintTo,
    },
    
    token_interface::{non_transferable_mint_initialize, NonTransferableMintInitialize, Token2022},
};
use anchor_lang::prelude::*;
use anchor_lang::solana_program::program::invoke_signed;
use anchor_lang::solana_program::system_instruction;
use anchor_spl::associated_token::AssociatedToken;

use crate::constant::seeds::{MINT_ACCOUNT, MINT_AUTHORITY};
use crate::errors::ProposalError;

#[derive(Accounts)]
pub struct MintSoulboundToUser<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(mut)]
    pub mint: Account<'info, anchor_spl::token::Mint>,
    
    #[account(
        init_if_needed,
        payer = payer,
        associated_token::mint = mint,
        associated_token::authority = user,
        associated_token::token_program = token_2022_program,
    )]
    pub user_token_account: Account<'info, anchor_spl::token::TokenAccount>,
    
    pub token_2022_program: Program<'info, Token2022>,

    /// CHECK: mint authority PDA
    #[account(
        seeds = [MINT_AUTHORITY],
        bump,
    )]
    pub mint_authority: UncheckedAccount<'info>,

    /// CHECK: user
    pub user: UncheckedAccount<'info>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl<'info> MintSoulboundToUser<'info> {
    pub fn handle_mint_soulbound_to_user(
        &mut self,
        bumps: &MintSoulboundToUserBumps,
    ) -> Result<()> {
        require!(
            self.user_token_account.amount == 0,
            ProposalError::ProposalAlreadyBacked
        );
        let mint_authority_seeds: &[&[u8]] = &[MINT_AUTHORITY, &[bumps.mint_authority]];

        // Allocate mint with NonTransferable extension if not created yet (only when using PDA)
        let mint_len = ExtensionType::try_calculate_account_len::<SplMint>(&[ExtensionType::NonTransferable])
            .unwrap();

        let (pda, pda_bump) = Pubkey::find_program_address(&[MINT_ACCOUNT], &crate::ID);
        if self.mint.key() == pda && self.mint.to_account_info().data_is_empty() {
            let lamports = Rent::get()?.minimum_balance(mint_len);
            let ix = system_instruction::create_account(
                &self.payer.key(),
                &self.mint.key(),
                lamports,
                mint_len as u64,
                &self.token_2022_program.key(),
            );
            invoke_signed(
                &ix,
                &[
                    self.payer.to_account_info(),
                    self.mint.to_account_info(),
                    self.system_program.to_account_info(),
                ],
                &[&[MINT_ACCOUNT, &[pda_bump]]],
            )?;
        }

        initialize_mint2(
            CpiContext::new(
                self.token_2022_program.to_account_info(),
                InitializeMint2 { mint: self.mint.to_account_info() },
            ),
            0,                                // decimals
            &self.mint_authority.key(),       // mint authority
            None,                             // freeze authority
        )?;

        non_transferable_mint_initialize(
            CpiContext::new(
                self.token_2022_program.to_account_info(),
                NonTransferableMintInitialize { 
                    token_program_id: self.token_2022_program.to_account_info(),
                    mint: self.mint.to_account_info(),
                },
            )
        )?;

        mint_to(
            CpiContext::new_with_signer(
                self.token_2022_program.to_account_info(),
                MintTo {
                    mint: self.mint.to_account_info(),
                    to: self.user_token_account.to_account_info(),
                    authority: self.mint_authority.to_account_info(),
                },
                &[mint_authority_seeds],
            ),
            1,
        )?;

        Ok(())
    }
}