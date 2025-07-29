pub const ANCHOR_DISCRIMINATOR: usize = 8;
pub const SECONDS_TO_DAYS: i64 = 86400;

pub const AMOUNT_TO_RAISE_PER_USER: u64 = 998_997_760;
pub const FEE_TO_DEDUCT: u64 = 2_000_000;   //0.002 SOl 

pub const TOTAL_MINT: u64 = 1_000_000_000; // 1 billion supply
pub const TOTAL_POOL_TOKENS: u64 = 150_000_000; // 15% of total Token supply
pub const MAKER_TOKEN_AMOUNT: u64 = 10_000_000;// 1% of the total Token supply 
pub const TOTAL_AIRDROP_AMOUNT_PER_MILESTONE: u64 = 140_000_000; // 14% of total Token supply

pub const MINIMUM_BACKERS: u64 = 1; // shud be 50
pub const MAXIMUM_BACKERS: u64 = 1000;

pub mod seeds {
    pub const MINT_ACCOUNT: &[u8] = b"mint_soulbound";
    pub const POOL_AUTHORITY_PREFIX: &[u8] = b"pool_authority";
    pub const VAULT_AUTHORITY: &[u8] = b"vault_authority";
    pub const BACKER: &[u8] = b"backer";
    pub const MINT_AUTHORITY: &[u8] = b"mint_authority";
    pub const PROPOSAL: &[u8] = b"proposal";
    pub const TOKEN_VAULT: &[u8] = b"token_vault";
    pub const MAKER: &[u8] = b"maker";
}
pub mod treasury {
    use anchor_lang::{prelude::Pubkey, solana_program::pubkey};
    pub const ID: Pubkey = pubkey!("76U9hvHNUNn7YV5FekSzDHzqnHETsUpDKq4cMj2dMxNi");
}

pub mod admin_pubkey {
    use anchor_lang::{prelude::Pubkey, solana_program::pubkey};
    pub const ID: Pubkey = pubkey!("D4VNMB6heKqVyiii4HjK2K7pEC9U3tVuNjCkFr3xNGfe");
}
