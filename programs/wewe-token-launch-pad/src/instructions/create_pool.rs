use std::u64;

use anchor_spl::token_interface::{TokenAccount, TokenInterface};

use crate::{
    const_pda,
    *,
};

use crate::constant::{INITIAL_POOL_LIQUIDITY, MAKER_TOKEN_AMOUNT, SECONDS_TO_DAYS};
use crate::event::CoinLaunched;
use crate::state::proposer::Proposal;
// use crate::constant::POOL_SIZE;
use anchor_lang::system_program::Transfer as NativeSolTransfer;
use anchor_spl::token::Transfer as TokenTransfer;

#[derive(Accounts)]
pub struct DammV2<'info> {
    #[account(mut)]
    pub proposal: Account<'info, Proposal>,

    #[account(
        mut, 
        token::mint = base_mint,
        token::token_program = token_base_program
    )]
    pub token_vault: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        mut, 
        token::mint = quote_mint,
        token::token_program = token_quote_program
    )]
    pub wsol_vault: Box<InterfaceAccount<'info, TokenAccount>>,

    /// CHECK: proposal maker
    pub maker: UncheckedAccount<'info>,

    #[account(
        init_if_needed,
        seeds = [b"base_vault", maker.key().as_ref(), base_mint.key().as_ref()],
        payer = payer,
        token::mint = base_mint,
        token::authority = maker,
        token::token_program = token_base_program,
        bump
    )]
    pub maker_token_account: Box<InterfaceAccount<'info, TokenAccount>>,

    /// CHECK: pool authority
    #[account(
        mut,
        address = const_pda::pool_authority::ID,
    )]
    pub pool_authority: AccountInfo<'info>,

    /// CHECK: 
    pool_config: AccountInfo<'info>,

    /// CHECK: pool
    #[account(mut)]
    pub pool: UncheckedAccount<'info>,

    /// CHECK: position nft mint for partner
    #[account(mut, signer)]
    pub first_position_nft_mint: UncheckedAccount<'info>,

    /// CHECK: position nft account for partner
    #[account(mut)]
    pub first_position_nft_account: UncheckedAccount<'info>,

    /// CHECK:
    #[account(mut)]
    pub first_position: UncheckedAccount<'info>,

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
    /// CHECK: base_vault
    #[account(
        init_if_needed,
        payer = payer,
        seeds = [b"base_vault", payer.key().as_ref(), base_mint.key().as_ref()],
        token::authority = payer,
        token::mint = base_mint,
        token::token_program = token_base_program,
        bump
    )]
    pub base_vault: Box<InterfaceAccount<'info, TokenAccount>>,
    /// CHECK: quote vault
    #[account(
        init_if_needed,
        payer = payer,
        seeds = [b"quote_vault", payer.key().as_ref(), quote_mint.key().as_ref()],
        token::authority = payer,
        token::mint = quote_mint,
        token::token_program = token_quote_program,
        bump
    )]
    pub quote_vault: Box<InterfaceAccount<'info, TokenAccount>>,
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
}

impl<'info> DammV2<'info> {
    pub fn create_pool(
        &self,
        liquidity: u128,
        sqrt_price: u128,
        bump: u8,
    ) -> Result<()> {
        // if self.proposal.is_rejected {
        //     // Check if the fundraising duration has been reached
        //     let current_time = Clock::get()?.unix_timestamp;
        //     require!(
        //         self.proposal.duration
        //             <= ((current_time - self.proposal.time_started) / SECONDS_TO_DAYS)
        //                 as u16,
        //         ProposalError::BackingNotEnded
        //     );
        // }

        let pool_authority_seeds = &[b"pool_authority".as_ref(), &[bump]];


        fund_creator_authority(
            self.proposal.get_lamports(),
            FundCreatorAuthorityAccounts {
                base_vault: &self.base_vault,
                quote_vault: &self.quote_vault,
                proposal: &self.proposal,
                token_program_a: &self.token_base_program,
                token_program_b: &self.token_quote_program,
                token_vault: &self.token_vault,
                wsol_vault: &self.wsol_vault,
                payer: &self.payer,
                system_program: &self.system_program,
                creator_authority: &self.pool_authority,
                maker: &self.maker_token_account,
            },
        )?;

        cp_amm::cpi::initialize_pool(
            CpiContext::new_with_signer(
                self.amm_program.to_account_info(),
                cp_amm::cpi::accounts::InitializePoolCtx {
                    creator: self.pool_authority.to_account_info(),
                    position_nft_mint: self.first_position_nft_mint.to_account_info(),
                    position_nft_account: self.first_position_nft_account.to_account_info(),
                    payer: self.pool_authority.to_account_info(),
                    config: self.pool_config.to_account_info(),
                    pool_authority: self.pool_authority.to_account_info(),
                    pool: self.pool.to_account_info(),
                    position: self.first_position.to_account_info(),
                    token_a_mint: self.base_mint.to_account_info(),
                    token_b_mint: self.quote_mint.to_account_info(),
                    token_a_vault: self.token_a_vault.to_account_info(),
                    token_b_vault: self.token_b_vault.to_account_info(),
                    payer_token_a: self.base_vault.to_account_info(),
                    payer_token_b: self.quote_vault.to_account_info(),
                    token_a_program: self.token_base_program.to_account_info(),
                    token_b_program: self.token_quote_program.to_account_info(),
                    token_2022_program: self.token_2022_program.to_account_info(),
                    system_program: self.system_program.to_account_info(),
                    event_authority: self.damm_event_authority.to_account_info(),
                    program: self.amm_program.to_account_info(),
                },
                &[&pool_authority_seeds[..]],
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
    });

        Ok(())
    }
}

