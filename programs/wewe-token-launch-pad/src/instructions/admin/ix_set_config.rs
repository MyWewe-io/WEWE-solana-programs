// set constant.rs values 
use crate::state::config::Configs;
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct SetConfig<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        init_if_needed,
        payer = authority,
        space = 8 + Configs::INIT_SPACE,
        seeds = [b"config"],
        bump
    )]
    pub config: Account<'info, Configs>,

    pub system_program: Program<'info, System>,
}

impl<'info> SetConfig<'info> {
    pub fn handle_set_config(
        &mut self,
        amount_to_raise_per_user: u64,
        total_mint: u64,
        total_pool_tokens: u64,
        maker_token_amount: u64,
        total_airdrop_amount_per_milestone: u64,
        min_backers: u64,
        max_backed_proposals: u64,
        refund_fee_basis_points: u16,
    ) -> Result<()> {
        self.config.set_inner(Configs {
            amount_to_raise_per_user: amount_to_raise_per_user,
            total_mint: total_mint,
            total_pool_tokens: total_pool_tokens,
            maker_token_amount: maker_token_amount,
            total_airdrop_amount_per_milestone: total_airdrop_amount_per_milestone,
            min_backers: min_backers,
            max_backed_proposals: max_backed_proposals,
            refund_fee_basis_points: refund_fee_basis_points, // 100 BPS = 1%
        });
        
        Ok(())
    }
}