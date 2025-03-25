use anchor_lang::prelude::*;

#[event]
pub struct ProposalCreated {
    pub maker: Pubkey,
    pub start_time: i64,
    pub duration: u16,
}

