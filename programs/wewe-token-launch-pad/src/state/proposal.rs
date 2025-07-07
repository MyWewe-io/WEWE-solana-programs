use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Proposal {
    pub maker: Pubkey,       // Proposal creator
    pub total_backing: u64, // Current raised amount
    pub time_started: i64,   // Start timestamp
    pub duration: u16,       // Duration in seconds
    pub bump: u8,            // PDA security bump
    pub backing_goal: u64,   // backing goal of proposal
    pub is_rejected: bool,   // proposal is rejected 
    pub proposal_id: u64, 
    pub is_pool_launched: bool,
    pub total_backers: u64,
}

