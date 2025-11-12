use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct BackerProposalCount {
    pub backer: Pubkey,
    pub active_count: u64,
}

