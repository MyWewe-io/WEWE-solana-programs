use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Proposer {
    pub maker: Pubkey,       // Fundraiser creator
    pub current_amount: u64, // Current raised amount
    pub time_started: i64,   // Start timestamp
    pub duration: u16,       // Duration in seconds
    pub bump: u8,            // PDA security bump
}