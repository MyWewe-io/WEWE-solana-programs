use {
    crate::{
        constant::{SECONDS_TO_DAYS, TOTAL_AMOUNT_TO_RAISE}, errors::ProposalError, event::AirdropClaimed, state::{backers::Backers, proposal::Proposal}
    },
    anchor_lang::prelude::*,
    anchor_spl::{
        associated_token::AssociatedToken,
        token::{transfer, Mint, Token, TokenAccount, Transfer},
    },
};

#[derive(Accounts)]
pub struct Claim<'info> {
    #[account(mut)]
    pub backer: Signer<'info>,
    pub maker: SystemAccount<'info>,
    pub proposal: Account<'info, Proposal>,

    #[account(mut)]
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
            TOTAL_AMOUNT_TO_RAISE >= self.proposal.total_backing,
            ProposalError::TargetNotMet
        );

        let cpi_program = self.token_program.to_account_info();

        // Transfer the funds from the vault to the contributor
        let cpi_accounts = Transfer {
            from: self.token_vault.to_account_info(),
            to: self.backer_token_account.to_account_info(),
            authority: self.proposal.to_account_info(),
        };

        // Signer seeds to sign the CPI on behalf of the fundraiser account
        let signer_seeds: [&[&[u8]]; 1] = [&[
            b"proposal".as_ref(),
            self.maker.to_account_info().key.as_ref(),
            &[self.proposal.bump],
        ]];

        // CPI context with signer since the fundraiser account is a PDA
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, &signer_seeds);

        let claim_amount = self.backer_account.claim_amount;
        // Transfer the funds from the vault to the contributor
        transfer(cpi_ctx, claim_amount)?;

        // set claim amount to zero, for succesive airdrops
        self.backer_account.claim_amount = 0;

        emit!(AirdropClaimed {
            proposal_address: self.proposal.key(),
            backer: self.backer.key(),
            amount: claim_amount,
        });

        Ok(())
    }
}
