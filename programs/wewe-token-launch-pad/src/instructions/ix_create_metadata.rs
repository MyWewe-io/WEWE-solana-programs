use crate::{
    constant::seeds::PROPOSAL,
    errors::ProposalError,
    state::proposal::Proposal,
};
use anchor_lang::prelude::*;
use anchor_spl::{
    metadata::{
        create_metadata_accounts_v3, mpl_token_metadata::types::DataV2, CreateMetadataAccountsV3,
    },
    token::Mint,
};

/// Creates token metadata account for a proposal.
/// 
/// This instruction can be called independently or together with `create_pool` 
/// in the same transaction. When called together, add this instruction FIRST, 
/// then add `create_pool` instruction to the same transaction.
/// 
/// Example (TypeScript):
/// ```typescript
/// const tx = new Transaction();
/// 
/// // Add metadata creation first
/// const createMetadataIx = await program.methods
///   .createMetadata()
///   .accounts({ ... })
///   .instruction();
/// tx.add(createMetadataIx);
/// 
/// // Add pool creation second
/// const createPoolIx = await program.methods
///   .createPool(sqrtPrice)
///   .accountsPartial({ ... })
///   .instruction();
/// tx.add(createPoolIx);
/// 
/// await provider.sendAndConfirm(tx, [signers]);
/// ```
#[derive(Accounts)]
pub struct CreateMetadata<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        constraint = proposal.mint_account == mint_account.key() @ ProposalError::IncorrectAccount
    )]
    pub proposal: Account<'info, Proposal>,

    #[account(mut)]
    pub mint_account: Account<'info, Mint>,

    /// CHECK: Validate address by deriving pda
    #[account(
        mut,
        seeds = [b"metadata", token_metadata_program.key().as_ref(), mint_account.key().as_ref()],
        bump,
        seeds::program = token_metadata_program.key(),
    )]
    pub metadata_account: UncheckedAccount<'info>,

    /// CHECK: Token metadata program
    pub token_metadata_program: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,

    /// CHECK: Rent sysvar
    pub rent: UncheckedAccount<'info>,
}

impl<'info> CreateMetadata<'info> {
    pub fn handle_create_metadata(
        &mut self,
        token_name: String,
        token_symbol: String,
        token_uri: String,
        _bumps: &CreateMetadataBumps,
    ) -> Result<()> {
        require!(!token_name.is_empty(), ProposalError::TokenNameEmpty);
        require!(!token_symbol.is_empty(), ProposalError::TokenSymbolEmpty);
        require!(!token_uri.is_empty(), ProposalError::TokenUriEmpty);
        require!(token_name.len() <= 32, ProposalError::LenthTooLong);
        require!(token_symbol.len() <= 10, ProposalError::LenthTooLong);
        require!(token_uri.len() <= 200, ProposalError::LenthTooLong);
        // Verify token_metadata_program is correct
        require!(
            self.token_metadata_program.key() == anchor_spl::metadata::ID,
            ProposalError::IncorrectAccount
        );

        // Verify rent sysvar is correct
        require!(
            self.rent.key() == anchor_lang::solana_program::sysvar::rent::ID,
            ProposalError::IncorrectAccount
        );

        // Check if metadata account already exists
        require!(
            self.metadata_account.lamports() == 0 || self.metadata_account.data_len() == 0,
            ProposalError::IncorrectAccount
        );

        // PDA signer seeds for proposal (mint authority)
        let proposal_signer_seeds: &[&[&[u8]]] = &[&[
            PROPOSAL,
            self.proposal.maker.as_ref(),
            &self.proposal.proposal_id.to_le_bytes(),
            &[self.proposal.bump],
        ]];

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

        Ok(())
    }
}

