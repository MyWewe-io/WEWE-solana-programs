use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Proposer {
    pub proposer: Pubkey,
    pub time_started: i64,
    pub duration: u16,
    pub protocol: str,
    pub bump: u8,
}