pub struct FundCreatorAuthorityAccounts<'b, 'info> {
    pub base_vault: &'b Box<InterfaceAccount<'info, TokenAccount>>,
    pub quote_vault: &'b Box<InterfaceAccount<'info, TokenAccount>>,
    pub proposal: &'b Account<'info, Proposal>,
    pub token_program_a: &'b Interface<'info, TokenInterface>,
    pub token_program_b: &'b Interface<'info, TokenInterface>,
    pub token_vault: &'b Box<InterfaceAccount<'info, TokenAccount>>,
    pub wsol_vault: &'b Box<InterfaceAccount<'info, TokenAccount>>,
    pub payer: &'b Signer<'info>,
    pub system_program: &'b Program<'info, System>,
    pub creator_authority: &'b AccountInfo<'info>,
    pub maker: &'b Box<InterfaceAccount<'info, TokenAccount>>,
}

pub fn fund_creator_authority<'b, 'info>(
    token_b_amount: u64,
    accounts: FundCreatorAuthorityAccounts<'b, 'info>,
) -> Result<()> {
    let FundCreatorAuthorityAccounts {
        base_vault,
        quote_vault,
        proposal,
        token_program_a,
        token_program_b,
        token_vault,
        wsol_vault,
        payer,
        system_program,
        creator_authority,
        maker,
    } = accounts;

    let signer_seeds: &[&[&[u8]]] = &[&[
        b"proposal",
        proposal.maker.as_ref(),
        &proposal.proposal_id.to_le_bytes(),
        &[proposal.bump],
    ]];

    // Fund creator PDA with token A and token B
    if INITIAL_POOL_LIQUIDITY > base_vault.amount {
        let amount = INITIAL_POOL_LIQUIDITY - base_vault.amount;
        anchor_spl::token::transfer(
            CpiContext::new_with_signer(
                token_program_a.to_account_info(),
                TokenTransfer {
                    from: token_vault.to_account_info(),
                    to: base_vault.to_account_info(),
                    authority: proposal.to_account_info(),
                },
                signer_seeds,
            ),
            amount,
        )?;
    }

    if token_b_amount > quote_vault.amount {
        let amount = token_b_amount - quote_vault.amount;
        // transfer sol to token account
        anchor_spl::token::transfer(
            CpiContext::new_with_signer(
                token_program_a.to_account_info(),
                TokenTransfer {
                    from: wsol_vault.to_account_info(),
                    to: quote_vault.to_account_info(),
                    authority: proposal.to_account_info(),
                },
                signer_seeds,
            ),
            amount,
        )?;
    }

    // Transfer 1% Tokens to proposal maker
    anchor_spl::token::transfer(
        CpiContext::new_with_signer(
            token_program_b.to_account_info(),
            TokenTransfer {
                from: token_vault.to_account_info(),
                to: maker.to_account_info(),
                authority: proposal.to_account_info(),
            },
            signer_seeds,
        ),
        MAKER_TOKEN_AMOUNT,
    )?;
    // Fund creator PDA with SOL to pay for account rental
    // let mut lamports: u64 = 0;

    // // Pool
    // lamports += Rent::get()?.minimum_balance(POOL_SIZE);
    // // LP mint
    // lamports += Rent::get()?.minimum_balance(Mint::LEN);
    //  a_vault_lp + b_vault_lp + creator LP ATA + protocol fee A + protocol fee B
    // let token_account_lamports = Rent::get()?.minimum_balance(TokenAccount);
    // lamports += token_account_lamports * 5;
    // LP mint Metadata
    // lamports += Rent::get()?.minimum_balance(679);
    // // Metaplex fee ...
    // lamports += 10_000_000;

    // msg!("Required lamports: {}", lamports);

    anchor_lang::system_program::transfer(
        CpiContext::new(
            system_program.to_account_info(),
            NativeSolTransfer {
                from: payer.to_account_info(),
                to: creator_authority.to_account_info(),
            },
        ),
        // lamports,
        34290400,
    )?;

    Ok(())
}
