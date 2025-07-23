use anchor_lang::prelude::*;

use crate::{constant::admin_pubkey, errors::ProposalError};

pub fn check(signer_account: &AccountInfo) -> Result<()> {
    // Check if signer === owner
    require_keys_eq!(
        signer_account.key(),
        admin_pubkey::ID,
        ProposalError::NotOwner
    );

    Ok(())
}
