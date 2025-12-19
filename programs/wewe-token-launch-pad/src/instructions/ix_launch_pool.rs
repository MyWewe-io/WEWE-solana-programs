use anchor_lang::system_program::{transfer, Transfer};
use anchor_spl::{
    associated_token::AssociatedToken,
    token,
    token::{mint_to, Mint, MintTo, Transfer as TokenTransfer},
    token_interface::{TokenAccount, TokenInterface},
};
use damm_v2_cpi::state::Config;
use damm_v2_cpi::params::fee_parameters::{BaseFeeParameters, PoolFeeParameters};
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
    /// CHECK: pool - must be derived from ["pool", config, max_mint, min_mint]
    #[account(mut)]
    pub pool: UncheckedAccount<'info>,
    /// CHECK: position nft mint for partner
    #[account(mut, signer)]
    pub position_nft_mint: UncheckedAccount<'info>,
    /// CHECK: damm pool authority
    pub damm_pool_authority: UncheckedAccount<'info>,
    /// CHECK: pool creator authority - must match config.pool_creator_authority
    pub pool_creator_authority: Signer<'info>,
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
    pub fn handle_create_pool(&mut self, _sqrt_price: u128) -> Result<()> {
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

        // Mint tokens to token vault at the start
        let proposal_signer_seeds: &[&[&[u8]]] = &[&[
            PROPOSAL,
            self.proposal.maker.as_ref(),
            &self.proposal.proposal_id.to_le_bytes(),
            &[self.proposal.bump],
        ]];
        
        // Load mint account to get decimals (address already validated by constraint)
        let mint_data = Mint::try_deserialize(&mut &self.mint_account.data.borrow()[..])?;
        // Use u128 for intermediate calculation to avoid overflow
        let pow = 10u128
            .saturating_pow(mint_data.decimals as u32);
        let amount_u128 = (self.config.total_mint as u128)
            .saturating_mul(pow);
        // Convert back to u64, saturating at u64::MAX if it doesn't fit
        let amount = amount_u128.min(u64::MAX as u128) as u64;
        
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

        // Load CP-AMM config only for pool PDA derivation and pool_creator_authority validation
        // All pool parameters come from our own Configs struct
        let pool_config = self.pool_config.load()?;
        
        // Validate pool_creator_authority matches config (or is public/default)
        require!(
            self.pool_creator_authority.key() == pool_config.pool_creator_authority || pool_config.pool_creator_authority == anchor_lang::solana_program::pubkey::Pubkey::default(),
            ProposalError::IncorrectAccount
        );
        
        // Validate pool PDA derivation matches expected seeds: ["pool", config, max_mint, min_mint]
        let min_mint = if self.base_mint.key() < self.quote_mint.key() {
            self.base_mint.key()
        } else {
            self.quote_mint.key()
        };
        let max_mint = if self.base_mint.key() > self.quote_mint.key() {
            self.base_mint.key()
        } else {
            self.quote_mint.key()
        };
        let (expected_pool, _) = anchor_lang::solana_program::pubkey::Pubkey::find_program_address(
            &[
                b"pool",
                self.pool_config.key().as_ref(),
                max_mint.as_ref(),
                min_mint.as_ref(),
            ],
            &damm_v2_cpi::ID,
        );
        require!(
            self.pool.key() == expected_pool,
            ProposalError::IncorrectAccount
        );
        
        let base_amount: u64 = self.config.total_pool_tokens * 10u64.pow(MINT_DECIMALS as u32);
        let quote_amount: u64 = self.proposal.total_backing;
        
        msg!("=== Amounts Check ===");
        msg!("Base amount (total_pool_tokens * 10^9): {}", base_amount);
        msg!("Quote amount (total_backing): {}", quote_amount);
        msg!("Token vault balance: {}", self.token_vault.amount);
        msg!("WSOL vault balance: {}", self.wsol_vault.amount);
        
        // Ensure amounts are non-zero
        if base_amount == 0 || quote_amount == 0 {
            msg!("ERROR: base_amount or quote_amount is zero!");
            return Err(ProposalError::InvalidParameters.into());
        }

        // Pool parameters: Only fee percentage is configurable, rest are hardcoded
        let cliff_fee_numerator = self.config.pool_cliff_fee_numerator;
        
        // Hardcoded values per requirements:
        // - base_fee_mode: 0 (static fee mode)
        // - first_factor/second_factor/third_factor: 0 (static fee, no rate limiting)
        // - collectFeeMode: 1 (always)
        // - dynamicFee: false (None)
        // - protocol/partner/referral fees: handled by CP-AMM program (hardcoded constants)
        // - activation_type: 0 (default)
        
        // Use MIN_SQRT_PRICE and MAX_SQRT_PRICE constants like the official Meteora reference
        // ref: https://github.com/MeteoraAg/dynamic-bonding-curve/blob/a7153a05bcdeda7e2de3d99a7fe01a21c3fce8f8/programs/dynamic-bonding-curve/src/instructions/migration/dynamic_amm_v2/migrate_damm_v2_initialize_pool.rs#L240
        let sqrt_min_price = MIN_SQRT_PRICE;
        let sqrt_max_price = MAX_SQRT_PRICE;
        
        // Calculate sqrt_price identically to Meteora reference
        // ref: https://github.com/MeteoraAg/damm-v2-sdk/blob/3d740ea8434af20a024d5d6fd08d60792dca9ca4/src/helpers/curve.ts#L36-L45
        // price = quote_amount / base_amount
        // sqrt_price = sqrt(price) * 2^64
        let float_price = quote_amount as f64 / base_amount as f64;
        let calculated_sqrt_price = (float_price.sqrt() * 2_f64.powf(64.0)) as u128;
        
        msg!("Price calculation: quote_amount={}, base_amount={}, float_price={}, calculated_sqrt_price={}", 
             quote_amount, base_amount, float_price, calculated_sqrt_price);
        
        // Clamp sqrt_price to valid range to avoid InvalidPriceRange error
        // DAMM v2 requires: MIN_SQRT_PRICE < sqrt_price < MAX_SQRT_PRICE
        // Use a reasonable margin from boundaries to ensure liquidity calculation works
        let min_safe_price = sqrt_min_price + (sqrt_max_price - sqrt_min_price) / 1000; // 0.1% from min
        let max_safe_price = sqrt_max_price - (sqrt_max_price - sqrt_min_price) / 1000; // 0.1% from max
        
        let sqrt_price = calculated_sqrt_price
            .max(min_safe_price)  // Ensure it's safely greater than min
            .min(max_safe_price); // Ensure it's safely less than max
        
        msg!("sqrt_price: calculated={}, clamped={} (min_safe={}, max_safe={}, absolute_min={}, absolute_max={})", 
             calculated_sqrt_price, sqrt_price, min_safe_price, max_safe_price, sqrt_min_price, sqrt_max_price);

        // Calculate liquidity identically to Meteora reference
        // ref: https://github.com/MeteoraAg/dynamic-bonding-curve/blob/a7153a05bcdeda7e2de3d99a7fe01a21c3fce8f8/programs/dynamic-bonding-curve/src/instructions/migration/dynamic_amm_v2/migrate_damm_v2_initialize_pool.rs#L240
        // Recreate exact function logic using safe methods
        use ruint::aliases::{U256, U512};
        
        msg!("=== Liquidity Calculation (Exact Meteora Function) ===");
        msg!("Inputs: base_amount={}, quote_amount={}, sqrt_price={}, min_sqrt_price={}, max_sqrt_price={}", 
             base_amount, quote_amount, sqrt_price, sqrt_min_price, sqrt_max_price);
        
        // fn get_initial_liquidity_from_amount_a(
        //     base_amount: u64,
        //     sqrt_max_price: u128,
        //     sqrt_price: u128,
        // ) -> Result<U512>
        let price_delta_a = U512::from(
            sqrt_max_price
                .checked_sub(sqrt_price)
                .ok_or(ProposalError::NumericalOverflow)?
        );
        let prod_a = U512::from(base_amount)
            .checked_mul(U512::from(sqrt_price))
            .ok_or(ProposalError::NumericalOverflow)?
            .checked_mul(U512::from(sqrt_max_price))
            .ok_or(ProposalError::NumericalOverflow)?;
        let liquidity_from_base = prod_a
            .checked_div(price_delta_a)
            .ok_or(ProposalError::NumericalOverflow)?;
        msg!("get_initial_liquidity_from_amount_a result: {}", liquidity_from_base);
        
        // fn get_initial_liquidity_from_amount_b(
        //     quote_amount: u64,
        //     sqrt_min_price: u128,
        //     sqrt_price: u128,
        // ) -> Result<u128>
        let price_delta_b = U256::from(
            sqrt_price
                .checked_sub(sqrt_min_price)
                .ok_or(ProposalError::NumericalOverflow)?
        );
        let quote_amount_u256 = U256::from(quote_amount)
            .checked_shl(128)
            .ok_or(ProposalError::NumericalOverflow)?;
        let liquidity_from_quote_u256 = quote_amount_u256
            .checked_div(price_delta_b)
            .ok_or(ProposalError::NumericalOverflow)?;
        let liquidity_from_quote = liquidity_from_quote_u256
            .try_into()
            .map_err(|_| ProposalError::TypeCastFailed)?;
        msg!("get_initial_liquidity_from_amount_b result: {}", liquidity_from_quote);
        
        // fn get_liquidity_for_adding_liquidity(...) -> Result<u128>
        let liquidity = if liquidity_from_base > U512::from(liquidity_from_quote) {
            msg!("Using liquidity_from_quote (smaller): {}", liquidity_from_quote);
            liquidity_from_quote
        } else {
            let liquidity_u128 = liquidity_from_base
                .try_into()
                .map_err(|_| ProposalError::TypeCastFailed)?;
            msg!("Using liquidity_from_base (smaller): {}", liquidity_u128);
            liquidity_u128
        };
        
        msg!("=== FINAL LIQUIDITY DELTA ===");
        msg!("liquidity_delta: {}", liquidity);
        
        // CRITICAL: Ensure liquidity is non-zero (liquidity delta must be > 0)
        if liquidity == 0 {
            msg!("ERROR: Calculated liquidity is ZERO! This will cause InvalidParameters error.");
            return Err(ProposalError::LiquidityCannotBeZero.into());
        }

        msg!("=== Pool Initialization Parameters ===");
        msg!("Liquidity (decimal): {}", liquidity);
        msg!("Base amount: {}, Quote amount: {}", base_amount, quote_amount);
        msg!("Sqrt price: {}, Min: {}, Max: {}", sqrt_price, sqrt_min_price, sqrt_max_price);
        msg!("Cliff fee numerator: {}", cliff_fee_numerator);
        msg!("Collect fee mode: 0");
        msg!("Base fee mode: 0, first_factor: 0, second_factor: [0;8], third_factor: 0");
        
        // Check vault balances before CPI call
        msg!("Token vault balance before CPI: {}", self.token_vault.amount);
        msg!("WSOL vault balance before CPI: {}", self.wsol_vault.amount);
        
        // msg!("About to call initialize_pool_with_dynamic_config CPI...");
        msg!("=== FINAL CPI PARAMETERS ===");
        msg!("Base amount: {}, Quote amount: {}", base_amount, quote_amount);
        msg!("Sqrt price: {}, Min: {}, Max: {}", sqrt_price, sqrt_min_price, sqrt_max_price);
        msg!("liquidity (liquidity_delta): {}", liquidity);
        msg!("cliff_fee_numerator: {}", cliff_fee_numerator);
        
        // Formula 1: L = base_amount * sqrt_price * MAX_SQRT_PRICE / (MAX_SQRT_PRICE - sqrt_price)
        let delta_base = sqrt_max_price.checked_sub(sqrt_price).unwrap_or(0);
        let base_u512 = U512::from(base_amount);
        let sqrt_price_u512 = U512::from(sqrt_price);
        let sqrt_max_price_u512 = U512::from(sqrt_max_price);
        let delta_base_u512 = U512::from(delta_base);
        
        let numerator_base = base_u512
            .checked_mul(sqrt_price_u512)
            .and_then(|x| x.checked_mul(sqrt_max_price_u512));
        let liquidity_from_base_formula = numerator_base
            .and_then(|num| num.checked_div(delta_base_u512));
        msg!("Formula 1 (from base): L = base({}) * sqrt_price({}) * max({}) / delta({})", 
             base_amount, sqrt_price, sqrt_max_price, delta_base);
        if let Some(liq) = liquidity_from_base_formula {
            msg!("  Result (U512): {}", liq);
        } else {
            msg!("  Result: None (overflow or division by zero)");
        }
        
        // Formula 2: L = (quote_amount * 2^128) / (sqrt_price - MIN_SQRT_PRICE)
        let delta_quote = sqrt_price.checked_sub(sqrt_min_price).unwrap_or(0);
        let quote_u256 = U256::from(quote_amount);
        let delta_quote_u256 = U256::from(delta_quote);
        
        let quote_shifted = quote_u256.checked_shl(128);
        let liquidity_from_quote_formula = quote_shifted
            .and_then(|shifted| shifted.checked_div(delta_quote_u256));
        msg!("Formula 2 (from quote): L = (quote({}) << 128) / delta({})", 
             quote_amount, delta_quote);
        if let Some(liq) = liquidity_from_quote_formula {
            msg!("  Result (U256): {}", liq);
            let liq_u128 = liq.to::<u128>();
            msg!("  Result as u128: {}", liq_u128);
        } else {
            msg!("  Result: None (overflow or division by zero)");
        }
        
        msg!("Final liquidity (min of both): {}", liquidity);
        
        // CRITICAL CHECK: Ensure liquidity is non-zero before CPI
        if liquidity == 0 {
            msg!("FATAL ERROR: liquidity is ZERO before CPI call! This will cause InvalidParameters.");
            return Err(ProposalError::LiquidityCannotBeZero.into());
        }

        // Attempt pool creation via CPI
        damm_v2_cpi::cpi::initialize_pool_with_dynamic_config(
            CpiContext::new_with_signer(
                self.amm_program.to_account_info(),
                damm_v2_cpi::cpi::accounts::InitializePoolWithDynamicConfigCtx {
                    creator: self.vault_authority.to_account_info(),
                    position_nft_mint: self.position_nft_mint.to_account_info(),
                    position_nft_account: self.position_nft_account.to_account_info(),
                    payer: self.vault_authority.to_account_info(),
                    pool_creator_authority: self.pool_creator_authority.to_account_info(),
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
                        cliff_fee_numerator: cliff_fee_numerator,
                        first_factor: 0, // number_of_period = 0 for static fees
                        second_factor: [0u8; 8], // period_frequency = 0 (static)
                        third_factor: 0u64, // reduction_factor = 0 (static)
                        base_fee_mode: 0, // FeeSchedulerLinear mode - static fee when all params are 0
                    },
                    padding: [0; 3],
                    dynamic_fee: None,
                },
                sqrt_min_price: sqrt_min_price,
                sqrt_max_price: sqrt_max_price,
                has_alpha_vault: false,
                liquidity,
                sqrt_price,
                activation_type: 0,
                collect_fee_mode: 0, // BothToken mode (matches reference implementation)
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
