use anchor_lang::prelude::*;
use anchor_lang::system_program::Transfer as NativeSolTransfer;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount, Transfer as TokenTransfer},
};
use dynamic_amm::instructions::CustomizableParams;

pub const POOL_SIZE: usize = 8 + 944;

#[derive(Accounts)]
pub struct InitializePoolFromProposerCreator<'info> {
    #[account(mut)]
    pub maker: Signer<'info>, // The proposer creator initializes the pool

    #[account(
        seeds = [b"proposer", maker.key().as_ref()],
        bump
    )]
    pub proposer: Account<'info, Proposer>, // The proposer exists but doesn't initialize the pool

    #[account(
        init_if_needed,
        associated_token::mint = token_a_mint,
        associated_token::authority = maker,
        payer = maker
    )]
    pub maker_token_a: Account<'info, TokenAccount>,

    #[account(
        init_if_needed,
        associated_token::mint = token_b_mint,
        associated_token::authority = maker,
        payer = maker
    )]
    pub maker_token_b: Account<'info, TokenAccount>,

    /// CHECK: Pool account (PDA)
    #[account(mut)]
    pub pool: UncheckedAccount<'info>,

    #[account(mut)]
    pub lp_mint: UncheckedAccount<'info>,

    pub token_a_mint: UncheckedAccount<'info>,
    pub token_b_mint: UncheckedAccount<'info>,

    #[account(mut)]
    pub a_vault: UncheckedAccount<'info>,

    #[account(mut)]
    pub b_vault: UncheckedAccount<'info>,

    #[account(mut)]
    pub a_token_vault: UncheckedAccount<'info>,

    #[account(mut)]
    pub b_token_vault: UncheckedAccount<'info>,

    #[account(mut)]
    pub a_vault_lp_mint: UncheckedAccount<'info>,

    #[account(mut)]
    pub b_vault_lp_mint: UncheckedAccount<'info>,

    #[account(mut)]
    pub a_vault_lp: UncheckedAccount<'info>,

    #[account(mut)]
    pub b_vault_lp: UncheckedAccount<'info>,

    #[account(mut)]
    pub maker_pool_lp: UncheckedAccount<'info>,

    #[account(mut)]
    pub protocol_token_a_fee: UncheckedAccount<'info>,

    #[account(mut)]
    pub protocol_token_b_fee: UncheckedAccount<'info>,

    pub rent: Sysvar<'info, Rent>,

    #[account(mut)]
    pub mint_metadata: UncheckedAccount<'info>,

    pub metadata_program: UncheckedAccount<'info>,

    pub vault_program: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,

    #[account(address = dynamic_amm::ID)]
    pub dynamic_amm_program: UncheckedAccount<'info>,
}

