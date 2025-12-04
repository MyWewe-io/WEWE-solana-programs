use anchor_lang::system_program::{transfer, Transfer};
use anchor_spl::{
    associated_token::AssociatedToken,
    token,
    token::{mint_to, MintTo, Transfer as TokenTransfer},
    token_interface::{TokenAccount, TokenInterface},
};
use cp_amm::state::Config;
use std::u64;

use crate::{
    const_pda::{self, const_authority::VAULT_BUMP},
    constant::{
        seeds::{PROPOSAL, TOKEN_VAULT, VAULT_AUTHORITY},
        *,
    },
    event::{CoinLaunched, ProposalRejected},
    state::{proposal::Proposal,config::Configs},
    *,
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
    pub vault_authority: SystemAccount<'info>,
    #[account(
        mut,
        seeds = [TOKEN_VAULT, vault_authority.key().as_ref(), base_mint.key().as_ref()],
        bump,
        token::mint = base_mint,
        token::authority = vault_authority,
        token::token_program = token_base_program,
      )]
    pub token_vault: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(
        init_if_needed,
        payer = payer,
        seeds = [TOKEN_VAULT, vault_authority.key().as_ref(), quote_mint.key().as_ref()],
        token::authority = vault_authority,
        token::mint = quote_mint,
        token::token_program = token_quote_program,
        bump,
    )]
    pub wsol_vault: Box<InterfaceAccount<'info, TokenAccount>>,
    /// CHECK: proposal maker
    #[account(constraint = maker.key() == proposal.maker @ ProposalError::IncorrectAccount)]
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
    pool_config: AccountLoader<'info, Config>,
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
    #[account(
        mut,
        constraint = base_mint.key() == proposal.mint_account @ ProposalError::IncorrectAccount
    )]
    pub base_mint: UncheckedAccount<'info>,
    /// CHECK: quote token mint
    #[account(
        mut,
        address = wsol_pubkey::ID
    )]
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
    pub config: Account<'info, Configs>,
}

