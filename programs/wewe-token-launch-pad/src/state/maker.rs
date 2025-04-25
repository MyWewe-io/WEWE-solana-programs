use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct MakerAccount {
    pub proposal_count: u64,
}
