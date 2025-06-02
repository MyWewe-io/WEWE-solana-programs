#![allow(unexpected_cfgs)]
use anchor_lang::prelude::*;

mod constant;
pub mod errors;
mod event;
pub mod instructions;
pub mod state;
mod utils;

use instructions::*;
use utils::*;

declare_program!(dynamic_amm);
declare_id!("F1oVBG5SQfKwd78DHUJtfwARJv8Mk8yAMoW6yp8hPi8u");

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

    pub fn deposit_sol(ctx: Context<Contribute>, proposal_index: u64) -> Result<()> {
        ctx.accounts.deposit_sol(proposal_index)?;

        Ok(())
    }

    pub fn refund(ctx: Context<Refund>, proposal_index: u64) -> Result<()> {
        ctx.accounts.refund(proposal_index)?;

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

    pub fn claim(ctx: Context<Claim>, proposal_index: u64) -> Result<()> {
        ctx.accounts.claim(proposal_index)?;

        Ok(())
    }

    #[access_control(check(&ctx.accounts.authority))]
    pub fn reject_proposal(ctx: Context<RejectProposal>, proposal_index: u64) -> Result<()> {
        ctx.accounts.reject_proposal(proposal_index)?;

        Ok(())
    }
}
