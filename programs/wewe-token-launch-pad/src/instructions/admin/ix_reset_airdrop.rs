use crate::{
    constant::seeds::BACKER,
    state::{backers::Backers, proposal::Proposal},
};
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct ResetAirdrop<'info> {
    pub authority: Signer<'info>,
    
    #[account(mut)]
    pub proposal: Account<'info, Proposal>,
    
    /// CHECK: backer account
    pub backer: AccountInfo<'info>,
    
    #[account(
        mut,
        seeds = [BACKER, proposal.key().as_ref(), backer.key().as_ref()],
        bump,
    )]
    pub backer_account: Account<'info, Backers>,
}

impl<'info> ResetAirdrop<'info> {
    pub fn handle_reset_airdrop(&mut self) -> Result<()> {
        // Reset the initial_airdrop_received flag to allow retrying airdrops
        self.backer_account.initial_airdrop_received = false;
        
        Ok(())
    }
}

