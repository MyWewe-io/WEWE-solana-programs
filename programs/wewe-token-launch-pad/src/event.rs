use anchor_lang::prelude::*;

#[event]
pub struct ProposalCreated {
    pub maker: Pubkey,
    pub proposal_address: Pubkey,
    pub start_time: i64,
    pub duration: u16,
}

#[event]
pub struct ProposalBacked {
    pub backer: Pubkey,
    pub proposal_backed: Pubkey,
    pub amount: u64,
}

#[event]
pub struct BackerRefunded {
    pub backer: Pubkey,
    pub amount: u64,
}