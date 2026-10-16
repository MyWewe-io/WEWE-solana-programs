use anchor_lang::prelude::*;

use crate::{constant::multisig_authority, errors::ProposalError};

/// Gate for reset_airdrop. Authorizes ONLY the Squads multisig vault, which is a
/// PDA with no private key — so the operation can only run via a Squads proposal +
/// threshold approval. Unlike the config-stored authorities this is immutable (a
/// hardcoded const, not a rotatable Configs field), which is the point: a single
/// compromised private key can never trigger a reset, even if the stored
/// configure_authority were somehow changed.
pub fn check_multisig(signer_account: &AccountInfo) -> Result<()> {
    require_keys_eq!(
        signer_account.key(),
        multisig_authority::ID,
        ProposalError::NotOwner
    );

    Ok(())
}
