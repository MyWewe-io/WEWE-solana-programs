use crate::{
    constant::seeds::PROPOSAL,
    errors::ProposalError,
    event::MilestoneStarted,
    state::proposal::Proposal,
};
use anchor_lang::prelude::*;
use anchor_spl::{
    metadata::{
        create_metadata_accounts_v3, mpl_token_metadata::types::DataV2, CreateMetadataAccountsV3,
        Metadata,
    },
    token::{set_authority, Mint, SetAuthority, Token},
};
use anchor_spl::token::spl_token::instruction::AuthorityType;

#[derive(Accounts)]
pub struct InitialiseMilestone<'info> {
    pub authority: Signer<'info>,
    #[account(mut)]
    pub proposal: Account<'info, Proposal>,
    /// CHECK: Mint account from proposal
    #[account(
        mut,
        address = proposal.mint_account @ ProposalError::IncorrectAccount
    )]
    pub mint_account: Account<'info, Mint>,
    /// CHECK: Metadata PDA derived from mint
    #[account(
        mut,
        seeds = [b"metadata", token_metadata_program.key().as_ref(), mint_account.key().as_ref()],
        bump,
        seeds::program = token_metadata_program.key(),
    )]
    pub metadata_account: UncheckedAccount<'info>,
    #[account(mut)]
    pub payer: Signer<'info>,
    pub token_metadata_program: Program<'info, Metadata>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

impl<'info> InitialiseMilestone<'info> {
    pub fn handle_initialise_milestone(&mut self) -> Result<()> {
        require!(!self.proposal.is_rejected, ProposalError::ProposalRejected);
        require!(self.proposal.is_pool_launched, ProposalError::TargetNotMet);
        require!(
            !self.proposal.milestone_active,
            ProposalError::NoMilestoneActive
        );

        let proposal_signer_seeds: &[&[&[u8]]] = &[&[
            PROPOSAL,
            self.proposal.maker.as_ref(),
            &self.proposal.proposal_id.to_le_bytes(),
            &[self.proposal.bump],
        ]];

        // Create metadata account if it doesn't exist
        // IMPORTANT: Metadata must be created BEFORE mint authority is revoked
        // Once mint authority is None, Anchor's CPI wrapper will fail due to signer privilege checks
        if self.metadata_account.data_is_empty() {
            // Only create metadata if mint authority still exists
            require!(
                self.mint_account.mint_authority.is_some(),
                ProposalError::TargetNotMet
            );

            create_metadata_accounts_v3(
            CpiContext::new(
                self.token_metadata_program.to_account_info(),
                CreateMetadataAccountsV3 {
                    metadata: self.metadata_account.to_account_info(),
                    mint: self.mint_account.to_account_info(),
                    mint_authority: self.proposal.to_account_info(),
                    update_authority: self.proposal.to_account_info(),
                    payer: self.payer.to_account_info(),
                    system_program: self.system_program.to_account_info(),
                    rent: self.rent.to_account_info(),
                },
            )
            .with_signer(proposal_signer_seeds),
            DataV2 {
                name: self.proposal.token_name.clone(),
                symbol: self.proposal.token_symbol.clone(),
                uri: self.proposal.token_uri.clone(),
                seller_fee_basis_points: 0,
                creators: None,
                collection: None,
                uses: None,
            },
                false, // Is mutable - set to false to make metadata immutable
                true,  // Update authority is signer - true because proposal PDA is signing
                None,  // Collection details
            )?;
        }

        // Revoke mint and freeze authority AFTER creating metadata
        // This ensures metadata is created while mint authority still exists
        // DexScreener checks the mint account's mint_authority field - if None, token shows as non-mintable
        if self.mint_account.mint_authority.is_some() {
            // Revoke mint authority to make token non-mintable
            set_authority(
                CpiContext::new(
                    self.token_program.to_account_info(),
                    SetAuthority {
                        current_authority: self.proposal.to_account_info(),
                        account_or_mint: self.mint_account.to_account_info(),
                    },
                )
                .with_signer(proposal_signer_seeds),
                AuthorityType::MintTokens,
                None, // None = revoke authority
            )?;

            // Revoke freeze authority to make token non-freezable
            set_authority(
                CpiContext::new(
                    self.token_program.to_account_info(),
                    SetAuthority {
                        current_authority: self.proposal.to_account_info(),
                        account_or_mint: self.mint_account.to_account_info(),
                    },
                )
                .with_signer(proposal_signer_seeds),
                AuthorityType::FreezeAccount,
                None, // None = revoke authority
            )?;
        }

        self.proposal.milestone_active = true;
        self.proposal.milestone_backers_weighted = 0;
        self.proposal.milestone_reputation_sum = 0;

        emit!(MilestoneStarted {
            proposal: self.proposal.key(),
            token_mint: self.proposal.mint_account.key(),
            cycle: self.proposal.current_airdrop_cycle,
        });
        Ok(())
    }
}
