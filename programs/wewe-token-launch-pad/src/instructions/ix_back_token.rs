use anchor_lang::{
    prelude::*,
    system_program::{transfer, Transfer},
};
use anchor_spl::token::{Mint, TokenAccount};

use crate::{
    const_pda,
    constant::{
        seeds::{BACKER, VAULT_AUTHORITY},
        *,
    },
    errors::ProposalError,
    event::ProposalBacked,
    state::{backers::Backers, proposal::Proposal},
};

#[derive(Accounts)]
pub struct Contribute<'info> {
    #[account(mut)]
    pub backer: Signer<'info>,

    /// CHECK: protocol treasury
    #[account(mut, address = treasury::ID)]
    pub wewe_vault: UncheckedAccount<'info>,

    #[account(mut)]
    pub proposal: Account<'info, Proposal>,

    /// CHECK: vault authority
    #[account(
        mut,
        seeds = [VAULT_AUTHORITY.as_ref()],
        bump,
    )]
    pub vault_authority: SystemAccount<'info>,

    #[account(
        mut,
        address = const_pda::const_authority::MINT,
    )]
    pub mint: Account<'info, Mint>,

    #[account(
        associated_token::mint = mint,
        associated_token::authority = backer,
        constraint = user_token_account.amount == 1 @ ProposalError::NotAuthorised,
    )]
    pub user_token_account: Account<'info, TokenAccount>,

    #[account(
        init,
        payer = backer,
        seeds = [BACKER, proposal.key().as_ref(), backer.key().as_ref()],
        bump,
        space = ANCHOR_DISCRIMINATOR + Backers::INIT_SPACE,
    )]
    pub backer_account: Account<'info, Backers>,

    pub system_program: Program<'info, System>,
}

impl<'info> Contribute<'info> {
    pub fn handle_deposit_sol(&mut self) -> Result<()> {
        let now = Clock::get()?.unix_timestamp;
        let elapsed = now.saturating_sub(self.proposal.time_started);

        require!(elapsed <= SECONDS_TO_DAYS, ProposalError::BackingEnded);
        require!(!self.proposal.is_rejected, ProposalError::ProposalRejected);
        require!(
            !self.proposal.is_pool_launched,
            ProposalError::PoolAlreadyLaunched
        );
        require!(
            self.proposal.total_backers < MAXIMUM_BACKERS,
            ProposalError::BackingGoalReached
        );
        require!(
            self.backer.key() != self.proposal.maker,
            ProposalError::CantBackOwnProposal
        );

        let amount = AMOUNT_TO_RAISE_PER_USER
            .checked_sub(FEE_TO_DEDUCT)
            .ok_or(ProposalError::NumericalOverflow)?;
        let program_id = self.system_program.to_account_info();

        transfer(
            CpiContext::new(
                program_id.clone(),
                Transfer {
                    from: self.backer.to_account_info(),
                    to: self.vault_authority.to_account_info(),
                },
            ),
            amount,
        )?;

        transfer(
            CpiContext::new(
                program_id,
                Transfer {
                    from: self.backer.to_account_info(),
                    to: self.wewe_vault.to_account_info(),
                },
            ),
            FEE_TO_DEDUCT,
        )?;

        self.backer_account.initial_airdrop_received = false;
        self.proposal.total_backing = self
            .proposal
            .total_backing
            .checked_add(amount)
            .ok_or(ProposalError::NumericalOverflow)?;
        self.proposal.total_backers = self
            .proposal
            .total_backers
            .checked_add(1)
            .ok_or(ProposalError::NumericalOverflow)?;
        self.backer_account.settle_cycle = 1;

        emit!(ProposalBacked {
            backer: self.backer.key(),
            proposal_backed: self.proposal.key(),
            backer_account: self.backer_account.key(),
        });

        Ok(())
    }
}
