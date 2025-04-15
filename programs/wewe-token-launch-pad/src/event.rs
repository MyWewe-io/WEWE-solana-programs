use anchor_lang::prelude::*;

#[event]
pub struct ProposalCreated {
    pub maker: Pubkey,
    pub start_time: i64,
    pub duration: u16,
}

#[event]
pub struct ProposalBacked {
    pub backer: Pubkey,
    pub proposal_backed: Pubkey,
    pub amount: u64,
}
