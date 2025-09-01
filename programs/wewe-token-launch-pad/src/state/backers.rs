use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Backers {
    pub claim_amount: u64,
    pub amount_updated_upto_cycle: u8,
    pub initial_airdrop_received: bool,
    pub contributed_lamports: u64,
    pub settle_cycle: u8,
}
