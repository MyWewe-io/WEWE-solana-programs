use anchor_lang::prelude::*;

use crate::{errors::ProposalError, state::config::Configs};

/// Gate for routine automated operations. Authorizes the HOT backend signer
/// (admin_authority) stored in the Configs account. Rotatable via set_config.
pub fn check(signer_account: &AccountInfo, config: &Account<Configs>) -> Result<()> {
    require_keys_eq!(
        signer_account.key(),
        config.admin_authority,
        ProposalError::NotOwner
    );

    Ok(())
}
