#![allow(unexpected_cfgs)]
use anchor_lang::prelude::*;

mod constant;
pub mod errors;
mod event;
pub mod instructions;
pub mod state;
mod utils;
mod const_pda;

use instructions::*;
use utils::*;
use errors::*;

declare_id!("GK1gjRGXjoRPD9LuzsSX8BWeEzT8pJNwFcbA8rY1g6E9");

#[program]
pub mod wewe_token_launch_pad {
    use super::*;

    pub fn create_proposal(
        ctx: Context<CreateProposal>,
        token_name: String,
        token_symbol: String,
        token_uri: String,
    ) -> Result<()> {
        ctx.accounts.create_proposal(
            token_name,
            token_symbol,
            token_uri,
            &ctx.bumps,
        )?;

        Ok(())
    }

    pub fn deposit_sol(ctx: Context<Contribute>) -> Result<()> {
        match ctx.accounts.deposit_sol() {
            Ok(_) => {},
            Err(_) => return Err(error!(ProposalError::ProposalAlreadyBacked)),
        }

        Ok(())
    }

    pub fn refund(ctx: Context<Refund>) -> Result<()> {
        ctx.accounts.refund()?;

        Ok(())
    }

    pub fn create_pool(
        ctx: Context<DammV2>,
        liquidity: u128,
        sqrt_price: u128,
    ) -> Result<()> {
    
        ctx.accounts.create_pool(liquidity, sqrt_price)
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
}
