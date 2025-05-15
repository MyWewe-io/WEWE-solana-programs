use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Backers {
    pub amount: u64,
}
