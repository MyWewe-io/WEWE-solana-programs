use anchor_lang::prelude::*;

use crate::{constant::chain_service_pubkey, errors::ProposalError};

pub fn check(signer_account: &AccountInfo) -> Result<()> {
    // Check if signer === owner
    require_keys_eq!(
        signer_account.key(),
        chain_service_pubkey::ID,
        ProposalError::NotOwner
    );

    Ok(())
}
