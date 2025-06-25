use std::ops::Sub;

use anchor_lang::prelude::*;
use anchor_spl::token::{self, SyncNative, Token, TokenAccount};

use crate::constant::{AMOUNT_TO_RAISE_PER_USER, FEE_TO_DEDUCT, SECONDS_TO_DAYS};
use crate::event::BackerRefunded;
use crate::errors::ProposalError;
use crate::state::{backers::Backers, proposer::Proposal};

#[derive(Accounts)]
pub struct Refund<'info> {
    #[account(mut)]
    pub backer: Signer<'info>,

    #[account(mut)]
    pub proposal: Account<'info, Proposal>,

    #[account(
        mut,
        seeds = [b"backer", proposal.key().as_ref(), backer.key().as_ref()],
        bump,
        close = backer,
    )]
    pub backer_account: Account<'info, Backers>,

    #[account(
        mut,
        associated_token::mint = wsol_mint,
        associated_token::authority = proposal,
    )]
    pub wsol_vault: Account<'info, TokenAccount>,

    pub wsol_mint: Account<'info, token::Mint>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

impl<'info> Refund<'info> {
    pub fn refund(&mut self) -> Result<()> {
        if !self.proposal.is_rejected {
            let current_time = Clock::get()?.unix_timestamp;
            require!(
                self.proposal.duration
                    <= ((current_time - self.proposal.time_started) / SECONDS_TO_DAYS) as u16,
                ProposalError::BackingNotEnded
            );
        }
    
        let refund_amount = AMOUNT_TO_RAISE_PER_USER.sub(FEE_TO_DEDUCT);
    
        // Sync WSOL vault to reflect actual lamports
        let signer_seeds: &[&[&[u8]]] = &[&[
            b"proposal",
            self.proposal.maker.as_ref(),
            &self.proposal.proposal_id.to_le_bytes(),
            &[self.proposal.bump],
        ]];
    
        token::sync_native(
            CpiContext::new(
                self.token_program.to_account_info(),
                token::SyncNative {
                    account: self.wsol_vault.to_account_info(),
                },
            ).with_signer(signer_seeds),
        )?;
    
        // Transfer SOL from proposal to backer
        anchor_lang::system_program::transfer(
            CpiContext::new_with_signer(
                self.system_program.to_account_info(),
                anchor_lang::system_program::Transfer {
                    from: self.proposal.to_account_info(),
                    to: self.backer.to_account_info(),
                },
                signer_seeds
            ),
            refund_amount,
        )?;
        
        self.proposal.current_amount -= refund_amount;
    
        emit!(BackerRefunded {
            backer: self.backer.key(),
            amount: refund_amount,
        });
    
        Ok(())
    }    
}
