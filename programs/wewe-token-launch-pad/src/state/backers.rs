use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Backers {
    pub reputation_score: u64,
    pub claimed_cycle: u8,
}
