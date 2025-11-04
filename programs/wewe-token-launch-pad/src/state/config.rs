use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]

pub struct Configs {
    pub amount_to_raise_per_user : u64,
    pub total_mint : u64,
    pub total_pool_tokens: u64,
    pub maker_token_amount : u64,
    pub total_airdrop_amount_per_milestone: u64,
    pub min_backers : u64,
    // Transfer fee configuration (basis points, 100 = 1%)
    pub transfer_fee_basis_points: u16,
    // Maximum fee in native token units (prevents unbounded fees)
    pub max_fee: u64,
}