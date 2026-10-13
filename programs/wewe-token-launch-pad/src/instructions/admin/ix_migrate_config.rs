// One-time migration: grow an existing Configs account to the current layout.
//
// The previous layout had no admin_authority / configure_authority fields. The old
// serialized data is a prefix of the new layout, so growing the account and zero-filling
// the new tail yields a valid Configs with both authorities = Pubkey::default(). That zeroed
// configure_authority is what arms the bootstrap branch of set_config, so the sequence is:
// upgrade program -> migrate_config -> set_config (upgrade authority seeds the authorities).
//
// The account is taken as AccountInfo, not Account<Configs>: the old data cannot deserialize
// as the new Configs (it is 64 bytes short), so Anchor must not try.
use crate::{errors::ProposalError, state::config::Configs};
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct MigrateConfig<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    /// CHECK: config PDA, resized manually here; validated by seeds + program ownership.
    #[account(mut, seeds = [b"config"], bump, owner = crate::ID)]
    pub config: AccountInfo<'info>,

    // Gate: only the program upgrade authority may migrate. The constraint ties program_data
    // to this program so a spoofed program_data cannot be substituted.
    #[account(constraint = program.programdata_address()? == Some(program_data.key()) @ ProposalError::NotOwner)]
    pub program: Program<'info, crate::program::WeweTokenLaunchPad>,
    pub program_data: Account<'info, ProgramData>,

    pub system_program: Program<'info, System>,
}

impl<'info> MigrateConfig<'info> {
    pub fn handle_migrate_config(&mut self) -> Result<()> {
        require!(
            self.program_data.upgrade_authority_address == Some(self.authority.key()),
            ProposalError::NotOwner
        );

        let info = self.config.to_account_info();
        let new_len: usize = 8 + Configs::INIT_SPACE;
        let old_len = info.data_len();

        // Idempotent: if already at (or past) the new size, nothing to do.
        if old_len >= new_len {
            return Ok(());
        }

        // Keep the account rent-exempt at the new size.
        let rent = Rent::get()?;
        let required = rent.minimum_balance(new_len);
        let current = info.lamports();
        if required > current {
            anchor_lang::system_program::transfer(
                CpiContext::new(
                    self.system_program.to_account_info(),
                    anchor_lang::system_program::Transfer {
                        from: self.authority.to_account_info(),
                        to: info.clone(),
                    },
                ),
                required - current,
            )?;
        }

        // Grow in place and zero-fill the appended bytes (admin_authority + configure_authority).
        info.realloc(new_len, true)?;

        Ok(())
    }
}
