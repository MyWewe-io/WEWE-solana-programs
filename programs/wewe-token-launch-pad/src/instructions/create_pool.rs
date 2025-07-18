use anchor_lang::system_program::{transfer, Transfer};
use anchor_spl::{
    associated_token::AssociatedToken,
    token,
    token::Transfer as TokenTransfer,
    token_interface::{TokenAccount, TokenInterface},
};
use std::{ops::Sub, u64};

use crate::const_pda::const_authority::VAULT_BUMP;
use crate::state::proposal::Proposal;
use crate::{const_pda, *};
use crate::{
    constant::*,
    event::{CoinLaunched, ProposalRejected},
};

#[derive(Accounts)]
pub struct DammV2<'info> {
    #[account(mut)]
    pub proposal: Account<'info, Proposal>,
    /// CHECK: vault authority
    #[account(
        mut,
        seeds = [
            VAULT_AUTHORITY.as_ref(),
        ],
        bump,
    )]
    pub vault_authority: UncheckedAccount<'info>,
    #[account(mut)]
    pub token_vault: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(
        init_if_needed,
        payer = payer,
        seeds = [b"token_vault", vault_authority.key().as_ref(), quote_mint.key().as_ref()],
        token::authority = vault_authority,
        token::mint = quote_mint,
        token::token_program = token_quote_program,
        bump,
    )]
    pub wsol_vault: Box<InterfaceAccount<'info, TokenAccount>>,
    /// CHECK: proposal maker token account
    pub maker: UncheckedAccount<'info>,
    #[account(
        init_if_needed,
        payer = payer,
        associated_token::mint = base_mint,
        associated_token::authority = maker,
        associated_token::token_program = token_base_program,
    )]
    pub maker_token_account: Box<InterfaceAccount<'info, TokenAccount>>,
    /// CHECK: pool authority
    #[account(
        mut,
        address = const_pda::const_authority::POOL_ID,
    )]
    pub pool_authority: AccountInfo<'info>,
    /// CHECK: pool config
    pool_config: AccountInfo<'info>,
    /// CHECK: pool
    #[account(mut)]
    pub pool: UncheckedAccount<'info>,
    /// CHECK: position nft mint for partner
    #[account(mut, signer)]
    pub position_nft_mint: UncheckedAccount<'info>,
    /// CHECK: damm pool authority
    pub damm_pool_authority: UncheckedAccount<'info>,
    /// CHECK: position nft account for partner
    #[account(mut)]
    pub position_nft_account: UncheckedAccount<'info>,
    /// CHECK:
    #[account(mut)]
    pub position: UncheckedAccount<'info>,
    /// CHECK:
    #[account(address = cp_amm::ID)]
    pub amm_program: UncheckedAccount<'info>,
    /// CHECK: base token mint
    #[account(mut)]
    pub base_mint: UncheckedAccount<'info>,
    /// CHECK: quote token mint
    #[account(mut)]
    pub quote_mint: UncheckedAccount<'info>,
    /// CHECK:
    #[account(mut)]
    pub token_a_vault: UncheckedAccount<'info>,
    /// CHECK:
    #[account(mut)]
    pub token_b_vault: UncheckedAccount<'info>,
    /// CHECK: payer
    #[account(mut)]
    pub payer: Signer<'info>,
    /// CHECK: token_program
    pub token_base_program: Interface<'info, TokenInterface>,
    /// CHECK: token_program
    pub token_quote_program: Interface<'info, TokenInterface>,
    /// CHECK: token_program
    pub token_2022_program: Interface<'info, TokenInterface>,
    /// CHECK: damm event authority
    pub damm_event_authority: UncheckedAccount<'info>,
    /// System program.
    pub system_program: Program<'info, System>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

