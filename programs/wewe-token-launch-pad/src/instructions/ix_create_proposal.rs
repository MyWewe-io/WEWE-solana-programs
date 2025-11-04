use std::ops::Sub;

use crate::{
    const_pda,
    constant::{
        seeds::{MAKER, PROPOSAL, TOKEN_VAULT, VAULT_AUTHORITY},
        treasury,
        ANCHOR_DISCRIMINATOR, MINT_DECIMALS,
    },
    errors::ProposalError,
    event::ProposalCreated,
    state::{maker::MakerAccount, proposal::Proposal,config::Configs},
};
use anchor_lang::prelude::*;
use std::str::FromStr;
use anchor_spl::{
    metadata::{
        create_metadata_accounts_v3, mpl_token_metadata::types::DataV2, CreateMetadataAccountsV3,
        Metadata,
    },
    token::{mint_to, MintTo, TokenAccount},
    token_interface::TokenInterface,
};

#[derive(Accounts)]
pub struct CreateProposal<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(mut)]
    pub maker: Signer<'info>,

    #[account(
        init_if_needed,
        payer = payer,
        seeds = [MAKER, maker.key().as_ref()],
        bump,
        space = ANCHOR_DISCRIMINATOR + MakerAccount::INIT_SPACE,
    )]
    pub maker_account: Account<'info, MakerAccount>,

    #[account(
        init,
        payer = payer,
        seeds = [PROPOSAL, maker.key().as_ref(), &maker_account.proposal_count.to_le_bytes()],
        bump,
        space = ANCHOR_DISCRIMINATOR + Proposal::INIT_SPACE,
    )]
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

    /// CHECK: Token-2022 mint account (will be initialized with transfer fee extension)
    /// Mint address derived from: seeds = [PROPOSAL, maker.key(), proposal_count, "mint"]
    #[account(mut)]
    pub mint_account: UncheckedAccount<'info>,
    
    /// CHECK: WEWE treasury account (receives transfer fees)
    #[account(address = treasury::ID)]
    pub wewe_treasury: UncheckedAccount<'info>,

    /// CHECK: Validate address by deriving pda
    #[account(
        mut,
        seeds = [b"metadata", token_metadata_program.key().as_ref(), mint_account.key().as_ref()],
        bump,
        seeds::program = token_metadata_program.key(),
    )]
    pub metadata_account: UncheckedAccount<'info>,

    #[account(
        init,
        seeds = [TOKEN_VAULT, vault_authority.key().as_ref(), mint_account.key().as_ref()],
        payer = payer,
        token::mint = mint_account,
        token::authority = vault_authority,
        token::token_program = token_program,
        bump,
    )]
    pub token_vault: Account<'info, TokenAccount>,

    #[account(
        mut,
        address = const_pda::const_authority::MINT,
    )]
    pub mint: Account<'info, anchor_spl::token::Mint>,
    #[account(
        associated_token::mint = mint,
        associated_token::authority = maker,
        constraint = user_token_account.amount == 1 @ ProposalError::NotAuthorised
    )]
    pub user_token_account: Account<'info, TokenAccount>,

    pub token_program: Interface<'info, TokenInterface>,
    pub token_metadata_program: Program<'info, Metadata>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
    pub config: Account<'info, Configs>,
}

