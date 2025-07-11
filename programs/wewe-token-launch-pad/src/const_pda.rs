use anchor_lang::solana_program::pubkey::Pubkey;
use const_crypto::ed25519;

pub mod const_authority {
    use super::*;

    const POOL_AUTHORITY_AND_BUMP: ([u8; 32], u8) = ed25519::derive_program_address(
        &[crate::constant::POOL_AUTHORITY_PREFIX],
        &cp_amm::ID_CONST.to_bytes(),
    );

    pub const POOL_ID: Pubkey = Pubkey::new_from_array(POOL_AUTHORITY_AND_BUMP.0);

    const VAULT_AUTHORITY_AND_BUMP: ([u8; 32], u8) = ed25519::derive_program_address(
        &[crate::constant::VAULT_AUTHORITY],
        &crate::ID_CONST.to_bytes(),
    );
    
    pub const VAULT_BUMP: u8 = VAULT_AUTHORITY_AND_BUMP.1;
    
    const MINT_ACCOUNT_AND_BUMP: ([u8; 32], u8) = ed25519::derive_program_address(
        &[crate::constant::MINT_ACCOUNT],
        &crate::ID_CONST.to_bytes(),
    );

    pub const MINT: Pubkey = Pubkey::new_from_array(MINT_ACCOUNT_AND_BUMP.0);
}