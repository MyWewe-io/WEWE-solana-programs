use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]

pub struct Configs {
    pub amount_to_raise_per_user : u64,
    pub total_mint : u64,
    pub total_pool_tokens: u64,
    pub maker_token_amount : u64,
    pub total_airdrop_amount_per_milestone: u64,
    pub min_backers : u64,
    pub max_backed_proposals : u64,
    pub refund_fee_basis_points : u16, // Fee percentage in basis points (100 = 1%, 250 = 2.5%, etc.)
    // Access-control authorities, stored on-chain so they can be rotated via set_config
    // WITHOUT a program upgrade. See utils::check / utils::check_configure_authority.
    pub admin_authority : Pubkey,     // hot backend signer: automated ops (mint, airdrop, snapshot, fees, milestones)
    pub configure_authority : Pubkey, // cold / Squads multisig: set_config + sensitive reset/emergency ops
}