use anchor_lang::prelude::*;

#[event]
pub struct ProposalCreated {
    pub maker: Pubkey,
    pub proposal_address: Pubkey,
    pub start_time: i64,
    pub duration: u16,
    pub token_name: String,
    pub token_symbol: String,
    pub token_uri: String,
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

#[event]
pub struct ProposalRejected {
    pub maker: Pubkey,
    pub proposal_address: Pubkey,
}