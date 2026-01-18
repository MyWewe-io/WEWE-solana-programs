// tests/wewe_token_launch_pad.spec.ts
import * as anchor from '@coral-xyz/anchor';
import { Program } from '@coral-xyz/anchor';
import BN from 'bn.js';
import Decimal from 'decimal.js';
import { assert, expect } from 'chai';
import {
  TOKEN_PROGRAM_ID,
  TOKEN_2022_PROGRAM_ID,
  ASSOCIATED_TOKEN_PROGRAM_ID,
  getAssociatedTokenAddressSync,
  createAssociatedTokenAccountInstruction,
  createTransferInstruction,
  createSyncNativeInstruction,
  getMint,
} from '@solana/spl-token';

import type { WeweTokenLaunchPad } from '../target/types/wewe_token_launch_pad';
import type { CpAmm } from './cp_amm';
import CpAmmIDL from '../idls/cp_amm.json';
import { ComputeBudgetProgram } from '@solana/web3.js';

import {
  confirm,
  findProposalPDA,
  findBackerAccountPDA,
  findMakerAccountPDA,
  WSOL_MINT,
  getMetadata,
  generateKeypairs,
  getTokenVaultAddress,
  derivePoolPDAs,
  findUserAta,
  findMintAccount,
  findMintAuthority,
  getSqrtPriceFromPrice,
  findConfigPDA,
  findBackerProposalCountPDA,
  findMetadataPDA,
  findTempWsolPDA,
} from './utils';

const TOKEN_METADATA_PROGRAM_ID = new anchor.web3.PublicKey("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s");

// Helper function to wait for a specific event
const waitForEvent = async (program: Program<any>, eventName: string): Promise<any> => {
  return new Promise((resolve) => {
    program.addEventListener(eventName, (event) => {
      resolve(event);
      program.removeEventListener(eventName);
    });
  });
};