impl<'info> CreateProposal<'info> {
    pub fn handle_create_proposal(
        &mut self,
        token_name: String,
        token_symbol: String,
        token_uri: String,
        bumps: &CreateProposalBumps,
    ) -> Result<()> {
        // PDA signer seeds
        let signer_seeds: &[&[&[u8]]] = &[&[
            PROPOSAL,
            self.maker.key.as_ref(),
            &self.maker_account.proposal_count.to_le_bytes(),
            &[bumps.proposal],
        ]];

        require!(token_name.len() <= 32, ProposalError::LenthTooLong);
        require!(token_symbol.len() <= 10, ProposalError::LenthTooLong);
        require!(token_uri.len() <= 200, ProposalError::LenthTooLong);

        // Initialize Token-2022 mint with transfer fee extension
        self.initialize_token2022_mint_with_transfer_fee(signer_seeds)?;

        create_metadata_accounts_v3(
            CpiContext::new(
                self.token_metadata_program.to_account_info(),
                CreateMetadataAccountsV3 {
                    metadata: self.metadata_account.to_account_info(),
                    mint: self.mint_account.to_account_info(),
                    mint_authority: self.proposal.to_account_info(),
                    update_authority: self.proposal.to_account_info(),
                    payer: self.payer.to_account_info(),
                    system_program: self.system_program.to_account_info(),
                    rent: self.rent.to_account_info(),
                },
            )
            .with_signer(signer_seeds),
            DataV2 {
                name: token_name.clone(),
                symbol: token_symbol.clone(),
                uri: token_uri.clone(),
                seller_fee_basis_points: 0,
                creators: None,
                collection: None,
                uses: None,
            },
            false, // Is mutable
            true,  // Update authority is signer
            None,  // Collection details
        )?;

        // Get mint decimals (use configured value for Token-2022)
        let decimals = MINT_DECIMALS;
        
        let pow = 10u64
            .checked_pow(decimals as u32)
            .ok_or(ProposalError::NumericalOverflow)?;
        let amount = self.config.total_mint // TOTAL_MINT
            .checked_mul(pow)
            .ok_or(ProposalError::NumericalOverflow)?;
        // Invoke the mint_to instruction on the token program
        mint_to(
            CpiContext::new(
                self.token_program.to_account_info(),
                MintTo {
                    mint: self.mint_account.to_account_info(),
                    to: self.token_vault.to_account_info(),
                    authority: self.proposal.to_account_info(),
                },
            )
            .with_signer(signer_seeds), // using PDA to sign,
            amount, // Mint tokens
        )?;

        let now = Clock::get()?.unix_timestamp;
        self.proposal.set_inner(Proposal {
            maker: self.maker.key(),
            mint_account: self.mint_account.key(),
            total_backing: 0,
            time_started: now,
            bump: bumps.proposal,
            is_rejected: false,
            proposal_id: self.maker_account.proposal_count,
            is_pool_launched: false,
            total_backers: 0,
            current_airdrop_cycle: 1,
            milestone_active: false,
            milestone_units_assigned: 0,
            milestone_backers_weighted: 0,
        });
        // increment proposal count for maker
        let idx = self.maker_account.proposal_count;
        self.maker_account.proposal_count =
            idx.checked_add(1).ok_or(ProposalError::NumericalOverflow)?;

        emit!(ProposalCreated {
            maker: self.maker.key(),
            proposal_address: self.proposal.key(),
            proposal_index: self.maker_account.proposal_count.sub(1),
            start_time: now,
            token_name,
            token_symbol,
            token_uri,
            mint_account: self.mint_account.key(),
            token_vault: self.token_vault.key(),
            metadata_account: self.metadata_account.key(),
            maker_account: self.maker_account.key(),
            proposal_bump: bumps.proposal,
        });

        Ok(())
    }

