use anchor_lang::prelude::*;
use anchor_spl::{
    metadata::{
        create_metadata_accounts_v3, mpl_token_metadata::types::DataV2, CreateMetadataAccountsV3,
        Metadata,
    },
    token::{mint_to, Mint, MintTo, Token, TokenAccount},
};
use crate::{
    const_pda,
    constant::{seeds::{PROPOSAL, TOKEN_VAULT, VAULT_AUTHORITY, MAKER}, ANCHOR_DISCRIMINATOR, TOTAL_MINT},
    errors::ProposalError,
    event::ProposalCreated,
    state::{maker::MakerAccount, proposal::Proposal},
};

#[derive(Accounts)]
pub struct CreateProposal<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    /// CHECK: maker of the proposal
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
    pub vault_authority: UncheckedAccount<'info>,

    #[account(
        init,
        payer = payer,
        mint::decimals = 9,
        mint::authority = proposal.key(),
        mint::freeze_authority = proposal.key(),
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
    pub token_metadata_program: Program<'info, Metadata>,
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
        // PDA signer seeds
        let signer_seeds: &[&[&[u8]]] = &[&[
            b"proposal",
            self.maker.key.as_ref(),
            &self.maker_account.proposal_count.to_le_bytes(),
            &[bumps.proposal],
        ]];

        create_metadata_accounts_v3(
            CpiContext::new(
                self.token_metadata_program.to_account_info(),
                CreateMetadataAccountsV3 {
                    metadata: self.metadata_account.to_account_info(),
                    mint: self.mint_account.to_account_info(),
                    mint_authority: self.proposal.to_account_info(),
                    update_authority: self.maker.to_account_info(),
                    payer: self.payer.to_account_info(),
                    system_program: self.system_program.to_account_info(),
                    rent: self.rent.to_account_info(),
                },
            )
            .with_signer(signer_seeds),
            DataV2 {
                name: token_name.clone(),
                symbol: token_symbol.clone(),
                uri: token_uri.clone(),
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
                    authority: self.proposal.to_account_info(),
                },
            )
            .with_signer(signer_seeds), // using PDA to sign,
            TOTAL_MINT * 10u64.pow(self.mint_account.decimals as u32), // Mint tokens
        )?;

        self.proposal.set_inner(Proposal {
            maker: self.maker.key(),
            total_backing: 0,
            time_started: Clock::get()?.unix_timestamp,
            bump: bumps.proposal,
            is_rejected: false,
            proposal_id: self.maker_account.proposal_count,
            is_pool_launched: false,
            total_backers: 0,
        });
        // increment proposal count for maker
        self.maker_account.proposal_count += 1;

        emit!(ProposalCreated {
            maker: self.maker.key(),
            proposal_address: self.proposal.key(),
            start_time: Clock::get()?.unix_timestamp,
            token_name,
            token_symbol,
            token_uri,
            mint_account: self.mint_account.key(),
            proposal_index: self.maker_account.proposal_count,
        });

        Ok(())
    }
}