/// Executes a Dynamic AMM initialize customizable constant product permissionless pool but with PDA as creator / payer.
///
/// # Arguments
///
/// * `ctx` - The context containing accounts and programs.
/// * `token_a_amount` - The amount of token a to be deposited.
/// * `token_b_amount` - The amount of token b to be deposited.
/// * `params` - The parameters for the pool.
///
/// # Returns
///
/// Returns a `Result` indicating success or failure.
pub fn handle_initialize_pool_from_proposer_creator(
    ctx: Context<InitializePoolFromProposerCreator>,
    token_a_amount: u64,
    token_b_amount: u64,
    params: CustomizableParams,
) -> Result<()> {
    // Verify that signer is the proposer creator
    require_keys_eq!(
        ctx.accounts.maker.key(),
        ctx.accounts.proposer.maker,
        PoolError::InvalidProposerCreator
    );

    // Continue with pool initialization
    fund_maker(
        token_a_amount,
        token_b_amount,
        FundMakerAccounts {
            maker_token_a: &ctx.accounts.maker_token_a,
            maker_token_b: &ctx.accounts.maker_token_b,
            token_program: &ctx.accounts.token_program,
            maker: &ctx.accounts.maker,
            system_program: &ctx.accounts.system_program,
        },
    )?;

    let accounts =
        dynamic_amm::cpi::accounts::InitializeCustomizablePermissionlessConstantProductPool {
            pool: ctx.accounts.pool.to_account_info(),
            token_a_mint: ctx.accounts.token_a_mint.to_account_info(),
            token_b_mint: ctx.accounts.token_b_mint.to_account_info(),
            a_vault: ctx.accounts.a_vault.to_account_info(),
            b_vault: ctx.accounts.b_vault.to_account_info(),
            a_token_vault: ctx.accounts.a_token_vault.to_account_info(),
            b_token_vault: ctx.accounts.b_token_vault.to_account_info(),
            a_vault_lp_mint: ctx.accounts.a_vault_lp_mint.to_account_info(),
            b_vault_lp_mint: ctx.accounts.b_vault_lp_mint.to_account_info(),
            a_vault_lp: ctx.accounts.a_vault_lp.to_account_info(),
            b_vault_lp: ctx.accounts.b_vault_lp.to_account_info(),
            payer_token_a: ctx.accounts.maker_token_a.to_account_info(),
            payer_token_b: ctx.accounts.maker_token_b.to_account_info(),
            payer_pool_lp: ctx.accounts.maker_pool_lp.to_account_info(),
            protocol_token_a_fee: ctx.accounts.protocol_token_a_fee.to_account_info(),
            protocol_token_b_fee: ctx.accounts.protocol_token_b_fee.to_account_info(),
            payer: ctx.accounts.maker.to_account_info(),
            rent: ctx.accounts.rent.to_account_info(),
            mint_metadata: ctx.accounts.mint_metadata.to_account_info(),
            metadata_program: ctx.accounts.metadata_program.to_account_info(),
            vault_program: ctx.accounts.vault_program.to_account_info(),
            token_program: ctx.accounts.token_program.to_account_info(),
            associated_token_program: ctx.accounts.associated_token_program.to_account_info(),
            lp_mint: ctx.accounts.lp_mint.to_account_info(),
            system_program: ctx.accounts.system_program.to_account_info(),
        };

    let cpi_context = CpiContext::new(
        ctx.accounts.dynamic_amm_program.to_account_info(),
        accounts,
    );

    dynamic_amm::cpi::initialize_customizable_permissionless_constant_product_pool(
        cpi_context,
        token_a_amount,
        token_b_amount,
        params,
    )
}

pub struct FundMakerAccounts<'b, 'info> {
    pub maker_token_a: &'b Account<'info, TokenAccount>,
    pub maker_token_b: &'b Account<'info, TokenAccount>,
    pub token_program: &'b Program<'info, Token>,
    pub maker: &'b Signer<'info>,
    pub system_program: &'b Program<'info, System>,
}

pub fn fund_maker<'b, 'info>(
    token_a_amount: u64,
    token_b_amount: u64,
    accounts: FundMakerAccounts<'b, 'info>,
) -> Result<()> {
    let FundMakerAccounts {
        maker_token_a,
        maker_token_b,
        token_program,
        maker,
        system_program,
    } = accounts;

    if token_a_amount > maker_token_a.amount {
        let amount = token_a_amount - maker_token_a.amount;
        anchor_spl::token::transfer(
            CpiContext::new(
                token_program.to_account_info(),
                TokenTransfer {
                    from: maker.to_account_info(),
                    to: maker_token_a.to_account_info(),
                    authority: maker.to_account_info(),
                },
            ),
            amount,
        )?;
    }

    if token_b_amount > maker_token_b.amount {
        let amount = token_b_amount - maker_token_b.amount;
        anchor_spl::token::transfer(
            CpiContext::new(
                token_program.to_account_info(),
                TokenTransfer {
                    from: maker.to_account_info(),
                    to: maker_token_b.to_account_info(),
                    authority: maker.to_account_info(),
                },
            ),
            amount,
        )?;
    }

    Ok(())
}
