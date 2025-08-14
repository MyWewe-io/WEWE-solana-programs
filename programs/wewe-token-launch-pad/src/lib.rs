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

declare_id!("JBC9QTYM8DsYWFkzk8GAZTUpfsUYMLha5REF6KomvPxs");

#[program]
pub mod wewe_token_launch_pad {
    use super::*;

    pub fn create_proposal(
        ctx: Context<CreateProposal>,
        token_name: String,
        token_symbol: String,
        token_uri: String,
    ) -> Result<()> {
        ctx.accounts
            .create_proposal(token_name, token_symbol, token_uri, &ctx.bumps)?;

        Ok(())
    }

    pub fn deposit_sol(ctx: Context<Contribute>) -> Result<()> {
        match ctx.accounts.deposit_sol() {
            Ok(_) => {}
            Err(_) => return Err(error!(ProposalError::ProposalAlreadyBacked)),
        }

        Ok(())
    }

    pub fn refund(ctx: Context<Refund>) -> Result<()> {
        ctx.accounts.refund()?;

        Ok(())
    }

    pub fn create_pool(ctx: Context<DammV2>, sqrt_price: u128) -> Result<()> {
        ctx.accounts.create_pool(sqrt_price)?;

        Ok(())
    }

    pub fn claim(ctx: Context<Claim>) -> Result<()> {
        ctx.accounts.claim()?;

        Ok(())
    }

    #[access_control(check(&ctx.accounts.authority))]
    pub fn reject_proposal(ctx: Context<RejectProposal>) -> Result<()> {
        ctx.accounts.reject_proposal()?;

        Ok(())
    }

    #[access_control(check(&ctx.accounts.authority))]
    pub fn update_airdrop_amount(ctx: Context<UpdateBacker>, amount: u64) -> Result<()> {
        ctx.accounts.update_airdrop_amount(amount)?;

        Ok(())
    }

    #[access_control(check(&ctx.accounts.payer))]
    pub fn mint_soulbound_to_user(ctx: Context<MintSoulboundToUser>) -> Result<()> {
        ctx.accounts.mint_soulbound_to_user(&ctx.bumps)?;

        Ok(())
    }

    #[access_control(check(&ctx.accounts.payer))]
    pub fn claim_pool_fee(
        ctx: Context<ClaimPositionFee>,
        user_wsol_amount: u64,
        user_token_amount: u64,
    ) -> Result<()> {
        ctx.accounts
            .claim_position_fee(user_wsol_amount, user_token_amount)?;

        Ok(())
    }

    #[access_control(check(&ctx.accounts.authority))]
    pub fn burn(ctx: Context<BurnTokens>, amount: u64) -> Result<()> {
        ctx.accounts.burn_tokens(amount)
    }

    #[access_control(check(&ctx.accounts.authority))]
    pub fn airdrop(ctx: Context<Airdrop>, amount: u64) -> Result<()> {
        ctx.accounts.airdrop(amount)
    }
}
