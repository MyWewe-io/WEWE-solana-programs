use {
    crate::{
        constant::{SECONDS_TO_DAYS, TOTAL_AMOUNT_TO_RAISE, TOTAL_MINT},
        errors::ProposalError,
        state::{backers::Backers, proposer::Proposal},
    },
    anchor_lang::prelude::*,
    anchor_spl::{
        associated_token::AssociatedToken,
        token::{transfer, Mint, Token, TokenAccount, Transfer},
    },
    std::ops::{Div, Mul},
};

#[derive(Accounts)]
pub struct TransferTokens<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    pub maker: SystemAccount<'info>,
    pub backer: SystemAccount<'info>,

    #[account(
        seeds = [b"proposer", maker.key().as_ref()],
        bump,
    )]
    pub proposal: Account<'info, Proposal>,

    #[account(
        mut,
        seeds = [b"mint", maker.key().as_ref()],
        bump
    )]
    pub mint_account: Account<'info, Mint>,

    #[account(
        associated_token::mint = mint_account,
        associated_token::authority = proposal,
    )]
    pub token_vault: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [b"backer", proposal.key().as_ref(), backer.key().as_ref()],
        bump,
        close=backer,
    )]
    pub backer_account: Account<'info, Backers>,

    #[account(
        init_if_needed,
        payer = payer,
        associated_token::mint = mint_account,
        associated_token::authority = backer,
    )]
    pub backer_token_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

pub fn transfer_tokens(ctx: Context<TransferTokens>) -> Result<()> {
    // Check if the fundraising duration has been reached
    let current_time = Clock::get()?.unix_timestamp;

    require!(
        ctx.accounts.proposal.duration
            >= ((current_time - ctx.accounts.proposal.time_started) / SECONDS_TO_DAYS) as u16,
        ProposalError::BackingNotEnded
    );

    require!(
        TOTAL_AMOUNT_TO_RAISE >= ctx.accounts.proposal.current_amount,
        ProposalError::TargetNotMet
    );

    let amount = ctx
        .accounts
        .backer_account
        .amount
        .mul(TOTAL_MINT)
        .div(ctx.accounts.proposal.backing_goal);

    let cpi_program = ctx.accounts.token_program.to_account_info();

    // Transfer the funds from the vault to the contributor
    let cpi_accounts = Transfer {
        from: ctx.accounts.token_vault.to_account_info(),
        to: ctx.accounts.backer_token_account.to_account_info(),
        authority: ctx.accounts.proposal.to_account_info(),
    };

    // Signer seeds to sign the CPI on behalf of the fundraiser account
    let signer_seeds: [&[&[u8]]; 1] = [&[
        b"proper".as_ref(),
        ctx.accounts.maker.to_account_info().key.as_ref(),
        &[ctx.accounts.proposal.bump],
    ]];

    // CPI context with signer since the fundraiser account is a PDA
    let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, &signer_seeds);

    // Transfer the funds from the vault to the contributor
    transfer(cpi_ctx, amount)?;

    msg!("Tokens transferred successfully.");

    Ok(())
}
