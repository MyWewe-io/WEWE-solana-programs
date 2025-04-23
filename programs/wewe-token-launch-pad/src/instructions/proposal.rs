use anchor_lang::prelude::*;

use crate::{event::ProposalCreated, state::proposer::Proposal};
use {
    crate::constant::{ANCHOR_DISCRIMINATOR, TOTAL_MINT},
    anchor_spl::{
        associated_token::AssociatedToken,
        metadata::{
            create_metadata_accounts_v3, mpl_token_metadata::types::DataV2,
            CreateMetadataAccountsV3, Metadata,
        },
        token::{mint_to, Mint, MintTo, Token, TokenAccount},
    },
};

#[derive(Accounts)]
#[instruction(_token_decimals: u8)]
pub struct CreateProposal<'info> {
    #[account(mut)]
    pub maker: Signer<'info>, // Creator of the proposal
    #[account(
        init,
        payer = maker,
        seeds = [b"proposer", maker.key().as_ref()],
        bump,
        space = ANCHOR_DISCRIMINATOR + Proposal::INIT_SPACE,
    )]
    pub proposal: Account<'info, Proposal>,

    #[account(
        init,
        payer = maker,
        seeds = [b"mint", maker.key().as_ref()],
        bump,
        mint::decimals = _token_decimals,
        mint::authority = mint_account.key(),
        mint::freeze_authority = mint_account.key(),
    )]
    pub mint_account: Account<'info, Mint>,

    /// CHECK: Validate address by deriving pda
    #[account(
        mut,
        seeds = [b"metadata", token_metadata_program.key().as_ref(), mint_account.key().as_ref()],
        bump,
        seeds::program = token_metadata_program.key(),
    )]
    pub metadata_account: UncheckedAccount<'info>,

    #[account(
        init_if_needed,
        payer = maker,
        associated_token::mint = mint_account,
        associated_token::authority = proposal,
    )]
    pub token_vault: Account<'info, TokenAccount>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,
    pub token_metadata_program: Program<'info, Metadata>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

impl<'info> CreateProposal<'info> {
    pub fn create_proposal(
        &mut self,
        _token_decimals: u8,
        backing_goal: u64,
        token_name: String,
        token_symbol: String,
        token_uri: String,
        duration: u16,
        bumps: &CreateProposalBumps,
    ) -> Result<()> {
        // PDA signer seeds
        let signer_seeds: &[&[&[u8]]] = &[&[b"mint", self.maker.key.as_ref(), &[bumps.mint_account]]];

        create_metadata_accounts_v3(
            CpiContext::new(
                self.token_metadata_program.to_account_info(),
                CreateMetadataAccountsV3 {
                    metadata: self.metadata_account.to_account_info(),
                    mint: self.mint_account.to_account_info(),
                    mint_authority: self.mint_account.to_account_info(),
                    update_authority: self.maker.to_account_info(),
                    payer: self.maker.to_account_info(),
                    system_program: self.system_program.to_account_info(),
                    rent: self.rent.to_account_info(),
                },
            )
            .with_signer(signer_seeds),
            DataV2 {
                name: token_name,
                symbol: token_symbol,
                uri: token_uri,
                seller_fee_basis_points: 0,
                creators: None,
                collection: None,
                uses: None,
            },
            false, // Is mutable
            true,  // Update authority is signer
            None,  // Collection details
        )?;
        
        // Invoke the mint_to instruction on the token program
        mint_to(
            CpiContext::new(
                self.token_program.to_account_info(),
                MintTo {
                    mint: self.mint_account.to_account_info(),
                    to: self.token_vault.to_account_info(),
                    authority: self.mint_account.to_account_info(),
                },
            )
            .with_signer(signer_seeds), // using PDA to sign,
            TOTAL_MINT * 10u64.pow(self.mint_account.decimals as u32), // Mint tokens
        )?;
       
        self.proposal.set_inner(Proposal {
            maker: self.maker.key(),
            current_amount: 0,
            time_started: Clock::get()?.unix_timestamp,
            duration,
            bump: bumps.proposal,
            backing_goal,
            is_rejected: false,
        });

        emit!(ProposalCreated {
            maker: self.maker.key(),
            proposal_address: self.proposal.key(),
            start_time: Clock::get()?.unix_timestamp,
            duration,
        });

        Ok(())
    }
}