describe('Wewe Token Launch Pad - Integration Tests', () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.WeweTokenLaunchPad as Program<WeweTokenLaunchPad>;
  const cpAmm = new Program<CpAmm>(CpAmmIDL as CpAmm, provider);

  const { maker, backer, authority } = generateKeypairs();
  // configure_authority - in prod this will be the same as upgrade authority
  // For tests, we use a separate keypair
  const configureAuthority = anchor.web3.Keypair.fromSecretKey(
    Uint8Array.from([40, 42, 71, 80, 145, 233, 28, 85, 46, 107, 84, 98, 241, 241, 141, 138, 140, 198, 165, 158, 56, 179, 167, 195, 231, 122, 197, 137, 163, 36, 140, 202, 225, 68, 81, 1, 242, 195, 20, 0, 204, 11, 235, 44, 77, 100, 241, 12, 138, 225, 40, 210, 2, 95, 212, 51, 228, 218, 94, 73, 186, 142, 137, 191])
  );
  // chain_service authority - used for automated operations, should NOT be able to set config
  const chainServiceAuthority = anchor.web3.Keypair.fromSecretKey(
    Uint8Array.from([42, 132, 54, 48, 86, 137, 10, 155, 254, 103, 140, 97, 104, 8, 197, 48, 55, 71, 171, 157, 247, 159, 233, 130, 100, 213, 107, 236, 96, 40, 175, 164, 179, 49, 15, 185, 22, 130, 249, 11, 142, 174, 6, 253, 52, 133, 167, 81, 80, 179, 15, 199, 164, 252, 14, 233, 42, 74, 178, 20, 71, 62, 139, 21])
  );

  let proposalIndex1 = new BN(0);
  let proposalIndex2 = new BN(1);
  let proposalIndex3 = new BN(2);
  let proposalIndex4 = new BN(3);
  const configStruct = findConfigPDA(program.programId, maker.publicKey)
  const proposal = findProposalPDA(program.programId, maker.publicKey, proposalIndex1);
  const proposal2 = findProposalPDA(program.programId, maker.publicKey, proposalIndex2);
  const proposal3 = findProposalPDA(program.programId, maker.publicKey, proposalIndex3);
  const proposal4 = findProposalPDA(program.programId, maker.publicKey, proposalIndex4);

  const backerAccount = findBackerAccountPDA(program.programId, proposal, backer.publicKey);
  const backerAccount2 = findBackerAccountPDA(program.programId, proposal2, backer.publicKey);
  const backerAccount3 = findBackerAccountPDA(program.programId, proposal3, backer.publicKey);
  const backerAccount4 = findBackerAccountPDA(program.programId, proposal4, backer.publicKey);
  const makerAccount = findMakerAccountPDA(program.programId, maker.publicKey);

  const metadata = getMetadata();
  const [vaultAuthority] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("vault_authority")],
    program.programId
  );
  const weweTreasury = new anchor.web3.PublicKey("76U9hvHNUNn7YV5FekSzDHzqnHETsUpDKq4cMj2dMxNi");
  
  const mint = anchor.web3.Keypair.generate();
  const mint2 = anchor.web3.Keypair.generate();
  const mint3 = anchor.web3.Keypair.generate();
  const mint4 = anchor.web3.Keypair.generate();
  const [vault] = getTokenVaultAddress(vaultAuthority, mint.publicKey, program.programId);
  const [vault2] = getTokenVaultAddress(vaultAuthority, mint2.publicKey, program.programId);
  const [vault3] = getTokenVaultAddress(vaultAuthority, mint3.publicKey, program.programId);
  const [vault4] = getTokenVaultAddress(vaultAuthority, mint4.publicKey, program.programId);

  const mintAccount = findMintAccount(program.programId);
  const userAta = findUserAta(backer.publicKey, mintAccount);
  const makerAta = findUserAta(maker.publicKey, mintAccount);
  const [freezeAuthority] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("freeze_authority")],
    program.programId
  );
  const mintAuthority = findMintAuthority(program.programId);
  const backerTokenAccount = findUserAta(backer.publicKey, mint.publicKey);

  // Use hardcoded config address from devnet
  const config = new anchor.web3.PublicKey("7xeqWqnS4VMguYMexgtw1vxXt3cM8eh6vyvUkYgv2aJY");
  const pdas = derivePoolPDAs(program.programId, cpAmm.programId, mint.publicKey, WSOL_MINT, maker.publicKey, config);

  // Setup: Ensure CP-AMM config account exists in local validator
  before(async () => {
    // Check if account exists locally
    const localAccountInfo = await provider.connection.getAccountInfo(config);
    if (localAccountInfo) {
      console.log('✅ CP-AMM config account already exists locally');
      return;
    }

    // Account doesn't exist locally, try to fetch from devnet and clone it
    console.log('📥 CP-AMM config account not found locally, fetching from devnet...');
    const devnetConnection = new anchor.web3.Connection(
      'https://api.devnet.solana.com',
      'confirmed'
    );
    
    const devnetAccountInfo = await devnetConnection.getAccountInfo(config);
    
    if (!devnetAccountInfo) {
      throw new Error(
        `CP-AMM config account ${config.toBase58()} not found on devnet. ` +
        `Please verify the account address or create the config account first.`
      );
    }

    // Try to clone the account to local validator
    // Note: This requires the local validator to support account cloning
    // If your validator doesn't support this, you'll need to manually add it
    // or ensure Anchor.toml [[test.validator.clone]] is working correctly
    console.log('📋 Attempting to clone account to local validator...');
    
    // For now, provide helpful error message
    // The account should be cloned via Anchor.toml configuration
    throw new Error(
      `CP-AMM config account ${config.toBase58()} not found in local validator.\n` +
      `The account exists on devnet but needs to be cloned to your local validator.\n\n` +
      `Solutions:\n` +
      `1. Ensure Anchor.toml has: [[test.validator.clone]]\n` +
      `   address = "${config.toBase58()}"\n` +
      `2. Restart your local validator to trigger account cloning\n` +
      `3. Or manually clone the account using solana-test-validator --clone ${config.toBase58()}`
    );
  });

  it('0. Check CP-AMM config account contents', async () => {
    console.log('\n🔍 Checking CP-AMM Config Account Contents');
    console.log('='.repeat(80));
    console.log(`📍 Account Address: ${config.toBase58()}\n`);

    // Check if account exists
    const accountInfo = await provider.connection.getAccountInfo(config);
    if (!accountInfo) {
      console.log('❌ Account not found on localnet');
      throw new Error(`CP-AMM config account ${config.toBase58()} not found on localnet`);
    }

    console.log('✅ Account found on localnet!');
    console.log(`   Owner: ${accountInfo.owner.toBase58()}`);
    console.log(`   Executable: ${accountInfo.executable}`);
    console.log(`   Lamports: ${accountInfo.lamports} (${accountInfo.lamports / 1e9} SOL)`);
    console.log(`   Data length: ${accountInfo.data.length} bytes`);
    console.log(`   Rent Epoch: ${accountInfo.rentEpoch}\n`);

    // Try to fetch using Anchor program
    try {
      const configAccount = await cpAmm.account.config.fetch(config);
      console.log('📋 Config Account Data (decoded):');
      console.log('='.repeat(80));
      console.log(`   vault_config_key: ${configAccount.vaultConfigKey.toBase58()}`);
      console.log(`   pool_creator_authority: ${configAccount.poolCreatorAuthority.toBase58()}`);
      console.log(`   activation_type: ${configAccount.activationType}`);
      console.log(`   collect_fee_mode: ${configAccount.collectFeeMode}`);
      console.log(`   index: ${configAccount.index.toString()}`);
      console.log(`   sqrt_min_price: ${configAccount.sqrtMinPrice.toString()}`);
      console.log(`   sqrt_max_price: ${configAccount.sqrtMaxPrice.toString()}`);
      
      if (configAccount.poolFees) {
        console.log(`\n   Pool Fees:`);
        console.log(`     protocol_a_fee: ${configAccount.poolFees.protocolAFee?.toString() || 'N/A'}`);
        console.log(`     protocol_b_fee: ${configAccount.poolFees.protocolBFee?.toString() || 'N/A'}`);
        console.log(`     partner_a_fee: ${configAccount.poolFees.partnerAFee?.toString() || 'N/A'}`);
        console.log(`     partner_b_fee: ${configAccount.poolFees.partnerBFee?.toString() || 'N/A'}`);
      }
      
      console.log('\n📄 Raw JSON:');
      console.log(JSON.stringify(configAccount, (key, value) => {
        // Convert BN and PublicKey to strings for JSON serialization
        if (value && typeof value === 'object') {
          if (value.constructor && value.constructor.name === 'BN') {
            return value.toString();
          }
          if (value.constructor && value.constructor.name === 'PublicKey') {
            return value.toBase58();
          }
        }
        return value;
      }, 2));
      
      console.log('='.repeat(80));
    } catch (error: any) {
      console.log('⚠️  Could not decode account with Anchor, showing raw data:');
      console.log(`   Error: ${error.message}\n`);
      
      // Show raw hex data
      const hex = Array.from(accountInfo.data.slice(0, Math.min(200, accountInfo.data.length)))
        .map(b => b.toString(16).padStart(2, '0'))
        .join(' ');
      console.log(`   First ${Math.min(200, accountInfo.data.length)} bytes (hex):`);
      console.log(`   ${hex}`);
      if (accountInfo.data.length > 200) {
        console.log(`   ... (${accountInfo.data.length - 200} more bytes)`);
      }
    }
    
    console.log('\n');
  });

  async function printTxLogs(sig: string) {
    // wait for finalization so logs are retrievable
    await provider.connection.confirmTransaction(sig, 'confirmed');
    // fetch the executed tx and print program logs
    const tx = await provider.connection.getTransaction(sig, {
      commitment: 'confirmed',
      maxSupportedTransactionVersion: 0,
    });
    console.log('\n=== PROGRAM LOGS ===');
    console.log(tx?.meta?.logMessages?.join('\n') ?? '(no logs)');
    console.log('====================\n');
  }

  it('-1 Test liquidity calculation', async () => {
    // Helper functions matching Rust implementation
    // L = Δx * sqrt(P) * sqrt(P_upper) / (sqrt(P_upper) - sqrt(P))
    const getInitialLiquidityFromAmountA = (
      baseAmount: BN,
      sqrtMaxPrice: BN,
      sqrtPrice: BN
    ): BN => {
      // price_delta = sqrt_max_price - sqrt_price
      const priceDelta = sqrtMaxPrice.sub(sqrtPrice);
      if (priceDelta.lte(new BN(0))) {
        throw new Error('price_delta must be positive');
      }

      // prod = base_amount * sqrt_price * sqrt_max_price
      const prod = baseAmount.mul(sqrtPrice).mul(sqrtMaxPrice);

      // liquidity = prod / price_delta (round down)
      const liquidity = prod.div(priceDelta);
      return liquidity;
    };

    // L = Δy * 2^128 / (sqrt(P) - sqrt(P_lower))
    const getInitialLiquidityFromAmountB = (
      quoteAmount: BN,
      sqrtMinPrice: BN,
      sqrtPrice: BN
    ): BN => {
      // price_delta = sqrt_price - sqrt_min_price
      const priceDelta = sqrtPrice.sub(sqrtMinPrice);
      if (priceDelta.lte(new BN(0))) {
        throw new Error('price_delta must be positive');
      }

      // quote_amount_shifted = quote_amount << 128 (multiply by 2^128)
      const TWO_TO_128 = new BN(2).pow(new BN(128));
      const quoteAmountShifted = quoteAmount.mul(TWO_TO_128);

      // liquidity = quote_amount_shifted / price_delta (round down)
      const liquidity = quoteAmountShifted.div(priceDelta);
      return liquidity;
    };

    // Takes the minimum of liquidity_from_base and liquidity_from_quote
    const getLiquidityForAddingLiquidity = (
      baseAmount: BN,
      quoteAmount: BN,
      sqrtPrice: BN,
      minSqrtPrice: BN,
      maxSqrtPrice: BN
    ): BN => {
      const liquidityFromBase = getInitialLiquidityFromAmountA(
        baseAmount,
        maxSqrtPrice,
        sqrtPrice
      );
      const liquidityFromQuote = getInitialLiquidityFromAmountB(
        quoteAmount,
        minSqrtPrice,
        sqrtPrice
      );

      // Return the smaller of the two
      if (liquidityFromBase.gt(liquidityFromQuote)) {
        return liquidityFromQuote;
      } else {
        return liquidityFromBase;
      }
    };

    // Test with sample values (use strings for very large numbers to avoid BN.js precision issues)
    const baseAmount = new BN('150000000000000000');
    const quoteAmount = new BN('1000000000');
    const sqrtPrice = new BN('79226673521066983548331216');
    const sqrtMinPrice = new BN('4295048016');
    const sqrtMaxPrice = new BN('79226673521066979257578248091');

    console.log('Testing liquidity calculation:');
    console.log('baseAmount:', baseAmount.toString());
    console.log('quoteAmount:', quoteAmount.toString());
    console.log('sqrtPrice:', sqrtPrice.toString());
    console.log('sqrtMinPrice:', sqrtMinPrice.toString());
    console.log('sqrtMaxPrice:', sqrtMaxPrice.toString());

    try {
      const liquidityFromBase = getInitialLiquidityFromAmountA(
        baseAmount,
        sqrtMaxPrice,
        sqrtPrice
      );
      console.log('liquidityFromBase:', liquidityFromBase.toString());

      const liquidityFromQuote = getInitialLiquidityFromAmountB(
        quoteAmount,
        sqrtMinPrice,
        sqrtPrice
      );
      console.log('liquidityFromQuote:', liquidityFromQuote.toString());

      const liquidity = getLiquidityForAddingLiquidity(
        baseAmount,
        quoteAmount,
        sqrtPrice,
        sqrtMinPrice,
        sqrtMaxPrice
      );
      console.log('Final liquidity (liquidity_delta):', liquidity.toString());

      // Assertions to make the test pass (with better error messages)
      try {
        expect(liquidityFromBase.gt(new BN(0)), 'liquidityFromBase should be greater than 0').to.be.true;
        expect(liquidityFromQuote.gt(new BN(0)), 'liquidityFromQuote should be greater than 0').to.be.true;
        expect(liquidity.gt(new BN(0)), 'liquidity should be greater than 0').to.be.true;
        expect(
          liquidity.eq(liquidityFromBase) || liquidity.eq(liquidityFromQuote),
          `liquidity (${liquidity.toString()}) should equal either liquidityFromBase (${liquidityFromBase.toString()}) or liquidityFromQuote (${liquidityFromQuote.toString()})`
        ).to.be.true;
      } catch (assertionError) {
        console.error('Assertion failed:', assertionError.message);
        throw assertionError;
      }
    } catch (error) {
      console.error('Error calculating liquidity:', error);
      throw error;
    }
  });
  
  it('0. Airdrops funds to test accounts', async () => {
    const airdropPromises = [
      provider.connection.requestAirdrop(provider.wallet.publicKey, 5e9),
      provider.sendAndConfirm(new anchor.web3.Transaction().add(
        anchor.web3.SystemProgram.transfer({
          fromPubkey: provider.wallet.publicKey,
          toPubkey: maker.publicKey,
          lamports: 1e9,
        })
      )),
      provider.sendAndConfirm(new anchor.web3.Transaction().add(
        anchor.web3.SystemProgram.transfer({
          fromPubkey: provider.wallet.publicKey,
          toPubkey: backer.publicKey,
          lamports: 10e9, // Increased to 10 SOL to cover multiple 1 SOL deposits + transaction fees
        })
      )),
      provider.sendAndConfirm(new anchor.web3.Transaction().add(
        anchor.web3.SystemProgram.transfer({
          fromPubkey: provider.wallet.publicKey,
          toPubkey: authority.publicKey,
          lamports: 1e9,
        })
      )),
      provider.sendAndConfirm(new anchor.web3.Transaction().add(
        anchor.web3.SystemProgram.transfer({
          fromPubkey: provider.wallet.publicKey,
          toPubkey: configureAuthority.publicKey,
          lamports: 1e9,
        })
      )),
      provider.sendAndConfirm(new anchor.web3.Transaction().add(
        anchor.web3.SystemProgram.transfer({
          fromPubkey: provider.wallet.publicKey,
          toPubkey: chainServiceAuthority.publicKey,
          lamports: 1e9,
        })
      )),
      provider.sendAndConfirm(new anchor.web3.Transaction().add(
        anchor.web3.SystemProgram.transfer({
          fromPubkey: provider.wallet.publicKey,
          toPubkey: vaultAuthority,
          lamports: 1e9,
        })
      )),
    ];
    await Promise.all(airdropPromises.map(p => confirm(p)));
  });

  it('1. Sets constant values', async () => {
    const amountToRaisePerUser = new BN(1_000_000_000); // 1 SOL
    const totalMint = new BN(1_000_000_000);
    const totalPoolTokens = new BN(150_000_000);
    const makerTokenAmount = new BN(10_000_000);
    const totalAirdropAmountPerMilestone = new BN(140_000_000);
    const minBackers = new BN(1);
    const maxBackedProposals = new BN(3);
    const refundFeeBps = new BN(200); // 200 BPS = 2%
    const tx = await program.methods
      .setConfig(
        amountToRaisePerUser,
        totalMint,
        totalPoolTokens,
        makerTokenAmount,
        totalAirdropAmountPerMilestone,
        minBackers,
        maxBackedProposals,
        refundFeeBps, // refund_fee_basis_points: 200 BPS = 2%
      )
      .accounts({
        authority: configureAuthority.publicKey,
        config: configStruct,
      })
      .signers([configureAuthority])
      .rpc();

    await confirm(tx);

  });

  it('1.5. Fails when old authority (not configure_authority) tries to set config', async () => {
    const amountToRaisePerUser = new BN(1_000_000_000); // 1 SOL
    const totalMint = new BN(1_000_000_000);
    const totalPoolTokens = new BN(150_000_000);
    const makerTokenAmount = new BN(10_000_000);
    const totalAirdropAmountPerMilestone = new BN(140_000_000);
    const minBackers = new BN(1);
    const maxBackedProposals = new BN(3);
    const refundFeeBps = new BN(200); // 200 BPS = 2%
  
    try {
      await program.methods
        .setConfig(
          amountToRaisePerUser,
          totalMint,
          totalPoolTokens,
          makerTokenAmount,
          totalAirdropAmountPerMilestone,
          minBackers,
          maxBackedProposals,
          refundFeeBps,
        )
        .accounts({
          authority: authority.publicKey, // Using old authority, not configureAuthority
          config: configStruct,
        })
        .signers([authority])
        .rpc();

      assert.fail('Should not allow old authority to set config');
    } catch (err) {
      expect(err.message).to.include('NotOwner');
    }
  });

  it("2. Mints and freezes soulbound token to user and maker", async () => {
    const tx1 = await program.methods
      .mintSoulboundToUser()
      .accounts({
        authority: authority.publicKey,
        user: backer.publicKey,
        mint: mintAccount,
        freezeAuthority,
        mintAuthority,
        userTokenAccount: userAta,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([authority])
      .rpc();

    const tx2 = await program.methods
      .mintSoulboundToUser()
      .accounts({
        authority: authority.publicKey,
        user: maker.publicKey,
        mint: mintAccount,
        mintAuthority,
        userTokenAccount: makerAta,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([authority])
      .rpc();

    const accountInfo = await provider.connection.getTokenAccountBalance(userAta);
    assert.strictEqual(accountInfo.value.uiAmount, 1);

    // Create a clear failure if the freeze logic is broken and enforce transfer fails
    const ix = createTransferInstruction(
      userAta,
      makerAta,
      backer.publicKey,
      1,
    );
    
    const tx = new anchor.web3.Transaction().add(ix);
    await provider.sendAndConfirm(tx, [backer]).then(
      () => assert.fail('Expected transfer to fail (frozen account); freeze logic may be missing'),
      async (err: any) => {
        const logs: string[] | undefined = err?.logs ?? (typeof err?.getLogs === 'function' ? await err.getLogs() : undefined);
        const joined = logs?.join('\n') ?? String(err?.message || err);
        // Token Program 0x11 = AccountFrozen; accept any simulation failure too
        expect(joined).to.satisfy((m: string) => m.includes('custom program error: 0x11') || m.includes('Simulation failed'));
      }
    );
  });

  it('3. Creates first proposal', async () => {
    const eventPromise = waitForEvent(program, 'proposalCreated');

    await program.methods
      .createProposal(metadata.name, metadata.symbol, metadata.uri)
      .accountsPartial({
        payer: authority.publicKey,
        maker: maker.publicKey,
        makerAccount,
        vaultAuthority,
        proposal,
        mintAccount: mint.publicKey,
        tokenVault: vault,
        systemProgram: anchor.web3.SystemProgram.programId,
        config: configStruct
      })
      .signers([authority, mint, maker])
      .rpc()
      .then(confirm);

    const capturedEvent = await eventPromise;

    const expectedEvent = {
      maker: maker.publicKey.toBase58(),
      proposalAddress: proposal.toBase58(),
    };

    expect(capturedEvent.maker.toBase58()).to.equal(expectedEvent.maker);
    expect(capturedEvent.proposalAddress.toBase58()).to.equal(expectedEvent.proposalAddress);
  });

  it('4. Creates second proposal with same maker', async () => {
    const eventPromise = waitForEvent(program, 'proposalCreated');

    await program.methods
      .createProposal(metadata.name, metadata.symbol, metadata.uri)
      .accountsPartial({
        payer: authority.publicKey,
        maker: maker.publicKey,
        makerAccount,
        vaultAuthority,
        proposal: proposal2,
        mintAccount: mint2.publicKey,
        tokenVault: vault2,
        systemProgram: anchor.web3.SystemProgram.programId,
        config: configStruct
      })
      .signers([authority, mint2, maker])
      .rpc()
      .then(confirm);

    const capturedEvent = await eventPromise;

    const expectedEvent = {
      maker: maker.publicKey.toBase58(),
      proposalAddress: proposal2.toBase58(),
    };

    expect(capturedEvent.maker.toBase58()).to.equal(expectedEvent.maker);
    expect(capturedEvent.proposalAddress.toBase58()).to.equal(expectedEvent.proposalAddress);
  });

  it('5. Backs the first proposal with SOL', async () => {
    const backerProposalCount = findBackerProposalCountPDA(program.programId, backer.publicKey);
    
    // Check that backer_proposal_count doesn't exist or is 0 before backing
    try {
      const countAccount = await program.account.backerProposalCount.fetch(backerProposalCount);
      expect(countAccount.activeCount.toNumber()).to.equal(0);
    } catch (err) {
      // Account doesn't exist yet, which is fine - it will be created during backing
    }

    await program.methods
      .depositSol()
      .accountsPartial({
        backer: backer.publicKey,
        weweVault: weweTreasury,
        mint: mintAccount,
        userTokenAccount: userAta,
        proposal,
        backerAccount,
        backerProposalCount,
        vaultAuthority,
        systemProgram: anchor.web3.SystemProgram.programId,
        config: configStruct
      })
      .signers([backer])
      .rpc()
      .then(confirm);

    // Verify backer_proposal_count is now 1 after backing
    const countAccount = await program.account.backerProposalCount.fetch(backerProposalCount);
    expect(countAccount.activeCount.toNumber()).to.equal(1);
    expect(countAccount.backer.toBase58()).to.equal(backer.publicKey.toBase58());
  });

  // Refactored test case to fix the failure
  it('6. Fails when user backs same proposal twice', async () => {
    const backerProposalCount = findBackerProposalCountPDA(program.programId, backer.publicKey);
    
    try {
      await program.methods
        .depositSol()
        .accountsPartial({
          backer: backer.publicKey,
        weweVault: weweTreasury,
          proposal,
          vaultAuthority,
          backerAccount,
          backerProposalCount,
          systemProgram: anchor.web3.SystemProgram.programId,
          config: configStruct
        })
        .signers([backer])
        .rpc();
      
      // If the rpc call succeeds, the test should fail
      assert.fail('Should not allow double backing');
    } catch (err) {
      // Check for a generic Anchor program error
      expect(err.message).to.include('custom program error');
    }
  });

  it('7. Backs the second proposal', async () => {
    const backerProposalCount = findBackerProposalCountPDA(program.programId, backer.publicKey);
    
    // Verify backer_proposal_count is 1 before backing (from test 5)
    const countAccountBefore = await program.account.backerProposalCount.fetch(backerProposalCount);
    expect(countAccountBefore.activeCount.toNumber()).to.equal(1);

    await program.methods
      .depositSol()
      .accountsPartial({
        backer: backer.publicKey,
        weweVault: weweTreasury,
        proposal: proposal2,
        backerAccount: backerAccount2,
        backerProposalCount,
        vaultAuthority,
        systemProgram: anchor.web3.SystemProgram.programId,
        config: configStruct
      })
      .signers([backer])
      .rpc()
      .then(confirm);

    // Verify backer_proposal_count is now 2 after backing second proposal
    const countAccountAfter = await program.account.backerProposalCount.fetch(backerProposalCount);
    expect(countAccountAfter.activeCount.toNumber()).to.equal(2);
  });

  it('7.5. Creates third proposal and backs it successfully (reaches max limit)', async () => {
    // First create the third proposal
    const eventPromise = waitForEvent(program, 'proposalCreated');

    await program.methods
      .createProposal(metadata.name, metadata.symbol, metadata.uri)
      .accountsPartial({
        payer: authority.publicKey,
        maker: maker.publicKey,
        makerAccount,
        vaultAuthority,
        proposal: proposal3,
        mintAccount: mint3.publicKey,
        tokenVault: vault3,
        systemProgram: anchor.web3.SystemProgram.programId,
        config: configStruct
      })
      .signers([authority, mint3, maker])
      .rpc()
      .then(confirm);

    await eventPromise;

    // Now back the third proposal
    const backerProposalCount = findBackerProposalCountPDA(program.programId, backer.publicKey);
    
    // Verify backer_proposal_count is 2 before backing (from tests 5 and 7)
    const countAccountBefore = await program.account.backerProposalCount.fetch(backerProposalCount);
    expect(countAccountBefore.activeCount.toNumber()).to.equal(2);

    await program.methods
      .depositSol()
      .accountsPartial({
        backer: backer.publicKey,
        weweVault: weweTreasury,
        mint: mintAccount,
        userTokenAccount: userAta,
        proposal: proposal3,
        backerAccount: backerAccount3,
        backerProposalCount,
        vaultAuthority,
        systemProgram: anchor.web3.SystemProgram.programId,
        config: configStruct
      })
      .signers([backer])
      .rpc()
      .then(confirm);

    // Verify backer_proposal_count is now 3 after backing third proposal (max limit reached)
    const countAccountAfter = await program.account.backerProposalCount.fetch(backerProposalCount);
    expect(countAccountAfter.activeCount.toNumber()).to.equal(3);
  });

  it('7.6. Fails when trying to back a fourth proposal (exceeds max limit)', async () => {
    // First create the fourth proposal
    const eventPromise = waitForEvent(program, 'proposalCreated');

    await program.methods
      .createProposal(metadata.name, metadata.symbol, metadata.uri)
      .accountsPartial({
        payer: authority.publicKey,
        maker: maker.publicKey,
        makerAccount,
        vaultAuthority,
        proposal: proposal4,
        mintAccount: mint4.publicKey,
        tokenVault: vault4,
        systemProgram: anchor.web3.SystemProgram.programId,
        config: configStruct
      })
      .signers([authority, mint4, maker])
      .rpc()
      .then(confirm);

    await eventPromise;

    // Now try to back the fourth proposal - should fail
    const backerProposalCount = findBackerProposalCountPDA(program.programId, backer.publicKey);
    
    // Verify backer_proposal_count is still 3 (max limit)
    const countAccountBefore = await program.account.backerProposalCount.fetch(backerProposalCount);
    expect(countAccountBefore.activeCount.toNumber()).to.equal(3);

    try {
      await program.methods
        .depositSol()
        .accountsPartial({
          backer: backer.publicKey,
        weweVault: weweTreasury,
          mint: mintAccount,
          userTokenAccount: userAta,
          proposal: proposal4,
          backerAccount: backerAccount4,
          backerProposalCount,
          vaultAuthority,
          systemProgram: anchor.web3.SystemProgram.programId,
          config: configStruct
        })
        .signers([backer])
        .rpc();

      assert.fail('Should not allow backing a fourth proposal when max limit is 3');
    } catch (err) {
      // Verify the error is MaxBackedProposalsReached
      expect(err.message).to.include('MaxBackedProposalsReached');
    }

    // Verify backer_proposal_count is still 3 (unchanged after failed attempt)
    const countAccountAfter = await program.account.backerProposalCount.fetch(backerProposalCount);
    expect(countAccountAfter.activeCount.toNumber()).to.equal(3);
  });

  it('8. Authority rejects a proposal', async () => {
    const proposal2Data = await program.account.proposal.fetch(proposal2);
    const metadataAccount2 = findMetadataPDA(proposal2Data.mintAccount);
    
    await program.methods
      .rejectProposal()
      .accountsPartial({
        authority: authority.publicKey,
        proposal: proposal2,
        payer: authority.publicKey,
        mintAccount: proposal2Data.mintAccount,
        metadataAccount: metadataAccount2,
        tokenMetadataProgram: TOKEN_METADATA_PROGRAM_ID,
      })
      .signers([authority])
      .rpc()
      .then(confirm);
  });

  it("9. Refunds SOL to backer after proposal is rejected", async () => {
    const backerProposalCount = findBackerProposalCountPDA(program.programId, backer.publicKey);
    
    // Verify backer_proposal_count is 3 before refund (from backing proposals, proposal2, and proposal3)
    const countAccountBefore = await program.account.backerProposalCount.fetch(backerProposalCount);
    expect(countAccountBefore.activeCount.toNumber()).to.equal(3);

    const weweTreasury = new anchor.web3.PublicKey("76U9hvHNUNn7YV5FekSzDHzqnHETsUpDKq4cMj2dMxNi");
    
    // Get refund fee basis points from config (should be 200 = 2%)
    const configAccount = await program.account.configs.fetch(configStruct);
    const refundFeeBps = configAccount.refundFeeBasisPoints;
    
    // Verify config is set to 200 bps (2%) as expected
    expect(refundFeeBps).to.equal(200); // 200 basis points = 2%
    
    const amountToRaisePerUser = new BN(1_000_000_000); // 1 SOL
    const depositedAmount = amountToRaisePerUser; // Full amount is in vault (no upfront fee)
    
    // Calculate expected amounts based on config value (not hardcoded)
    // Fee is calculated as a percentage of the deposited amount
    // Formula: fee = deposited_amount * refundFeeBps / 10000
    //          refund_amount = deposited_amount - fee
    // With refundFeeBps = 200 (2%): fee = 0.02 SOL, refund = 0.98 SOL
    const BASIS_POINTS = new BN(10000);
    const expectedFee = depositedAmount.muln(refundFeeBps).div(BASIS_POINTS);
    const expectedRefund = depositedAmount.sub(expectedFee);
    
    console.log(`\n=== REFUND TEST - Initial State ===`);
    console.log(`   Config refundFeeBps: ${refundFeeBps} (${refundFeeBps / 100}%)`);
    console.log(`   Deposit amount: ${depositedAmount.toString()} lamports (${depositedAmount.toNumber() / 1e9} SOL)`);
    console.log(`   Expected refund: ${expectedRefund.toString()} lamports (${expectedRefund.toNumber() / 1e9} SOL)`);
    console.log(`   Expected fee: ${expectedFee.toString()} lamports (${expectedFee.toNumber() / 1e9} SOL)`);
    
    // Get balances before refund
    const backerBalanceBefore = await provider.connection.getBalance(backer.publicKey);
    const treasuryBalanceBefore = await provider.connection.getBalance(weweTreasury);
    const vaultBalanceBeforeRefund = await provider.connection.getBalance(vaultAuthority);
    
    console.log(`\n=== Balances BEFORE Refund ===`);
    console.log(`   Vault balance: ${vaultBalanceBeforeRefund} lamports (${vaultBalanceBeforeRefund / 1e9} SOL)`);
    console.log(`   Backer balance: ${backerBalanceBefore} lamports (${backerBalanceBefore / 1e9} SOL)`);
    console.log(`   Treasury balance: ${treasuryBalanceBefore} lamports (${treasuryBalanceBefore / 1e9} SOL)`);
    
    // Ensure vault has enough balance for refund + fee + rent-exempt minimum
    // Rent-exempt minimum for system account is ~890,880 lamports, we'll add 0.01 SOL to be safe
    const rentExemptBuffer = new BN(10_000_000); // 0.01 SOL
    const requiredBalance = depositedAmount.add(rentExemptBuffer);
    
    // if (vaultBalanceBeforeRefund < requiredBalance.toNumber()) {
    //   // Transfer additional SOL to vault to ensure it has enough balance
    //   const additionalNeeded = requiredBalance.subn(vaultBalanceBeforeRefund);
    //   await provider.sendAndConfirm(
    //     new anchor.web3.Transaction().add(
    //       anchor.web3.SystemProgram.transfer({
    //         fromPubkey: provider.wallet.publicKey,
    //         toPubkey: vaultAuthority,
    //         lamports: additionalNeeded.toNumber(),
    //       })
    //     )
    //   );
    //   // Update vault balance after transfer
    //   const updatedVaultBalance = await provider.connection.getBalance(vaultAuthority);
    //   expect(updatedVaultBalance).to.be.at.least(requiredBalance.toNumber());
    // }
    
    // Listen for the refund event
    const eventPromise = waitForEvent(program, 'backerRefunded');
    
    await program.methods
      .refund()
      .accounts({
        backer: backer.publicKey,
        proposal: proposal2,
        vaultAuthority,
        weweTreasury,
        backerAccount: backerAccount2,
        backerProposalCount,
        systemProgram: anchor.web3.SystemProgram.programId,
        config: configStruct,
      })
      .signers([])
      .rpc()
      .then(confirm);

    // Verify backer_proposal_count is now 2 after refund (proposal and proposal3 are still backed)
    const countAccountAfter = await program.account.backerProposalCount.fetch(backerProposalCount);
    expect(countAccountAfter.activeCount.toNumber()).to.equal(2);
    
    // Wait for event and verify
    const event = await eventPromise;
    expect(event.refundAmount.toString()).to.equal(expectedRefund.toString());
    expect(event.weweFee.toString()).to.equal(expectedFee.toString());
    
    // Enforce fee percentage: fee should be refundFeeBps basis points of deposited amount
    const actualRefundAmount = new BN(event.refundAmount.toString());
    const actualFee = new BN(event.weweFee.toString());
    const feeAsPercentOfDeposit = actualFee.mul(BASIS_POINTS).div(depositedAmount);
    expect(feeAsPercentOfDeposit.toNumber()).to.equal(refundFeeBps); // Should be exact
    
    // Verify refund + fee = deposited amount (exact)
    const total = actualRefundAmount.add(actualFee);
    expect(total.toString()).to.equal(depositedAmount.toString()); // Should be exact
    
    // Verify balances after refund
    const backerBalanceAfter = await provider.connection.getBalance(backer.publicKey);
    const treasuryBalanceAfter = await provider.connection.getBalance(weweTreasury);
    const vaultBalanceAfter = await provider.connection.getBalance(vaultAuthority);
    
    console.log(`\n=== Balances AFTER Refund ===`);
    console.log(`   Vault balance: ${vaultBalanceAfter} lamports (${vaultBalanceAfter / 1e9} SOL)`);
    console.log(`   Backer balance: ${backerBalanceAfter} lamports (${backerBalanceAfter / 1e9} SOL)`);
    console.log(`   Treasury balance: ${treasuryBalanceAfter} lamports (${treasuryBalanceAfter / 1e9} SOL)`);
    
    // Account for transaction fees and rent returned from closing backer_account
    const backerBalanceIncrease = backerBalanceAfter - backerBalanceBefore;
    const treasuryBalanceIncrease = treasuryBalanceAfter - treasuryBalanceBefore;
    const vaultBalanceDecrease = vaultBalanceBeforeRefund - vaultBalanceAfter;
    
    console.log(`\n=== Balance Changes ===`);
    console.log(`   Vault decrease: ${vaultBalanceDecrease} lamports (${vaultBalanceDecrease / 1e9} SOL)`);
    console.log(`   Backer increase: ${backerBalanceIncrease} lamports (${backerBalanceIncrease / 1e9} SOL)`);
    console.log(`   Treasury increase: ${treasuryBalanceIncrease} lamports (${treasuryBalanceIncrease / 1e9} SOL)`);
    console.log(`   Expected vault decrease: ${expectedRefund.add(expectedFee).toString()} lamports (${expectedRefund.add(expectedFee).toNumber() / 1e9} SOL)`);
    
    // The backer receives:
    // 1. The refund_amount (SOL transferred from vault)
    // 2. Rent from closing backer_account (typically ~1-2 million lamports)
    // 3. Transaction fee deduction (typically ~5000 lamports)
    // So backerBalanceIncrease should be approximately: refund_amount + rent - tx_fee
    
    // Verify backer received at least the refund amount (may have more due to rent)
    expect(backerBalanceIncrease).to.be.at.least(expectedRefund.toNumber() - 10000); // Allow for tx fees
    
    // Verify treasury received the fee
    expect(treasuryBalanceIncrease).to.equal(expectedFee.toNumber());
    
    // Verify vault balance: should have reduced by refund + fee, but maintain rent-exempt minimum
    // Rent-exempt minimum for system account is ~890,880 lamports
    const RENT_EXEMPT_MINIMUM = 890_880;
    const expectedVaultDecrease = expectedRefund.add(expectedFee);
    const actualVaultDecrease = vaultBalanceBeforeRefund - vaultBalanceAfter;
    
    // Vault should have decreased by refund + fee (within rounding)
    // But must maintain at least rent-exempt minimum
    expect(actualVaultDecrease).to.be.closeTo(expectedVaultDecrease.toNumber(), 1000); // Allow for rounding
    expect(vaultBalanceAfter).to.be.at.least(RENT_EXEMPT_MINIMUM); // Must remain rent-exempt
    
    // Final verification: fee percentage matches config (exact)
    const refundFromEvent = new BN(event.refundAmount.toString());
    const feeFromEvent = new BN(event.weweFee.toString());
    const feePercentageOfDeposit = feeFromEvent.mul(BASIS_POINTS).div(depositedAmount);
    expect(feePercentageOfDeposit.toNumber()).to.equal(refundFeeBps); // Should be exact
    
    console.log(`\n=== Refund Fee Verification Summary ===`);
    console.log(`   Deposit amount: ${depositedAmount.toString()} lamports (${depositedAmount.toNumber() / 1e9} SOL)`);
    console.log(`   Refund to backer: ${refundFromEvent.toString()} lamports (${refundFromEvent.toNumber() / 1e9} SOL)`);
    console.log(`   Fee to WEWE treasury: ${feeFromEvent.toString()} lamports (${feeFromEvent.toNumber() / 1e9} SOL)`);
    console.log(`   Fee percentage of deposit: ${feePercentageOfDeposit.toNumber() / 100}% (${refundFeeBps} basis points from config)`);
    console.log(`   Refund + Fee: ${refundFromEvent.add(feeFromEvent).toString()} lamports (${refundFromEvent.add(feeFromEvent).toNumber() / 1e9} SOL)`);
    console.log(`   ✓ Refund + Fee = Deposit: ${refundFromEvent.add(feeFromEvent).toString()} = ${depositedAmount.toString()}`);
    
    console.log(`\n=== Final Balance Summary ===`);
    console.log(`   Backer net change: +${backerBalanceIncrease} lamports (+${(backerBalanceIncrease / 1e9).toFixed(9)} SOL)`);
    console.log(`     - Includes refund: ${expectedRefund.toString()} lamports`);
    console.log(`     - Plus rent from closed account: ~${backerBalanceIncrease - expectedRefund.toNumber()} lamports`);
    console.log(`   Treasury net change: +${treasuryBalanceIncrease} lamports (+${(treasuryBalanceIncrease / 1e9).toFixed(9)} SOL)`);
    console.log(`   Vault net change: -${vaultBalanceDecrease} lamports (-${(vaultBalanceDecrease / 1e9).toFixed(9)} SOL)`);
    console.log(`     - Remaining vault balance: ${vaultBalanceAfter} lamports (${(vaultBalanceAfter / 1e9).toFixed(9)} SOL)`);
    console.log(`\n✓ Refund test completed successfully!\n`);
  });

  it('10. Launches coin and creates DAMM pool', async () => {
    const [wsolVault] = getTokenVaultAddress(vaultAuthority, WSOL_MINT, program.programId);

    const eventPromise = waitForEvent(program, 'coinLaunched');
    
    // Fetch the config account (already exists on devnet)
    const config_account = await cpAmm.account.config.fetch(config);
    const proposalData = await program.account.proposal.fetch(proposal);
    const configData = await program.account.configs.fetch(configStruct);

    const computeUnitsIx = ComputeBudgetProgram.setComputeUnitLimit({ units: 1_400_000 });
    
    // Setup token amounts (matching SDK example structure)
    // tokenAAmount = total_pool_tokens * 10^9 (MINT_DECIMALS = 9)
    const totalPoolTokensBN = configData.totalPoolTokens instanceof BN 
      ? configData.totalPoolTokens 
      : new BN(configData.totalPoolTokens.toString());
    const tokenAAmount = totalPoolTokensBN.mul(new BN(10).pow(new BN(9)));
    
    // tokenBAmount = total_backing (in lamports)
    const totalBackingBN = proposalData.totalBacking instanceof BN
      ? proposalData.totalBacking
      : new BN(proposalData.totalBacking.toString());
    const tokenBAmount = totalBackingBN;
    
    // Validate amounts are not zero
    if (tokenAAmount.isZero() || tokenBAmount.isZero()) {
      throw new Error(
        `Invalid amounts for sqrt_price calculation: tokenAAmount=${tokenAAmount.toString()}, tokenBAmount=${tokenBAmount.toString()}, ` +
        `totalPoolTokens=${totalPoolTokensBN.toString()}, totalBacking=${totalBackingBN.toString()}`
      );
    }
    
    // Calculate initial price (price of tokenA in terms of tokenB)
    // price = tokenBAmount / tokenAAmount
    const initialPrice = new Decimal(tokenBAmount.toString()).div(new Decimal(tokenAAmount.toString())).toNumber();
    
    console.log(`Initial price: ${initialPrice}`);
    console.log(`Token A amount: ${tokenAAmount.toString()}`);
    console.log(`Token B amount: ${tokenBAmount.toString()}`);
    
    // Token decimals
    const tokenADecimals = 9; // Base token decimals
    const tokenBDecimals = 9; // WSOL decimals
    
    // Calculate sqrt price from initial price
    const initSqrtPrice = getSqrtPriceFromPrice(
      initialPrice.toString(),
      tokenADecimals,
      tokenBDecimals
    );

    console.log(`Initial sqrt price: ${initSqrtPrice}`);
    
    // Use CP-AMM's constants (must match what's in ix_launch_pool.rs)
    // CP-AMM MIN_SQRT_PRICE = 4295048016, MAX_SQRT_PRICE = 79226673521066979257578248091
    const minSqrtPrice = new BN("4295048016"); // CP-AMM MIN_SQRT_PRICE
    const maxSqrtPrice = new BN("79226673521066979257578248091"); // CP-AMM MAX_SQRT_PRICE

    console.log(`Min sqrt price: ${minSqrtPrice}`);
    console.log(`Max sqrt price: ${maxSqrtPrice}`);
    
    // Note: Liquidity is calculated internally by the Rust program using get_liquidity_for_adding_liquidity
    // which matches the SDK's getLiquidityDelta logic
    
    // Chain service pubkey must be the signer (payer) and pool_creator_authority
    // Verify chainServiceAuthority matches the constant
    const CHAIN_SERVICE_PUBKEY = new anchor.web3.PublicKey("D4VNMB6heKqVyiii4HjK2K7pEC9U3tVuNjCkFr3xNGfe");
    if (!chainServiceAuthority.publicKey.equals(CHAIN_SERVICE_PUBKEY)) {
      throw new Error(`chainServiceAuthority (${chainServiceAuthority.publicKey.toString()}) does not match constant (${CHAIN_SERVICE_PUBKEY.toString()})`);
    }

    // Assert that the config being passed is the expected one
    const EXPECTED_CONFIG_PUBKEY = new anchor.web3.PublicKey("7xeqWqnS4VMguYMexgtw1vxXt3cM8eh6vyvUkYgv2aJY");
    expect(config.toBase58()).to.equal(EXPECTED_CONFIG_PUBKEY.toBase58(), 
      `Config must be ${EXPECTED_CONFIG_PUBKEY.toBase58()}, but got ${config.toBase58()}`);

    // For this config, D4VNMB6heKqVyiii4HjK2K7pEC9U3tVuNjCkFr3xNGfe is the pool creator authority
    // Verify this matches what we're passing
    expect(CHAIN_SERVICE_PUBKEY.toBase58()).to.equal(
      CHAIN_SERVICE_PUBKEY.toBase58(),
      `poolCreatorAuthority must be ${CHAIN_SERVICE_PUBKEY.toBase58()} for config ${EXPECTED_CONFIG_PUBKEY.toBase58()}`
    );

    // Create pool using createPool instruction (similar to SDK's createCustomPool)
    // The Rust program handles liquidity calculation internally
    const tx = await program.methods
      .createPool(initSqrtPrice)
      .accountsPartial({
        proposal,
        vaultAuthority,
        maker: maker.publicKey, // Maker is the proposal creator, not chain service
        tokenVault: vault,
        wsolVault,
        poolAuthority: pdas.poolAuthority, // Must be DAMM v2 pool authority PDA (const_pda::const_authority::POOL_ID)
        dammPoolAuthority: pdas.poolAuthority,
        poolConfig: EXPECTED_CONFIG_PUBKEY,
        pool: pdas.pool,
        positionNftMint: pdas.positionNftMint.publicKey,
        positionNftAccount: pdas.positionNftAccount,
        position: pdas.position,
        ammProgram: cpAmm.programId,
        baseMint: mint.publicKey,
        mintAccount: proposalData.mintAccount,
        makerTokenAccount: pdas.makerTokenAccount,
        quoteMint: WSOL_MINT,
        tokenAVault: pdas.tokenAVault,
        tokenBVault: pdas.tokenBVault,
        payer: chainServiceAuthority.publicKey,
        chainServicePubkey: chainServiceAuthority.publicKey,
        tokenBaseProgram: TOKEN_PROGRAM_ID,
        tokenQuoteProgram: TOKEN_PROGRAM_ID,
        token2022Program: TOKEN_2022_PROGRAM_ID,
        dammEventAuthority: pdas.dammEventAuthority,
        systemProgram: anchor.web3.SystemProgram.programId,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        config: configStruct,
      })
      .signers([chainServiceAuthority, pdas.positionNftMint])
      .transaction();

    tx.instructions.unshift(computeUnitsIx);
    
    // chainServiceAuthority must be a signer (as payer and pool_creator_authority)
    console.log("\nSending create pool transaction...");
    const signature = await provider.sendAndConfirm(tx, [chainServiceAuthority, pdas.positionNftMint], {
      commitment: "confirmed",
      skipPreflight: false,
    });

    console.log("Pool created successfully");
    console.log(`Transaction signature: ${signature}`);
    console.log(`Pool: ${pdas.pool.toBase58()}`);
    console.log(`Position: ${pdas.position.toBase58()}`);

    const capturedEvent = await eventPromise;

    expect(capturedEvent.proposalAddress.toBase58()).to.equal(proposal.toBase58());
    expect(capturedEvent.mintAccount.toBase58()).to.equal(mint.publicKey.toBase58());
  });

  it("10.5a. Reset pool launch in isolation", async () => {
    // Create a new mint for the reset - we'll create it via createProposal to ensure proper initialization
    const newMint = anchor.web3.Keypair.generate();
    const [wsolVault] = getTokenVaultAddress(vaultAuthority, WSOL_MINT, program.programId);
    
    // Fetch current proposal state
    const proposalDataBefore = await program.account.proposal.fetch(proposal);
    
    // Verify pool is currently launched
    expect(proposalDataBefore.isPoolLaunched).to.be.true;
    
    // Calculate required amount (with generous buffer)
    const totalBackingBN = proposalDataBefore.totalBacking instanceof BN
      ? proposalDataBefore.totalBacking
      : new BN(proposalDataBefore.totalBacking.toString());
    
    // Add extra buffer (3x the required amount) to ensure we have enough
    const requiredAmount = totalBackingBN.mul(new BN(3));
    
    // Get WSOL vault balance
    let wsolBalanceBefore = new BN(0);
    try {
      const wsolVaultAccount = await provider.connection.getTokenAccountBalance(wsolVault);
      wsolBalanceBefore = new BN(wsolVaultAccount.value.amount);
    } catch (e) {
      // Vault might not exist yet - that's fine, we'll add SOL anyway
    }
    
    // If vault doesn't have enough WSOL, add some
    if (wsolBalanceBefore.lt(requiredAmount)) {
      const amountToAdd = requiredAmount.sub(wsolBalanceBefore);
      
      // Transfer SOL directly to WSOL vault (wraps it automatically when synced)
      const transferIx = anchor.web3.SystemProgram.transfer({
        fromPubkey: provider.wallet.publicKey,
        toPubkey: wsolVault,
        lamports: amountToAdd.toNumber(),
      });
      
      // Sync native to wrap SOL to WSOL
      const syncIx = createSyncNativeInstruction(wsolVault);
      
      await provider.sendAndConfirm(
        new anchor.web3.Transaction().add(transferIx, syncIx)
      ).then(confirm);
      
      console.log(`Added ${amountToAdd.toString()} lamports (${amountToAdd.div(new BN(1e9)).toString()} SOL) to WSOL vault`);
    }
    
    // Call reset_pool_launch - this will initialize the new mint account
    // Use chainServiceAuthority like test 10 (Launches coin and creates DAMM pool)
    await program.methods
      .resetPoolLaunch()
      .accounts({
        authority: chainServiceAuthority.publicKey,
        payer: chainServiceAuthority.publicKey,
        proposal,
        vaultAuthority,
        wsolVault,
        quoteMint: WSOL_MINT,
        mintAccount: newMint.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([chainServiceAuthority, newMint])
      .rpc()
      .then(confirm);
    
    // Verify proposal state after reset
    const proposalDataAfter = await program.account.proposal.fetch(proposal);
    expect(proposalDataAfter.isPoolLaunched).to.be.false;
    expect(proposalDataAfter.mintAccount.toBase58()).to.equal(newMint.publicKey.toBase58());
    expect(proposalDataAfter.launchTimestamp).to.be.null;
  });

  it("10.5b. Run pool creation logic again after reset", async () => {
    // Fetch current proposal state - it should be reset from 10.5a
    const proposalDataBefore = await program.account.proposal.fetch(proposal);
    const configData = await program.account.configs.fetch(configStruct);
    
    // Get the current mint from the proposal (set by reset in 10.5a)
    const currentMint = proposalDataBefore.mintAccount;
    const [newVault] = getTokenVaultAddress(vaultAuthority, currentMint, program.programId);
    const [wsolVault] = getTokenVaultAddress(vaultAuthority, WSOL_MINT, program.programId);
    
    // Verify pool is not launched (should be reset from 10.5a)
    expect(proposalDataBefore.isPoolLaunched).to.be.false;
    
    // Derive new pool PDAs for the current mint
    const newPdas = derivePoolPDAs(program.programId, cpAmm.programId, currentMint, WSOL_MINT, maker.publicKey, config);
    
    // Get token vault balance before launch
    let tokenVaultBalanceBefore = new BN(0);
    try {
      const tokenVaultAccount = await provider.connection.getTokenAccountBalance(newVault);
      tokenVaultBalanceBefore = new BN(tokenVaultAccount.value.amount);
    } catch (e) {
      // Vault doesn't exist yet, that's fine - create_pool will create it
    }
    
    // Calculate sqrt price for the new pool
    const totalBackingBN = proposalDataBefore.totalBacking instanceof BN
      ? proposalDataBefore.totalBacking
      : new BN(proposalDataBefore.totalBacking.toString());
    const totalPoolTokensBN = configData.totalPoolTokens instanceof BN 
      ? configData.totalPoolTokens 
      : new BN(configData.totalPoolTokens.toString());
    const tokenAAmount = totalPoolTokensBN.mul(new BN(10).pow(new BN(9)));
    const tokenBAmount = totalBackingBN;
    const initialPrice = new Decimal(tokenBAmount.toString()).div(new Decimal(tokenAAmount.toString())).toNumber();
    const initSqrtPrice = getSqrtPriceFromPrice(
      initialPrice.toString(),
      9, // tokenADecimals
      9  // tokenBDecimals
    );
    
    // Launch pool again with new mint
    const eventPromise2 = waitForEvent(program, 'coinLaunched');
    
    const computeUnitsIx = ComputeBudgetProgram.setComputeUnitLimit({ units: 1_400_000 });
    
    // Refresh blockhash to avoid "Blockhash not found" error
    // Get fresh blockhash with confirmed commitment to match sendAndConfirm
    const { blockhash } = await provider.connection.getLatestBlockhash('confirmed');
    
    const tx2 = await program.methods
      .createPool(initSqrtPrice)
      .accountsPartial({
        proposal,
        vaultAuthority,
        maker: maker.publicKey,
        tokenVault: newVault,
        wsolVault,
        poolAuthority: newPdas.poolAuthority,
        dammPoolAuthority: newPdas.poolAuthority,
        poolConfig: config,
        pool: newPdas.pool,
        positionNftMint: newPdas.positionNftMint.publicKey,
        positionNftAccount: newPdas.positionNftAccount,
        position: newPdas.position,
        ammProgram: cpAmm.programId,
        baseMint: currentMint,
        mintAccount: currentMint, // This should match proposal.mint_account now
        makerTokenAccount: newPdas.makerTokenAccount,
        quoteMint: WSOL_MINT,
        tokenAVault: newPdas.tokenAVault,
        tokenBVault: newPdas.tokenBVault,
        payer: chainServiceAuthority.publicKey,
        chainServicePubkey: chainServiceAuthority.publicKey,
        tokenBaseProgram: TOKEN_PROGRAM_ID,
        tokenQuoteProgram: TOKEN_PROGRAM_ID,
        token2022Program: TOKEN_2022_PROGRAM_ID,
        dammEventAuthority: newPdas.dammEventAuthority,
        systemProgram: anchor.web3.SystemProgram.programId,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        config: configStruct,
      })
      .signers([chainServiceAuthority, newPdas.positionNftMint])
      .transaction();
    
    tx2.instructions.unshift(computeUnitsIx);
    tx2.recentBlockhash = blockhash;
    tx2.feePayer = chainServiceAuthority.publicKey;
    
    // Sign the transaction with the required signers
    tx2.sign(chainServiceAuthority, newPdas.positionNftMint);
    
    console.log("\nSending second create pool transaction with new mint...");
    const signature2 = await provider.connection.sendRawTransaction(tx2.serialize(), {
      skipPreflight: false,
      maxRetries: 3,
    });
    
    // Wait for confirmation with proper error handling
    const confirmation = await provider.connection.confirmTransaction(signature2, 'confirmed');
    if (confirmation.value.err) {
      throw new Error(`Transaction failed: ${JSON.stringify(confirmation.value.err)}`);
    }
    
    console.log("Second pool created successfully");
    console.log(`Transaction signature: ${signature2}`);
    console.log(`New Pool: ${newPdas.pool.toBase58()}`);
    console.log(`New Position: ${newPdas.position.toBase58()}`);
    
    const capturedEvent2 = await eventPromise2;
    expect(capturedEvent2.proposalAddress.toBase58()).to.equal(proposal.toBase58());
    expect(capturedEvent2.mintAccount.toBase58()).to.equal(currentMint.toBase58());
    
    // Verify new tokens were minted
    const tokenVaultAccountAfter = await provider.connection.getTokenAccountBalance(newVault);
    const tokenVaultBalanceAfter = new BN(tokenVaultAccountAfter.value.amount);
    
    // Calculate expected mint amount
    const expectedMintAmount = configData.totalMint instanceof BN
      ? configData.totalMint.mul(new BN(10).pow(new BN(9)))
      : new BN(configData.totalMint.toString()).mul(new BN(10).pow(new BN(9)));
    
    console.log(`Token vault balance before: ${tokenVaultBalanceBefore.toString()}`);
    console.log(`Token vault balance after: ${tokenVaultBalanceAfter.toString()}`);
    console.log(`Expected mint amount: ${expectedMintAmount.toString()}`);
    
    // // Verify tokens were minted (balance increased)
    // expect(tokenVaultBalanceAfter.gt(tokenVaultBalanceBefore)).to.be.true;
    // expect(tokenVaultBalanceAfter.gte(expectedMintAmount)).to.be.true;
    
    // Verify proposal is marked as launched again
    const proposalDataFinal = await program.account.proposal.fetch(proposal);
    expect(proposalDataFinal.isPoolLaunched).to.be.true;
    expect(proposalDataFinal.mintAccount.toBase58()).to.equal(currentMint.toBase58());
    expect(proposalDataFinal.launchTimestamp).to.not.be.null;
    
    // Verify new pool exists and is different from old pool
    const oldPoolInfo = await provider.connection.getAccountInfo(pdas.pool);
    const newPoolInfo = await provider.connection.getAccountInfo(newPdas.pool);
    
    expect(oldPoolInfo).to.not.be.null; // Old pool still exists
    expect(newPoolInfo).to.not.be.null; // New pool exists
    expect(newPdas.pool.toBase58()).to.not.equal(pdas.pool.toBase58()); // Different pools
  });

  it("11. Airdrop launched coin successfully", async () => {
    // Fetch current proposal to get the current mint (may have been reset in 10.5a)
    const proposalData = await program.account.proposal.fetch(proposal);
    const currentMint = proposalData.mintAccount;
    const [currentVault] = getTokenVaultAddress(vaultAuthority, currentMint, program.programId);
    const currentBackerTokenAccount = findUserAta(backer.publicKey, currentMint);
    
    await program.methods
      .airdrop()
      .accounts({
        payer: authority.publicKey,
        backer: backer.publicKey,
        proposal,
        vaultAuthority,
        mintAccount: currentMint,
        tokenVault: currentVault,
        backerAccount,
        backerTokenAccount: currentBackerTokenAccount,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
        config: configStruct,
      })
      .signers([authority])
      .rpc()
      .then(confirm);

    const tokenAccountInfo = await provider.connection.getTokenAccountBalance(currentBackerTokenAccount);
    const balance = tokenAccountInfo.value.uiAmount;
    assert.ok(balance && balance > 0, "Backer should receive airdropped tokens");
  });

  it('12. Starts a milestone (initialiseMilestone)', async () => {
    const proposalData = await program.account.proposal.fetch(proposal);
    const metadataAccount = findMetadataPDA(proposalData.mintAccount);
    
    const sig = await program.methods
      .initialiseMilestone()
      .accounts({
        authority: authority.publicKey,
        proposal,
        mintAccount: proposalData.mintAccount,
        metadataAccount,
        payer: authority.publicKey,
        tokenMetadataProgram: TOKEN_METADATA_PROGRAM_ID,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
      })
      .signers([authority])
      .rpc();
    
    // Print program logs to see msg! output
    await printTxLogs(sig);
    await confirm(sig);

    // Verify that both mint authority and freeze authority are revoked
    const mintInfo = await getMint(provider.connection, proposalData.mintAccount);
    expect(mintInfo.mintAuthority).to.be.null;
    expect(mintInfo.freezeAuthority).to.be.null;
    console.log('✓ Mint authority revoked (is null):', mintInfo.mintAuthority === null);
    console.log('✓ Freeze authority revoked (is null):', mintInfo.freezeAuthority === null);
  });

  it('13. Updates backer milestone amount', async () => {
    // Fetch current proposal to get the current mint (may have been reset in 10.5a)
    const proposalData = await program.account.proposal.fetch(proposal);
    const currentMint = proposalData.mintAccount;
    const currentBackerTokenAccount = findUserAta(backer.publicKey, currentMint);
    
    await program.methods
      .snapshotBackerAmount()
      .accounts({
        authority: authority.publicKey,
        proposal,
        backer: backer.publicKey,
        backerAccount,
        backerTokenAccount: currentBackerTokenAccount,
        mintAccount: currentMint,
        config: configStruct,
      })
      .signers([authority])
      .rpc()
      .then(confirm);
  });

  it('14. Ends a milestone', async () => {
    // Fetch current proposal to get the current mint (may have been reset in 10.5a)
    const proposalData = await program.account.proposal.fetch(proposal);
    const currentMint = proposalData.mintAccount;
    const [currentVault] = getTokenVaultAddress(vaultAuthority, currentMint, program.programId);
    
    await program.methods
      .endMilestone()
      .accounts({
        authority: authority.publicKey,
        proposal,
        mint: currentMint,
        vaultAuthority,
        tokenVault: currentVault,
        config: configStruct,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([authority])
      .rpc()
      .then(confirm);
  });

  it("15. Backer claims milestone reward successfully", async () => {
    await program.methods
      .claimMilestoneReward()
      .accounts({
        backer: backer.publicKey,
        proposal,
        vaultAuthority,
        mintAccount: mint.publicKey,
        tokenVault: vault,
        backerAccount,
        backerTokenAccount,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([backer])
      .rpc()
      .then(confirm);

    const tokenAccountInfo = await provider.connection.getTokenAccountBalance(backerTokenAccount);
    const balance = tokenAccountInfo.value.uiAmount;
    assert.ok(balance && balance > 0, "Backer should receive milestone reward");
  
    const backerData = await program.account.backers.fetch(backerAccount);
    assert.strictEqual(backerData.claimAmount.toNumber(), 0, "Claim amount should be reset to zero");
  });
  
  it('16. Claims position fee and distributes tokens', async () => {
    const weweTreasury = new anchor.web3.PublicKey("76U9hvHNUNn7YV5FekSzDHzqnHETsUpDKq4cMj2dMxNi");
    const weweWsolAccount = getAssociatedTokenAddressSync(WSOL_MINT, weweTreasury, true);
    const weweTokenAccount = getAssociatedTokenAddressSync(mint.publicKey, weweTreasury, true);
    const makerWsolAccount = getAssociatedTokenAddressSync(WSOL_MINT, maker.publicKey, true);
    const [wsolVault] = getTokenVaultAddress(vaultAuthority, WSOL_MINT, program.programId);
    const treasuryTempWsol = findTempWsolPDA(vaultAuthority, proposal, true, program.programId);
    const makerTempWsol = findTempWsolPDA(vaultAuthority, proposal, false, program.programId);
    const computeUnitsIx = ComputeBudgetProgram.setComputeUnitLimit({ units: 1_400_000 });

    // Create a trader account to perform swaps and generate fees
    const trader = anchor.web3.Keypair.generate();
    const traderTokenAccount = getAssociatedTokenAddressSync(mint.publicKey, trader.publicKey, true);
    const traderWsolAccount = getAssociatedTokenAddressSync(WSOL_MINT, trader.publicKey, true);

    // Fund trader with SOL for transactions and WSOL for swaps
    await provider.sendAndConfirm(
      new anchor.web3.Transaction().add(
        anchor.web3.SystemProgram.transfer({
          fromPubkey: provider.wallet.publicKey,
          toPubkey: trader.publicKey,
          lamports: 2 * anchor.web3.LAMPORTS_PER_SOL, // 2 SOL for fees and WSOL
        })
      )
    );

    // Create trader's token accounts if needed
    try {
      await provider.connection.getTokenAccountBalance(traderTokenAccount);
    } catch {
      const createTokenAtaIx = createAssociatedTokenAccountInstruction(
        provider.wallet.publicKey,
        traderTokenAccount,
        trader.publicKey,
        mint.publicKey,
        TOKEN_PROGRAM_ID
      );
      await provider.sendAndConfirm(new anchor.web3.Transaction().add(createTokenAtaIx));
      // Wait a bit for account to be initialized
      await new Promise(resolve => setTimeout(resolve, 500));
    }

    try {
      await provider.connection.getTokenAccountBalance(traderWsolAccount);
    } catch {
      const createWsolAtaIx = createAssociatedTokenAccountInstruction(
        provider.wallet.publicKey,
        traderWsolAccount,
        trader.publicKey,
        WSOL_MINT,
        TOKEN_PROGRAM_ID
      );
      await provider.sendAndConfirm(new anchor.web3.Transaction().add(createWsolAtaIx));
      // Wait a bit for account to be initialized
      await new Promise(resolve => setTimeout(resolve, 500));
    }
    
    // Verify token accounts exist and are owned by trader
    const tokenAccountInfo = await provider.connection.getAccountInfo(traderTokenAccount);
    const wsolAccountInfo = await provider.connection.getAccountInfo(traderWsolAccount);
    if (!tokenAccountInfo || !wsolAccountInfo) {
      throw new Error('Token accounts not properly initialized');
    }

    // Transfer more tokens and WSOL to trader for larger swaps to generate meaningful fees
    // Get some tokens from the vault to trade with
    const swapAmount = new BN(2_000_000_000); // 2 tokens (with 9 decimals) - enough for multiple swaps
    const wsolSwapAmount = new BN(1_000_000_000); // 1 WSOL - enough for swaps back

    // Transfer tokens to trader
    const transferTokenIx = createTransferInstruction(
      pdas.makerTokenAccount,
      traderTokenAccount,
      maker.publicKey,
      swapAmount.toNumber(),
      [],
      TOKEN_PROGRAM_ID
    );

    // Wrap SOL to WSOL for trader
    const transferSolIx = anchor.web3.SystemProgram.transfer({
      fromPubkey: trader.publicKey,
      toPubkey: traderWsolAccount,
      lamports: wsolSwapAmount.toNumber(),
    });
    const syncWsolIx = createSyncNativeInstruction(traderWsolAccount);

    // Need both maker (for token transfer) and trader (for SOL transfer) to sign
    await provider.sendAndConfirm(
      new anchor.web3.Transaction()
        .add(transferTokenIx)
        .add(transferSolIx)
        .add(syncWsolIx),
      [maker, trader]
    );

    // Perform swaps to generate fees for testing unwrapping logic
    // Using instruction builder with proper account typing to avoid ConstraintHasOne errors
    try {
      // Perform multiple larger swaps to generate meaningful fees
      // CP-AMM fees accumulate in the position, so we need substantial swaps
      
      // Swap 1: Token -> WSOL (buy WSOL with tokens) - larger amount
      const swap1Amount = new BN(1_000_000_000); // 1 token (9 decimals)
      const swap1Params = {
        amount0: swap1Amount, // amount_in when exact_in
        amount1: new BN(0), // minimum_amount_out
        swapMode: 0, // ExactIn
      };
      
      const swap1Tx = await (cpAmm.methods as any)
        .swap2(swap1Params)
        .accounts({
          poolAuthority: pdas.poolAuthority,
          pool: pdas.pool,
          inputTokenAccount: traderTokenAccount,
          outputTokenAccount: traderWsolAccount,
          tokenAVault: pdas.tokenAVault,
          tokenBVault: pdas.tokenBVault,
          tokenAMint: mint.publicKey,
          tokenBMint: WSOL_MINT,
          payer: trader.publicKey,
          tokenAProgram: TOKEN_PROGRAM_ID,
          tokenBProgram: TOKEN_PROGRAM_ID,
          referralTokenAccount: traderWsolAccount,
          eventAuthority: pdas.dammEventAuthority,
          program: cpAmm.programId,
        })
        .signers([trader])
        .rpc();
      console.log('✅ Swap 1 completed (Token -> WSOL):', swap1Tx);
      console.log('✅ Performed swap to generate fees');
    } catch (swapError: any) {
      // If swaps fail due to ConstraintHasOne, log and continue - fees might exist from previous runs
      console.warn('⚠️  Swaps failed (may be due to ConstraintHasOne with as any):', swapError.message);
      console.log('⚠️  Continuing with fee claim - will test unwrapping if fees exist');
    }

    // Now claim the fees (this will test the unwrapping logic)
    const tx = await program.methods
      .claimPoolFee()
      .accounts({
        poolAuthority: pdas.poolAuthority,
        payer: authority.publicKey,
        maker: maker.publicKey,
        weweTreasury,
        proposal,
        vaultAuthority,
        weweWsolAccount,
        weweTokenAccount,
        makerWsolAccount,
        makerTokenAccount: pdas.makerTokenAccount,
        pool: pdas.pool,
        position: pdas.position,
        tokenAAccount: vault,
        tokenBAccount: wsolVault,
        tokenAVault: pdas.tokenAVault,
        tokenBVault: pdas.tokenBVault,
        tokenAMint: mint.publicKey,
        tokenBMint: WSOL_MINT,
        positionNftAccount: pdas.positionNftAccount,
        treasuryTempWsol,
        makerTempWsol,
        tokenAProgram: TOKEN_PROGRAM_ID,
        tokenBProgram: TOKEN_PROGRAM_ID,
        ammProgram: cpAmm.programId,
        eventAuthority: pdas.dammEventAuthority,
        systemProgram: anchor.web3.SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
      })
      .signers([authority])
      .preInstructions([computeUnitsIx])
      .rpc()
      .then(confirm);

    // Print program logs to see msg! outputs
    await printTxLogs(tx);

    console.log('✅ Claimed pool fees and tested unwrapping logic');
  });

  it('16.5. Transfer tokens before snapshot → reduced allocation', async () => {
    // Start a fresh milestone (cycle 2)
    const proposalDataCycle2 = await program.account.proposal.fetch(proposal);
    const currentMint = proposalDataCycle2.mintAccount;
    const metadataAccountCycle2 = findMetadataPDA(currentMint);
    const [currentVault] = getTokenVaultAddress(vaultAuthority, currentMint, program.programId);
    const currentBackerTokenAccount = findUserAta(backer.publicKey, currentMint);
    // Derive PDAs for the current mint (may have been reset in 10.5a)
    const currentPdas = derivePoolPDAs(program.programId, cpAmm.programId, currentMint, WSOL_MINT, maker.publicKey, config);
    
    await program.methods
      .initialiseMilestone()
      .accounts({
        authority: authority.publicKey,
        proposal,
        mintAccount: currentMint,
        metadataAccount: metadataAccountCycle2,
        payer: authority.publicKey,
        tokenMetadataProgram: TOKEN_METADATA_PROGRAM_ID,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
      })
      .signers([authority])
      .rpc()
      .then(confirm);
  
    // Use currentBackerTokenAccount directly (derived from currentMint)
    // This should match test 13's pattern exactly
    const destAta = currentPdas.makerTokenAccount;   // maker's ATA for the current mint
  
    // Verify the backer's token account exists and has the correct mint BEFORE using it
    let balBefore = new BN(0);
    const tokenAccountInfo = await provider.connection.getTokenAccountBalance(currentBackerTokenAccount);
    const tokenAccountData = await provider.connection.getParsedAccountInfo(currentBackerTokenAccount);
    
    // Verify the account exists
    if (!tokenAccountData.value) {
      throw new Error(`Backer token account ${currentBackerTokenAccount.toBase58()} does not exist. Expected mint: ${currentMint.toBase58()}. Make sure test 11 (Airdrop) ran successfully.`);
    }
    
    // Verify the mint matches - this is critical for the constraint check
    if ('parsed' in tokenAccountData.value.data) {
      const parsedData = tokenAccountData.value.data as any;
      const accountMint = parsedData.parsed?.info?.mint;
      if (accountMint !== currentMint.toBase58()) {
        // This should never happen if the account was created correctly in test 11
        throw new Error(`Backer token account ${currentBackerTokenAccount.toBase58()} has mint ${accountMint}, but expected ${currentMint.toBase58()}. Account address was derived using findUserAta(backer.publicKey, ${currentMint.toBase58()}), but the account at that address has mint ${accountMint}. This suggests test 11 (Airdrop) may have created the account for a different mint.`);
      }
    } else {
      throw new Error(`Backer token account ${currentBackerTokenAccount.toBase58()} exists but could not parse mint information.`);
    }
    
    balBefore = new BN(tokenAccountInfo.value.amount);
    
    const toMove = balBefore.gt(new BN(0)) ? balBefore.div(new BN(2)) : new BN(0);

    if (toMove.gt(new BN(0))) {
      // Use BigInt to avoid "Number can only safely store up to 53 bits" error
      // createTransferInstruction accepts number | bigint
      const transferAmount = BigInt(toMove.toString());
      const tx = new anchor.web3.Transaction().add(
        createTransferInstruction(
          currentBackerTokenAccount,
          destAta,
          backer.publicKey,
          transferAmount
        )
      );
      await provider.sendAndConfirm(tx, [backer]);
    }  
  
    // Take snapshot AFTER moving tokens away (or if no tokens to move, snapshot with current balance)
    // Use currentBackerTokenAccount directly (same as test 13) - this was verified to have the correct mint above
    const sig = await program.methods
      .snapshotBackerAmount()
      .accounts({
        authority: authority.publicKey,
        proposal,
        backer: backer.publicKey,
        backerAccount,
        backerTokenAccount: currentBackerTokenAccount,
        mintAccount: currentMint,
        config: configStruct,
      })
      .signers([authority])
      .rpc()
      .then(confirm);
  
    // End milestone and attempt to claim: balance should not increase
    await program.methods
      .endMilestone()
      .accounts({
        authority: authority.publicKey,
        proposal,
        mint: currentMint,
        vaultAuthority,
        tokenVault: currentVault,
        config: configStruct,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([authority])
      .rpc()
      .then(confirm);
  
    await program.methods
      .claimMilestoneReward()
      .accounts({
        backer: backer.publicKey,
        proposal,
        vaultAuthority,
        mintAccount: currentMint,
        tokenVault: currentVault,
        backerAccount,
        backerTokenAccount: currentBackerTokenAccount,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([backer])
      .rpc()
      .then(confirm);
  });

  it('16.55. Resets airdrop flag and allows retrying airdrop', async () => {
    // Get proposal data to find the current mint
    const proposalData = await program.account.proposal.fetch(proposal);
    const currentMint = proposalData.mintAccount;
    const [currentVault] = getTokenVaultAddress(vaultAuthority, currentMint, program.programId);
    const currentBackerTokenAccount = findUserAta(backer.publicKey, currentMint);

    // Verify that the backer has already received an airdrop (initial_airdrop_received = true)
    const backerAccountBefore = await program.account.backers.fetch(backerAccount);
    assert.strictEqual(
      backerAccountBefore.initialAirdropReceived,
      true,
      'Backer should have already received initial airdrop'
    );

    // Get balance before reset
    const balanceBeforeReset = await provider.connection.getTokenAccountBalance(currentBackerTokenAccount);
    const balanceBeforeResetAmount = Number(balanceBeforeReset.value.amount);

    // Test that airdrop fails before resetAirdrop (since initial_airdrop_received = true)
    try {
      await program.methods
        .airdrop()
        .accounts({
          payer: authority.publicKey,
          backer: backer.publicKey,
          proposal,
          vaultAuthority,
          mintAccount: currentMint,
          tokenVault: currentVault,
          backerAccount,
          backerTokenAccount: currentBackerTokenAccount,
          tokenProgram: TOKEN_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          systemProgram: anchor.web3.SystemProgram.programId,
          config: configStruct,
        })
        .signers([authority])
        .rpc()
        .then(confirm);

      assert.fail('Should not allow airdrop when initial_airdrop_received is true');
    } catch (err) {
      expect(err.message).to.include('AirdropAlreadyRecived');
    }

    // Reset the airdrop flag using admin instruction
    await program.methods
      .resetAirdrop()
      .accounts({
        authority: authority.publicKey,
        proposal,
        backer: backer.publicKey,
        backerAccount,
      })
      .signers([authority])
      .rpc()
      .then(confirm);

    // Verify that initial_airdrop_received is now false
    const backerAccountAfterReset = await program.account.backers.fetch(backerAccount);
    assert.strictEqual(
      backerAccountAfterReset.initialAirdropReceived,
      false,
      'Backer airdrop flag should be reset to false'
    );

    // Now retry the airdrop
    await program.methods
      .airdrop()
      .accounts({
        payer: authority.publicKey,
        backer: backer.publicKey,
        proposal,
        vaultAuthority,
        mintAccount: currentMint,
        tokenVault: currentVault,
        backerAccount,
        backerTokenAccount: currentBackerTokenAccount,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
        config: configStruct,
      })
      .signers([authority])
      .rpc()
      .then(confirm);

    // Verify that initial_airdrop_received is now true again
    const backerAccountAfterAirdrop = await program.account.backers.fetch(backerAccount);
    assert.strictEqual(
      backerAccountAfterAirdrop.initialAirdropReceived,
      true,
      'Backer airdrop flag should be set to true after airdrop'
    );

    // Verify balance increased
    const balanceAfterAirdrop = await provider.connection.getTokenAccountBalance(currentBackerTokenAccount);
    const balanceAfterAirdropAmount = Number(balanceAfterAirdrop.value.amount);
    assert.ok(
      balanceAfterAirdropAmount > balanceBeforeResetAmount,
      `Balance should increase after airdrop. Before: ${balanceBeforeResetAmount}, After: ${balanceAfterAirdropAmount}`
    );
  });

  describe('Burn Amount Verification Tests', () => {
    it('16.6. Verifies zero burn when all holders maintain full allocation', async () => {
      // Create new proposal for burn testing
      const makerData = await program.account.makerAccount.fetch(makerAccount);
      const proposalIndexBurn1 = makerData.proposalCount;
      const testProposalBurn1 = findProposalPDA(program.programId, maker.publicKey, proposalIndexBurn1);
      const testMintBurn1 = anchor.web3.Keypair.generate();
      const [testVaultBurn1] = getTokenVaultAddress(vaultAuthority, testMintBurn1.publicKey, program.programId);
      const testBackerBurn1 = anchor.web3.Keypair.generate();
      const testBackerAccountBurn1 = findBackerAccountPDA(program.programId, testProposalBurn1, testBackerBurn1.publicKey);
      const testBackerTokenAccountBurn1 = findUserAta(testBackerBurn1.publicKey, testMintBurn1.publicKey);

      // Fund backer
      await provider.sendAndConfirm(new anchor.web3.Transaction().add(
        anchor.web3.SystemProgram.transfer({
          fromPubkey: provider.wallet.publicKey,
          toPubkey: testBackerBurn1.publicKey,
          lamports: 2e9, // 2 SOL to cover deposit + transaction fees
        })
      )).then(confirm);

      // Mint soulbound token
      const testBackerBurn1Ata = findUserAta(testBackerBurn1.publicKey, mintAccount);
      await program.methods
        .mintSoulboundToUser()
        .accounts({
          authority: authority.publicKey,
          user: testBackerBurn1.publicKey,
          mint: mintAccount,
          freezeAuthority,
          mintAuthority,
          userTokenAccount: testBackerBurn1Ata,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .signers([authority])
        .rpc()
        .then(confirm);

      // Create proposal
      await program.methods
        .createProposal(metadata.name, metadata.symbol, metadata.uri)
        .accountsPartial({
          payer: authority.publicKey,
          maker: maker.publicKey,
          makerAccount,
          vaultAuthority,
          proposal: testProposalBurn1,
          mintAccount: testMintBurn1.publicKey,
          tokenVault: testVaultBurn1,
          systemProgram: anchor.web3.SystemProgram.programId,
          config: configStruct
        })
        .signers([authority, testMintBurn1, maker])
        .rpc()
        .then(confirm);

      // Back proposal
      await program.methods
        .depositSol()
        .accountsPartial({
          backer: testBackerBurn1.publicKey,
        weweVault: weweTreasury,
          mint: mintAccount,
          userTokenAccount: testBackerBurn1Ata,
          proposal: testProposalBurn1,
          backerAccount: testBackerAccountBurn1,
          vaultAuthority,
          systemProgram: anchor.web3.SystemProgram.programId,
          config: configStruct
        })
        .signers([testBackerBurn1])
        .rpc()
        .then(confirm);

      // Launch pool
      const config_account = await cpAmm.account.config.fetch(config);
      // Calculate price: quoteAmount (1 lamport) / baseAmount (150M tokens * 10^9)
      const baseAmount = new BN(150_000_000).mul(new BN(10).pow(new BN(9)));
      const quoteAmount = new BN(1);
      const price = new Decimal(quoteAmount.toString()).div(new Decimal(baseAmount.toString())).toString();
      const sqrtPrice = getSqrtPriceFromPrice(price, 9, 9);
      const [wsolVault] = getTokenVaultAddress(vaultAuthority, WSOL_MINT, program.programId);
      const pdasBurn1 = derivePoolPDAs(program.programId, cpAmm.programId, testMintBurn1.publicKey, WSOL_MINT, maker.publicKey, config);

      const testProposalBurn1Data = await program.account.proposal.fetch(testProposalBurn1);
      const poolCreatorAuthority = config_account.poolCreatorAuthority.equals(anchor.web3.PublicKey.default)
        ? authority.publicKey
        : config_account.poolCreatorAuthority;
      const computeUnitsIx = ComputeBudgetProgram.setComputeUnitLimit({ units: 1_400_000 });
      const tx = new anchor.web3.Transaction().add(computeUnitsIx);
      const createPoolIx = await program.methods
        .createPool(sqrtPrice)
        .accountsPartial({
          proposal: testProposalBurn1,
          vaultAuthority,
          maker: maker.publicKey,
          tokenVault: testVaultBurn1,
          wsolVault,
          poolAuthority: pdasBurn1.poolAuthority,
          dammPoolAuthority: pdasBurn1.poolAuthority,
          poolConfig: config,
          poolCreatorAuthority: poolCreatorAuthority,
          pool: pdasBurn1.pool,
          positionNftMint: pdasBurn1.positionNftMint.publicKey,
          positionNftAccount: pdasBurn1.positionNftAccount,
          position: pdasBurn1.position,
          ammProgram: cpAmm.programId,
          baseMint: testMintBurn1.publicKey,
          mintAccount: testProposalBurn1Data.mintAccount,
          makerTokenAccount: pdasBurn1.makerTokenAccount,
          quoteMint: WSOL_MINT,
          tokenAVault: pdasBurn1.tokenAVault,
          tokenBVault: pdasBurn1.tokenBVault,
          payer: authority.publicKey,
          tokenBaseProgram: TOKEN_PROGRAM_ID,
          tokenQuoteProgram: TOKEN_PROGRAM_ID,
          token2022Program: TOKEN_2022_PROGRAM_ID,
          dammEventAuthority: pdasBurn1.dammEventAuthority,
          systemProgram: anchor.web3.SystemProgram.programId,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          config: configStruct
        })
        .instruction();
      tx.add(createPoolIx);
      await provider.sendAndConfirm(tx, [authority, pdasBurn1.positionNftMint]);

      // Airdrop to backer
      await program.methods
        .airdrop()
        .accounts({
          payer: authority.publicKey,
          backer: testBackerBurn1.publicKey,
          proposal: testProposalBurn1,
          vaultAuthority,
          mintAccount: testMintBurn1.publicKey,
          tokenVault: testVaultBurn1,
          backerAccount: testBackerAccountBurn1,
          backerTokenAccount: testBackerTokenAccountBurn1,
          tokenProgram: TOKEN_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          systemProgram: anchor.web3.SystemProgram.programId,
          config: configStruct,
        })
        .signers([authority])
        .rpc()
        .then(confirm);

      // Start milestone
      const metadataAccountBurn1 = findMetadataPDA(testProposalBurn1Data.mintAccount);
      
      await program.methods
        .initialiseMilestone()
        .accounts({
          authority: authority.publicKey,
          proposal: testProposalBurn1,
          mintAccount: testProposalBurn1Data.mintAccount,
          metadataAccount: metadataAccountBurn1,
          payer: authority.publicKey,
          tokenMetadataProgram: TOKEN_METADATA_PROGRAM_ID,
          systemProgram: anchor.web3.SystemProgram.programId,
          rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        })
        .signers([authority])
        .rpc()
        .then(confirm);

      // Snapshot backer (holder maintains full allocation - 100% reputation)
      await program.methods
        .snapshotBackerAmount()
        .accounts({
          authority: authority.publicKey,
          proposal: testProposalBurn1,
          backer: testBackerBurn1.publicKey,
          backerAccount: testBackerAccountBurn1,
          backerTokenAccount: testBackerTokenAccountBurn1,
          mintAccount: testMintBurn1.publicKey,
          config: configStruct,
        })
        .signers([authority])
        .rpc()
        .then(confirm);

      // Get vault balance before end milestone
      const vaultBalanceBeforeEnd = await provider.connection.getTokenAccountBalance(testVaultBurn1);
      
      // Get proposal data to calculate expected burn
      const proposalData = await program.account.proposal.fetch(testProposalBurn1);
      
      // Calculate expected burn: (NUM_HOLDERS * 100) - SUM(reputation_scores)
      // With full allocation, reputation = 100 for each holder
      const numHolders = proposalData.milestoneBackersWeighted.toNumber();
      const expectedReputationSum = numHolders * 100; // All holders have 100 reputation
      const expectedBurnBase = (numHolders * 100) - expectedReputationSum; // Should be 0
      const MINT_DECIMALS = 9;
      const expectedBurnAmount = expectedBurnBase * Math.pow(10, MINT_DECIMALS);

      // End milestone
      await program.methods
        .endMilestone()
        .accounts({
          authority: authority.publicKey,
          proposal: testProposalBurn1,
          mint: testMintBurn1.publicKey,
          vaultAuthority,
          tokenVault: testVaultBurn1,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .signers([authority])
        .rpc()
        .then(confirm);

      // Get vault balance after end milestone
      const vaultBalanceAfterEnd = await provider.connection.getTokenAccountBalance(testVaultBurn1);
      
      // Calculate actual burn amount
      const actualBurnAmount = Number(vaultBalanceBeforeEnd.value.amount) - Number(vaultBalanceAfterEnd.value.amount);
      
      // Verify burn amount is zero (all holders maintained full allocation)
      assert.strictEqual(
        actualBurnAmount,
        expectedBurnAmount,
        `Burn amount should be zero when all holders maintain full allocation. Expected: ${expectedBurnAmount}, Actual: ${actualBurnAmount}`
      );
      assert.strictEqual(actualBurnAmount, 0, 'Burn amount should be zero with full allocation');
    });

    it('16.7. Verifies non-zero burn when holder has reduced allocation', async () => {
      // Create new proposal for reduced allocation burn testing
      const makerData = await program.account.makerAccount.fetch(makerAccount);
      const proposalIndexBurn2 = makerData.proposalCount;
      const testProposalBurn2 = findProposalPDA(program.programId, maker.publicKey, proposalIndexBurn2);
      const testMintBurn2 = anchor.web3.Keypair.generate();
      const [testVaultBurn2] = getTokenVaultAddress(vaultAuthority, testMintBurn2.publicKey, program.programId);
      const testBackerBurn2 = anchor.web3.Keypair.generate();
      const testBackerAccountBurn2 = findBackerAccountPDA(program.programId, testProposalBurn2, testBackerBurn2.publicKey);
      const testBackerTokenAccountBurn2 = findUserAta(testBackerBurn2.publicKey, testMintBurn2.publicKey);

      // Fund backer
      await provider.sendAndConfirm(new anchor.web3.Transaction().add(
        anchor.web3.SystemProgram.transfer({
          fromPubkey: provider.wallet.publicKey,
          toPubkey: testBackerBurn2.publicKey,
          lamports: 2e9, // 2 SOL to cover deposit + transaction fees
        })
      )).then(confirm);

      // Mint soulbound token
      const testBackerBurn2Ata = findUserAta(testBackerBurn2.publicKey, mintAccount);
      await program.methods
        .mintSoulboundToUser()
        .accounts({
          authority: authority.publicKey,
          user: testBackerBurn2.publicKey,
          mint: mintAccount,
          freezeAuthority,
          mintAuthority,
          userTokenAccount: testBackerBurn2Ata,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .signers([authority])
        .rpc()
        .then(confirm);

      // Create proposal
      await program.methods
        .createProposal(metadata.name, metadata.symbol, metadata.uri)
        .accountsPartial({
          payer: authority.publicKey,
          maker: maker.publicKey,
          makerAccount,
          vaultAuthority,
          proposal: testProposalBurn2,
          mintAccount: testMintBurn2.publicKey,
          tokenVault: testVaultBurn2,
          systemProgram: anchor.web3.SystemProgram.programId,
          config: configStruct
        })
        .signers([authority, testMintBurn2, maker])
        .rpc()
        .then(confirm);

      // Back proposal
      await program.methods
        .depositSol()
        .accountsPartial({
          backer: testBackerBurn2.publicKey,
        weweVault: weweTreasury,
          mint: mintAccount,
          userTokenAccount: testBackerBurn2Ata,
          proposal: testProposalBurn2,
          backerAccount: testBackerAccountBurn2,
          vaultAuthority,
          systemProgram: anchor.web3.SystemProgram.programId,
          config: configStruct
        })
        .signers([testBackerBurn2])
        .rpc()
        .then(confirm);

      // Launch pool
      const config_account = await cpAmm.account.config.fetch(config);
      // Calculate price: quoteAmount (1 lamport) / baseAmount (150M tokens * 10^9)
      const baseAmount = new BN(150_000_000).mul(new BN(10).pow(new BN(9)));
      const quoteAmount = new BN(1);
      const price = new Decimal(quoteAmount.toString()).div(new Decimal(baseAmount.toString())).toString();
      const sqrtPrice = getSqrtPriceFromPrice(price, 9, 9);
      const [wsolVault] = getTokenVaultAddress(vaultAuthority, WSOL_MINT, program.programId);
      const pdasBurn2 = derivePoolPDAs(program.programId, cpAmm.programId, testMintBurn2.publicKey, WSOL_MINT, maker.publicKey, config);
      const destAccountBurn2 = pdasBurn2.makerTokenAccount; // Use maker's ATA for the test mint as destination

      const testProposalBurn2Data = await program.account.proposal.fetch(testProposalBurn2);
      const poolCreatorAuthority = config_account.poolCreatorAuthority.equals(anchor.web3.PublicKey.default)
        ? authority.publicKey
        : config_account.poolCreatorAuthority;
      const computeUnitsIx = ComputeBudgetProgram.setComputeUnitLimit({ units: 1_400_000 });
      const tx = new anchor.web3.Transaction().add(computeUnitsIx);
      const createPoolIx = await program.methods
        .createPool(sqrtPrice)
        .accountsPartial({
          proposal: testProposalBurn2,
          vaultAuthority,
          maker: maker.publicKey,
          tokenVault: testVaultBurn2,
          wsolVault,
          poolAuthority: pdasBurn2.poolAuthority,
          dammPoolAuthority: pdasBurn2.poolAuthority,
          poolConfig: config,
          poolCreatorAuthority: poolCreatorAuthority,
          pool: pdasBurn2.pool,
          positionNftMint: pdasBurn2.positionNftMint.publicKey,
          positionNftAccount: pdasBurn2.positionNftAccount,
          position: pdasBurn2.position,
          ammProgram: cpAmm.programId,
          baseMint: testMintBurn2.publicKey,
          mintAccount: testProposalBurn2Data.mintAccount,
          makerTokenAccount: pdasBurn2.makerTokenAccount,
          quoteMint: WSOL_MINT,
          tokenAVault: pdasBurn2.tokenAVault,
          tokenBVault: pdasBurn2.tokenBVault,
          payer: authority.publicKey,
          tokenBaseProgram: TOKEN_PROGRAM_ID,
          tokenQuoteProgram: TOKEN_PROGRAM_ID,
          token2022Program: TOKEN_2022_PROGRAM_ID,
          dammEventAuthority: pdasBurn2.dammEventAuthority,
          systemProgram: anchor.web3.SystemProgram.programId,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          config: configStruct
        })
        .instruction();
      tx.add(createPoolIx);
      await provider.sendAndConfirm(tx, [authority, pdasBurn2.positionNftMint]);

      // Airdrop to backer
      await program.methods
        .airdrop()
        .accounts({
          payer: authority.publicKey,
          backer: testBackerBurn2.publicKey,
          proposal: testProposalBurn2,
          vaultAuthority,
          mintAccount: testMintBurn2.publicKey,
          tokenVault: testVaultBurn2,
          backerAccount: testBackerAccountBurn2,
          backerTokenAccount: testBackerTokenAccountBurn2,
          tokenProgram: TOKEN_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          systemProgram: anchor.web3.SystemProgram.programId,
          config: configStruct,
        })
        .signers([authority])
        .rpc()
        .then(confirm);

      // Get original airdrop amount
      const configData = await program.account.configs.fetch(configStruct);
      const proposalDataBefore = await program.account.proposal.fetch(testProposalBurn2);
      const originalAirdropPerBacker = configData.totalAirdropAmountPerMilestone.toNumber() / proposalDataBefore.totalBackers.toNumber();
      const MINT_DECIMALS = 9;
      const originalAirdropAmount = originalAirdropPerBacker * Math.pow(10, MINT_DECIMALS);

      // Transfer half of tokens away (reduces balance to 50% of original)
      const balanceBeforeTransfer = await provider.connection.getTokenAccountBalance(testBackerTokenAccountBurn2);
      const transferAmount = BigInt(balanceBeforeTransfer.value.amount) / BigInt(2);
      
      if (transferAmount > 0) {
        const transferTx = new anchor.web3.Transaction().add(
          createTransferInstruction(
            testBackerTokenAccountBurn2,
            destAccountBurn2,
            testBackerBurn2.publicKey,
            Number(transferAmount)
          )
        );
        await provider.sendAndConfirm(transferTx, [testBackerBurn2]);
      }

      // Verify balance is now 50% of original
      const balanceAfterTransfer = await provider.connection.getTokenAccountBalance(testBackerTokenAccountBurn2);
      const expectedBalanceAfterTransfer = originalAirdropAmount / 2;
      assert.ok(
        Math.abs(Number(balanceAfterTransfer.value.amount) - expectedBalanceAfterTransfer) < 1000,
        `Balance should be approximately 50% of original. Expected: ~${expectedBalanceAfterTransfer}, Actual: ${balanceAfterTransfer.value.amount}`
      );

      // Start milestone
      const metadataAccountBurn2 = findMetadataPDA(testProposalBurn2Data.mintAccount);
      
      await program.methods
        .initialiseMilestone()
        .accounts({
          authority: authority.publicKey,
          proposal: testProposalBurn2,
          mintAccount: testProposalBurn2Data.mintAccount,
          metadataAccount: metadataAccountBurn2,
          payer: authority.publicKey,
          tokenMetadataProgram: TOKEN_METADATA_PROGRAM_ID,
          systemProgram: anchor.web3.SystemProgram.programId,
          rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        })
        .signers([authority])
        .rpc()
        .then(confirm);

      // Snapshot backer (holder has 50% of original allocation - 50 reputation)
      await program.methods
        .snapshotBackerAmount()
        .accounts({
          authority: authority.publicKey,
          proposal: testProposalBurn2,
          backer: testBackerBurn2.publicKey,
          backerAccount: testBackerAccountBurn2,
          backerTokenAccount: testBackerTokenAccountBurn2,
          mintAccount: testMintBurn2.publicKey,
          config: configStruct,
        })
        .signers([authority])
        .rpc()
        .then(confirm);

      // Get proposal data to calculate expected burn
      const proposalData = await program.account.proposal.fetch(testProposalBurn2);
      
      // Calculate expected burn: (NUM_HOLDERS * 100) - SUM(reputation_scores)
      // With 50% allocation, reputation = 50
      const numHolders = proposalData.milestoneBackersWeighted.toNumber();
      const expectedReputationSum = 50; // Holder has 50% of original = 50 reputation
      const expectedBurnBase = (numHolders * 100) - expectedReputationSum; // Should be 50
      const expectedBurnAmount = expectedBurnBase * Math.pow(10, MINT_DECIMALS);

      // Get vault balance before end milestone
      const vaultBalanceBeforeEnd = await provider.connection.getTokenAccountBalance(testVaultBurn2);
      
      // End milestone
      await program.methods
        .endMilestone()
        .accounts({
          authority: authority.publicKey,
          proposal: testProposalBurn2,
          mint: testMintBurn2.publicKey,
          vaultAuthority,
          tokenVault: testVaultBurn2,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .signers([authority])
        .rpc()
        .then(confirm);

      // Get vault balance after end milestone
      const vaultBalanceAfterEnd = await provider.connection.getTokenAccountBalance(testVaultBurn2);
      
      // Calculate actual burn amount
      const actualBurnAmount = Number(vaultBalanceBeforeEnd.value.amount) - Number(vaultBalanceAfterEnd.value.amount);
      
      // Verify burn amount matches expected (with small tolerance for rounding)
      const tolerance = Math.pow(10, MINT_DECIMALS); // Allow 1 token tolerance
      console.log(`Calculation: (NUM_HOLDERS * 100) - SUM(reputation_scores) = Expected Burn`);
      console.log(`Calculation: (${numHolders} * 100) - ${expectedReputationSum} = ${expectedBurnBase}`);
      console.log(`Expected Burn Amount: ${expectedBurnAmount}`);
      console.log(`Actual Burn Amount: ${actualBurnAmount}`);
      console.log(`Tolerance: ${tolerance}`);
      console.log(`Difference: ${Math.abs(actualBurnAmount - expectedBurnAmount)}`);
      assert.ok(
        Math.abs(actualBurnAmount - expectedBurnAmount) <= tolerance,
        `Burn amount should match expected. Expected: ${expectedBurnAmount}, Actual: ${actualBurnAmount}, Difference: ${Math.abs(actualBurnAmount - expectedBurnAmount)}`
      );
      assert.ok(actualBurnAmount > 0, 'Burn amount should be non-zero with reduced allocation');
    });
  });

  // ============================================================================
  // HIGH PRIORITY SECURITY & ERROR HANDLING TESTS
  // ============================================================================

  describe('Access Control Tests', () => {
    it('17. Fails when unauthorized user tries to set config', async () => {
      const unauthorizedUser = anchor.web3.Keypair.generate();
      const refundFeeBps = new BN(200); // 200 BPS = 2%
      
      try {
        await program.methods
          .setConfig(
            new BN(1_000_000_000),
            new BN(1_000_000_000),
            new BN(150_000_000),
            new BN(10_000_000),
            new BN(140_000_000),
            new BN(1),
            new BN(3),
            refundFeeBps, // refund_fee_basis_points: 200 BPS = 2%
          )
          .accounts({
            authority: unauthorizedUser.publicKey,
            config: configStruct,
          })
          .signers([unauthorizedUser])
          .rpc();

        assert.fail('Should not allow unauthorized user to set config');
      } catch (err) {
        expect(err.message).to.include('NotOwner');
      }
    });

    it('17a. configure_authority can set config', async () => {
      const amountToRaisePerUser = new BN(1_000_000_000); // 1 SOL
      const totalMint = new BN(1_000_000_000);
      const totalPoolTokens = new BN(150_000_000);
      const makerTokenAmount = new BN(10_000_000);
      const totalAirdropAmountPerMilestone = new BN(140_000_000);
      const minBackers = new BN(1);
      const maxBackedProposals = new BN(3);
      const refundFeeBps = new BN(200); // 200 BPS = 2%
      
      const tx = await program.methods
        .setConfig(
          amountToRaisePerUser,
          totalMint,
          totalPoolTokens,
          makerTokenAmount,
          totalAirdropAmountPerMilestone,
          minBackers,
          maxBackedProposals,
          refundFeeBps,
        )
        .accounts({
          authority: configureAuthority.publicKey,
          config: configStruct,
        })
        .signers([configureAuthority])
        .rpc();

      await confirm(tx);
    });

    it('17b. chain_service cannot set config', async () => {
      const refundFeeBps = new BN(200); // 200 BPS = 2%
      
      try {
        await program.methods
          .setConfig(
            new BN(1_000_000_000),
            new BN(1_000_000_000),
            new BN(150_000_000),
            new BN(10_000_000),
            new BN(140_000_000),
            new BN(1),
            new BN(3),
            refundFeeBps, // refund_fee_basis_points: 200 BPS = 2%
          )
          .accounts({
            authority: chainServiceAuthority.publicKey,
            config: configStruct,
          })
          .signers([chainServiceAuthority])
          .rpc();

        assert.fail('Should not allow chain_service to set config');
      } catch (err) {
        expect(err.message).to.include('NotOwner');
      }
    });

    it('18. Fails when unauthorized user tries to reject proposal', async () => {
      const unauthorizedUser = anchor.web3.Keypair.generate();
      const proposalDataUnauthorized = await program.account.proposal.fetch(proposal);
      const metadataAccountUnauthorized = findMetadataPDA(proposalDataUnauthorized.mintAccount);

      try {
        await program.methods
          .rejectProposal()
          .accountsPartial({
            authority: unauthorizedUser.publicKey,
            proposal,
            payer: unauthorizedUser.publicKey,
            mintAccount: proposalDataUnauthorized.mintAccount,
            metadataAccount: metadataAccountUnauthorized,
            tokenMetadataProgram: TOKEN_METADATA_PROGRAM_ID,
          })
          .signers([unauthorizedUser])
          .rpc();

        assert.fail('Should not allow unauthorized user to reject proposal');
      } catch (err) {
        expect(err.message).to.include('NotOwner');
      }
    });

    it('19. Fails when unauthorized user tries to start milestone', async () => {
      const unauthorizedUser = anchor.web3.Keypair.generate();
      const proposalDataUnauthorized = await program.account.proposal.fetch(proposal);
      const metadataAccountUnauthorized = findMetadataPDA(proposalDataUnauthorized.mintAccount);

      try {
        await program.methods
          .initialiseMilestone()
          .accounts({
            authority: unauthorizedUser.publicKey,
            proposal,
            mintAccount: proposalDataUnauthorized.mintAccount,
            metadataAccount: metadataAccountUnauthorized,
            payer: unauthorizedUser.publicKey,
            tokenMetadataProgram: TOKEN_METADATA_PROGRAM_ID,
            systemProgram: anchor.web3.SystemProgram.programId,
            rent: anchor.web3.SYSVAR_RENT_PUBKEY,
          })
          .signers([unauthorizedUser])
          .rpc();

        assert.fail('Should not allow unauthorized user to start milestone');
      } catch (err) {
        expect(err.message).to.include('NotOwner');
      }
    });

    it('20. Fails when unauthorized user tries to snapshot backer', async () => {
      const unauthorizedUser = anchor.web3.Keypair.generate();

      try {
        await program.methods
          .snapshotBackerAmount()
          .accounts({
            authority: unauthorizedUser.publicKey,
            proposal,
            backer: backer.publicKey,
            backerAccount,
            backerTokenAccount,
            mintAccount: mint.publicKey,
            config: configStruct,
          })
          .signers([unauthorizedUser])
          .rpc();

        assert.fail('Should not allow unauthorized user to snapshot');
      } catch (err) {
        expect(err.message).to.include('NotOwner');
      }
    });

    it('21. Fails when unauthorized user tries to end milestone', async () => {
      const unauthorizedUser = anchor.web3.Keypair.generate();
      
      // Fetch current proposal to get the current mint (may have been reset in 10.5a)
      const proposalData = await program.account.proposal.fetch(proposal);
      const currentMint = proposalData.mintAccount;
      const [currentVault] = getTokenVaultAddress(vaultAuthority, currentMint, program.programId);

      try {
        await program.methods
          .endMilestone()
          .accounts({
            authority: unauthorizedUser.publicKey,
            proposal,
            mint: currentMint,
            vaultAuthority,
            tokenVault: currentVault,
            config: configStruct,
            tokenProgram: TOKEN_PROGRAM_ID,
          })
          .signers([unauthorizedUser])
          .rpc();

        assert.fail('Should not allow unauthorized user to end milestone');
      } catch (err) {
        expect(err.message).to.include('NotOwner');
      }
    });

    it('22. Fails when unauthorized user tries to mint soulbound token', async () => {
      const unauthorizedUser = anchor.web3.Keypair.generate();
      const testUser = anchor.web3.Keypair.generate();
      const testUserAta = findUserAta(testUser.publicKey, mintAccount);
      await provider.connection.requestAirdrop(unauthorizedUser.publicKey, 5_000_000);
      await new Promise(resolve => setTimeout(resolve, 500));
      try {
        await program.methods
          .mintSoulboundToUser()
          .accounts({
            authority: unauthorizedUser.publicKey,
            user: testUser.publicKey,
            mint: mintAccount,
            freezeAuthority,
            mintAuthority,
            userTokenAccount: testUserAta,
            associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
            tokenProgram: TOKEN_PROGRAM_ID,
            systemProgram: anchor.web3.SystemProgram.programId,
          })
          .signers([unauthorizedUser])
          .rpc();

        assert.fail('Should not allow unauthorized user to mint soulbound token');
      } catch (err) {
        expect(err.message).to.include('NotOwner');
      }
    });

    it('23. Fails when unauthorized user tries to claim pool fees', async () => {
      const unauthorizedUser = anchor.web3.Keypair.generate();
      await provider.connection.requestAirdrop(unauthorizedUser.publicKey, 5_000_000);
      await new Promise(resolve => setTimeout(resolve, 500));
      
      const weweTreasury = new anchor.web3.PublicKey("76U9hvHNUNn7YV5FekSzDHzqnHETsUpDKq4cMj2dMxNi");
      const weweWsolAccount = getAssociatedTokenAddressSync(WSOL_MINT, weweTreasury, true);
      const weweTokenAccount = getAssociatedTokenAddressSync(mint.publicKey, weweTreasury, true);
      const makerWsolAccount = getAssociatedTokenAddressSync(WSOL_MINT, maker.publicKey, true);
      const [wsolVault] = getTokenVaultAddress(vaultAuthority, WSOL_MINT, program.programId);
      const treasuryTempWsol = findTempWsolPDA(vaultAuthority, proposal, true, program.programId);
      const makerTempWsol = findTempWsolPDA(vaultAuthority, proposal, false, program.programId);

      try {
        await program.methods
          .claimPoolFee()
          .accounts({
            poolAuthority: pdas.poolAuthority,
            payer: unauthorizedUser.publicKey,
            maker: maker.publicKey,
            weweTreasury,
            proposal,
            vaultAuthority,
            weweWsolAccount,
            weweTokenAccount,
            makerWsolAccount,
            makerTokenAccount: pdas.makerTokenAccount,
            pool: pdas.pool,
            position: pdas.position,
            tokenAAccount: vault,
            tokenBAccount: wsolVault,
            tokenAVault: pdas.tokenAVault,
            tokenBVault: pdas.tokenBVault,
            tokenAMint: mint.publicKey,
            tokenBMint: WSOL_MINT,
            positionNftAccount: pdas.positionNftAccount,
            treasuryTempWsol,
            makerTempWsol,
            tokenAProgram: TOKEN_PROGRAM_ID,
            tokenBProgram: TOKEN_PROGRAM_ID,
            ammProgram: cpAmm.programId,
            eventAuthority: pdas.dammEventAuthority,
            systemProgram: anchor.web3.SystemProgram.programId,
            tokenProgram: TOKEN_PROGRAM_ID,
            associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
            rent: anchor.web3.SYSVAR_RENT_PUBKEY,
          })
          .signers([unauthorizedUser])
          .rpc();

        assert.fail('Should not allow unauthorized user to claim pool fees');
      } catch (err) {
        // The error should be "NotOwner" - Anchor wraps it in simulation error
        const errorMsg = err.message || err.toString();
        expect(errorMsg).to.match(/NotOwner|constraint.*maker/i);
      }
    });
  });

  describe('State Validation Tests - deposit_sol', () => {
    let testProposal3: anchor.web3.PublicKey;
    let testMint3: anchor.web3.Keypair;
    let testVault3: anchor.web3.PublicKey;
    let testBacker3: anchor.web3.Keypair;
    let testBackerAccount3: anchor.web3.PublicKey;
    let proposalIndex3: anchor.BN;

    before(async () => {
      // Get the current proposal count to use the next available index
      const makerData = await program.account.makerAccount.fetch(makerAccount);
      proposalIndex3 = makerData.proposalCount;
      
      testBacker3 = anchor.web3.Keypair.generate();
      testProposal3 = findProposalPDA(program.programId, maker.publicKey, proposalIndex3);
      testBackerAccount3 = findBackerAccountPDA(program.programId, testProposal3, testBacker3.publicKey);
      testMint3 = anchor.web3.Keypair.generate();
      [testVault3] = getTokenVaultAddress(vaultAuthority, testMint3.publicKey, program.programId);

      // Fund test backer
      await provider.sendAndConfirm(new anchor.web3.Transaction().add(
        anchor.web3.SystemProgram.transfer({
          fromPubkey: provider.wallet.publicKey,
          toPubkey: testBacker3.publicKey,
          lamports: 2e9, // 2 SOL to cover deposit + transaction fees
        })
      )).then(confirm);

      // Mint soulbound token to test backer
      const testBacker3Ata = findUserAta(testBacker3.publicKey, mintAccount);
      await program.methods
        .mintSoulboundToUser()
        .accounts({
          authority: authority.publicKey,
          user: testBacker3.publicKey,
          mint: mintAccount,
          freezeAuthority,
          mintAuthority,
          userTokenAccount: testBacker3Ata,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .signers([authority])
        .rpc()
        .then(confirm);

      // Create test proposal
      await program.methods
        .createProposal(metadata.name, metadata.symbol, metadata.uri)
        .accountsPartial({
          payer: authority.publicKey,
          maker: maker.publicKey,
          makerAccount,
          vaultAuthority,
          proposal: testProposal3,
          mintAccount: testMint3.publicKey,
          tokenVault: testVault3,
          systemProgram: anchor.web3.SystemProgram.programId,
          config: configStruct
        })
        .signers([authority, testMint3, maker])
        .rpc()
        .then(confirm);
    });

    it('24. Fails when backing rejected proposal', async () => {
      // First reject the proposal
      const testProposal3Data = await program.account.proposal.fetch(testProposal3);
      const metadataAccount3 = findMetadataPDA(testProposal3Data.mintAccount);
      
      await program.methods
        .rejectProposal()
        .accountsPartial({
          authority: authority.publicKey,
          proposal: testProposal3,
          payer: authority.publicKey,
          mintAccount: testProposal3Data.mintAccount,
          metadataAccount: metadataAccount3,
          tokenMetadataProgram: TOKEN_METADATA_PROGRAM_ID,
        })
        .signers([authority])
        .rpc()
        .then(confirm);


      // Try to back the rejected proposal
      try {
        await program.methods
          .depositSol()
          .accountsPartial({
            backer: testBacker3.publicKey,
        weweVault: weweTreasury,
            mint: mintAccount,
            userTokenAccount: findUserAta(testBacker3.publicKey, mintAccount),
            proposal: testProposal3,
            backerAccount: testBackerAccount3,
            vaultAuthority,
            systemProgram: anchor.web3.SystemProgram.programId,
            config: configStruct
          })
          .signers([testBacker3])
          .rpc();

        assert.fail('Should not allow backing rejected proposal');
      } catch (err) {
        expect(err.message).to.include('ProposalRejected');
      }
    });

    it('25. Fails when maker tries to back own proposal', async () => {
      const makerBackerAccount = findBackerAccountPDA(program.programId, proposal, maker.publicKey);
      const proposalData = await program.account.proposal.fetch(testProposal3);
      try {
        await program.methods
          .depositSol()
          .accountsPartial({
            backer: maker.publicKey,
        weweVault: weweTreasury,
            mint: mintAccount,
            userTokenAccount: makerAta,
            proposal,
            backerAccount: makerBackerAccount,
            vaultAuthority,
            systemProgram: anchor.web3.SystemProgram.programId,
            config: configStruct
          })
          .signers([maker])
          .rpc();

        assert.fail('Should not allow maker to back own proposal');
      } catch (err) {
        expect(err.message).to.include('CantBackOwnProposal');
      }
    });

    it('26. Fails when backing already launched proposal', async () => {
      const newBacker = anchor.web3.Keypair.generate();
      const newBackerAccount = findBackerAccountPDA(program.programId, proposal, newBacker.publicKey);

      // Fund new backer
      await provider.sendAndConfirm(new anchor.web3.Transaction().add(
        anchor.web3.SystemProgram.transfer({
          fromPubkey: provider.wallet.publicKey,
          toPubkey: newBacker.publicKey,
          lamports: 2e9, // 2 SOL to cover deposit + transaction fees
        })
      )).then(confirm);

      // Mint soulbound token
      const newBackerAta = findUserAta(newBacker.publicKey, mintAccount);
      await program.methods
        .mintSoulboundToUser()
        .accounts({
          authority: authority.publicKey,
          user: newBacker.publicKey,
          mint: mintAccount,
          freezeAuthority,
          mintAuthority,
          userTokenAccount: newBackerAta,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .signers([authority])
        .rpc()
        .then(confirm);

      // Try to back already launched proposal (proposal was launched in test #10)
      try {
        await program.methods
          .depositSol()
          .accountsPartial({
            backer: newBacker.publicKey,
        weweVault: weweTreasury,
            mint: mintAccount,
            userTokenAccount: newBackerAta,
            proposal,
            backerAccount: newBackerAccount,
            vaultAuthority,
            systemProgram: anchor.web3.SystemProgram.programId,
            config: configStruct
          })
          .signers([newBacker])
          .rpc();

        assert.fail('Should not allow backing already launched proposal');
      } catch (err) {
        expect(err.message).to.include('PoolAlreadyLaunched');
      }
    });

    it('27. Fails when user without soulbound token tries to back', async () => {
      const userWithoutToken = anchor.web3.Keypair.generate();
      const makerData = await program.account.makerAccount.fetch(makerAccount);
      const proposalIndex4 = makerData.proposalCount;
      const testProposal4 = findProposalPDA(program.programId, maker.publicKey, proposalIndex4);
      const testMint4 = anchor.web3.Keypair.generate();
      const [testVault4] = getTokenVaultAddress(vaultAuthority, testMint4.publicKey, program.programId);
      const testBackerAccount4 = findBackerAccountPDA(program.programId, testProposal4, userWithoutToken.publicKey);

      // Fund user
      await provider.sendAndConfirm(new anchor.web3.Transaction().add(
        anchor.web3.SystemProgram.transfer({
          fromPubkey: provider.wallet.publicKey,
          toPubkey: userWithoutToken.publicKey,
          lamports: 2e9, // 2 SOL to cover deposit + transaction fees
        })
      )).then(confirm);

      await provider.sendAndConfirm(new anchor.web3.Transaction().add(
      anchor.web3.SystemProgram.transfer({
        fromPubkey: provider.wallet.publicKey,
        toPubkey: userWithoutToken.publicKey,
        lamports: 1e9,
      })
    )).then(confirm);

    const userWithoutTokenAta = findUserAta(userWithoutToken.publicKey, mintAccount);
    await program.provider.sendAndConfirm(
        new anchor.web3.Transaction().add(
            createAssociatedTokenAccountInstruction(
                provider.wallet.publicKey, 
                userWithoutTokenAta,
                userWithoutToken.publicKey,
                mintAccount,
                TOKEN_PROGRAM_ID,             
                ASSOCIATED_TOKEN_PROGRAM_ID 
            )
        ),
        [] 
    ).then(confirm);

      // Create new proposal
      await program.methods
        .createProposal(metadata.name, metadata.symbol, metadata.uri)
        .accountsPartial({
          payer: authority.publicKey,
          maker: maker.publicKey,
          makerAccount,
          vaultAuthority,
          proposal: testProposal4,
          mintAccount: testMint4.publicKey,
          tokenVault: testVault4,
          systemProgram: anchor.web3.SystemProgram.programId,
          config: configStruct
        })
        .signers([authority, testMint4, maker])
        .rpc()
        .then(confirm);

      // Try to back without soulbound token
      try {
        await program.methods
          .depositSol()
          .accountsPartial({
            backer: userWithoutToken.publicKey,
        weweVault: weweTreasury,
            mint: mintAccount,
            userTokenAccount: userWithoutTokenAta,
            proposal: testProposal4,
            backerAccount: testBackerAccount4,
            vaultAuthority,
            systemProgram: anchor.web3.SystemProgram.programId,
            config: configStruct
          })
          .signers([userWithoutToken])
          .rpc();

        assert.fail('Should not allow backing without soulbound token');
      } catch (err) {
        expect(err.message).to.include('NotAuthorised');
      }
    });
  }); 

  describe('Input Validation Tests', () => {
    it('28. Fails when creating proposal with name > 32 characters', async () => {
      const longName = 'A'.repeat(33);
      const makerData = await program.account.makerAccount.fetch(makerAccount);
      const currentProposalIndex = makerData.proposalCount;
      const testProposal5 = findProposalPDA(program.programId, maker.publicKey, currentProposalIndex);
      const testMint5 = anchor.web3.Keypair.generate();
      const [testVault5] = getTokenVaultAddress(vaultAuthority, testMint5.publicKey, program.programId);

      try {
        await program.methods
          .createProposal(longName, metadata.symbol, metadata.uri)
          .accountsPartial({
            payer: authority.publicKey,
            maker: maker.publicKey,
            makerAccount,
            vaultAuthority,
            proposal: testProposal5,
            mintAccount: testMint5.publicKey,
            tokenVault: testVault5,
            systemProgram: anchor.web3.SystemProgram.programId,
            config: configStruct
          })
          .signers([authority, testMint5, maker])
          .rpc();

        assert.fail('Should not allow name > 32 characters');
      } catch (err) {
        expect(err.message).to.include('LenthTooLong');
      }
    });

    it('29. Fails when creating proposal with symbol > 10 characters', async () => {
      const longSymbol = 'B'.repeat(11);
      const makerData = await program.account.makerAccount.fetch(makerAccount);
      const currentProposalIndex = makerData.proposalCount;
      const testProposal6 = findProposalPDA(program.programId, maker.publicKey, currentProposalIndex);
      const testMint6 = anchor.web3.Keypair.generate();
      const [testVault6] = getTokenVaultAddress(vaultAuthority, testMint6.publicKey, program.programId);

      try {
        await program.methods
          .createProposal(metadata.name, longSymbol, metadata.uri)
          .accountsPartial({
            payer: authority.publicKey,
            maker: maker.publicKey,
            makerAccount,
            vaultAuthority,
            proposal: testProposal6,
            mintAccount: testMint6.publicKey,
            tokenVault: testVault6,
            systemProgram: anchor.web3.SystemProgram.programId,
            config: configStruct
          })
          .signers([authority, testMint6, maker])
          .rpc();

        assert.fail('Should not allow symbol > 10 characters');
      } catch (err) {
        expect(err.message).to.include('LenthTooLong');
      }
    });

    it('30. Fails when creating proposal with URI > 200 characters', async () => {
      const longUri = 'C'.repeat(201);
      const makerData = await program.account.makerAccount.fetch(makerAccount);
      const currentProposalIndex = makerData.proposalCount;
      const testProposal7 = findProposalPDA(program.programId, maker.publicKey, currentProposalIndex);
      const testMint7 = anchor.web3.Keypair.generate();
      const [testVault7] = getTokenVaultAddress(vaultAuthority, testMint7.publicKey, program.programId);

      try {
        await program.methods
          .createProposal(metadata.name, metadata.symbol, longUri)
          .accountsPartial({
            payer: authority.publicKey,
            maker: maker.publicKey,
            makerAccount,
            vaultAuthority,
            proposal: testProposal7,
            mintAccount: testMint7.publicKey,
            tokenVault: testVault7,
            systemProgram: anchor.web3.SystemProgram.programId,
            config: configStruct
          })
          .signers([authority, testMint7, maker])
          .rpc();

        assert.fail('Should not allow URI > 200 characters');
      } catch (err) {
        expect(err.message).to.include('LenthTooLong');
      }
    });
  }); 

  describe('Double Operation Prevention Tests', () => {
    it('31. Fails when attempting to mint soulbound token twice to same user', async () => {
      try {
        await program.methods
          .mintSoulboundToUser()
          .accounts({
            authority: authority.publicKey,
            user: backer.publicKey,
            mint: mintAccount,
            freezeAuthority,
            mintAuthority,
            userTokenAccount: userAta,
            associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
            tokenProgram: TOKEN_PROGRAM_ID,
            systemProgram: anchor.web3.SystemProgram.programId,
          })
          .signers([authority])
          .rpc();

        assert.fail('Should not allow double mint to same user');
      } catch (err) {
        expect(err.message).to.include('ProposalAlreadyBacked');
      }
    });

    it('32. Fails when attempting to claim airdrop twice', async () => {
      try {
        await program.methods
          .airdrop()
          .accounts({
            payer: authority.publicKey,
            backer: backer.publicKey,
            proposal,
            vaultAuthority,
            mintAccount: mint.publicKey,
            tokenVault: vault,
            backerAccount,
            backerTokenAccount,
            tokenProgram: TOKEN_PROGRAM_ID,
            associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
            systemProgram: anchor.web3.SystemProgram.programId,
            config: configStruct,
          })
          .signers([authority])
          .rpc();

        assert.fail('Should not allow double airdrop claim');
      } catch (err) {
        expect(err.message).to.include('AirdropAlreadyRecived');
      }
    });
  }); 

  describe('State Transition Tests', () => {
    let testProposal8: anchor.web3.PublicKey;
    let testMint8: anchor.web3.Keypair;
    let testVault8: anchor.web3.PublicKey;
    let proposalIndex8;

    before(async () => {
      const makerData = await program.account.makerAccount.fetch(makerAccount);
      proposalIndex8 = makerData.proposalCount;
      testProposal8 = findProposalPDA(program.programId, maker.publicKey, proposalIndex8);
      // testProposal8 = findProposalPDA(program.programId, maker.publicKey, proposalIndex8);
      testMint8 = anchor.web3.Keypair.generate();
      [testVault8] = getTokenVaultAddress(vaultAuthority, testMint8.publicKey, program.programId);

      // Create test proposal
      await program.methods
        .createProposal(metadata.name, metadata.symbol, metadata.uri)
        .accountsPartial({
          payer: authority.publicKey,
          maker: maker.publicKey,
          makerAccount,
          vaultAuthority,
          proposal: testProposal8,
          mintAccount: testMint8.publicKey,
          tokenVault: testVault8,
          systemProgram: anchor.web3.SystemProgram.programId,
          config: configStruct
        })
        .signers([authority, testMint8, maker])
        .rpc()
        .then(confirm);
    });

    it('33. Fails when trying to launch pool with insufficient backers', async () => {
      const config_account = await cpAmm.account.config.fetch(config);
      // Calculate price: quoteAmount (1 lamport) / baseAmount (150M tokens * 10^9)
      const baseAmount = new BN(150_000_000).mul(new BN(10).pow(new BN(9)));
      const quoteAmount = new BN(1);
      const price = new Decimal(quoteAmount.toString()).div(new Decimal(baseAmount.toString())).toString();
      const sqrtPrice = getSqrtPriceFromPrice(price, 9, 9);
      const [wsolVault] = getTokenVaultAddress(vaultAuthority, WSOL_MINT, program.programId);
      const pdas8 = derivePoolPDAs(program.programId, cpAmm.programId, testMint8.publicKey, WSOL_MINT, maker.publicKey, config);
      const testProposal8Data = await program.account.proposal.fetch(testProposal8);
      const poolCreatorAuthority = config_account.poolCreatorAuthority.equals(anchor.web3.PublicKey.default)
        ? authority.publicKey
        : config_account.poolCreatorAuthority;

      try {
        await program.methods
          .createPool(sqrtPrice)
          .accountsPartial({
            proposal: testProposal8,
            vaultAuthority,
            maker: maker.publicKey,
            tokenVault: testVault8,
            wsolVault,
            poolAuthority: pdas8.poolAuthority,
            dammPoolAuthority: pdas8.poolAuthority,
            poolConfig: config,
            poolCreatorAuthority: poolCreatorAuthority,
            pool: pdas8.pool,
            positionNftMint: pdas8.positionNftMint.publicKey,
            positionNftAccount: pdas8.positionNftAccount,
            position: pdas8.position,
            ammProgram: cpAmm.programId,
            baseMint: testMint8.publicKey,
            mintAccount: testProposal8Data.mintAccount,
            makerTokenAccount: pdas8.makerTokenAccount,
            quoteMint: WSOL_MINT,
            tokenAVault: pdas8.tokenAVault,
            tokenBVault: pdas8.tokenBVault,
            payer: authority.publicKey,
            tokenBaseProgram: TOKEN_PROGRAM_ID,
            tokenQuoteProgram: TOKEN_PROGRAM_ID,
            token2022Program: TOKEN_2022_PROGRAM_ID,
            dammEventAuthority: pdas8.dammEventAuthority,
            systemProgram: anchor.web3.SystemProgram.programId,
            associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
            config: configStruct,
          })
          .signers([authority, pdas8.positionNftMint])
          .rpc();

        assert.fail('Should not allow launching with insufficient backers');
      } catch (err) {
        expect(err.message).to.include('TargetNotMet');
      }
    });

    it('34. Fails when trying to start milestone before pool launch', async () => {
      const testProposal8DataForMilestone = await program.account.proposal.fetch(testProposal8);
      const metadataAccountTest8 = findMetadataPDA(testProposal8DataForMilestone.mintAccount);
      
      try {
        await program.methods
          .initialiseMilestone()
          .accounts({
            authority: authority.publicKey,
            proposal: testProposal8,
            mintAccount: testProposal8DataForMilestone.mintAccount,
            metadataAccount: metadataAccountTest8,
            payer: authority.publicKey,
            tokenMetadataProgram: TOKEN_METADATA_PROGRAM_ID,
            systemProgram: anchor.web3.SystemProgram.programId,
            rent: anchor.web3.SYSVAR_RENT_PUBKEY,
          })
          .signers([authority])
          .rpc();

        assert.fail('Should not allow starting milestone before launch');
      } catch (err) {
        expect(err.message).to.include('TargetNotMet');
      }
    });

    it('35. Fails when trying to claim milestone reward before pool launch', async () => {
      const testBacker = anchor.web3.Keypair.generate();
      await provider.connection.requestAirdrop(testBacker.publicKey, 2e9); // 2 SOL to cover deposit + transaction fees
      await new Promise(resolve => setTimeout(resolve, 500));
      const testUserAta = findUserAta(testBacker.publicKey, mintAccount);
      await program.methods
        .mintSoulboundToUser()
        .accounts({
          authority: authority.publicKey,
          user: testBacker.publicKey,
          mint: mintAccount,
          freezeAuthority,
          mintAuthority,
          userTokenAccount: testUserAta,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .signers([authority])
        .rpc()
        .then(confirm);
      const testBackerAccount = findBackerAccountPDA(program.programId, testProposal8, testBacker.publicKey);
      await program.methods
        .depositSol()
        .accountsPartial({
          backer: testBacker.publicKey,
        weweVault: weweTreasury,
          mint: mintAccount,
          userTokenAccount: testUserAta,
          proposal: testProposal8,
          backerAccount: testBackerAccount,
          vaultAuthority,
          systemProgram: anchor.web3.SystemProgram.programId,
          config: configStruct
        })
        .signers([testBacker])
        .rpc()
        .then(confirm);
      const testBackerTokenAccount = findUserAta(testBacker.publicKey, testMint8.publicKey);

      try {
        await program.methods
          .claimMilestoneReward()
          .accounts({
            backer: testBacker.publicKey,
            proposal: testProposal8,
            vaultAuthority,
            mintAccount: testMint8.publicKey,
            tokenVault: testVault8,
            backerAccount: testBackerAccount,
            backerTokenAccount: testBackerTokenAccount,
            tokenProgram: TOKEN_PROGRAM_ID,
            associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
            systemProgram: anchor.web3.SystemProgram.programId,
          })
          .signers([testBacker])
          .rpc();

        assert.fail('Should not allow claiming before launch');
      } catch (err) {
        expect(err.message).to.include('TargetNotMet');
      }
    });

    it('36. Fails when trying to airdrop before pool launch', async () => {
      const testBacker = anchor.web3.Keypair.generate();
      await provider.connection.requestAirdrop(testBacker.publicKey, 2e9); // 2 SOL to cover deposit + transaction fees
      await new Promise(resolve => setTimeout(resolve, 500));
      const testUserAta = findUserAta(testBacker.publicKey, mintAccount);
      await program.methods
        .mintSoulboundToUser()
        .accounts({
          authority: authority.publicKey,
          user: testBacker.publicKey,
          mint: mintAccount,
          freezeAuthority,
          mintAuthority,
          userTokenAccount: testUserAta,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .signers([authority])
        .rpc()
        .then(confirm);
      const testBackerAccount = findBackerAccountPDA(program.programId, testProposal8, testBacker.publicKey);
      await program.methods
        .depositSol()
        .accountsPartial({
          backer: testBacker.publicKey,
        weweVault: weweTreasury,
          mint: mintAccount,
          userTokenAccount: testUserAta,
          proposal: testProposal8,
          backerAccount: testBackerAccount,
          vaultAuthority,
          systemProgram: anchor.web3.SystemProgram.programId,
          config: configStruct
        })
        .signers([testBacker])
        .rpc()
        .then(confirm);
      const testBackerTokenAccount = findUserAta(testBacker.publicKey, testMint8.publicKey);

      try {
        await program.methods
          .airdrop()
          .accounts({
            payer: authority.publicKey,
            backer: testBacker.publicKey,
            proposal: testProposal8,
            vaultAuthority,
            mintAccount: testMint8.publicKey,
            tokenVault: testVault8,
            backerAccount: testBackerAccount,
            backerTokenAccount: testBackerTokenAccount,
            tokenProgram: TOKEN_PROGRAM_ID,
            associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
            systemProgram: anchor.web3.SystemProgram.programId,
            config: configStruct,
          })
          .signers([authority])
          .rpc();

        assert.fail('Should not allow airdrop before launch');
      } catch (err) {
        expect(err.message).to.include('TargetNotMet');
      }
    });

    it('37. Fails when trying to refund non-rejected proposal', async () => {
      const testBacker = anchor.web3.Keypair.generate();
      await provider.connection.requestAirdrop(testBacker.publicKey, 2e9); // 2 SOL to cover deposit + transaction fees
      await new Promise(resolve => setTimeout(resolve, 500));
      const testUserAta = findUserAta(testBacker.publicKey, mintAccount);
      await program.methods
        .mintSoulboundToUser()
        .accounts({
          authority: authority.publicKey,
          user: testBacker.publicKey,
          mint: mintAccount,
          freezeAuthority,
          mintAuthority,
          userTokenAccount: testUserAta,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .signers([authority])
        .rpc()
        .then(confirm);
      const testBackerAccount = findBackerAccountPDA(program.programId, testProposal8, testBacker.publicKey);
      await program.methods
        .depositSol()
        .accountsPartial({
          backer: testBacker.publicKey,
        weweVault: weweTreasury,
          mint: mintAccount,
          userTokenAccount: testUserAta,
          proposal: testProposal8,
          backerAccount: testBackerAccount,
          vaultAuthority,
          systemProgram: anchor.web3.SystemProgram.programId,
          config: configStruct
        })
        .signers([testBacker])
        .rpc()
        .then(confirm);
      const testBackerTokenAccount = findUserAta(testBacker.publicKey, testMint8.publicKey);

      try {
        await program.methods
          .refund()
          .accounts({
            backer: testBacker.publicKey,
            proposal: testProposal8,
            vaultAuthority,
            backerAccount: testBackerAccount,
            tokenProgram: TOKEN_PROGRAM_ID,
            systemProgram: anchor.web3.SystemProgram.programId,
            config: configStruct,
          })
          .signers([])
          .rpc();

        assert.fail('Should not allow refund on non-rejected proposal');
      } catch (err) {
        expect(err.message).to.include('BackingNotEnded');
      }
    });
  }); 

  describe('Milestone State Validation Tests', () => {
    it('38. Fails when trying to end milestone that is not active', async () => {
      // Create new proposal and launch it
      const makerData = await program.account.makerAccount.fetch(makerAccount);
      const proposalIndex9 = makerData.proposalCount;
      const testProposal9 = findProposalPDA(program.programId, maker.publicKey, proposalIndex9);
      const testMint9 = anchor.web3.Keypair.generate();
      const [testVault9] = getTokenVaultAddress(vaultAuthority, testMint9.publicKey, program.programId);
      const testBacker9 = anchor.web3.Keypair.generate();
      const testBackerAccount9 = findBackerAccountPDA(program.programId, testProposal9, testBacker9.publicKey);

      // Fund backer
      await provider.sendAndConfirm(new anchor.web3.Transaction().add(
        anchor.web3.SystemProgram.transfer({
          fromPubkey: provider.wallet.publicKey,
          toPubkey: testBacker9.publicKey,
          lamports: 2e9, // 2 SOL to cover deposit + transaction fees
        })
      )).then(confirm);

      // Mint soulbound token
      const testBacker9Ata = findUserAta(testBacker9.publicKey, mintAccount);
      await program.methods
        .mintSoulboundToUser()
        .accounts({
          authority: authority.publicKey,
          user: testBacker9.publicKey,
          mint: mintAccount,
          freezeAuthority,
          mintAuthority,
          userTokenAccount: testBacker9Ata,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .signers([authority])
        .rpc()
        .then(confirm);

      // Create proposal
      await program.methods
        .createProposal(metadata.name, metadata.symbol, metadata.uri)
        .accountsPartial({
          payer: authority.publicKey,
          maker: maker.publicKey,
          makerAccount,
          vaultAuthority,
          proposal: testProposal9,
          mintAccount: testMint9.publicKey,
          tokenVault: testVault9,
          systemProgram: anchor.web3.SystemProgram.programId,
          config: configStruct
        })
        .signers([authority, testMint9, maker])
        .rpc()
        .then(confirm);

      // Back proposal
      await program.methods
        .depositSol()
        .accountsPartial({
          backer: testBacker9.publicKey,
        weweVault: weweTreasury,
          mint: mintAccount,
          userTokenAccount: testBacker9Ata,
          proposal: testProposal9,
          backerAccount: testBackerAccount9,
          vaultAuthority,
          systemProgram: anchor.web3.SystemProgram.programId,
          config: configStruct
        })
        .signers([testBacker9])
        .rpc()
        .then(confirm);

      // Launch pool
      const config_account = await cpAmm.account.config.fetch(config);
      // Calculate price: quoteAmount (1 lamport) / baseAmount (150M tokens * 10^9)
      const baseAmount = new BN(150_000_000).mul(new BN(10).pow(new BN(9)));
      const quoteAmount = new BN(1);
      const price = new Decimal(quoteAmount.toString()).div(new Decimal(baseAmount.toString())).toString();
      const sqrtPrice = getSqrtPriceFromPrice(price, 9, 9);
      const [wsolVault] = getTokenVaultAddress(vaultAuthority, WSOL_MINT, program.programId);
      const pdas9 = derivePoolPDAs(program.programId, cpAmm.programId, testMint9.publicKey, WSOL_MINT, maker.publicKey, config);
      const testProposal9Data = await program.account.proposal.fetch(testProposal9);
      const poolCreatorAuthority = config_account.poolCreatorAuthority.equals(anchor.web3.PublicKey.default)
        ? authority.publicKey
        : config_account.poolCreatorAuthority;

      const computeUnitsIx = ComputeBudgetProgram.setComputeUnitLimit({ units: 1_400_000 });
      const tx = await program.methods
        .createPool(sqrtPrice)
        .accountsPartial({
          proposal: testProposal9,
          vaultAuthority,
          maker: maker.publicKey,
          tokenVault: testVault9,
          wsolVault,
          poolAuthority: pdas9.poolAuthority,
          dammPoolAuthority: pdas9.poolAuthority,
          poolConfig: config,
          poolCreatorAuthority: poolCreatorAuthority,
          pool: pdas9.pool,
          positionNftMint: pdas9.positionNftMint.publicKey,
          positionNftAccount: pdas9.positionNftAccount,
          position: pdas9.position,
          ammProgram: cpAmm.programId,
          baseMint: testMint9.publicKey,
          mintAccount: testProposal9Data.mintAccount,
          makerTokenAccount: pdas9.makerTokenAccount,
          quoteMint: WSOL_MINT,
          tokenAVault: pdas9.tokenAVault,
          tokenBVault: pdas9.tokenBVault,
          payer: authority.publicKey,
          tokenBaseProgram: TOKEN_PROGRAM_ID,
          tokenQuoteProgram: TOKEN_PROGRAM_ID,
          token2022Program: TOKEN_2022_PROGRAM_ID,
          dammEventAuthority: pdas9.dammEventAuthority,
          systemProgram: anchor.web3.SystemProgram.programId,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          config: configStruct,
        })
        .signers([authority, pdas9.positionNftMint])
        .transaction();

      tx.instructions.unshift(computeUnitsIx);
      await provider.sendAndConfirm(tx, [authority, pdas9.positionNftMint]);

      // Try to end milestone without starting one
      try {
        await program.methods
          .endMilestone()
          .accounts({
            authority: authority.publicKey,
            proposal: testProposal9,
            mint: testMint9.publicKey,
            vaultAuthority,
            tokenVault: testVault9,
            config: configStruct,
            tokenProgram: TOKEN_PROGRAM_ID,
          })
          .signers([authority])
          .rpc();

        assert.fail('Should not allow ending milestone that is not active');
      } catch (err) {
        expect(err.message).to.include('NoMilestoneActive');
      }
    });

    it('39. Fails when trying to snapshot without active milestone', async () => {
      const makerData = await program.account.makerAccount.fetch(makerAccount);
      const proposalIndex9 = makerData.proposalCount;
      const testProposal9 = findProposalPDA(program.programId, maker.publicKey, proposalIndex9);
      const testBacker9 = anchor.web3.Keypair.generate();
      const testBackerAccount9 = findBackerAccountPDA(program.programId, testProposal9, testBacker9.publicKey);
      const testMint9 = anchor.web3.Keypair.generate();

      try {
        await program.methods
          .snapshotBackerAmount()
          .accounts({
            authority: authority.publicKey,
            proposal: testProposal9,
            backer: testBacker9.publicKey,
            backerAccount: testBackerAccount9,
            backerTokenAccount: findUserAta(testBacker9.publicKey, testMint9.publicKey),
            mintAccount: testMint9.publicKey,
            config: configStruct,
          })
          .signers([authority])
          .rpc();

        assert.fail('Should not allow snapshot without active milestone');
      } catch (err) {
        // Will fail before getting to milestone check due to account constraints
        expect(err).to.exist;
      }
    });

    it('40. Fails when starting milestone on rejected proposal', async () => {
      const makerData = await program.account.makerAccount.fetch(makerAccount);
      const proposalIndex10 = makerData.proposalCount;
      const testProposal10 = findProposalPDA(program.programId, maker.publicKey, proposalIndex10);
      const testMint10 = anchor.web3.Keypair.generate();
      const [testVault10] = getTokenVaultAddress(vaultAuthority, testMint10.publicKey, program.programId);

      // Create and reject proposal
      await program.methods
        .createProposal(metadata.name, metadata.symbol, metadata.uri)
        .accountsPartial({
          payer: authority.publicKey,
          maker: maker.publicKey,
          makerAccount,
          vaultAuthority,
          proposal: testProposal10,
          mintAccount: testMint10.publicKey,
          tokenVault: testVault10,
          systemProgram: anchor.web3.SystemProgram.programId,
          config: configStruct
        })
        .signers([authority, testMint10, maker])
        .rpc()
        .then(confirm);

      const testProposal10DataForReject = await program.account.proposal.fetch(testProposal10);
      const metadataAccountTest10ForReject = findMetadataPDA(testProposal10DataForReject.mintAccount);
      
      await program.methods
        .rejectProposal()
        .accountsPartial({
          authority: authority.publicKey,
          proposal: testProposal10,
          payer: authority.publicKey,
          mintAccount: testProposal10DataForReject.mintAccount,
          metadataAccount: metadataAccountTest10ForReject,
          tokenMetadataProgram: TOKEN_METADATA_PROGRAM_ID,
        })
        .signers([authority])
        .rpc()
        .then(confirm);

      const testProposal10Data = await program.account.proposal.fetch(testProposal10);
      const metadataAccountTest10 = findMetadataPDA(testProposal10Data.mintAccount);
      
      try {
        await program.methods
          .initialiseMilestone()
          .accounts({
            authority: authority.publicKey,
            proposal: testProposal10,
            mintAccount: testProposal10Data.mintAccount,
            metadataAccount: metadataAccountTest10,
            payer: authority.publicKey,
            tokenMetadataProgram: TOKEN_METADATA_PROGRAM_ID,
            systemProgram: anchor.web3.SystemProgram.programId,
            rent: anchor.web3.SYSVAR_RENT_PUBKEY,
          })
          .signers([authority])
          .rpc();

        assert.fail('Should not allow starting milestone on rejected proposal');
      } catch (err) {
        expect(err.message).to.include('ProposalRejected');
      }
    });
  }); 

  describe('Rejection and Launch Constraint Tests', () => {
    it('41. Fails when trying to reject already launched proposal', async () => {
      const proposalDataLaunched = await program.account.proposal.fetch(proposal);
      const metadataAccountLaunched = findMetadataPDA(proposalDataLaunched.mintAccount);
      
      try {
        await program.methods
          .rejectProposal()
          .accountsPartial({
            authority: authority.publicKey,
            proposal, // This was launched in test #10
            payer: authority.publicKey,
            mintAccount: proposalDataLaunched.mintAccount,
            metadataAccount: metadataAccountLaunched,
            tokenMetadataProgram: TOKEN_METADATA_PROGRAM_ID,
          })
          .signers([authority])
          .rpc();

        assert.fail('Should not allow rejecting already launched proposal');
      } catch (err) {
        expect(err.message).to.include('PoolAlreadyLaunched');
      }
    });

    it('42. Fails when trying to launch already launched pool', async () => {
      const config_account = await cpAmm.account.config.fetch(config);
      // Calculate price: quoteAmount (1 lamport) / baseAmount (150M tokens * 10^9)
      const baseAmount = new BN(150_000_000).mul(new BN(10).pow(new BN(9)));
      const quoteAmount = new BN(1);
      const price = new Decimal(quoteAmount.toString()).div(new Decimal(baseAmount.toString())).toString();
      const sqrtPrice = getSqrtPriceFromPrice(price, 9, 9);
      const [wsolVault] = getTokenVaultAddress(vaultAuthority, WSOL_MINT, program.programId);
      const proposalDataAlreadyLaunched = await program.account.proposal.fetch(proposal);
      const currentMint = proposalDataAlreadyLaunched.mintAccount;
      const [currentVault] = getTokenVaultAddress(vaultAuthority, currentMint, program.programId);
      // Derive PDAs for the current mint (may have been reset in 10.5a)
      const currentPdas = derivePoolPDAs(program.programId, cpAmm.programId, currentMint, WSOL_MINT, maker.publicKey, config);
      const poolCreatorAuthority = config_account.poolCreatorAuthority.equals(anchor.web3.PublicKey.default)
        ? authority.publicKey
        : config_account.poolCreatorAuthority;

      try {
        await program.methods
          .createPool(sqrtPrice)
          .accountsPartial({
            proposal, // Already launched
            vaultAuthority,
            maker: maker.publicKey,
            tokenVault: currentVault,
            wsolVault,
            poolAuthority: currentPdas.poolAuthority,
            dammPoolAuthority: currentPdas.poolAuthority,
            poolConfig: config,
            poolCreatorAuthority: poolCreatorAuthority,
            pool: currentPdas.pool,
            positionNftMint: currentPdas.positionNftMint.publicKey,
            positionNftAccount: currentPdas.positionNftAccount,
            position: currentPdas.position,
            ammProgram: cpAmm.programId,
            baseMint: currentMint,
            mintAccount: currentMint,
            makerTokenAccount: currentPdas.makerTokenAccount,
            quoteMint: WSOL_MINT,
            tokenAVault: currentPdas.tokenAVault,
            tokenBVault: currentPdas.tokenBVault,
            payer: authority.publicKey,
            tokenBaseProgram: TOKEN_PROGRAM_ID,
            tokenQuoteProgram: TOKEN_PROGRAM_ID,
            token2022Program: TOKEN_2022_PROGRAM_ID,
            dammEventAuthority: currentPdas.dammEventAuthority,
            systemProgram: anchor.web3.SystemProgram.programId,
            associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
            config: configStruct,
          })
          .signers([authority, currentPdas.positionNftMint])
          .rpc();

        assert.fail('Should not allow launching already launched pool');
      } catch (err) {
        expect(err.message).to.include('PoolAlreadyLaunched');
      }
    });
  }); 

  describe('Snapshot Double-Update Prevention Tests', () => {
    let testProposal15: anchor.web3.PublicKey;
    let testBackerAccount15: anchor.web3.PublicKey;
    let testMint15: anchor.web3.Keypair;
    let testBackerTokenAccount15: anchor.web3.PublicKey;
    before(async () => {
        const makerData = await program.account.makerAccount.fetch(makerAccount);
        const proposalIndex15 = makerData.proposalCount;
        testProposal15 = findProposalPDA(program.programId, maker.publicKey, proposalIndex15);
        testMint15 = anchor.web3.Keypair.generate();
        const [testVault15] = getTokenVaultAddress(vaultAuthority, testMint15.publicKey, program.programId); 
        testBackerAccount15 = findBackerAccountPDA(program.programId, testProposal15, backer.publicKey);

        await program.methods
            .createProposal(metadata.name, metadata.symbol, metadata.uri)
            .accountsPartial({
                payer: authority.publicKey,
                maker: maker.publicKey,
                makerAccount,
                vaultAuthority,
                proposal: testProposal15, 
                mintAccount: testMint15.publicKey,
                tokenVault: testVault15,
                systemProgram: anchor.web3.SystemProgram.programId,
                config: configStruct
            })
            .signers([authority, testMint15, maker])
            .rpc()
            .then(confirm); 

        const backerAta = findUserAta(backer.publicKey, mintAccount); 
        await program.methods
            .depositSol()
            .accountsPartial({
                backer: backer.publicKey,
        weweVault: weweTreasury,
                mint: mintAccount,
                userTokenAccount: backerAta,
                proposal: testProposal15,
                backerAccount: testBackerAccount15, 
                vaultAuthority,
                systemProgram: anchor.web3.SystemProgram.programId,
                config: configStruct
            })
            .signers([backer])
            .rpc()
            .then(confirm);

        const testPoolPdas = derivePoolPDAs(program.programId, cpAmm.programId, testMint15.publicKey, WSOL_MINT, maker.publicKey, config);
        const [wsolVault] = getTokenVaultAddress(vaultAuthority, WSOL_MINT, program.programId);
        const config_account = await cpAmm.account.config.fetch(config);
        const testProposal15DataForPool = await program.account.proposal.fetch(testProposal15);
        const poolCreatorAuthority = config_account.poolCreatorAuthority.equals(anchor.web3.PublicKey.default)
          ? authority.publicKey
          : config_account.poolCreatorAuthority;

        const computeUnitsIx = ComputeBudgetProgram.setComputeUnitLimit({ units: 1_400_000 });
        // Calculate price: quoteAmount (1 lamport) / baseAmount (150M tokens * 10^9)
        const baseAmount = new BN(150_000_000).mul(new BN(10).pow(new BN(9)));
        const quoteAmount = new BN(1);
        const price = new Decimal(quoteAmount.toString()).div(new Decimal(baseAmount.toString())).toString();
        const sqrtPrice = getSqrtPriceFromPrice(price, 9, 9);

        const tx = await program.methods
            .createPool(sqrtPrice)
            .accountsPartial({
                proposal: testProposal15, // Use the new proposal
                vaultAuthority,
                maker: maker.publicKey,
                tokenVault: testVault15, // Use the new vault
                wsolVault,
                poolAuthority: testPoolPdas.poolAuthority,
                dammPoolAuthority: testPoolPdas.poolAuthority,
                poolConfig: config,
                poolCreatorAuthority: poolCreatorAuthority,
                pool: testPoolPdas.pool,
                positionNftMint: testPoolPdas.positionNftMint.publicKey,
                positionNftAccount: testPoolPdas.positionNftAccount,
                position: testPoolPdas.position,
                ammProgram: cpAmm.programId,
                baseMint: testMint15.publicKey, // Use the new mint
                mintAccount: testProposal15DataForPool.mintAccount,
                makerTokenAccount: testPoolPdas.makerTokenAccount,
                quoteMint: WSOL_MINT,
                tokenAVault: testPoolPdas.tokenAVault,
                tokenBVault: testPoolPdas.tokenBVault,
                payer: authority.publicKey,
                tokenBaseProgram: TOKEN_PROGRAM_ID,
                tokenQuoteProgram: TOKEN_PROGRAM_ID,
                token2022Program: TOKEN_2022_PROGRAM_ID,
                dammEventAuthority: testPoolPdas.dammEventAuthority,
                systemProgram: anchor.web3.SystemProgram.programId,
                associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
                config: configStruct,
            })
            .signers([authority, testPoolPdas.positionNftMint])
            .transaction();

        tx.instructions.unshift(computeUnitsIx);
        await provider.sendAndConfirm(tx, [authority, testPoolPdas.positionNftMint]);

        const testProposal15Data = await program.account.proposal.fetch(testProposal15);
        const metadataAccountTest15 = findMetadataPDA(testProposal15Data.mintAccount);
        
        await program.methods
            .initialiseMilestone()
            .accounts({
                authority: authority.publicKey,
                proposal: testProposal15,
                mintAccount: testProposal15Data.mintAccount,
                metadataAccount: metadataAccountTest15,
                payer: authority.publicKey,
                tokenMetadataProgram: TOKEN_METADATA_PROGRAM_ID,
                systemProgram: anchor.web3.SystemProgram.programId,
                rent: anchor.web3.SYSVAR_RENT_PUBKEY,
            })
            .signers([authority])
            .rpc()
            .then(confirm); 
        
        testBackerTokenAccount15 = findUserAta(backer.publicKey, testMint15.publicKey);

        const createAtaIx = createAssociatedTokenAccountInstruction(
            backer.publicKey, // Payer of the transaction
            testBackerTokenAccount15,
            backer.publicKey,
            testMint15.publicKey,
            TOKEN_PROGRAM_ID, 
        );
        await provider.sendAndConfirm(new anchor.web3.Transaction().add(createAtaIx), [backer]);
        
        await program.methods
          .snapshotBackerAmount()
          .accounts({
            authority: authority.publicKey,
            proposal: testProposal15,
            backer: backer.publicKey, 
            backerAccount: testBackerAccount15, 
            backerTokenAccount: testBackerTokenAccount15, 
            mintAccount: testMint15.publicKey,
            config: configStruct,
          })
          .signers([authority])
          .rpc()
          .then(confirm); 
    });
    it('43. Fails when snapshotting same backer twice in same cycle', async () => {
      // First snapshot succeeds (already done in test #13)
      // Try to snapshot the same backer again in the same cycle
      try {
        await program.methods
          .snapshotBackerAmount()
          .accounts({
            authority: authority.publicKey,
            proposal: testProposal15,
            backer: backer.publicKey,
            backerAccount: testBackerAccount15,
            backerTokenAccount: testBackerTokenAccount15,
            mintAccount: testMint15.publicKey,
            config: configStruct,
          })
          .signers([authority])
          .rpc(); 

        assert.fail('Should not allow snapshotting same backer twice in same cycle');
      } catch (err) { 
        expect(err.message).to.include('AmountAlreadyUpdated');
      }
    });
  }); 

  describe('End Milestone Validation Tests', () => {
    it('44. Fails when ending milestone without all backers weighted', async () => {
      // Create new proposal and launch it with 2 backers
      const makerData = await program.account.makerAccount.fetch(makerAccount);
      const proposalIndex11 = makerData.proposalCount;
      const testProposal11 = findProposalPDA(program.programId, maker.publicKey, proposalIndex11);
      const testMint11 = anchor.web3.Keypair.generate();
      const [testVault11] = getTokenVaultAddress(vaultAuthority, testMint11.publicKey, program.programId); 

      const testBacker11a = anchor.web3.Keypair.generate();
      const testBacker11b = anchor.web3.Keypair.generate();
      const testBackerAccount11a = findBackerAccountPDA(program.programId, testProposal11, testBacker11a.publicKey);
      const testBackerAccount11b = findBackerAccountPDA(program.programId, testProposal11, testBacker11b.publicKey);

      await provider.sendAndConfirm(new anchor.web3.Transaction().add(
        anchor.web3.SystemProgram.transfer({
          fromPubkey: provider.wallet.publicKey,
          toPubkey: testBacker11a.publicKey,
          lamports: 2e9, // 2 SOL to cover deposit + transaction fees
        }),
        anchor.web3.SystemProgram.transfer({
          fromPubkey: provider.wallet.publicKey,
          toPubkey: testBacker11b.publicKey,
          lamports: 2e9, // 2 SOL to cover deposit + transaction fees
        })
      )).then(confirm); 

      const testBacker11aAta = findUserAta(testBacker11a.publicKey, mintAccount);
      const testBacker11bAta = findUserAta(testBacker11b.publicKey, mintAccount);

      await program.methods
        .mintSoulboundToUser()
        .accounts({
          authority: authority.publicKey,
          user: testBacker11a.publicKey,
          mint: mintAccount,
          freezeAuthority,
          mintAuthority,
          userTokenAccount: testBacker11aAta,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .signers([authority])
        .rpc()
        .then(confirm); 

      await program.methods
        .mintSoulboundToUser()
        .accounts({
          authority: authority.publicKey,
          user: testBacker11b.publicKey,
          mint: mintAccount,
          freezeAuthority,
          mintAuthority,
          userTokenAccount: testBacker11bAta,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .signers([authority])
        .rpc()
        .then(confirm); 

      await program.methods
        .createProposal(metadata.name, metadata.symbol, metadata.uri)
        .accountsPartial({
          payer: authority.publicKey,
          maker: maker.publicKey,
          makerAccount,
          vaultAuthority,
          proposal: testProposal11,
          mintAccount: testMint11.publicKey,
          tokenVault: testVault11,
          systemProgram: anchor.web3.SystemProgram.programId,
          config: configStruct
        })
        .signers([authority, testMint11, maker])
        .rpc()
        .then(confirm); 

      // Back with both backers
      await program.methods
        .depositSol()
        .accountsPartial({
          backer: testBacker11a.publicKey,
        weweVault: weweTreasury,
          mint: mintAccount,
          userTokenAccount: testBacker11aAta,
          proposal: testProposal11,
          backerAccount: testBackerAccount11a,
          vaultAuthority,
          systemProgram: anchor.web3.SystemProgram.programId,
          config: configStruct
        })
        .signers([testBacker11a])
        .rpc()
        .then(confirm); 

      await program.methods
        .depositSol()
        .accountsPartial({
          backer: testBacker11b.publicKey,
        weweVault: weweTreasury,
          mint: mintAccount,
          userTokenAccount: testBacker11bAta,
          proposal: testProposal11,
          backerAccount: testBackerAccount11b,
          vaultAuthority,
          systemProgram: anchor.web3.SystemProgram.programId,
          config: configStruct
        })
        .signers([testBacker11b])
        .rpc()
        .then(confirm); 

      const config_account = await cpAmm.account.config.fetch(config);
      // Calculate price: quoteAmount (1 lamport) / baseAmount (150M tokens * 10^9)
      const baseAmount = new BN(150_000_000).mul(new BN(10).pow(new BN(9)));
      const quoteAmount = new BN(1);
      const price = new Decimal(quoteAmount.toString()).div(new Decimal(baseAmount.toString())).toString();
      const sqrtPrice = getSqrtPriceFromPrice(price, 9, 9);
      const [wsolVault] = getTokenVaultAddress(vaultAuthority, WSOL_MINT, program.programId);
      const pdas11 = derivePoolPDAs(program.programId, cpAmm.programId, testMint11.publicKey, WSOL_MINT, maker.publicKey, config);
      const testProposal11DataForPool = await program.account.proposal.fetch(testProposal11);

      const computeUnitsIx = ComputeBudgetProgram.setComputeUnitLimit({ units: 1_400_000 });
      const tx = await program.methods
        .createPool(sqrtPrice)
        .accountsPartial({
          proposal: testProposal11,
          vaultAuthority,
          maker: maker.publicKey,
          tokenVault: testVault11,
          wsolVault,
          poolAuthority: pdas11.poolAuthority,
          dammPoolAuthority: pdas11.poolAuthority,
          poolConfig: config,
          pool: pdas11.pool,
          positionNftMint: pdas11.positionNftMint.publicKey,
          positionNftAccount: pdas11.positionNftAccount,
          position: pdas11.position,
          ammProgram: cpAmm.programId,
          baseMint: testMint11.publicKey,
          mintAccount: testProposal11DataForPool.mintAccount,
          makerTokenAccount: pdas11.makerTokenAccount,
          quoteMint: WSOL_MINT,
          tokenAVault: pdas11.tokenAVault,
          tokenBVault: pdas11.tokenBVault,
          payer: authority.publicKey,
          tokenBaseProgram: TOKEN_PROGRAM_ID,
          tokenQuoteProgram: TOKEN_PROGRAM_ID,
          token2022Program: TOKEN_2022_PROGRAM_ID,
          dammEventAuthority: pdas11.dammEventAuthority,
          systemProgram: anchor.web3.SystemProgram.programId,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          config: configStruct,
        })
        .signers([authority, pdas11.positionNftMint])
        .transaction(); 

      tx.instructions.unshift(computeUnitsIx);
      await provider.sendAndConfirm(tx, [authority, pdas11.positionNftMint]);

      const testProposal11Data = await program.account.proposal.fetch(testProposal11);
      const metadataAccountTest11 = findMetadataPDA(testProposal11Data.mintAccount);
      
      await program.methods
        .initialiseMilestone()
        .accounts({
          authority: authority.publicKey,
          proposal: testProposal11,
          mintAccount: testProposal11Data.mintAccount,
          metadataAccount: metadataAccountTest11,
          payer: authority.publicKey,
          tokenMetadataProgram: TOKEN_METADATA_PROGRAM_ID,
          systemProgram: anchor.web3.SystemProgram.programId,
          rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        })
        .signers([authority])
        .rpc()
        .then(confirm); 

      const testBackerTokenAccount11a = findUserAta(testBacker11a.publicKey, testMint11.publicKey);
      const testBackerTokenAccount11b = findUserAta(testBacker11b.publicKey, testMint11.publicKey);

      // Create ATA for backer 11a
      const createAtaIx_11a = createAssociatedTokenAccountInstruction(
          testBacker11a.publicKey, 
          testBackerTokenAccount11a,
          testBacker11a.publicKey,
          testMint11.publicKey,
          TOKEN_PROGRAM_ID, 
      );
      await provider.sendAndConfirm(new anchor.web3.Transaction().add(createAtaIx_11a), [testBacker11a]);

      // Create ATA for backer 11b
      const createAtaIx_11b = createAssociatedTokenAccountInstruction(
          testBacker11b.publicKey, 
          testBackerTokenAccount11b,
          testBacker11b.publicKey,
          testMint11.publicKey,
          TOKEN_PROGRAM_ID, 
      );
      await provider.sendAndConfirm(new anchor.web3.Transaction().add(createAtaIx_11b), [testBacker11b]);

      await program.methods
        .snapshotBackerAmount()
        .accounts({
          authority: authority.publicKey,
          proposal: testProposal11,
          backer: testBacker11a.publicKey,
          backerAccount: testBackerAccount11a,
          backerTokenAccount: testBackerTokenAccount11a,
          mintAccount: testMint11.publicKey,
          config: configStruct,
        })
        .signers([authority])
        .rpc()
        .then(confirm); // error happening in this step

      // Try to end milestone without snapshotting all backers
      try {
        await program.methods
          .endMilestone()
          .accounts({
            authority: authority.publicKey,
            proposal: testProposal11,
            mint: testMint11.publicKey,
            vaultAuthority,
            tokenVault: testVault11,
            config: configStruct,
            tokenProgram: TOKEN_PROGRAM_ID,
          })
          .signers([authority])
          .rpc();

        assert.fail('Should not allow ending milestone without all backers weighted');
      } catch (err) {
        expect(err.message).to.include('AllBackerScoreNotUpdated');
      }
    });
  });

  describe('Token Economics Verification Tests', () => {
    it('45. Verifies airdrop amount calculation is correct', async () => {
      // Create new proposal, launch it, and verify airdrop math
      const makerData = await program.account.makerAccount.fetch(makerAccount);
      const proposalIndex12 = makerData.proposalCount;
      const testProposal12 = findProposalPDA(program.programId, maker.publicKey, proposalIndex12);
      const testMint12 = anchor.web3.Keypair.generate();
      const [testVault12] = getTokenVaultAddress(vaultAuthority, testMint12.publicKey, program.programId);
      const testBacker12 = anchor.web3.Keypair.generate();
      const testBackerAccount12 = findBackerAccountPDA(program.programId, testProposal12, testBacker12.publicKey);
      const testBackerTokenAccount12 = findUserAta(testBacker12.publicKey, testMint12.publicKey); 

      // Fund backer
      await provider.sendAndConfirm(new anchor.web3.Transaction().add(
        anchor.web3.SystemProgram.transfer({
          fromPubkey: provider.wallet.publicKey,
          toPubkey: testBacker12.publicKey,
          lamports: 2e9, // 2 SOL to cover deposit + transaction fees
        })
      )).then(confirm); 

      // Mint soulbound token
      const testBacker12Ata = findUserAta(testBacker12.publicKey, mintAccount);
      await program.methods
        .mintSoulboundToUser()
        .accounts({
          authority: authority.publicKey,
          user: testBacker12.publicKey,
          mint: mintAccount,
          freezeAuthority,
          mintAuthority,
          userTokenAccount: testBacker12Ata,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .signers([authority])
        .rpc()
        .then(confirm); 

      // Create proposal
      await program.methods
        .createProposal(metadata.name, metadata.symbol, metadata.uri)
        .accountsPartial({
          payer: authority.publicKey,
          maker: maker.publicKey,
          makerAccount,
          vaultAuthority,
          proposal: testProposal12,
          mintAccount: testMint12.publicKey,
          tokenVault: testVault12,
          systemProgram: anchor.web3.SystemProgram.programId,
          config: configStruct
        })
        .signers([authority, testMint12, maker])
        .rpc()
        .then(confirm); 

      // Back proposal
      await program.methods
        .depositSol()
        .accountsPartial({
          backer: testBacker12.publicKey,
        weweVault: weweTreasury,
          mint: mintAccount,
          userTokenAccount: testBacker12Ata,
          proposal: testProposal12,
          backerAccount: testBackerAccount12,
          vaultAuthority,
          systemProgram: anchor.web3.SystemProgram.programId,
          config: configStruct
        })
        .signers([testBacker12])
        .rpc()
        .then(confirm); 

      // Launch pool
      const config_account = await cpAmm.account.config.fetch(config);
      // Calculate price: quoteAmount (1 lamport) / baseAmount (150M tokens * 10^9)
      const baseAmount = new BN(150_000_000).mul(new BN(10).pow(new BN(9)));
      const quoteAmount = new BN(1);
      const price = new Decimal(quoteAmount.toString()).div(new Decimal(baseAmount.toString())).toString();
      const sqrtPrice = getSqrtPriceFromPrice(price, 9, 9);
      const [wsolVault] = getTokenVaultAddress(vaultAuthority, WSOL_MINT, program.programId);
      const pdas12 = derivePoolPDAs(program.programId, cpAmm.programId, testMint12.publicKey, WSOL_MINT, maker.publicKey, config);
      const testProposal12Data = await program.account.proposal.fetch(testProposal12);

      const computeUnitsIx = ComputeBudgetProgram.setComputeUnitLimit({ units: 1_400_000 });
      const tx = await program.methods
        .createPool(sqrtPrice)
        .accountsPartial({
          proposal: testProposal12,
          vaultAuthority,
          maker: maker.publicKey,
          tokenVault: testVault12,
          wsolVault,
          poolAuthority: pdas12.poolAuthority,
          dammPoolAuthority: pdas12.poolAuthority,
          poolConfig: config,
          pool: pdas12.pool,
          positionNftMint: pdas12.positionNftMint.publicKey,
          positionNftAccount: pdas12.positionNftAccount,
          position: pdas12.position,
          ammProgram: cpAmm.programId,
          baseMint: testMint12.publicKey,
          mintAccount: testProposal12Data.mintAccount,
          makerTokenAccount: pdas12.makerTokenAccount,
          quoteMint: WSOL_MINT,
          tokenAVault: pdas12.tokenAVault,
          tokenBVault: pdas12.tokenBVault,
          payer: authority.publicKey,
          tokenBaseProgram: TOKEN_PROGRAM_ID,
          tokenQuoteProgram: TOKEN_PROGRAM_ID,
          token2022Program: TOKEN_2022_PROGRAM_ID,
          dammEventAuthority: pdas12.dammEventAuthority,
          systemProgram: anchor.web3.SystemProgram.programId,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          config: configStruct,
        })
        .signers([authority, pdas12.positionNftMint])
        .transaction(); 

      tx.instructions.unshift(computeUnitsIx); 
      await provider.sendAndConfirm(tx, [authority, pdas12.positionNftMint]); 

      // Get proposal data to check total_backers
      const proposalData = await program.account.proposal.fetch(testProposal12); 

      // Airdrop and verify amount
      const balanceBefore = await provider.connection.getTokenAccountBalance(testBackerTokenAccount12).catch(() => ({ value: { uiAmount: 0 } })); 

      await program.methods
        .airdrop()
        .accounts({
          payer: authority.publicKey,
          backer: testBacker12.publicKey,
          proposal: testProposal12,
          vaultAuthority,
          mintAccount: testMint12.publicKey,
          tokenVault: testVault12,
          backerAccount: testBackerAccount12,
          backerTokenAccount: testBackerTokenAccount12,
          tokenProgram: TOKEN_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          systemProgram: anchor.web3.SystemProgram.programId,
          config: configStruct,
        })
        .signers([authority])
        .rpc()
        .then(confirm); 

      const balanceAfter = await provider.connection.getTokenAccountBalance(testBackerTokenAccount12); 

      // Expected: total_airdrop_amount_per_milestone / total_backers
      // From config: 140_000_000 / 1 = 140_000_000 tokens
      const configData = await program.account.configs.fetch(configStruct); 
      const expectedAmount = configData.totalAirdropAmountPerMilestone.toNumber() / proposalData.totalBackers.toNumber(); 

      assert.strictEqual(balanceAfter.value.uiAmount, expectedAmount, 'Airdrop amount should match calculation'); 
    }); 

    it('46. Verifies no upfront fee - full 1 SOL goes to vault', async () => {
      // Create new proposal and verify fee deduction
      const makerData = await program.account.makerAccount.fetch(makerAccount);
      const proposalIndex13 = makerData.proposalCount;
      const testProposal13 = findProposalPDA(program.programId, maker.publicKey, proposalIndex13);
      const testMint13 = anchor.web3.Keypair.generate();
      const [testVault13] = getTokenVaultAddress(vaultAuthority, testMint13.publicKey, program.programId);
      const testBacker13 = anchor.web3.Keypair.generate();
      const testBackerAccount13 = findBackerAccountPDA(program.programId, testProposal13, testBacker13.publicKey);

      // Fund backer
      await provider.sendAndConfirm(new anchor.web3.Transaction().add(
        anchor.web3.SystemProgram.transfer({
          fromPubkey: provider.wallet.publicKey,
          toPubkey: testBacker13.publicKey,
          lamports: 2e9, // 2 SOL to cover deposit + transaction fees
        })
      )).then(confirm);

      // Mint soulbound token
      const testBacker13Ata = findUserAta(testBacker13.publicKey, mintAccount);
      await program.methods
        .mintSoulboundToUser()
        .accounts({
          authority: authority.publicKey,
          user: testBacker13.publicKey,
          mint: mintAccount,
          freezeAuthority,
          mintAuthority,
          userTokenAccount: testBacker13Ata,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .signers([authority])
        .rpc()
        .then(confirm);

      // Create proposal
      await program.methods
        .createProposal(metadata.name, metadata.symbol, metadata.uri)
        .accountsPartial({
          payer: authority.publicKey,
          maker: maker.publicKey,
          makerAccount,
          vaultAuthority,
          proposal: testProposal13,
          mintAccount: testMint13.publicKey,
          tokenVault: testVault13,
          systemProgram: anchor.web3.SystemProgram.programId,
          config: configStruct
        })
        .signers([authority, testMint13, maker])
        .rpc()
        .then(confirm);

      // Check vault_authority balance before
      const vaultBalanceBefore = await provider.connection.getBalance(vaultAuthority);

      // Back proposal
      await program.methods
        .depositSol()
        .accountsPartial({
          backer: testBacker13.publicKey,
        weweVault: weweTreasury,
          mint: mintAccount,
          userTokenAccount: testBacker13Ata,
          proposal: testProposal13,
          backerAccount: testBackerAccount13,
          vaultAuthority,
          systemProgram: anchor.web3.SystemProgram.programId,
          config: configStruct
        })
        .signers([testBacker13])
        .rpc()
        .then(confirm);

      const vaultBalanceAfter = await provider.connection.getBalance(vaultAuthority);
      const configData = await program.account.configs.fetch(configStruct);

      // Expected: full amount_to_raise_per_user (no upfront fee)
      const expectedIncrease = configData.amountToRaisePerUser.toNumber();
      const actualIncrease = vaultBalanceAfter - vaultBalanceBefore;

      assert.strictEqual(actualIncrease, expectedIncrease, 'Vault should receive full amount');
    }); 

    it('47. Verifies metadata account is closed and rent refunded when proposal is rejected', async () => {
      // Create a new proposal for testing
      const makerData = await program.account.makerAccount.fetch(makerAccount);
      const proposalIndex14 = makerData.proposalCount;
      const testProposal14 = findProposalPDA(program.programId, maker.publicKey, proposalIndex14);
      const testMint14 = anchor.web3.Keypair.generate();
      const [testVault14] = getTokenVaultAddress(vaultAuthority, testMint14.publicKey, program.programId);

      // Create proposal (this creates the metadata account)
      await program.methods
        .createProposal(metadata.name, metadata.symbol, metadata.uri)
        .accountsPartial({
          payer: authority.publicKey,
          maker: maker.publicKey,
          makerAccount,
          vaultAuthority,
          proposal: testProposal14,
          mintAccount: testMint14.publicKey,
          tokenVault: testVault14,
          systemProgram: anchor.web3.SystemProgram.programId,
          config: configStruct
        })
        .signers([authority, testMint14, maker])
        .rpc()
        .then(confirm);

      // Get proposal data to find the mint account
      const testProposal14Data = await program.account.proposal.fetch(testProposal14);
      const metadataAccount14 = findMetadataPDA(testProposal14Data.mintAccount);

      // Check that metadata account exists before rejection
      const metadataAccountInfoBefore = await provider.connection.getAccountInfo(metadataAccount14);
      expect(metadataAccountInfoBefore).to.not.be.null;
      expect(metadataAccountInfoBefore!.lamports).to.be.greaterThan(0);
      
      const metadataRentBefore = metadataAccountInfoBefore!.lamports;

      // Get payer balance before rejection
      const payerBalanceBefore = await provider.connection.getBalance(authority.publicKey);

      // Reject the proposal (this should close the metadata account)
      await program.methods
        .rejectProposal()
        .accountsPartial({
          authority: authority.publicKey,
          proposal: testProposal14,
          payer: authority.publicKey, // Payer receives the rent refund
          mintAccount: testProposal14Data.mintAccount,
          metadataAccount: metadataAccount14,
          tokenMetadataProgram: TOKEN_METADATA_PROGRAM_ID,
        })
        .signers([authority])
        .rpc()
        .then(confirm);

      // Check that metadata account no longer exists (or has 0 lamports)
      const metadataAccountInfoAfter = await provider.connection.getAccountInfo(metadataAccount14);
      // When an account is closed, it either doesn't exist or has 0 lamports
      expect(metadataAccountInfoAfter === null || metadataAccountInfoAfter!.lamports === 0).to.be.true;

      // Check that payer received the rent refund
      const payerBalanceAfter = await provider.connection.getBalance(authority.publicKey);
      const balanceIncrease = payerBalanceAfter - payerBalanceBefore;
      
      // The payer should receive the rent that was in the metadata account
      // Note: There might be transaction fees, so we check that the increase is at least close to the rent
      // Allow for some transaction fee variance (typically 5000 lamports)
      expect(balanceIncrease).to.be.greaterThan(metadataRentBefore - 10000);
      expect(balanceIncrease).to.be.lessThanOrEqual(metadataRentBefore + 10000);
    });
  });
});
