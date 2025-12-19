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
    pub max_backed_proposals : u64,
    pub refund_fee_basis_points : u16, // Fee percentage in basis points (100 = 1%, 250 = 2.5%, etc.)
    
    // CP-AMM Pool Parameters - Only fee percentage is configurable
    // Fee numerator (denominator is 100,000,000, so 20_000_000 = 2%, 2_000_000 = 0.2%)
    pub pool_cliff_fee_numerator: u64,
}