use anchor_lang::prelude::*;

use crate::{
    errors::ProposalError,
    state::{config::Configs, proposal::Proposal},
};

#[derive(Accounts)]
pub struct EmergencyUnlock<'info> {
    pub authority: Signer<'info>,

    #[account(seeds = [b"config"], bump)]
    pub config: Account<'info, Configs>,

    #[account(mut)]
    pub proposal: Account<'info, Proposal>,

    /// CHECK: Pool account to verify it doesn't exist or is invalid
    pub pool_account: AccountInfo<'info>,
}

impl<'info> EmergencyUnlock<'info> {
    pub fn handle_emergency_unlock(&mut self) -> Result<()> {
        // Sensitive op — require the cold configure_authority even if the lib.rs
        // #[access_control] wrapper is ever removed.
        require_keys_eq!(
            self.authority.key(),
            self.config.configure_authority,
            ProposalError::UnauthorizedEmergencyUnlock
        );

        // Require that pool launch flag is set (otherwise no need to unlock)
        require!(
            self.proposal.is_pool_launched,
            ProposalError::PoolAlreadyLaunched
        );

        // Require that emergency unlock hasn't been used before
        require!(
            !self.proposal.emergency_unlocked,
            ProposalError::PoolAlreadyLaunched
        );

        // Verify pool doesn't actually exist or is invalid
        // If pool account is empty or owned by system program, it wasn't created
        let pool_exists = !self.pool_account.data_is_empty()
            && *self.pool_account.owner != anchor_lang::system_program::ID;
        
        // Option 1: Pool doesn't exist - allow unlock
        if !pool_exists {
            self.proposal.is_pool_launched = false;
            self.proposal.emergency_unlocked = true;
            return Ok(());
        }

        // Option 2: Check if sufficient time has passed (24 hours = 86400 seconds)
        let clock = Clock::get()?;
        if let Some(launch_timestamp) = self.proposal.launch_timestamp {
            let elapsed = clock.unix_timestamp.saturating_sub(launch_timestamp);
            require!(
                elapsed >= 86400, // 24 hours
                ProposalError::TooSoonForEmergencyUnlock
            );
            
            // Even if pool exists, if enough time has passed and admin confirms,
            // allow unlock (admin can verify pool state manually)
            self.proposal.is_pool_launched = false;
            self.proposal.emergency_unlocked = true;
            return Ok(());
        }

        // If no launch timestamp and pool exists, don't allow unlock
        // (shouldn't happen, but safety check)
        require!(
            !pool_exists,
            ProposalError::PoolActuallyExists
        );

        Ok(())
    }
}

