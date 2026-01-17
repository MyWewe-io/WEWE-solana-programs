use anchor_lang::prelude::*;
use anchor_lang::system_program;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{close_account, initialize_account, CloseAccount, InitializeAccount, Token, ID as TOKEN_PROGRAM_ID},
    token_interface::{TokenAccount, TokenInterface},
};

use crate::{
    const_pda::{self, const_authority::VAULT_BUMP},
    constant::{seeds::VAULT_AUTHORITY, seeds::TOKEN_VAULT, treasury},
    errors::ProposalError,
    event::PositionFeeClaimed,
    state::proposal::Proposal,
};

#[derive(Accounts)]
pub struct ClaimPositionFee<'info> {
    /// CHECK: pool authority
    #[account(
        mut,
        address = const_pda::const_authority::POOL_ID,
    )]
    pub pool_authority: UncheckedAccount<'info>,

    #[account(mut)]
    pub payer: Signer<'info>,

    /// CHECK: maker of the propposal (needs to be mutable for SOL transfers)
    #[account(
        mut,
        constraint = maker.key() == proposal.maker @ ProposalError::NotOwner
    )]
    pub maker: UncheckedAccount<'info>,

    /// CHECK: owner of the propposal (needs to be mutable for SOL transfers)
    #[account(mut, address = treasury::ID)]
    pub wewe_treasury: UncheckedAccount<'info>,

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

    /// CHECK:
    pub token_a_mint: UncheckedAccount<'info>,

    /// CHECK: Token B mint (should be WSOL_MINT for unwrapping to work)
    /// The temporary account must be initialized with this mint to be a WSOL token account
    pub token_b_mint: UncheckedAccount<'info>,

    /// WSOL account - can be owned by treasury or vault_authority (we'll use as temp account)
    #[account(
        init_if_needed,
        payer = payer,
        associated_token::mint = token_b_mint,
        associated_token::authority = wewe_treasury,
    )]
    pub wewe_wsol_account: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        init_if_needed,
        payer = payer,
        associated_token::mint = token_a_mint,
        associated_token::authority = wewe_treasury,
    )]
    pub wewe_token_account: Box<InterfaceAccount<'info, TokenAccount>>,

    /// WSOL account - can be owned by maker or vault_authority (we'll use as temp account)
    #[account(
        init_if_needed,
        payer = payer,
        associated_token::authority = maker,
        associated_token::mint = token_b_mint,
    )]
    pub maker_wsol_account: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        init_if_needed,
        payer = payer,
        associated_token::mint = token_a_mint,
        associated_token::authority = maker,
    )]
    pub maker_token_account: Box<InterfaceAccount<'info, TokenAccount>>,

    /// CHECK: pool address
    pub pool: UncheckedAccount<'info>,

    /// CHECK: position address (owned by CP-AMM program, not our program)
    /// Using UncheckedAccount with mut - need mutability for CPI
    /// Adding constraint to explicitly allow CP-AMM ownership to bypass Anchor's owner validation
    #[account(mut)]
    pub position: UncheckedAccount<'info>,

    /// The user token a account - vault for token A
    #[account(
        mut,
        seeds = [TOKEN_VAULT, vault_authority.key().as_ref(), token_a_mint.key().as_ref()],
        bump,
    )]
    pub token_a_account: Box<InterfaceAccount<'info, TokenAccount>>,

    /// The user token b account - vault for token B  
    #[account(
        mut,
        seeds = [TOKEN_VAULT, vault_authority.key().as_ref(), token_b_mint.key().as_ref()],
        bump,
    )]
    pub token_b_account: Box<InterfaceAccount<'info, TokenAccount>>,

    /// The vault token account for input token
    #[account(mut)]
    pub token_a_vault: Box<InterfaceAccount<'info, TokenAccount>>,

    /// The vault token account for output token
    #[account(mut)]
    pub token_b_vault: Box<InterfaceAccount<'info, TokenAccount>>,

    /// CHECK:
    pub position_nft_account: UncheckedAccount<'info>,

    /// CHECK: Temporary WSOL account for treasury unwrapping (PDA derived from our program)
    /// PDA: [b"temp_wsol", vault_authority, proposal, b"treasury"]
    /// Changed to use our program ID so we can create it with invoke_signed
    #[account(
        mut,
        constraint = {
            let (expected_pda, _) = Pubkey::find_program_address(
                &[
                    b"temp_wsol",
                    vault_authority.key().as_ref(),
                    proposal.key().as_ref(),
                    b"treasury",
                ],
                &crate::ID, // Use our program ID so we can create it
            );
            treasury_temp_wsol.key() == expected_pda
        } @ ProposalError::IncorrectAccount
    )]
    pub treasury_temp_wsol: UncheckedAccount<'info>,

    /// CHECK: Temporary WSOL account for maker unwrapping (PDA derived from our program)
    /// PDA: [b"temp_wsol", vault_authority, proposal, b"maker"]
    /// Changed to use our program ID so we can create it with invoke_signed
    #[account(
        mut,
        constraint = {
            let (expected_pda, _) = Pubkey::find_program_address(
                &[
                    b"temp_wsol",
                    vault_authority.key().as_ref(),
                    proposal.key().as_ref(),
                    b"maker",
                ],
                &crate::ID, // Use our program ID so we can create it
            );
            maker_temp_wsol.key() == expected_pda
        } @ ProposalError::IncorrectAccount
    )]
    pub maker_temp_wsol: UncheckedAccount<'info>,

    pub token_a_program: Interface<'info, TokenInterface>,

    pub token_b_program: Interface<'info, TokenInterface>,

    /// CHECK: amm program address
    #[account(address = damm_v2_cpi::ID)]
    pub amm_program: UncheckedAccount<'info>,

    /// CHECK:
    pub event_authority: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>,
}

