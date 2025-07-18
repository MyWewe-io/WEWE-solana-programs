use anchor_lang::prelude::*;

use crate::{constant::OWNER, errors::ProposalError};

pub fn check(signer_account: &AccountInfo) -> Result<()> {
    // Check if signer === owner
    require_keys_eq!(
        signer_account.key(),
        OWNER,
        ProposalError::NotOwner
    );

    Ok(())
}
