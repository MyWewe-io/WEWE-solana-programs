use anchor_lang::prelude::*;

use crate::{constant::configure_authority_pubkey, errors::ProposalError};

pub fn check_configure_authority(signer_account: &AccountInfo) -> Result<()> {
    // Check if signer === configure_authority (for set_config operations)
    require_keys_eq!(
        signer_account.key(),
        configure_authority_pubkey::ID,
        ProposalError::NotOwner
    );

    Ok(())
}