impl<'info> DammV2<'info> {
    pub fn create_pool(&mut self) -> Result<()> {
        let is_owner = self.payer.key() == OWNER;
        let is_maker = self.payer.key() == self.proposal.maker;
        require!(is_maker || is_owner, ProposalError::NotOwner);

        let current_time = Clock::get()?.unix_timestamp;
        let time_passed = current_time - self.proposal.time_started;

        if time_passed >= SECONDS_TO_DAYS && self.proposal.total_backers < MINIMUM_BACKERS {
            self.proposal.is_rejected = true;
            emit!(ProposalRejected {
                maker: self.proposal.maker,
                proposal_address: self.proposal.key(),
            });
        }

        require!(!self.proposal.is_rejected, ProposalError::ProposalRejected);
        require!(
            !self.proposal.is_pool_launched,
            ProposalError::PoolAlreadyLaunched
        );
        require!(
            self.proposal.total_backers >= MINIMUM_BACKERS,
            ProposalError::TargetNotMet
        );

        let signer_seeds: &[&[&[u8]]] = &[&[b"vault_authority", &[VAULT_BUMP]]];
    
        fund_creator_authority(FundCreatorAuthorityAccounts {
            proposal: &self.proposal,
            wsol_vault: &self.wsol_vault,
            payer: &self.payer,
            system_program: &self.system_program,
            creator_authority: &self.vault_authority,
            maker_token_account: &self.maker_token_account,
            token_program_a: &self.token_base_program,
            token_vault: &self.token_vault,
        })?;

        let base_amount: u128 = TOTAL_POOL_TOKENS as u128;
        let quote_amount: u128 = self.proposal.total_backing as u128;

        let liquidity = integer_sqrt(base_amount.checked_mul(quote_amount).unwrap());

        let ratio = (quote_amount << 64) / base_amount;
        let sqrt_price = integer_sqrt(ratio);

        cp_amm::cpi::initialize_pool(
            CpiContext::new_with_signer(
                self.amm_program.to_account_info(),
                cp_amm::cpi::accounts::InitializePoolCtx {
                    creator: self.pool_authority.to_account_info(),
                    position_nft_mint: self.position_nft_mint.to_account_info(),
                    position_nft_account: self.position_nft_account.to_account_info(),
                    payer: self.vault_authority.to_account_info(),
                    config: self.pool_config.to_account_info(),
                    pool_authority: self.damm_pool_authority.to_account_info(),
                    pool: self.pool.to_account_info(),
                    position: self.position.to_account_info(),
                    token_a_mint: self.base_mint.to_account_info(),
                    token_b_mint: self.quote_mint.to_account_info(),
                    token_a_vault: self.token_a_vault.to_account_info(),
                    token_b_vault: self.token_b_vault.to_account_info(),
                    payer_token_a: self.token_vault.to_account_info(),
                    payer_token_b: self.wsol_vault.to_account_info(),
                    token_a_program: self.token_base_program.to_account_info(),
                    token_b_program: self.token_quote_program.to_account_info(),
                    token_2022_program: self.token_2022_program.to_account_info(),
                    system_program: self.system_program.to_account_info(),
                    event_authority: self.damm_event_authority.to_account_info(),
                    program: self.amm_program.to_account_info(),
                },
                signer_seeds,
            ),
            cp_amm::InitializePoolParameters {
                liquidity,
                sqrt_price,
                activation_point: None,
            },
        )?;

        emit!(CoinLaunched {
            proposal_address: self.proposal.key(),
            mint_account: self.base_mint.key(),
            total_sol_raised: self.proposal.total_backing,
            pool_address: self.pool.key(),
        });

        Ok(())
    }
}

pub struct FundCreatorAuthorityAccounts<'b, 'info> {
    pub proposal: &'b Account<'info, Proposal>,
    pub token_program_a: &'b Interface<'info, TokenInterface>,
    pub token_vault: &'b Box<InterfaceAccount<'info, TokenAccount>>,
    pub wsol_vault: &'b Box<InterfaceAccount<'info, TokenAccount>>,
    pub payer: &'b Signer<'info>,
    pub system_program: &'b Program<'info, System>,
    pub creator_authority: &'b AccountInfo<'info>,
    pub maker_token_account: &'b Box<InterfaceAccount<'info, TokenAccount>>,
}

pub fn fund_creator_authority<'b, 'info>(
    accounts: FundCreatorAuthorityAccounts<'b, 'info>,
) -> Result<()> {
    let FundCreatorAuthorityAccounts {
        proposal,
        wsol_vault,
        payer,
        system_program,
        creator_authority,
        maker_token_account,
        token_program_a,
        token_vault,
    } = accounts;

    let signer_seeds: &[&[&[u8]]] = &[&[b"vault_authority", &[VAULT_BUMP]]];

    let wsol_amount = wsol_vault.amount;

    let program_id = system_program.to_account_info();
    let cpi_context = CpiContext::new_with_signer(
        program_id,
        Transfer {
            from: creator_authority.to_account_info(),
            to: wsol_vault.to_account_info(),
        },
        signer_seeds,
    );

    transfer(cpi_context, proposal.total_backing.sub(wsol_amount))?;

    // Sync the native token to reflect the new SOL balance as wSOL
    let cpi_accounts = token::SyncNative {
        account: wsol_vault.to_account_info(),
    };
    let cpi_program = wsol_vault.to_account_info();
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
    token::sync_native(cpi_ctx)?;

    // Transfer 1% Tokens to proposal maker
    anchor_spl::token::transfer(
        CpiContext::new_with_signer(
            token_program_a.to_account_info(),
            TokenTransfer {
                from: token_vault.to_account_info(),
                to: maker_token_account.to_account_info(),
                authority: creator_authority.to_account_info(),
            },
            signer_seeds,
        ),
        MAKER_TOKEN_AMOUNT * 10u64.pow(9 as u32),
    )?;

    let program_id = system_program.to_account_info();
    let cpi_context = CpiContext::new(
        program_id,
        Transfer {
            from: payer.to_account_info(),
            to: creator_authority.to_account_info(),
        },
    );

    transfer(cpi_context, 50_000_000)?;

    Ok(())
}

fn integer_sqrt(value: u128) -> u128 {
    if value == 0 {
        return 0;
    }
    let mut z = value;
    let mut x = (value >> 1) + 1;
    while x < z {
        z = x;
        x = (value / x + x) >> 1;
    }
    z
}
