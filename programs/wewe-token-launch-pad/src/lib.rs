#![allow(unexpected_cfgs)]
use anchor_lang::prelude::*;

mod const_pda;
mod constant;
pub mod errors;
mod event;
pub mod instructions;
pub mod state;
mod utils;

use errors::*;
use instructions::*;
use utils::*;

declare_id!("Dr6KaoKH13wjV9Jq7vnz4XzDPWBPEfxqwJbo82akf3u7");
#[program]
pub mod wewe_token_launch_pad {
    use super::*;

    pub fn create_proposal(
        ctx: Context<CreateProposal>,
        token_name: String,
        token_symbol: String,
        token_uri: String,
    ) -> Result<()> {
        ctx.accounts.handle_create_proposal(token_name, token_symbol, token_uri, &ctx.bumps)
    }

    pub fn deposit_sol(ctx: Context<Contribute>) -> Result<()> {
        match ctx.accounts.handle_deposit_sol() {
            Ok(_) => {}
            Err(_) => return Err(error!(ProposalError::ProposalAlreadyBacked)),
        }

        Ok(())
    }

    pub fn refund(ctx: Context<Refund>) -> Result<()> {
        ctx.accounts.handle_refund()
    }

    pub fn create_pool(ctx: Context<DammV2>, sqrt_price: u128) -> Result<()> {
        ctx.accounts.handle_create_pool(sqrt_price)
    }

    pub fn claim_milestone_reward(ctx: Context<Claim>) -> Result<()> {
        ctx.accounts.handler_claim_milestone_reward()
    }

    #[access_control(check(&ctx.accounts.authority))]
    pub fn reject_proposal(ctx: Context<RejectProposal>) -> Result<()> {
        ctx.accounts.handle_reject_proposal()
    }

    #[access_control(check(&ctx.accounts.authority))]
    pub fn snapshot_backer_amount(ctx: Context<SnapshotBacker>) -> Result<()> {
        ctx.accounts.handle_snapshot()
    }

    #[access_control(check(&ctx.accounts.payer))]
    pub fn mint_soulbound_to_user(ctx: Context<MintSoulboundToUser>) -> Result<()> {
        ctx.accounts.handle_mint_soulbound_to_user(&ctx.bumps)
    }

    #[access_control(check(&ctx.accounts.payer))]
    pub fn claim_pool_fee(ctx: Context<ClaimPositionFee>) -> Result<()> {
        ctx.accounts.handle_claim_position_fee()
    }

    #[access_control(check(&ctx.accounts.authority))]
    pub fn initialise_milestone(ctx: Context<InitialiseMilestone>) -> Result<()> {
        ctx.accounts.handle_initialise_milestone()
    }

    pub fn airdrop(ctx: Context<Airdrop>) -> Result<()> {
        ctx.accounts.handle_airdrop()
    }

    #[access_control(check(&ctx.accounts.authority))]
    pub fn end_milestone(ctx: Context<EndMilestone>) -> Result<()> {
        ctx.accounts.handle_end_milestone()
    }
}
