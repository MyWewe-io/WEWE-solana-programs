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

declare_id!("14LLwL8ixmeex2Ab4irrLJe1Nrxwj3N9CuYVq3vnwPbb");
#[program]
pub mod wewe_token_launch_pad {
    use super::*;

    #[access_control(check_configure_authority(&ctx.accounts.authority))]
    pub fn set_config(
        ctx: Context<SetConfig>,
        amount_to_raise_per_user: u64,
        total_mint: u64,
        total_pool_tokens: u64,
        maker_token_amount: u64,
        total_airdrop_amount_per_milestone: u64,
        min_backers: u64,
        max_backed_proposals: u64,
        refund_fee_basis_points: u16,
    ) -> Result<()> {
        ctx.accounts.handle_set_config(
            amount_to_raise_per_user,
            total_mint,
            total_pool_tokens,
            maker_token_amount,
            total_airdrop_amount_per_milestone,
            min_backers,
            max_backed_proposals,
            refund_fee_basis_points,
        )?;
        Ok(())
    }

    pub fn create_proposal(
        ctx: Context<CreateProposal>,
        token_name: String,
        token_symbol: String,
        token_uri: String,
    ) -> Result<()> {
        ctx.accounts.handle_create_proposal(token_name, token_symbol, token_uri, &ctx.bumps)
    }

    pub fn deposit_sol(ctx: Context<Contribute>) -> Result<()> {
        ctx.accounts.handle_deposit_sol()
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

    // TODO: Figure out if this is needed
    // #[access_control(check(&ctx.accounts.authority))]
    // pub fn emergency_unlock(ctx: Context<EmergencyUnlock>) -> Result<()> {
    //     ctx.accounts.handle_emergency_unlock()
    // }

    #[access_control(check(&ctx.accounts.authority))]
    pub fn snapshot_backer_amount(ctx: Context<SnapshotBacker>) -> Result<()> {
        ctx.accounts.handle_snapshot()
    }

    #[access_control(check(&ctx.accounts.authority))]
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
