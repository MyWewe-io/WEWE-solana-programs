use anchor_lang::prelude::*;
use anchor_spl::{
    metadata::Metadata,
    token::Mint,
};

use crate::{
    constant::seeds::PROPOSAL,
    errors::ProposalError,
    event::ProposalRejected,
    state::proposal::Proposal,
};

#[derive(Accounts)]
pub struct RejectProposal<'info> {
    pub authority: Signer<'info>,
    #[account(mut)]
    pub proposal: Account<'info, Proposal>,
    /// CHECK: Mint account from proposal
    #[account(
        address = proposal.mint_account @ ProposalError::IncorrectAccount
    )]
    pub mint_account: Account<'info, Mint>,
    /// CHECK: Metadata PDA derived from mint - manually validated to avoid account resolution depth issues
    #[account(mut)]
    pub metadata_account: UncheckedAccount<'info>,
    /// CHECK: Payer account to receive rent from closing metadata
    #[account(mut)]
    pub payer: Signer<'info>,
    pub token_metadata_program: Program<'info, Metadata>,
}

impl<'info> RejectProposal<'info> {
    pub fn handle_reject_proposal(&mut self) -> Result<()> {
        require!(
            !self.proposal.is_pool_launched,
            ProposalError::PoolAlreadyLaunched
        );
        require!(!self.proposal.is_rejected, ProposalError::ProposalRejected);
        
        // Validate metadata account address matches expected PDA
        let token_metadata_program_key = self.token_metadata_program.key();
        let expected_metadata_pda = anchor_lang::solana_program::pubkey::Pubkey::find_program_address(
            &[
                b"metadata",
                token_metadata_program_key.as_ref(),
                self.mint_account.key().as_ref(),
            ],
            &token_metadata_program_key,
        ).0;
        require!(
            self.metadata_account.key() == expected_metadata_pda,
            ProposalError::IncorrectAccount
        );
        
        self.proposal.is_rejected = true;

        // Close metadata account if it exists (metadata is mutable at proposal creation)
        // CloseMetadataAccountV1 instruction:
        // - Discriminator: 42 (u8)
        // - Accounts: metadata (writable), owner (signer), payer (signer, writable)
        if !self.metadata_account.data_is_empty() {
            let proposal_id_bytes = self.proposal.proposal_id.to_le_bytes();
            let seeds: &[&[u8]] = &[
                PROPOSAL,
                self.proposal.maker.as_ref(),
                &proposal_id_bytes,
                &[self.proposal.bump],
            ];
            let signer_seeds: &[&[&[u8]]] = &[seeds];

            // Construct CloseMetadataAccountV1 instruction
            // Discriminator: 42 (CloseMetadataAccountV1)
            let instruction_data = vec![42u8];
            
            // Accounts in order for CloseMetadataAccountV1:
            // 1. metadata (writable)
            // 2. owner (signer) - update authority  
            // 3. payer (signer, writable) - receives rent
            let accounts = vec![
                anchor_lang::solana_program::instruction::AccountMeta::new(
                    self.metadata_account.key(),
                    false, // not signer
                ),
                anchor_lang::solana_program::instruction::AccountMeta::new_readonly(
                    self.proposal.key(),
                    true, // signer (owner/update authority)
                ),
                anchor_lang::solana_program::instruction::AccountMeta::new(
                    self.payer.key(),
                    true, // signer and writable (receives rent)
                ),
            ];

            let instruction = anchor_lang::solana_program::instruction::Instruction {
                program_id: self.token_metadata_program.key(),
                accounts,
                data: instruction_data,
            };

            anchor_lang::solana_program::program::invoke_signed(
                &instruction,
                &[
                    self.metadata_account.to_account_info(),
                    self.proposal.to_account_info(),
                    self.payer.to_account_info(),
                    self.token_metadata_program.to_account_info(),
                ],
                signer_seeds,
            )?;
        }

        emit!(ProposalRejected {
            maker: self.proposal.maker,
            proposal_address: self.proposal.key(),
            mint_account: self.proposal.mint_account.key(),
        });

        Ok(())
    }
}
