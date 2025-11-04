use crate::{
    constant::{admin_pubkey, seeds::PROPOSAL, treasury},
    errors::ProposalError,
    state::{proposal::Proposal, config::Configs},
};
use anchor_lang::prelude::*;
use anchor_spl::token_interface::TokenInterface;
use std::str::FromStr;

#[derive(Accounts)]
pub struct UpdateTransferFee<'info> {
    /// CHECK: Authority (must be admin or proposal maker)
    #[account(mut)]
    pub authority: Signer<'info>,

    /// CHECK: Proposal PDA (transfer fee config authority)
    #[account(
        mut,
        seeds = [PROPOSAL, proposal.maker.as_ref(), &proposal.proposal_id.to_le_bytes()],
        bump = proposal.bump,
    )]
    pub proposal: Account<'info, Proposal>,

    /// CHECK: Mint account (Token-2022)
    #[account(
        mut,
        constraint = mint.key() == proposal.mint_account @ ProposalError::IncorrectAccount
    )]
    pub mint: UncheckedAccount<'info>,

    /// CHECK: WEWE treasury account (receives transfer fees)
    #[account(address = treasury::ID)]
    pub wewe_treasury: UncheckedAccount<'info>,

    /// CHECK: Token-2022 program
    pub token_program: Interface<'info, TokenInterface>,

    pub config: Account<'info, Configs>,
}

impl<'info> UpdateTransferFee<'info> {
    pub fn handle_update_transfer_fee(&self) -> Result<()> {
        // Verify authority is either admin or proposal maker
        require!(
            self.authority.key() == admin_pubkey::ID || self.authority.key() == self.proposal.maker,
            ProposalError::NotOwner
        );

        let signer_seeds: &[&[&[u8]]] = &[&[
            PROPOSAL,
            self.proposal.maker.as_ref(),
            &self.proposal.proposal_id.to_le_bytes(),
            &[self.proposal.bump],
        ]];

        // Update transfer fee config using raw instruction
        // Instruction discriminator: 40 (for update_transfer_fee_config)
        let token_2022_program_id = anchor_lang::solana_program::pubkey::Pubkey::from_str(
            "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb"
        ).unwrap();
        
        let mut instruction_data = Vec::new();
        instruction_data.push(40u8); // Instruction discriminator for update_transfer_fee_config
        
        // Older transfer fee (epoch, maximum_fee)
        instruction_data.extend_from_slice(&0u64.to_le_bytes()); // epoch
        instruction_data.extend_from_slice(&self.config.max_fee.to_le_bytes()); // maximum_fee
        
        // Newer transfer fee (epoch, maximum_fee)
        instruction_data.extend_from_slice(&0u64.to_le_bytes()); // epoch
        instruction_data.extend_from_slice(&self.config.max_fee.to_le_bytes()); // maximum_fee
        
        // Withdraw withheld authority (treasury)
        instruction_data.extend_from_slice(self.wewe_treasury.key().as_ref());
        
        // Build account metas
        let mut account_infos = Vec::new();
        account_infos.push(anchor_lang::solana_program::instruction::AccountMeta::new(
            self.mint.key(),
            false,
        ));
        account_infos.push(anchor_lang::solana_program::instruction::AccountMeta::new_readonly(
            self.proposal.key(),
            true, // is_signer (PDA will sign)
        ));
        
        let update_fee_ix = anchor_lang::solana_program::instruction::Instruction {
            program_id: token_2022_program_id,
            accounts: account_infos,
            data: instruction_data,
        };
        
        // Invoke update_transfer_fee_config
        anchor_lang::solana_program::program::invoke_signed(
            &update_fee_ix,
            &[
                self.mint.to_account_info(),
                self.proposal.to_account_info(),
                self.token_program.to_account_info(),
            ],
            signer_seeds,
        )?;

        Ok(())
    }
}

