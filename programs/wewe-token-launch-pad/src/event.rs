use anchor_lang::prelude::*;

#[event]
pub struct ProposalCreated {
    pub maker: Pubkey,
    pub proposal_address: Pubkey,
    pub proposal_index: u64,
    pub start_time: i64,
    pub token_name: String,
    pub token_symbol: String,
    pub token_uri: String,
    pub mint_account: Pubkey,
}

#[event]
pub struct ProposalBacked {
    pub backer: Pubkey,
    pub proposal_backed: Pubkey,
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

#[event]
pub struct AirdropClaimed {
    pub proposal_address: Pubkey,
    pub backer: Pubkey,
    pub amount: u64,
}

#[event]
pub struct CoinLaunched {
    pub mint_account: Pubkey,
    pub proposal_address: Pubkey,
    pub total_sol_raised: u64,
    pub pool_address: Pubkey,
}