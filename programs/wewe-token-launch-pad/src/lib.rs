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

declare_id!("AvBFyeWVxa297Xjj2qDpBsMU1VDWfNhGxWRT9eaZLnoY");

#[program]
pub mod wewe_token_launch_pad {
    use super::*;

    pub fn create_proposal(
        ctx: Context<CreateProposal>,
        duration: u16,
        backing_goal: u64,
        token_name: String,
        token_symbol: String,
        token_uri: String,
        token_decimals: u8,
    ) -> Result<()> {
        ctx.accounts.create_proposal(
            token_decimals,
            backing_goal,
            token_name,
            token_symbol,
            token_uri,
            duration,
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
        bump: u8,
    ) -> Result<()> {
    
        ctx.accounts.create_pool(liquidity, sqrt_price, bump)
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
}
