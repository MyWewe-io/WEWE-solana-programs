use {
    crate::{
        constant::{AMOUNT_PER_AIRDROP, AMOUNT_TO_RAISE_PER_USER, SECONDS_TO_DAYS, TOTAL_AMOUNT_TO_RAISE},
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
pub struct Claim<'info> {
    #[account(mut)]
    pub backer: Signer<'info>,
    pub maker: SystemAccount<'info>,

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
        payer = backer,
        associated_token::mint = mint_account,
        associated_token::authority = backer,
    )]
    pub backer_token_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl<'info> Claim<'info> {
    pub fn claim(&mut self) -> Result<()> {
        // Check if the fundraising duration has been reached
        let current_time = Clock::get()?.unix_timestamp;

        require!(
            self.proposal.duration
                >= ((current_time - self.proposal.time_started) / SECONDS_TO_DAYS) as u16,
            ProposalError::BackingNotEnded
        );

        require!(
            TOTAL_AMOUNT_TO_RAISE >= self.proposal.current_amount,
            ProposalError::TargetNotMet
        );

        let mut airdrop_percent =
            get_airdrop_percent(self.backer_account.reputation_score).div(100);
        if self.backer_account.claimed_cycle == 0 {
            airdrop_percent = 1;
        }

        let amount = AMOUNT_TO_RAISE_PER_USER
            .mul(AMOUNT_PER_AIRDROP)
            .div(self.proposal.backing_goal)
            .mul(airdrop_percent);

        let cpi_program = self.token_program.to_account_info();

        // Transfer the funds from the vault to the contributor
        let cpi_accounts = Transfer {
            from: self.token_vault.to_account_info(),
            to: self.backer_token_account.to_account_info(),
            authority: self.proposal.to_account_info(),
        };

        // Signer seeds to sign the CPI on behalf of the fundraiser account
        let signer_seeds: [&[&[u8]]; 1] = [&[
            b"proposer".as_ref(),
            self.maker.to_account_info().key.as_ref(),
            &[self.proposal.bump],
        ]];

        // CPI context with signer since the fundraiser account is a PDA
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, &signer_seeds);

        // Transfer the funds from the vault to the contributor
        transfer(cpi_ctx, amount)?;

        msg!("Tokens transferred successfully.");

        Ok(())
    }
}

// Get airdrop percent based on reputation score
pub fn get_airdrop_percent(reputation_score: u64) -> u64 {
    match reputation_score {
        100 => 100,
        70..=99 => 75,
        50..=69 => 50,
        30..=49 => 25,
        _ => 0,
    }
}
