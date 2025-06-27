// tests/wewe_token_launch_pad.spec.ts
import * as anchor from '@coral-xyz/anchor';
import { Program } from '@coral-xyz/anchor';
import BN from 'bn.js';
import { assert, expect } from 'chai';
import {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  TOKEN_PROGRAM_ID,
  TOKEN_2022_PROGRAM_ID,
} from '@solana/spl-token';

import type { WeweTokenLaunchPad } from '../target/types/wewe_token_launch_pad';
import type { CpAmm } from '../target/types/cp_amm';
import CpAmmIDL from '../idls/damm_v2.json';

import {
  confirm,
  findProposalPDA,
  findBackerAccountPDA,
  findMakerAccountPDA,
  WSOL_MINT,
  getMetadata,
  generateKeypairs,
  getTokenVaultAddress,
  createDammConfig,
  derivePoolPDAs,
} from './utils';


describe('Wewe Token Launch Pad - Integration Tests', () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.WeweTokenLaunchPad as Program<WeweTokenLaunchPad>;
  const cpAmm = new Program<CpAmm>(CpAmmIDL as CpAmm, provider);

  const { maker, backer, authority, mint, mint2 } = generateKeypairs();

  let proposalIndex = new BN(0);
  const proposal = findProposalPDA(program.programId, maker.publicKey, proposalIndex);

  proposalIndex = new BN(1);
  const proposal2 = findProposalPDA(program.programId, maker.publicKey, proposalIndex);

  const backerAccount = findBackerAccountPDA(program.programId, proposal, backer.publicKey);
  const makerAccount = findMakerAccountPDA(program.programId, maker.publicKey);

  const metadata = getMetadata();
  const vault = getTokenVaultAddress(mint.publicKey, proposal);
  const vault2 = getTokenVaultAddress(mint2.publicKey, proposal2);
  const wsolVault = getTokenVaultAddress(WSOL_MINT, proposal);
  const wsolVault2 = getTokenVaultAddress(WSOL_MINT, proposal2);

  it('Airdrops funds to test accounts', async () => {
    await confirm(provider.connection.requestAirdrop(maker.publicKey, 1e9));
    await confirm(provider.connection.requestAirdrop(backer.publicKey, 2e9));
    await confirm(provider.connection.requestAirdrop(authority.publicKey, 1e9));
  });

  it('Creates first proposal', async () => {
    let capturedEvent: any = null;
    const listener = await program.addEventListener('proposalCreated', (event) => capturedEvent = event);

    const tx = await program.methods
      .createProposal(9, new BN(50_000_000), metadata.name, metadata.symbol, metadata.uri, 0)
      .accountsPartial({
        maker: maker.publicKey,
        makerAccount,
        proposal,
        mintAccount: mint.publicKey,
        tokenVault: vault,
        wsolVault,
        wsolMint: WSOL_MINT,
        systemProgram: anchor.web3.SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
      })
      .signers([maker, mint])
      .rpc()
      .then(confirm);

    // Wait for the event to be captured
    await new Promise((resolve) => setTimeout(resolve, 5000));

    // Remove the listener
    await program.removeEventListener(listener);
    const expectedEvent = {
      maker: maker.publicKey.toBase58(),
      proposalAddress: proposal.toBase58(),
      duration: 9,
    };

    // Assert event fields
    expect(capturedEvent.maker.toBase58()).to.equal(expectedEvent.maker);
    expect(capturedEvent.proposalAddress.toBase58()).to.equal(expectedEvent.proposalAddress);
    expect(capturedEvent.duration).to.equal(expectedEvent.duration);
  });

  it('Creates second proposal with same maker', async () => {
    let capturedEvent: any = null;
    const listener = await program.addEventListener('proposalCreated', (event) => capturedEvent = event);

    const tx = await program.methods
      .createProposal(9, new BN(50_000_000), metadata.name, metadata.symbol, metadata.uri, 100)
      .accountsPartial({
        maker: maker.publicKey,
        makerAccount,
        proposal: proposal2,
        mintAccount: mint2.publicKey,
        tokenVault: vault2,
        wsolVault: wsolVault2,
        wsolMint: WSOL_MINT,
        systemProgram: anchor.web3.SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
      })
      .signers([maker, mint2])
      .rpc()
      .then(confirm);

    await program.removeEventListener(listener);
    const expectedEvent = {
      maker: maker.publicKey.toBase58(),
      proposalAddress: proposal2.toBase58(),
      duration: 9,
    };

    // Assert event fields
    expect(capturedEvent.maker.toBase58()).to.equal(expectedEvent.maker);
    expect(capturedEvent.proposalAddress.toBase58()).to.equal(expectedEvent.proposalAddress);
    expect(capturedEvent.duration).to.equal(expectedEvent.duration);
  });

  it('Backs the first proposal with SOL', async () => {
    const tx = await program.methods
      .depositSol()
      .accountsPartial({
        backer: backer.publicKey,
        proposal,
        backerAccount,
        wsolVault,
        wsolMint: WSOL_MINT,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([backer])
      .rpc()
      .then(confirm);

    console.log('Backed proposal with SOL. Tx:', tx);
  });

  it('Fails when user backs same proposal twice', async () => {
    try {
      await program.methods
        .depositSol()
        .accountsPartial({
          backer: backer.publicKey,
          proposal,
          backerAccount,
          wsolVault,
          wsolMint: WSOL_MINT,
          tokenProgram: TOKEN_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .signers([backer])
        .rpc();

      assert.fail('Should not allow double backing');
    } catch (err) {
      expect(err.message).to.match(/custom program error/);
    }
  });

  it('Authority rejects a proposal', async () => {
    const tx = await program.methods
      .rejectProposal()
      .accountsPartial({
        authority: authority.publicKey,
        proposal: proposal2,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([authority])
      .rpc()
      .then(confirm);

    console.log('Proposal rejected. Tx:', tx);
  });

  it('Launches coin and creates DAMM pool', async () => {
    const liquidity = new BN(100_000_000_000);
    const sqrtPrice = new BN(1000);
    const CONFIG_INDEX = new BN(10);
    const admin = anchor.web3.Keypair.generate();
    await confirm(provider.connection.requestAirdrop(admin.publicKey, 1e9));

    const config = new anchor.web3.PublicKey('8CNy9goNQNLM4wtgRw528tUQGMKD3vSuFRZY2gLGLLvF');
    // const config = createDammConfig(cpAmm.programId, CONFIG_INDEX);
    // await cpAmm.methods
    //   .createConfig(CONFIG_INDEX, {
    //     poolFees: {
    //       baseFee: { cliffFeeNumerator: new BN(2_500_000), numberOfPeriod: 0, reductionFactor: new BN(0), periodFrequency: new BN(0), feeSchedulerMode: 0 },
    //       protocolFeePercent: 10,
    //       partnerFeePercent: 0,
    //       referralFeePercent: 0,
    //       dynamicFee: null,
    //     },
    //     sqrtMinPrice: new BN("4295048016"),
    //     sqrtMaxPrice: new BN("79226673521066979257578248091"),
    //     vaultConfigKey: anchor.web3.PublicKey.default,
    //     poolCreatorAuthority: anchor.web3.PublicKey.default,
    //     activationType: 0,
    //     collectFeeMode: 0,
    //   })
    //   .accountsPartial({ config, admin: admin.publicKey, systemProgram: anchor.web3.SystemProgram.programId })
    //   .signers([admin])
    //   .rpc()
    //   .then(confirm);

    const pdas = derivePoolPDAs(program.programId, mint.publicKey, WSOL_MINT, maker.publicKey, config);

    let capturedEvent: any = null;
    const listener = await program.addEventListener('coinLaunched', (event) => capturedEvent = event);

    const tx = await program.methods
      .createPool(liquidity, sqrtPrice, pdas.poolAuthorityBump)
      .accountsPartial({
        proposal,
        maker: maker.publicKey,
        tokenVault: vault,
        wsolVault,
        makerTokenAccount: pdas.baseVault,
        poolAuthority: pdas.poolAuthority,
        poolConfig: config,
        pool: pdas.pool,
        firstPositionNftMint: pdas.positionNftMint.publicKey,
        firstPositionNftAccount: pdas.positionNftAccount,
        firstPosition: pdas.position,
        ammProgram: cpAmm.programId,
        baseMint: mint.publicKey,
        quoteMint: WSOL_MINT,
        tokenAVault: pdas.tokenAVault,
        tokenBVault: pdas.tokenBVault,
        baseVault: pdas.baseVault,
        quoteVault: pdas.quoteVault,
        payer: maker.publicKey,
        tokenBaseProgram: TOKEN_PROGRAM_ID,
        tokenQuoteProgram: TOKEN_PROGRAM_ID,
        token2022Program: TOKEN_2022_PROGRAM_ID,
        dammEventAuthority: pdas.dammEventAuthority.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([maker, pdas.positionNftMint])
      .rpc()

    await program.removeEventListener(listener);

    expect(capturedEvent.proposalAddress.toBase58()).to.equal(proposal.toBase58());
    expect(capturedEvent.mintAccount.toBase58()).to.equal(mint.publicKey.toBase58());
    console.log('Pool created. Tx:', tx);
  });
});
