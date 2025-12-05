use std::ops::Sub;

use crate::{
    const_pda,
    constant::{
        seeds::{MAKER, PROPOSAL, TOKEN_VAULT, VAULT_AUTHORITY},
        ANCHOR_DISCRIMINATOR, MINT_DECIMALS,
    },
    errors::ProposalError,
    event::ProposalCreated,
    state::{maker::MakerAccount, proposal::Proposal,config::Configs},
};
use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};

#[derive(Accounts)]
pub struct CreateProposal<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(mut)]
    pub maker: Signer<'info>,

    #[account(
        init_if_needed,
        payer = payer,
        seeds = [MAKER, maker.key().as_ref()],
        bump,
        space = ANCHOR_DISCRIMINATOR + MakerAccount::INIT_SPACE,
    )]
    pub maker_account: Account<'info, MakerAccount>,

    #[account(
        init,
        payer = payer,
        seeds = [PROPOSAL, maker.key().as_ref(), &maker_account.proposal_count.to_le_bytes()],
        bump,
        space = ANCHOR_DISCRIMINATOR + Proposal::INIT_SPACE,
    )]
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
        init,
        payer = payer,
        mint::decimals = MINT_DECIMALS,
        mint::authority = proposal.key(),
        mint::freeze_authority = proposal.key(),
    )]
    pub mint_account: Account<'info, Mint>,

    #[account(
        init,
        seeds = [TOKEN_VAULT, vault_authority.key().as_ref(), mint_account.key().as_ref()],
        payer = payer,
        token::mint = mint_account,
        token::authority = vault_authority,
        bump,
    )]
    pub token_vault: Account<'info, TokenAccount>,

    #[account(
        mut,
        address = const_pda::const_authority::MINT,
    )]
    pub mint: Account<'info, Mint>,
    #[account(
        associated_token::mint = mint,
        associated_token::authority = maker,
        constraint = user_token_account.amount == 1 @ ProposalError::NotAuthorised
    )]
    pub user_token_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub config: Account<'info, Configs>,
}

impl<'info> CreateProposal<'info> {
    pub fn handle_create_proposal(
        &mut self,
        token_name: String,
        token_symbol: String,
        token_uri: String,
        bumps: &CreateProposalBumps,
    ) -> Result<()> {
        require!(token_name.len() <= 32, ProposalError::LenthTooLong);
        require!(token_symbol.len() <= 10, ProposalError::LenthTooLong);
        require!(token_uri.len() <= 200, ProposalError::LenthTooLong);

        let now = Clock::get()?.unix_timestamp;
        self.proposal.set_inner(Proposal {
            maker: self.maker.key(),
            mint_account: self.mint_account.key(),
            total_backing: 0,
            time_started: now,
            bump: bumps.proposal,
            is_rejected: false,
            proposal_id: self.maker_account.proposal_count,
            is_pool_launched: false,
            total_backers: 0,
            current_airdrop_cycle: 1,
            milestone_active: false,
            milestone_units_assigned: 0,
            milestone_backers_weighted: 0,
            milestone_reputation_sum: 0,
            launch_timestamp: None,
            emergency_unlocked: false,
        });
        // increment proposal count for maker
        let idx = self.maker_account.proposal_count;
        self.maker_account.proposal_count =
            idx.checked_add(1).ok_or(ProposalError::NumericalOverflow)?;

        // Derive metadata account address (even though it doesn't exist yet)
        let mint_key = self.mint_account.key();
        let metadata_seeds: &[&[u8]] = &[
            b"metadata",
            anchor_spl::metadata::ID.as_ref(),
            mint_key.as_ref(),
        ];
        let (metadata_account, _) = Pubkey::find_program_address(metadata_seeds, &anchor_spl::metadata::ID);

        emit!(ProposalCreated {
            maker: self.maker.key(),
            proposal_address: self.proposal.key(),
            proposal_index: self.maker_account.proposal_count.sub(1),
            start_time: now,
            token_name,
            token_symbol,
            token_uri,
            mint_account: self.mint_account.key(),
            token_vault: self.token_vault.key(),
            metadata_account,
            maker_account: self.maker_account.key(),
            proposal_bump: bumps.proposal,
        });

        Ok(())
    }
}