    fn initialize_token2022_mint_with_transfer_fee(
        &self,
        signer_seeds: &[&[&[u8]]],
    ) -> Result<()> {
        // Token-2022 mint constants
        const MINT_SIZE: usize = 82; // Standard mint size
        const TRANSFER_FEE_CONFIG_EXTENSION_SIZE: usize = 72; // TransferFeeConfig extension size
        
        // Calculate total space needed: base mint + transfer fee extension
        let mint_space = MINT_SIZE;
        let extension_len = TRANSFER_FEE_CONFIG_EXTENSION_SIZE;
        let total_space = mint_space + extension_len;

        // Check if mint account is already initialized
        if self.mint_account.lamports() > 0 {
            return Ok(()); // Already initialized
        }

        // Rent for the account
        let rent = Rent::get()?;
        let rent_lamports = rent.minimum_balance(total_space);

        // Create the account
        anchor_lang::solana_program::program::invoke(
            &anchor_lang::solana_program::system_instruction::create_account(
                &self.payer.key(),
                &self.mint_account.key(),
                rent_lamports,
                total_space as u64,
                &anchor_lang::solana_program::pubkey::Pubkey::from_str(
                    "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb"
                ).unwrap(),
            ),
            &[
                self.payer.to_account_info(),
                self.mint_account.to_account_info(),
                self.system_program.to_account_info(),
            ],
        )?;

        // Initialize the mint using Token-2022's initialize_mint2 instruction
        // This initializes the mint base state with the extension space reserved
        let mint_authority_key = self.proposal.key();
        let freeze_authority_key = Some(self.proposal.key());
        
        // Build initialize_mint2 instruction manually
        // Token-2022 initialize_mint2 instruction format:
        // - instruction discriminator (1 byte)
        // - mint_authority (32 bytes)
        // - freeze_authority (Option<Pubkey>, 1 byte + 32 bytes if Some)
        // - decimals (1 byte)
        // - extension types (variable)
        let mut instruction_data = Vec::new();
        
        // Instruction discriminator for initialize_mint2 (36)
        instruction_data.push(36u8);
        
        // Mint authority
        instruction_data.extend_from_slice(mint_authority_key.as_ref());
        
        // Freeze authority (Some)
        instruction_data.push(1u8); // Some
        instruction_data.extend_from_slice(freeze_authority_key.unwrap().as_ref());
        
        // Decimals
        instruction_data.push(MINT_DECIMALS);
        
        // Extension types (TransferFeeConfig = 1)
        let extension_type: u16 = 1; // TransferFeeConfig extension type
        instruction_data.extend_from_slice(&extension_type.to_le_bytes());
        
        // Build account metas
        let mut account_infos = Vec::new();
        account_infos.push(anchor_lang::solana_program::instruction::AccountMeta::new(
            self.mint_account.key(),
            false,
        ));
        account_infos.push(anchor_lang::solana_program::instruction::AccountMeta::new_readonly(
            self.proposal.key(),
            true, // is_signer (PDA will sign)
        ));
        account_infos.push(anchor_lang::solana_program::instruction::AccountMeta::new_readonly(
            anchor_lang::solana_program::sysvar::rent::id(),
            false,
        ));
        account_infos.push(anchor_lang::solana_program::instruction::AccountMeta::new_readonly(
            anchor_lang::solana_program::system_program::id(),
            false,
        ));
        
        let token_2022_program_id = anchor_lang::solana_program::pubkey::Pubkey::from_str(
            "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb"
        ).unwrap();
        
        let initialize_mint_ix = anchor_lang::solana_program::instruction::Instruction {
            program_id: token_2022_program_id,
            accounts: account_infos,
            data: instruction_data,
        };
        
        // Invoke initialize_mint2
        anchor_lang::solana_program::program::invoke_signed(
            &initialize_mint_ix,
            &[
                self.mint_account.to_account_info(),
                self.proposal.to_account_info(),
                self.rent.to_account_info(),
                self.system_program.to_account_info(),
                self.token_program.to_account_info(),
            ],
            signer_seeds,
        )?;

        // Now initialize transfer fee extension using raw instruction
        // Build initialize_transfer_fee_config instruction
        // Instruction discriminator: 39 (for initialize_transfer_fee_config)
        let mut fee_instruction_data = Vec::new();
        fee_instruction_data.push(39u8); // Instruction discriminator
        
        // Older transfer fee (epoch, maximum_fee)
        fee_instruction_data.extend_from_slice(&0u64.to_le_bytes()); // epoch
        fee_instruction_data.extend_from_slice(&self.config.max_fee.to_le_bytes()); // maximum_fee
        
        // Newer transfer fee (epoch, maximum_fee)
        fee_instruction_data.extend_from_slice(&0u64.to_le_bytes()); // epoch
        fee_instruction_data.extend_from_slice(&self.config.max_fee.to_le_bytes()); // maximum_fee
        
        // Withdraw withheld authority (treasury)
        fee_instruction_data.extend_from_slice(self.wewe_treasury.key().as_ref());
        
        // Build account metas for initialize_transfer_fee_config
        let mut fee_account_infos = Vec::new();
        fee_account_infos.push(anchor_lang::solana_program::instruction::AccountMeta::new(
            self.mint_account.key(),
            false,
        ));
        fee_account_infos.push(anchor_lang::solana_program::instruction::AccountMeta::new_readonly(
            self.proposal.key(),
            true, // is_signer (PDA will sign)
        ));
        
        let initialize_fee_ix = anchor_lang::solana_program::instruction::Instruction {
            program_id: token_2022_program_id,
            accounts: fee_account_infos,
            data: fee_instruction_data,
        };
        
        // Invoke initialize_transfer_fee_config
        anchor_lang::solana_program::program::invoke_signed(
            &initialize_fee_ix,
            &[
                self.mint_account.to_account_info(),
                self.proposal.to_account_info(),
                self.token_program.to_account_info(),
            ],
            signer_seeds,
        )?;
        
        // Note: The transfer fee percentage (basis points) will be set via
        // update_transfer_fee instruction after initialization

        Ok(())
    }
}
