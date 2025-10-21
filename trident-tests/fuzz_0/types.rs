use borsh::{BorshDeserialize, BorshSerialize};
use trident_fuzz::fuzzing::*;

/// File containing all custom types which can be used
/// in transactions and instructions or invariant checks.
///
/// You can define your own custom types here.

#[derive(Debug, BorshDeserialize, BorshSerialize, Clone, Default)]
pub struct AirdropClaimed {
    pub proposal_address: TridentPubkey,

    pub backer: TridentPubkey,

    pub backer_account: TridentPubkey,

    pub mint_account: TridentPubkey,

    pub vault_account: TridentPubkey,

    pub recipient_account: TridentPubkey,

    pub amount: u64,
}

#[derive(Debug, BorshDeserialize, BorshSerialize, Clone, Default)]
pub struct BackerMilestoneSettled {
    pub proposal: TridentPubkey,

    pub backer: TridentPubkey,

    pub cycle: u8,

    pub alloc_units: u64,
}

#[derive(Debug, BorshDeserialize, BorshSerialize, Clone, Default)]
pub struct BackerRefunded {
    pub backer: TridentPubkey,

    pub backer_account: TridentPubkey,

    pub proposal_address: TridentPubkey,

    pub amount: u64,
}

#[derive(Debug, BorshDeserialize, BorshSerialize, Clone, Default)]
pub struct Backers {
    pub claim_amount: u64,

    pub initial_airdrop_received: bool,

    pub settle_cycle: u8,

    pub claimed_upto: u8,
}

#[derive(Debug, BorshDeserialize, BorshSerialize, Clone, Default)]
pub struct BaseFeeConfig {
    pub cliff_fee_numerator: u64,

    pub fee_scheduler_mode: u8,

    pub padding: [u8; 5],

    pub number_of_period: u16,

    pub period_frequency: u64,

    pub reduction_factor: u64,
}

#[derive(Debug, BorshDeserialize, BorshSerialize, Clone, Default)]
pub struct CoinLaunched {
    pub proposal_address: TridentPubkey,

    pub mint_account: TridentPubkey,

    pub quote_mint: TridentPubkey,

    pub total_sol_raised: u64,

    pub pool_address: TridentPubkey,

    pub token_vault: TridentPubkey,

    pub wsol_vault: TridentPubkey,

    pub maker: TridentPubkey,

    pub maker_token_account: TridentPubkey,

    pub position: TridentPubkey,

    pub position_nft_account: TridentPubkey,

    pub sqrt_price: u128,

    pub liquidity: u128,
}

#[derive(Debug, BorshDeserialize, BorshSerialize, Clone, Default)]
pub struct Config {
    pub vault_config_key: TridentPubkey,

    pub pool_creator_authority: TridentPubkey,

    pub pool_fees: PoolFeesConfig,

    pub activation_type: u8,

    pub collect_fee_mode: u8,

    pub config_type: u8,

    pub _padding_0: [u8; 5],

    pub index: u64,

    pub sqrt_min_price: u128,

    pub sqrt_max_price: u128,

    pub _padding_1: [u64; 10],
}

#[derive(Debug, BorshDeserialize, BorshSerialize, Clone, Default)]
pub struct Configs {
    pub amount_to_raise_per_user: u64,

    pub total_mint: u64,

    pub total_pool_tokens: u64,

    pub maker_token_amount: u64,

    pub total_airdrop_amount_per_milestone: u64,

    pub min_backers: u64,
}

#[derive(Debug, BorshDeserialize, BorshSerialize, Clone, Default)]
pub struct DynamicFeeConfig {
    pub initialized: u8,

    pub padding: [u8; 7],

    pub max_volatility_accumulator: u32,

    pub variable_fee_control: u32,

    pub bin_step: u16,

    pub filter_period: u16,

    pub decay_period: u16,

    pub reduction_factor: u16,

    pub padding_1: [u8; 8],

    pub bin_step_u128: u128,
}

#[derive(Debug, BorshDeserialize, BorshSerialize, Clone, Default)]
pub struct MakerAccount {
    pub proposal_count: u64,
}

#[derive(Debug, BorshDeserialize, BorshSerialize, Clone, Default)]
pub struct MilestoneEnded {
    pub proposal: TridentPubkey,

    pub cycle: u8,

    pub burned_units: u64,
}

#[derive(Debug, BorshDeserialize, BorshSerialize, Clone, Default)]
pub struct MilestoneStarted {
    pub proposal: TridentPubkey,

    pub cycle: u8,

    pub token_mint: TridentPubkey,
}

#[derive(Debug, BorshDeserialize, BorshSerialize, Clone, Default)]
pub struct PoolFeesConfig {
    pub base_fee: BaseFeeConfig,

    pub dynamic_fee: DynamicFeeConfig,

    pub protocol_fee_percent: u8,

    pub partner_fee_percent: u8,

    pub referral_fee_percent: u8,

    pub padding_0: [u8; 5],

    pub padding_1: [u64; 5],
}

#[derive(Debug, BorshDeserialize, BorshSerialize, Clone, Default)]
pub struct PositionFeeClaimed {
    pub proposal: TridentPubkey,

    pub maker: TridentPubkey,

    pub user: TridentPubkey,

    pub user_token_amount: u64,

    pub user_wsol_amount: u64,

    pub token_mint: TridentPubkey,

    pub wsol_mint: TridentPubkey,
}

#[derive(Debug, BorshDeserialize, BorshSerialize, Clone, Default)]
pub struct Proposal {
    pub maker: TridentPubkey,

    pub mint_account: TridentPubkey,

    pub time_started: i64,

    pub bump: u8,

    pub is_rejected: bool,

    pub proposal_id: u64,

    pub is_pool_launched: bool,

    pub total_backers: u64,

    pub total_backing: u64,

    pub current_airdrop_cycle: u8,

    pub milestone_active: bool,

    pub milestone_units_assigned: u64,

    pub milestone_backers_weighted: u64,
}

#[derive(Debug, BorshDeserialize, BorshSerialize, Clone, Default)]
pub struct ProposalBacked {
    pub backer: TridentPubkey,

    pub backer_account: TridentPubkey,

    pub proposal_backed: TridentPubkey,
}

#[derive(Debug, BorshDeserialize, BorshSerialize, Clone, Default)]
pub struct ProposalCreated {
    pub maker: TridentPubkey,

    pub proposal_address: TridentPubkey,

    pub proposal_index: u64,

    pub start_time: i64,

    pub token_name: String,

    pub token_symbol: String,

    pub token_uri: String,

    pub mint_account: TridentPubkey,

    pub metadata_account: TridentPubkey,

    pub token_vault: TridentPubkey,

    pub maker_account: TridentPubkey,

    pub proposal_bump: u8,
}

#[derive(Debug, BorshDeserialize, BorshSerialize, Clone, Default)]
pub struct ProposalRejected {
    pub maker: TridentPubkey,

    pub mint_account: TridentPubkey,

    pub proposal_address: TridentPubkey,
}
