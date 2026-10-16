use anchor_lang::prelude::*;

use crate::{errors::ProposalError, state::config::Configs};

/// Gate for sensitive operations (set_config, reset_airdrop, reset_pool_launch,
/// emergency_unlock). Authorizes only the cold / multisig configure_authority stored
/// in the Configs account.
pub fn check_configure_authority(signer_account: &AccountInfo, config: &Account<Configs>) -> Result<()> {
    require_keys_eq!(
        signer_account.key(),
        config.configure_authority,
        ProposalError::NotOwner
    );

    Ok(())
}
