use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Backers {
    pub claim_amount: u64,
    pub initial_airdrop_received: bool,
    pub settle_cycle: u8,
    pub claimed_upto: u8,
}
