#![allow(unexpected_cfgs)]
use anchor_lang::prelude::*;

pub mod state;
pub mod instructions;
pub mod errors;
mod constant;
mod event;
mod utils;

pub use instructions::*;

declare_program!(dynamic_amm);
declare_id!("DtwC3LsBgwnp6Cuc6MExnijmwh7WLXS5Hdr7XpdyF1qZ");

#[program]
pub mod wewe_token_launch_pad {
    use super::*;

    pub fn create_proposal(ctx: Context<CreateProposal>, duration: u16, backing_goal: u64, token_name: String,
        token_symbol: String,
        token_uri: String,
        token_decimals: u8) -> Result<()> {

        ctx.accounts.create_proposal(duration, backing_goal, token_name, token_symbol, token_uri, token_decimals, &ctx.bumps)?;
        
        Ok(())
    }

    pub fn deposit_sol(ctx: Context<Contribute>, amount: u64) -> Result<()> {

        ctx.accounts.deposit_sol(amount)?;

        Ok(())
    }

    pub fn refund(ctx: Context<Refund>) -> Result<()> {

        ctx.accounts.refund()?;

        Ok(())
    }

    // pub fn initialize_dynamic_amm_customizable_permissionless_pool(
    //     ctx: Context<DynamicAmmInitializeCustomizablePermissionlessPool>,
    //     token_a_amount: u64,
    //     token_b_amount: u64,
    //     params: dynamic_amm::types::CustomizableParams,
    // ) -> Result<()> {
        
    //     instructions::handle_initialize_customizable_permissionless_pool(ctx, token_a_amount, token_b_amount, params)?;

    //     Ok(())
    // }

    pub fn transfer_tokens(ctx: Context<TransferTokens>) -> Result<()> {

        instructions::transfer_tokens(ctx)?;

        Ok(())
    }

}