impl<'info> ClaimPositionFee<'info> {
    pub fn handle_claim_position_fee(&mut self) -> Result<()> {
        // Access control: The maker account is already validated in Accounts struct
        // Anyone can call this function as long as they provide the correct maker account
        // The maker constraint ensures only the correct maker can be specified
        
        let vault_authority_seeds: &[&[u8]] = &[VAULT_AUTHORITY, &[VAULT_BUMP]];

        let pre_a = self.token_a_account.amount;
        let pre_b = self.token_b_account.amount;

        msg!("Claiming position fees - Token A before: {}, Token B (WSOL) before: {}", pre_a, pre_b);
        damm_v2_cpi::cpi::claim_position_fee(CpiContext::new_with_signer(
            self.amm_program.to_account_info(),
            damm_v2_cpi::cpi::accounts::ClaimPositionFeeCtx {
                pool_authority: self.pool_authority.to_account_info(),
                pool: self.pool.to_account_info(),
                position: self.position.to_account_info(),
                token_a_account: self.token_a_account.to_account_info(),
                token_b_account: self.token_b_account.to_account_info(),
                token_a_vault: self.token_a_vault.to_account_info(),
                token_b_vault: self.token_b_vault.to_account_info(),
                token_a_mint: self.token_a_mint.to_account_info(),
                token_b_mint: self.token_b_mint.to_account_info(),
                position_nft_account: self.position_nft_account.to_account_info(),
                owner: self.vault_authority.to_account_info(),
                token_a_program: self.token_a_program.to_account_info(),
                token_b_program: self.token_b_program.to_account_info(),
                event_authority: self.event_authority.to_account_info(),
                program: self.amm_program.to_account_info(),
            },
            &[&vault_authority_seeds[..]],
        ))?;

        self.token_a_account.reload()?;
        self.token_b_account.reload()?;

        let claimed_token_a = self.token_a_account.amount.saturating_sub(pre_a);
        let claimed_token_b = self.token_b_account.amount.saturating_sub(pre_b);

        msg!("Claimed fees - Token A: {}, Token B (WSOL): {}", claimed_token_a, claimed_token_b);

        // If nothing was claimed, we're done
        if claimed_token_a == 0 && claimed_token_b == 0 {
            msg!("No fees to distribute");
            return Ok(());
        }

        #[inline]
        fn split_even(amount: u64) -> (u64, u64) {
            let half = amount / 2;
            let remainder = amount % 2;
            (half + remainder, half)
        }

        let (treasury_a, maker_a) = split_even(claimed_token_a);
        let (treasury_b, maker_b) = split_even(claimed_token_b);

        msg!("Fee split - Token A: treasury={}, maker={}", treasury_a, maker_a);
        msg!("Fee split - Token B (WSOL): treasury={}, maker={}", treasury_b, maker_b);

        if treasury_a > 0 {
            msg!("Transferring {} token A to treasury", treasury_a);
            anchor_spl::token::transfer(
                CpiContext::new_with_signer(
                    self.token_a_program.to_account_info(),
                    anchor_spl::token::Transfer {
                        from: self.token_a_account.to_account_info(),
                        to: self.wewe_token_account.to_account_info(),
                        authority: self.vault_authority.to_account_info(),
                    },
                    &[&vault_authority_seeds[..]],
                ),
                treasury_a,
            )?;
            msg!("✅ Transferred {} token A to treasury", treasury_a);
        }

        if maker_a > 0 {
            msg!("Transferring {} token A to maker", maker_a);
            anchor_spl::token::transfer(
                CpiContext::new_with_signer(
                    self.token_a_program.to_account_info(),
                    anchor_spl::token::Transfer {
                        from: self.token_a_account.to_account_info(),
                        to: self.maker_token_account.to_account_info(),
                        authority: self.vault_authority.to_account_info(),
                    },
                    &[&vault_authority_seeds[..]],
                ),
                maker_a,
            )?;
            msg!("✅ Transferred {} token A to maker", maker_a);
        }

        // Unwrap WSOL (token_b) to SOL before transferring
        // We use PDA-derived temporary accounts (validated at account struct level)
        if treasury_b > 0 {
            msg!("Processing treasury WSOL unwrap: {} WSOL", treasury_b);
            // Initialize the temporary WSOL account if it doesn't exist
            if self.treasury_temp_wsol.data_is_empty() {
                msg!("Initializing treasury temporary WSOL PDA account");
                // Derive PDA using OUR program ID so we can create it with invoke_signed
                // The PDA will be owned by our program, but we'll transfer ownership to token program
                // Actually wait - we need the PDA to be owned by token program for initialize_account to work
                // So we need to derive it with TOKEN_PROGRAM_ID but create it differently
                // The real solution: Use our program ID for the PDA, then it can be created with invoke_signed
                // But the test derives it with TOKEN_PROGRAM_ID, so we need to match that
                // Derive PDA using our program ID so we can create it with invoke_signed
                let (expected_pda, treasury_bump) = Pubkey::find_program_address(
                    &[
                        b"temp_wsol",
                        self.vault_authority.key().as_ref(),
                        self.proposal.key().as_ref(),
                        b"treasury",
                    ],
                    &crate::ID, // Use our program ID
                );
                
                // Verify the PDA matches what was passed in
                require_keys_eq!(
                    expected_pda,
                    self.treasury_temp_wsol.key(),
                    ProposalError::IncorrectAccount
                );
                
                let vault_key = self.vault_authority.key();
                let proposal_key = self.proposal.key();
                let bump_array = [treasury_bump];
                let treasury_seeds: &[&[u8]] = &[
                    b"temp_wsol",
                    vault_key.as_ref(),
                    proposal_key.as_ref(),
                    b"treasury",
                    &bump_array,
                ];
                
                // Create and initialize the PDA token account in one step
                // Standard token account size is 165 bytes
                let account_len = 165u64;
                let rent = Rent::get()?;
                let rent_lamports = rent.minimum_balance(account_len as usize);
                
                // Create the account owned by token program with rent-exempt balance
                // Now that the PDA is derived with our program ID, we can create it with invoke_signed
                msg!("Creating treasury temp WSOL PDA account with {} lamports for rent", rent_lamports);
                system_program::create_account(
                    CpiContext::new_with_signer(
                        self.system_program.to_account_info(),
                        system_program::CreateAccount {
                            from: self.payer.to_account_info(),
                            to: self.treasury_temp_wsol.to_account_info(),
                        },
                        &[treasury_seeds],
                    ),
                    rent_lamports,
                    account_len,
                    &self.token_b_program.key(), // Owner is token program, but PDA is derived from our program
                )?;
                
                // Initialize the PDA as a token account
                msg!("Initializing treasury temp WSOL PDA as token account");
                initialize_account(
                    CpiContext::new_with_signer(
                        self.token_b_program.to_account_info(),
                        InitializeAccount {
                            account: self.treasury_temp_wsol.to_account_info(),
                            mint: self.token_b_mint.to_account_info(),
                            authority: self.vault_authority.to_account_info(),
                            rent: self.rent.to_account_info(),
                        },
                        &[treasury_seeds],
                    ),
                )?;
                msg!("✅ Treasury temp WSOL PDA initialized");
            }

            // Transfer WSOL to temporary PDA account
            msg!("Transferring {} WSOL to treasury temp PDA for unwrapping", treasury_b);
            anchor_spl::token::transfer(
                CpiContext::new_with_signer(
                    self.token_b_program.to_account_info(),
                    anchor_spl::token::Transfer {
                        from: self.token_b_account.to_account_info(),
                        to: self.treasury_temp_wsol.to_account_info(),
                        authority: self.vault_authority.to_account_info(),
                    },
                    &[&vault_authority_seeds[..]],
                ),
                treasury_b,
            )?;
            msg!("✅ WSOL transferred to treasury temp PDA");

            // Close the temporary WSOL account to unwrap it to SOL
            // The SOL (lamports) will be sent to the treasury account
            msg!("Closing treasury temp WSOL PDA to unwrap {} WSOL to SOL", treasury_b);
            close_account(
                CpiContext::new_with_signer(
                    self.token_b_program.to_account_info(),
                    CloseAccount {
                        account: self.treasury_temp_wsol.to_account_info(),
                        destination: self.wewe_treasury.to_account_info(),
                        authority: self.vault_authority.to_account_info(),
                    },
                    &[&vault_authority_seeds[..]],
                ),
            )?;
            msg!("✅ Unwrapped {} WSOL to SOL to treasury", treasury_b);
        }

        if maker_b > 0 {
            msg!("Processing maker WSOL unwrap: {} WSOL", maker_b);
            // Initialize the temporary WSOL account if it doesn't exist
            if self.maker_temp_wsol.data_is_empty() {
                msg!("Initializing maker temporary WSOL PDA account");
                // Derive PDA using our program ID so we can create it with invoke_signed
                let (expected_pda, maker_bump) = Pubkey::find_program_address(
                    &[
                        b"temp_wsol",
                        self.vault_authority.key().as_ref(),
                        self.proposal.key().as_ref(),
                        b"maker",
                    ],
                    &crate::ID, // Use our program ID
                );
                
                // Verify the PDA matches what was passed in
                require_keys_eq!(
                    expected_pda,
                    self.maker_temp_wsol.key(),
                    ProposalError::IncorrectAccount
                );
                
                let vault_key = self.vault_authority.key();
                let proposal_key = self.proposal.key();
                let bump_array = [maker_bump];
                let maker_seeds: &[&[u8]] = &[
                    b"temp_wsol",
                    vault_key.as_ref(),
                    proposal_key.as_ref(),
                    b"maker",
                    &bump_array,
                ];
                
                // Create and initialize the PDA token account in one step
                // Standard token account size is 165 bytes
                let account_len = 165u64;
                let rent = Rent::get()?;
                let rent_lamports = rent.minimum_balance(account_len as usize);
                
                // Create the account owned by token program with rent-exempt balance
                // Now that the PDA is derived with our program ID, we can create it with invoke_signed
                msg!("Creating maker temp WSOL PDA account with {} lamports for rent", rent_lamports);
                system_program::create_account(
                    CpiContext::new_with_signer(
                        self.system_program.to_account_info(),
                        system_program::CreateAccount {
                            from: self.payer.to_account_info(),
                            to: self.maker_temp_wsol.to_account_info(),
                        },
                        &[maker_seeds],
                    ),
                    rent_lamports,
                    account_len,
                    &self.token_b_program.key(), // Owner is token program, but PDA is derived from our program
                )?;
                
                // Initialize the PDA as a token account
                msg!("Initializing maker temp WSOL PDA as token account");
                initialize_account(
                    CpiContext::new_with_signer(
                        self.token_b_program.to_account_info(),
                        InitializeAccount {
                            account: self.maker_temp_wsol.to_account_info(),
                            mint: self.token_b_mint.to_account_info(),
                            authority: self.vault_authority.to_account_info(),
                            rent: self.rent.to_account_info(),
                        },
                        &[maker_seeds],
                    ),
                )?;
                msg!("✅ Maker temp WSOL PDA initialized");
            }

            // Transfer WSOL to temporary PDA account
            msg!("Transferring {} WSOL to maker temp PDA for unwrapping", maker_b);
            anchor_spl::token::transfer(
                CpiContext::new_with_signer(
                    self.token_b_program.to_account_info(),
                    anchor_spl::token::Transfer {
                        from: self.token_b_account.to_account_info(),
                        to: self.maker_temp_wsol.to_account_info(),
                        authority: self.vault_authority.to_account_info(),
                    },
                    &[&vault_authority_seeds[..]],
                ),
                maker_b,
            )?;
            msg!("✅ WSOL transferred to maker temp PDA");

            // Close the temporary WSOL account to unwrap it to SOL and send it to maker
            msg!("Closing maker temp WSOL PDA to unwrap {} WSOL to SOL", maker_b);
            close_account(
                CpiContext::new_with_signer(
                    self.token_b_program.to_account_info(),
                    CloseAccount {
                        account: self.maker_temp_wsol.to_account_info(),
                        destination: self.maker.to_account_info(),
                        authority: self.vault_authority.to_account_info(),
                    },
                    &[&vault_authority_seeds[..]],
                ),
            )?;
            msg!("✅ Unwrapped {} WSOL to SOL to maker", maker_b);
        }

        emit!(PositionFeeClaimed {
            proposal: self.proposal.key(),
            maker: self.maker.key(),
            user: self.payer.key(),
            user_token_amount: claimed_token_a,
            user_wsol_amount: claimed_token_b,
            token_mint: self.token_a_mint.key(),
            wsol_mint: self.token_b_mint.key(),
        });

        Ok(())
    }
}
