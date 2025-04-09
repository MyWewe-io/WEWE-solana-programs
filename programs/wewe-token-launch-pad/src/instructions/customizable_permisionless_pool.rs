use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount, Transfer as TokenTransfer};
use crate::dlmm;

#[derive(Accounts)]
pub struct InitializeLbPair<'info> {
    #[account(mut)]
    pub lb_pair: UncheckedAccount<'info>,

    #[account(mut)]
    pub bin_array_bitmap_extension: UncheckedAccount<'info>,

    #[account(mut)]
    pub token_mint_x: UncheckedAccount<'info>,

    pub token_mint_y: UncheckedAccount<'info>,

    #[account(mut)]
    pub reserve_x: UncheckedAccount<'info>,

    #[account(mut)]
    pub reserve_y: UncheckedAccount<'info>,

    #[account(mut)]
    pub oracle: UncheckedAccount<'info>,

    #[account(mut)]
    pub user_token_x: UncheckedAccount<'info>,

    #[account(mut)]
    pub preset_parameter: UncheckedAccount<'info>,

    #[account(mut)]
    pub funder: Signer<'info>,

    // #[account(address = Token2022::id())]
    pub token_program: UncheckedAccount<'info>,

    pub system_program: UncheckedAccount<'info>,

    pub event_authority: UncheckedAccount<'info>,

    pub rent: Sysvar<'info, Rent>,

    pub program: UncheckedAccount<'info>,
}

pub fn handle_initialize_pool_from_proposer_creator<'a, 'b, 'c, 'info>(
    ctx: Context<'a, 'b, 'c, 'info, InitializeLbPair<'info>>,
    token_a_amount: u64,
    token_b_amount: u64,
) -> Result<()> {

    let accounts =
        dlmm::cpi::accounts::InitializeLbPair{
            lb_pair: ctx.accounts.lb_pair.to_account_info(),
            bin_array_bitmap_extension: Some(ctx.accounts.bin_array_bitmap_extension.to_account_info()),
            token_mint_x: ctx.accounts.token_mint_x.to_account_info(),
            token_mint_y: ctx.accounts.token_mint_y.to_account_info(),
            reserve_x: ctx.accounts.reserve_x.to_account_info(),
            reserve_y: ctx.accounts.reserve_y.to_account_info(),
            oracle: ctx.accounts.oracle.to_account_info(),
            funder: ctx.accounts.funder.to_account_info(),
            token_program: ctx.accounts.token_program.to_account_info(),
            system_program: ctx.accounts.system_program.to_account_info(),
            rent: ctx.accounts.rent.to_account_info(),
            event_authority: ctx.accounts.event_authority.to_account_info(),
            program: ctx.accounts.program.to_account_info(),
            preset_parameter: ctx.accounts.preset_parameter.to_account_info(),
        };

    let cpi_context = CpiContext::new(ctx.accounts.program.to_account_info(), accounts)
    .with_remaining_accounts(ctx.remaining_accounts.to_vec());
    
    dlmm::cpi::initialize_lb_pair(
        cpi_context, 1, 2 )
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
