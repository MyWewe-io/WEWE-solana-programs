use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Proposal {
    pub maker: Pubkey,
    pub mint_account: Pubkey,
    pub time_started: i64,
    pub bump: u8,
    pub is_rejected: bool,
    pub proposal_id: u64,
    pub is_pool_launched: bool,
    pub total_backers: u64,
    pub total_backing: u64,
    pub current_airdrop_cycle: u8,
    pub milestone_active: bool,
    pub milestone_units_assigned: u64,
    pub milestone_backers_weighted: u64,
    pub milestone_reputation_sum: u64,
    pub launch_timestamp: Option<i64>,
    pub emergency_unlocked: bool,
    #[max_len(32)]
    pub token_name: String,
    #[max_len(10)]
    pub token_symbol: String,
    #[max_len(200)]
    pub token_uri: String,
}
