use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Proposal {
    pub maker: Pubkey,       // Proposal creator
    pub mint_account: Pubkey,
    pub total_backing: u64, // Current raised amount
    pub time_started: i64,   // Start timestamp
    pub bump: u8,            // PDA security bump
    pub is_rejected: bool,   // proposal is rejected 
    pub proposal_id: u64, 
    pub is_pool_launched: bool,
    pub total_backers: u64,
    pub current_airdrop_cycle: u8,
}
