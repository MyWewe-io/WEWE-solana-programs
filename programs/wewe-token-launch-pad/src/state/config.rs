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
}