impl<'info> DammV2<'info> {
    pub fn handle_create_pool(&mut self, sqrt_price: u128) -> Result<()> {
        let is_owner = self.payer.key() == chain_service_pubkey::ID;
        let is_maker = self.payer.key() == self.proposal.maker;
        require!(is_maker || is_owner, ProposalError::NotOwner);

        let now = Clock::get()?.unix_timestamp;
        let elapsed = now.saturating_sub(self.proposal.time_started);
        if elapsed >= SECONDS_TO_DAYS && self.proposal.total_backers < self.config.min_backers // MINIMUM_BACKERS 
        {
            self.proposal.is_rejected = true;
            emit!(ProposalRejected {
                maker: self.proposal.maker,
                proposal_address: self.proposal.key(),
                mint_account: self.proposal.mint_account.key(),
            });
        }
        require!(!self.proposal.is_rejected, ProposalError::ProposalRejected);
        require!(
            !self.proposal.is_pool_launched,
            ProposalError::PoolAlreadyLaunched
        );
        require!(
            self.proposal.total_backers >= self.config.min_backers, // MINIMUM_BACKERS,
            ProposalError::TargetNotMet
        );

        let signer_seeds: &[&[&[u8]]] = &[&[VAULT_AUTHORITY, &[VAULT_BUMP]]];

                // Mint tokens to token_vault if not already minted (lazy minting for cost savings)
        // Check if tokens need to be minted by checking vault balance
        if self.token_vault.amount == 0 {
            let pow = 10u64
                .checked_pow(MINT_DECIMALS as u32)
                .ok_or(ProposalError::NumericalOverflow)?;
            let amount = self.config.total_mint
                .checked_mul(pow)
                .ok_or(ProposalError::NumericalOverflow)?;
            
            // Construct proposal PDA signer seeds for minting
            let proposal_signer_seeds: &[&[&[u8]]] = &[&[
                PROPOSAL,
                self.proposal.maker.as_ref(),
                &self.proposal.proposal_id.to_le_bytes(),
                &[self.proposal.bump],
            ]];
            
            // Mint tokens to token_vault using proposal PDA as authority
            mint_to(
                CpiContext::new(
                    self.token_base_program.to_account_info(),
                    MintTo {
                        mint: self.base_mint.to_account_info(),
                        to: self.token_vault.to_account_info(),
                        authority: self.proposal.to_account_info(),
                    },
                )
                .with_signer(proposal_signer_seeds),
                amount,
            )?;
            
            // Reload token_vault after minting to get updated balance
            self.token_vault.reload()?;
        }
        

        fund_creator_authority(FundCreatorAuthorityAccounts {
            proposal: &self.proposal,
            wsol_vault: &self.wsol_vault,
            system_program: &self.system_program,
            creator_authority: &self.vault_authority,
            maker_token_account: &self.maker_token_account,
            token_program_a: &self.token_base_program,
            token_vault: &self.token_vault,
            maker_amount: self.config.maker_token_amount,
        })?;

        // Reload vault accounts after fund_creator_authority to get updated balances
        // This is necessary because fund_creator_authority modifies the vault balances
        self.token_vault.reload()?;
        self.wsol_vault.reload()?;

        // Note: Vault accounts are already validated by Anchor's account constraints
        // (PDA derivation is checked via seeds constraints)
        
        // Capture vault balances BEFORE pool creation CPI
        let token_vault_before = self.token_vault.amount;
        let wsol_vault_before = self.wsol_vault.amount;

        let config = self.pool_config.load()?;
        let base_amount: u64 = self.config.total_pool_tokens * 10u64.pow(MINT_DECIMALS as u32); // TOTAL_POOL_TOKENS * 10u64.pow(MINT_DECIMALS as u32);
        let quote_amount: u64 = self.proposal.total_backing;

        let liquidity = get_liquidity_for_adding_liquidity(
            base_amount,
            quote_amount,
            sqrt_price,
            config.sqrt_min_price,
            config.sqrt_max_price,
        )?;

        // Attempt pool creation via CPI
        cp_amm::cpi::initialize_pool(
            CpiContext::new_with_signer(
                self.amm_program.to_account_info(),
                cp_amm::cpi::accounts::InitializePoolCtx {
                    creator: self.vault_authority.to_account_info(),
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

        // Reload accounts after CPI to get updated balances for validation
        self.token_vault.reload()?;
        self.wsol_vault.reload()?;

        // Primary validation: DAMM v2 CPI success means pool was created correctly.
        // DAMM v2 validates all fund transfers, liquidity calculations, and pool state internally.
        // We perform additional validation to ensure the pool is swapable.
        
        // Validate pool account exists, is initialized, and is swapable
        // This checks: liquidity > 0, sqrt_price > 0, pool_status == enabled
        validate_pool_account_exists(&self.pool.to_account_info())?;
        
        // Verify funds were transferred (non-zero check)
        // Exact amounts are validated by DAMM v2 internally
        validate_pool_creation(
            &self.token_vault,
            &self.wsol_vault,
            token_vault_before,
            wsol_vault_before,
            base_amount,
            quote_amount,
        )?;

        // Only set flag AFTER all validations pass
        let now = Clock::get()?.unix_timestamp;
        self.proposal.is_pool_launched = true;
        self.proposal.launch_timestamp = Some(now);

        emit!(CoinLaunched {
            proposal_address: self.proposal.key(),
            mint_account: self.base_mint.key(),
            quote_mint: self.quote_mint.key(),
            total_sol_raised: self.proposal.total_backing,
            pool_address: self.pool.key(),
            token_vault: self.token_vault.key(),
            wsol_vault: self.wsol_vault.key(),
            maker: self.proposal.maker,
            maker_token_account: self.maker_token_account.key(),
            position: self.position.key(),
            position_nft_account: self.position_nft_account.key(),
            sqrt_price,
            liquidity,
        });

        Ok(())
    }
}

pub struct FundCreatorAuthorityAccounts<'b, 'info> {
    pub proposal: &'b Account<'info, Proposal>,
    pub token_program_a: &'b Interface<'info, TokenInterface>,
    pub token_vault: &'b Box<InterfaceAccount<'info, TokenAccount>>,
    pub wsol_vault: &'b Box<InterfaceAccount<'info, TokenAccount>>,
    pub system_program: &'b Program<'info, System>,
    pub creator_authority: &'b AccountInfo<'info>,
    pub maker_token_account: &'b Box<InterfaceAccount<'info, TokenAccount>>,
    pub maker_amount: u64,
}

pub fn fund_creator_authority<'b, 'info>(
    accounts: FundCreatorAuthorityAccounts<'b, 'info>,
) -> Result<()> {
    let FundCreatorAuthorityAccounts {
        proposal,
        wsol_vault,
        system_program,
        creator_authority,
        maker_token_account,
        token_program_a,
        token_vault,
        maker_amount
    } = accounts;

    let signer_seeds: &[&[&[u8]]] = &[&[VAULT_AUTHORITY, &[VAULT_BUMP]]];

    let program_id = system_program.to_account_info();
    let cpi_context = CpiContext::new_with_signer(
        program_id,
        Transfer {
            from: creator_authority.to_account_info(),
            to: wsol_vault.to_account_info(),
        },
        signer_seeds,
    );

    transfer(cpi_context, proposal.total_backing)?;

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
        maker_amount * 1 * 10u64.pow(MINT_DECIMALS as u32),
    )?;

    Ok(())
}
