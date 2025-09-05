use anchor_lang::prelude::*;

use std::ops::Sub;
use crate::{
    const_pda,
    constant::{
        seeds::{MAKER, PROPOSAL, TOKEN_VAULT, VAULT_AUTHORITY},
        ANCHOR_DISCRIMINATOR, TOTAL_MINT,
    },
    errors::ProposalError,
    event::ProposalCreated,
    state::{maker::MakerAccount, proposal::Proposal},
};
use anchor_lang::system_program::{transfer, Transfer};
use anchor_spl::{token_2022::spl_token_2022::{self, extension::ExtensionType}, token_interface::{
    token_metadata_initialize, Mint, Token2022, TokenAccount, TokenMetadataInitialize, MintTo, mint_to
}};
use spl_token_metadata_interface::state::TokenMetadata;
use spl_type_length_value::variable_len_pack::VariableLenPack;
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
        seeds = [
            VAULT_AUTHORITY.as_ref(),
        ],
        bump,
    )]
    pub vault_authority: SystemAccount<'info>,

    #[account(
        init,
        payer = payer,
        mint::decimals = 9,
        mint::authority = proposal.key(),
        extensions::metadata_pointer::authority = payer,
        extensions::metadata_pointer::metadata_address = mint_account,
    )]
    pub mint_account: InterfaceAccount<'info, Mint>,

    #[account(
        init,
        seeds = [TOKEN_VAULT, vault_authority.key().as_ref(), mint_account.key().as_ref()],
        payer = payer,
        token::mint = mint_account,
        token::authority = vault_authority,
        bump,
    )]
    pub token_vault: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        address = const_pda::const_authority::MINT,
    )]
    pub mint: InterfaceAccount<'info, Mint>,
    #[account(
        associated_token::mint = mint,
        associated_token::authority = maker,
        constraint = user_token_account.amount == 1 @ ProposalError::NotAuthorised
    )]
    pub user_token_account: InterfaceAccount<'info, TokenAccount>,

    pub token_program: Program<'info, Token2022>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

impl<'info> CreateProposal<'info> {
    pub fn create_proposal(
        &mut self,
        token_name: String,
        token_symbol: String,
        token_uri: String,
        bumps: &CreateProposalBumps,
    ) -> Result<()> {
        // PDA signer seeds for proposal
        let signer_seeds: &[&[&[u8]]] = &[&[
            PROPOSAL,
            self.maker.key.as_ref(),
            &self.maker_account.proposal_count.to_le_bytes(),
            &[bumps.proposal],
        ]];

        require!(token_name.len() <= 32, ProposalError::LenthTooLong);
        require!(token_symbol.len() <= 10, ProposalError::LenthTooLong);
        require!(token_uri.len() <= 200, ProposalError::LenthTooLong);

        let token_metadata = TokenMetadata {
            name: token_name.clone(),
            symbol: token_symbol.clone(),
            uri: token_uri.clone(),
            ..Default::default()
        };

        let base_mint_space = ExtensionType::try_calculate_account_len::<spl_token_2022::state::Mint>(
            &[ExtensionType::MetadataPointer],
        )?;
        let meta_len = token_metadata.get_packed_len()?;
        let total_space = base_mint_space + 4 + meta_len; // 4 for TLV header
        let lamports = (Rent::get()?).minimum_balance(total_space);

        transfer(
            CpiContext::new(
                self.system_program.to_account_info(),
                Transfer {
                    from: self.payer.to_account_info(),
                    to: self.mint_account.to_account_info(),
                },
            ),
            lamports,
        )?;

        // Initialize token metadata
        token_metadata_initialize(
            CpiContext::new_with_signer(
                self.token_program.to_account_info(),
                TokenMetadataInitialize {
                    mint: self.mint_account.to_account_info(),
                    program_id: self.token_program.to_account_info(),
                    mint_authority: self.proposal.to_account_info(),
                    update_authority: self.proposal.to_account_info(),
                    metadata: self.mint_account.to_account_info(),
                },
                signer_seeds,
            ),
            token_name.clone(),
            token_symbol.clone(),
            token_uri.clone(),
        )?;

        let pow = 10u64
            .checked_pow(self.mint_account.decimals as u32)
            .ok_or(ProposalError::NumericalOverflow)?;
        let amount = TOTAL_MINT
            .checked_mul(pow)
            .ok_or(ProposalError::NumericalOverflow)?;

        // Mint tokens to token_vault
        mint_to(
            CpiContext::new_with_signer(
                self.token_program.to_account_info(),
                MintTo {
                    mint: self.mint_account.to_account_info(),
                    to: self.token_vault.to_account_info(),
                    authority: self.proposal.to_account_info(),
                },
                signer_seeds,
            ),
            amount,
        )?;

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
            current_airdrop_cycle: 0,
            milestone_active: false,
            milestone_units_assigned: 0,
            milestone_backers_weighted: 0,
        });

        // Increment proposal count
        let idx = self.maker_account.proposal_count;
        self.maker_account.proposal_count =
            idx.checked_add(1).ok_or(ProposalError::NumericalOverflow)?;

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
            maker_account: self.maker_account.key(),
            proposal_bump: bumps.proposal,
        });

        Ok(())
    }
}