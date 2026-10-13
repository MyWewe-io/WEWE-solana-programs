// set constant.rs values
use crate::{errors::ProposalError, state::config::Configs};
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

    // Used to authorize the first set_config (bootstrap): proves `authority` is the program's
    // upgrade authority. The constraint ties program_data to this program so a spoofed
    // program_data cannot be substituted. Ignored once a configure_authority is stored.
    #[account(constraint = program.programdata_address()? == Some(program_data.key()) @ ProposalError::NotOwner)]
    pub program: Program<'info, crate::program::WeweTokenLaunchPad>,
    pub program_data: Account<'info, ProgramData>,

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
        admin_authority: Pubkey,
        configure_authority: Pubkey,
    ) -> Result<()> {
        // Access control depends on the PRE-write stored authority:
        //   - First call (configure_authority unset / zeroed): only the program upgrade
        //     authority may seed the config authorities.
        //   - Thereafter: only the currently stored configure_authority may update config,
        //     which lets it rotate either authority with no program upgrade.
        let current_configure_authority = self.config.configure_authority;
        if current_configure_authority == Pubkey::default() {
            require!(
                self.program_data.upgrade_authority_address == Some(self.authority.key()),
                ProposalError::NotOwner
            );
        } else {
            require_keys_eq!(
                self.authority.key(),
                current_configure_authority,
                ProposalError::NotOwner
            );
        }

        // Never allow a zeroed authority, which would lock out future config access.
        require!(
            configure_authority != Pubkey::default() && admin_authority != Pubkey::default(),
            ProposalError::IncorrectAccount
        );

        self.config.set_inner(Configs {
            amount_to_raise_per_user: amount_to_raise_per_user,
            total_mint: total_mint,
            total_pool_tokens: total_pool_tokens,
            maker_token_amount: maker_token_amount,
            total_airdrop_amount_per_milestone: total_airdrop_amount_per_milestone,
            min_backers: min_backers,
            max_backed_proposals: max_backed_proposals,
            refund_fee_basis_points: refund_fee_basis_points, // 100 BPS = 1%
            admin_authority: admin_authority,
            configure_authority: configure_authority,
        });

        Ok(())
    }
}
