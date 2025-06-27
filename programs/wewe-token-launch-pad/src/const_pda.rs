use anchor_lang::solana_program::pubkey::Pubkey;
use const_crypto::ed25519;

pub mod pool_authority {
    use super::*;

    const POOL_AUTHORITY_AND_BUMP: ([u8; 32], u8) = ed25519::derive_program_address(
        &[crate::constant::POOL_AUTHORITY_PREFIX],
        &crate::ID_CONST.to_bytes(),
    );

    pub const ID: Pubkey = Pubkey::new_from_array(POOL_AUTHORITY_AND_BUMP.0);
}