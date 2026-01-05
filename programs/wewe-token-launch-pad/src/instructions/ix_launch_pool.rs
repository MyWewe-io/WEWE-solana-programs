use anchor_lang::system_program::{transfer, Transfer};
use anchor_spl::{
    associated_token::AssociatedToken,
    token,
    token::{mint_to, Mint, MintTo, Transfer as TokenTransfer},
    token_interface::{TokenAccount, TokenInterface},
};
use damm_v2_cpi::{params::fee_parameters::{BaseFeeParameters, PoolFeeParameters}};
use damm_v2_cpi::constants::{MIN_SQRT_PRICE, MAX_SQRT_PRICE};
use std::u64;

use crate::{
    const_pda::{self, const_authority::VAULT_BUMP},
    constant::{
        seeds::{PROPOSAL, TOKEN_VAULT, VAULT_AUTHORITY},
        *,
    },
    event::{CoinLaunched, ProposalRejected},
    state::{proposal::Proposal,config::Configs},
    utils::pool_liqudity::get_liquidity_delta,
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
    pool_config: UncheckedAccount<'info>,
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
    #[account(address = damm_v2_cpi::ID)]
    pub amm_program: UncheckedAccount<'info>,
    /// CHECK: base token mint
    #[account(
        mut,
        constraint = base_mint.key() == proposal.mint_account @ ProposalError::IncorrectAccount
    )]
    pub base_mint: UncheckedAccount<'info>,
    /// Mint account for minting tokens - derived from proposal.mint_account
    /// CHECK: Address validated against proposal, deserialized manually
    #[account(
        mut,
        address = proposal.mint_account @ ProposalError::IncorrectAccount
    )]
    pub mint_account: UncheckedAccount<'info>,
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
    /// CHECK: chain service pubkey (constrained to constant, not passed as parameter)
    #[account(
        address = chain_service_pubkey::ID @ ProposalError::NotOwner
    )]
    pub chain_service_pubkey: AccountInfo<'info>,
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
        // Chain service pubkey must be the signer (payer) and pool_creator_authority
        require!(
            self.payer.key() == chain_service_pubkey::ID,
            ProposalError::NotOwner
        );

        let now = Clock::get()?.unix_timestamp;
        let elapsed = now.saturating_sub(self.proposal.time_started);
        if elapsed >= SECONDS_TO_DAYS * 3 && self.proposal.total_backers < self.config.min_backers // MINIMUM_BACKERS 
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

        // Mint tokens to token vault at the start
        let proposal_signer_seeds: &[&[&[u8]]] = &[&[
            PROPOSAL,
            self.proposal.maker.as_ref(),
            &self.proposal.proposal_id.to_le_bytes(),
            &[self.proposal.bump],
        ]];
        
        // Load mint account to get decimals (address already validated by constraint)
        let mint_data = Mint::try_deserialize(&mut &self.mint_account.data.borrow()[..])?;
        let pow = 10u64
            .checked_pow(mint_data.decimals as u32)
            .ok_or(ProposalError::NumericalOverflow)?;
        let amount = self.config.total_mint
            .checked_mul(pow)
            .ok_or(ProposalError::NumericalOverflow)?;
        
        mint_to(
            CpiContext::new(
                self.token_base_program.to_account_info(),
                MintTo {
                    mint: self.mint_account.to_account_info(),
                    to: self.token_vault.to_account_info(),
                    authority: self.proposal.to_account_info(),
                },
            )
            .with_signer(proposal_signer_seeds),
            amount,
        )?;

        // Reload token vault after minting to get updated balance
        self.token_vault.reload()?;

        let signer_seeds: &[&[&[u8]]] = &[&[VAULT_AUTHORITY, &[VAULT_BUMP]]];
        

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

        // Reload vaults after funding to get updated balances
        self.token_vault.reload()?;
        self.wsol_vault.reload()?;

        // Note: Vault accounts are already validated by Anchor's account constraints
        // (PDA derivation is checked via seeds constraints)
        
        // Capture vault balances BEFORE pool creation CPI (after minting and funding)
        let token_vault_before = self.token_vault.amount;
        let wsol_vault_before = self.wsol_vault.amount;

        // Calculate token amounts (matching SDK flow: tokenAAmount and tokenBAmount)
        // tokenAAmount = total_pool_tokens * 10^decimals (base token amount)
        let base_amount: u64 = self.config.total_pool_tokens * 10u64.pow(MINT_DECIMALS as u32);
        // tokenBAmount = total_backing in lamports (quote token amount, WSOL)
        let quote_amount: u64 = self.proposal.total_backing;

        // Use MIN_SQRT_PRICE and MAX_SQRT_PRICE constants (matching SDK: MIN_SQRT_PRICE, MAX_SQRT_PRICE)
        // The config values may be 0 or uninitialized, but we need the actual constants
        let sqrt_min_price = MIN_SQRT_PRICE;
        let sqrt_max_price = MAX_SQRT_PRICE;

        // Calculate liquidity delta (matching SDK: cpAmm.getLiquidityDelta)
        // This calculates the minimum liquidity from both token amounts to ensure balanced pool
        // Formula: min(
        //   L_base = base_amount * sqrt_price * sqrt_max_price / (sqrt_max_price - sqrt_price),
        //   L_quote = quote_amount * 2^128 / (sqrt_price - sqrt_min_price)
        // )
        let liquidity = get_liquidity_delta(
            base_amount,
            quote_amount,
            sqrt_price,
            sqrt_min_price,
            sqrt_max_price,
        )?;

        msg!("liquidity: {}", liquidity);

        // require!(false, ProposalError::NumericalOverflow);

        // Create pool via CPI (matching SDK: cpAmm.createCustomPool)
        // Parameters match SDK flow:
        // - tokenAAmount -> base_amount
        // - tokenBAmount -> quote_amount
        // - initSqrtPrice -> sqrt_price (calculated from price in test)
        // - liquidityDelta -> liquidity (calculated above)
        // - sqrtMinPrice -> MIN_SQRT_PRICE
        // - sqrtMaxPrice -> MAX_SQRT_PRICE
        // - poolFees -> configured fee parameters
        // - activationType -> 1 (Timestamp)
        // - collectFeeMode -> 1 (BothToken)
        damm_v2_cpi::cpi::initialize_pool_with_dynamic_config(
            CpiContext::new_with_signer(
                self.amm_program.to_account_info(),
                damm_v2_cpi::cpi::accounts::InitializePoolWithDynamicConfigCtx {
                    creator: self.vault_authority.to_account_info(),
                    position_nft_mint: self.position_nft_mint.to_account_info(),
                    position_nft_account: self.position_nft_account.to_account_info(),
                    payer: self.vault_authority.to_account_info(),
                    pool_creator_authority: self.chain_service_pubkey.to_account_info(),
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
            damm_v2_cpi::InitializeCustomizablePoolParameters {
                pool_fees: PoolFeeParameters {
                    base_fee: BaseFeeParameters {
                        cliff_fee_numerator: 990_000_000, // 50% fee (denominator)
                        base_fee_mode: damm_v2_cpi::state::fee::BaseFeeMode::FeeSchedulerLinear as u8,
                        first_factor: 100,
                        second_factor: 1u64.to_le_bytes(),
                        third_factor: 9_700_000, // Ending Fee = Cliff Fee Numerator − (Number Of Periods × Reduction Factor)
                        /*
                            firstFactor: number // numberOfPeriod
                            secondFactor: BN // periodFrequency
                            thirdFactor: BN // reductionFactor
                            baseFeeMode: BaseFeeMode // 0 or 1
                         */

                    },
                    dynamic_fee: None,
                    ..Default::default()
                },
                sqrt_min_price: MIN_SQRT_PRICE,
                sqrt_max_price: MAX_SQRT_PRICE,
                has_alpha_vault: false, // Matching SDK: hasAlphaVault: false
                liquidity, // liquidityDelta from SDK
                sqrt_price, // initSqrtPrice from SDK
                activation_type: 1, // Matching SDK: ActivationType.Timestamp
                collect_fee_mode: 1, 
                activation_point: None, // Matching SDK: activationPoint: null
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