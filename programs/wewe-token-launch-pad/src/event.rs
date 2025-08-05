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
    pub token_vault: Pubkey,
    pub metadata_account: Pubkey,
    pub maker_account: Pubkey,
    pub proposal_bump: u8,
}


#[event]
pub struct ProposalBacked {
    pub backer: Pubkey,
    pub backer_account: Pubkey,
    pub proposal_backed: Pubkey,
}

#[event]
pub struct BackerRefunded {
    pub backer: Pubkey,
    pub backer_account: Pubkey,
    pub proposal_address: Pubkey,
    pub amount: u64,
}

#[event]
pub struct ProposalRejected {
    pub maker: Pubkey,
    pub mint_account: Pubkey,
    pub proposal_address: Pubkey,
}

#[event]
pub struct AirdropClaimed {
    pub proposal_address: Pubkey,
    pub backer: Pubkey,
    pub backer_account: Pubkey,
    pub mint_account: Pubkey,
    pub vault_account: Pubkey,
    pub recipient_account: Pubkey,
    pub amount: u64,
}

#[event]
pub struct CoinLaunched {
    pub proposal_address: Pubkey,
    pub mint_account: Pubkey,
    pub quote_mint: Pubkey,
    pub total_sol_raised: u64,
    pub pool_address: Pubkey,
    pub token_vault: Pubkey,
    pub wsol_vault: Pubkey,
    pub maker: Pubkey,
    pub maker_token_account: Pubkey,
    pub position: Pubkey,
    pub position_nft_account: Pubkey,
    pub sqrt_price: u128,
    pub liquidity: u128,
}

#[event]
pub struct PositionFeeClaimed {
    pub proposal: Pubkey,
    pub maker: Pubkey,
    pub user: Pubkey,
    pub user_token_amount: u64,
    pub user_wsol_amount: u64,
    pub token_mint: Pubkey,
    pub wsol_mint: Pubkey,
}

#[event]
pub struct AirdropClaimUpdated {
    pub proposal: Pubkey,
    pub backer: Pubkey,
    pub amount: u64,
    pub cycle: u8,
}

#[event]
pub struct TokensBurned {
    pub proposal: Pubkey,
    pub amount: u64,
    pub cycle: u8,
}