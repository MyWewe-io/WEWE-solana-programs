import * as anchor from '@coral-xyz/anchor';
import { Program } from '@coral-xyz/anchor';
import BN from 'bn.js';

import {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  TOKEN_PROGRAM_ID,
  TOKEN_2022_PROGRAM_ID,
  getAssociatedTokenAddressSync,
} from '@solana/spl-token';
import type { WeweTokenLaunchPad } from '../target/types/wewe_token_launch_pad.ts';
import { assert, expect } from 'chai';

describe('wewe_token_launch_pad', () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.WeweTokenLaunchPad as Program<WeweTokenLaunchPad>;

  const maker = anchor.web3.Keypair.generate();

  const backer = anchor.web3.Keypair.generate();

  let proposalCount = new BN(0);
  const proposal = anchor.web3.PublicKey.findProgramAddressSync([Buffer.from('proposal'), maker.publicKey.toBuffer(), proposalCount.toArrayLike(Buffer, "le", 8)], program.programId)[0];

  proposalCount = new BN(1);
  const proposal2 = anchor.web3.PublicKey.findProgramAddressSync([Buffer.from('proposal'), maker.publicKey.toBuffer(), proposalCount.toArrayLike(Buffer, "le", 8)], program.programId)[0];

  const backer_account = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from('backer'), proposal.toBuffer(), backer.publicKey.toBuffer()],
    program.programId,
  )[0];

  const maker_account = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from('maker'), maker.publicKey.toBuffer()],
    program.programId,
  )[0];

  // mint PublicKey prefixed with wewe: weweca2xonYzkEE1HGv8yqfBwwTzyHGyXx4B6sKaGmx
  const mint = anchor.web3.Keypair.fromSecretKey(Uint8Array.from([120, 242, 147, 132, 61, 86, 151, 209, 148, 93, 109, 242, 239, 178, 189, 134, 74, 139, 56, 122, 168, 40, 166, 101, 75, 200, 91, 214, 253, 44, 238, 50, 14, 0, 67, 220, 240, 46, 161, 156, 27, 124, 159, 138, 189, 13, 94, 61, 144, 123, 166, 53, 197, 39, 157, 168, 196, 163, 91, 187, 245, 3, 156, 19]));

  // mint PublicKey prefixed with wewe: wewesHFmfmCQtD7kGgq1vATe2sE2ehDGQJsQ9hRiHQ4
  const mint2 = anchor.web3.Keypair.fromSecretKey(Uint8Array.from([225, 221, 220, 42, 149, 90, 160, 131, 244, 182, 127, 56, 41, 199, 93, 110, 174, 0, 14, 141, 87, 60, 191, 10, 74, 94, 100, 50, 75, 15, 173, 37, 14, 0, 67, 226, 134, 115, 217, 145, 253, 4, 100, 186, 147, 181, 23, 98, 31, 67, 199, 140, 101, 137, 76, 233, 93, 210, 70, 44, 195, 91, 37, 225])
  );

  const metadata = {
    name: 'Solana Gold',
    symbol: 'GOLDSOL',
    uri: 'https://raw.githubusercontent.com/solana-developers/program-examples/new-examples/tokens/tokens/.assets/spl-token.json',
  };

  const vault = getAssociatedTokenAddressSync(mint.publicKey, proposal, true);

  const vault2 = getAssociatedTokenAddressSync(mint2.publicKey, proposal2, true);

  const authority = anchor.web3.Keypair.fromSecretKey(Uint8Array.from([42, 132, 54, 48, 86, 137, 10, 155, 254, 103, 140, 97, 104, 8, 197, 48, 55, 71, 171, 157, 247, 159, 233, 130, 100, 213, 107, 236, 96, 40, 175, 164, 179, 49, 15, 185, 22, 130, 249, 11, 142, 174, 6, 253, 52, 133, 167, 81, 80, 179, 15, 199, 164, 252, 14, 233, 42, 74, 178, 20, 71, 62, 139, 21])
  );

  const WSOL_MINT = new anchor.web3.PublicKey("So11111111111111111111111111111111111111112");

  const wsol_vault = getAssociatedTokenAddressSync(WSOL_MINT, proposal, true);
  const wsol_vault2 = getAssociatedTokenAddressSync(WSOL_MINT, proposal2, true);


  const confirm = async (signature: string): Promise<string> => {
    const block = await provider.connection.getLatestBlockhash();
    await provider.connection.confirmTransaction({
      signature,
      ...block,
    });
    return signature;
  };

  it('Test Preparation', async () => {
    const airdrop_maker = await provider.connection.requestAirdrop(maker.publicKey, 1 * anchor.web3.LAMPORTS_PER_SOL).then(confirm);
    console.log('\nAirdropped 1 SOL to maker', airdrop_maker);

    const airdrop_backer = await provider.connection.requestAirdrop(backer.publicKey, 2 * anchor.web3.LAMPORTS_PER_SOL).then(confirm);
    console.log('\nAirdropped 1 SOL to backer', airdrop_backer);

    const airdrop_authority = await provider.connection.requestAirdrop(authority.publicKey, 1 * anchor.web3.LAMPORTS_PER_SOL).then(confirm);
    console.log('\nAirdropped 1 SOL to backer', airdrop_authority);
  });

  it('Create proposal', async () => {
    const amount_to_raise = new BN(50000000);
    let capturedEvent: any = null;

    // Set up the event listener
    const listener = await program.addEventListener('proposalCreated', (event, slot) => {
      capturedEvent = event;
    });

    // Run the transaction
    const tx = await program.methods
      .createProposal(9, amount_to_raise, metadata.name, metadata.symbol, metadata.uri, 0)
      .accountsPartial({
        maker: maker.publicKey,
        makerAccount: maker_account,
        proposal,
        mintAccount: mint.publicKey,
        tokenVault: vault,
        wsolVault: wsol_vault,
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

    // Assert the event was received
    expect(capturedEvent).to.not.be.null;

    // Expected values
    const expectedEvent = {
      maker: maker.publicKey.toBase58(),
      proposalAddress: proposal.toBase58(),
      duration: 9,
    };

    // Assert event fields
    expect(capturedEvent.maker.toBase58()).to.equal(expectedEvent.maker);
    expect(capturedEvent.proposalAddress.toBase58()).to.equal(expectedEvent.proposalAddress);
    expect(capturedEvent.duration).to.equal(expectedEvent.duration);
    console.log(capturedEvent.mintAccount);
    console.log('\nInitialized proposal Account');
    console.log('Your transaction signature', tx);
  });


  it('Create proposal with same account', async () => {
    const amount_to_raise = new BN(50000000);
    let capturedEvent: any = null;

    // Set up the event listener
    const listener = await program.addEventListener('proposalCreated', (event, slot) => {
      capturedEvent = event;
    });

    // Run the transaction
    const tx = await program.methods
      .createProposal(9, amount_to_raise, metadata.name, metadata.symbol, metadata.uri, 100)
      .accountsPartial({
        maker: maker.publicKey,
        makerAccount: maker_account,
        proposal: proposal2,
        mintAccount: mint2.publicKey,
        tokenVault: vault2,
        wsolVault: wsol_vault2,
        wsolMint: WSOL_MINT,
        systemProgram: anchor.web3.SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
      })
      .signers([maker, mint2])
      .rpc()
      .then(confirm);

    // Wait for the event to be captured
    await new Promise((resolve) => setTimeout(resolve, 5000));

    // Remove the listener
    await program.removeEventListener(listener);

    // Assert the event was received
    expect(capturedEvent).to.not.be.null;

    // Expected values
    const expectedEvent = {
      maker: maker.publicKey.toBase58(),
      proposalAddress: proposal2.toBase58(),
      duration: 9,
    };

    // Assert event fields
    expect(capturedEvent.maker.toBase58()).to.equal(expectedEvent.maker);
    expect(capturedEvent.proposalAddress.toBase58()).to.equal(expectedEvent.proposalAddress);
    expect(capturedEvent.duration).to.equal(expectedEvent.duration);
    console.log(capturedEvent.mintAccount);
    console.log('\nInitialized proposal Account');
    console.log('Your transaction signature', tx);
  });

  it('back a proposal', async () => {
    const tx = await program.methods
      .depositSol()
      .accountsPartial({
        backer: backer.publicKey,
        proposal,
        backerAccount: backer_account,
        wsolVault: wsol_vault,
        wsolMint: WSOL_MINT,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([backer])
      .rpc()
      .then(confirm);

    console.log('\nContributed to proposal', tx);
    console.log('Your transaction signature', tx);
  });

  it('fails when user backs the same proposal twice', async () => {
    try {
      const tx = await program.methods
        .depositSol()
        .accountsPartial({
          backer: backer.publicKey,
          proposal,
          backerAccount: backer_account,
          wsolVault: wsol_vault,
          wsolMint: WSOL_MINT,
          tokenProgram: TOKEN_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .signers([backer])
        .rpc();

      assert.fail('Second contribution should have failed but succeeded:', tx);
    } catch (err) {
      expect(err.message).match(/custom program error/);
    }
  });

  it('reject a proposal from authority', async () => {
    const tx = await program.methods
      .rejectProposal()
      .accountsPartial({
        authority: authority.publicKey,
        proposal,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([authority])
      .rpc()
      .then(confirm);

    console.log('\nContributed to proposal', tx);
    console.log('Your transaction signature', tx);
  });

  // it('it refunds backing by unwrapping WSOL', async () => {
  //   const tx = await program.methods
  //     .refund()
  //     .accountsPartial({
  //       backer: backer.publicKey,
  //       proposal,
  //       backerAccount: backer_account,
  //       wsolVault: wsol_vault,
  //       wsolMint: WSOL_MINT,
  //       tokenProgram: TOKEN_PROGRAM_ID,
  //       systemProgram: anchor.web3.SystemProgram.programId,
  //     })
  //     .signers([backer])
  //     .rpc()
  //     .then(confirm);

  //   console.log('\nRefunded contributions (WSOL unwrap)', tx);

  //   const balance = await provider.connection.getBalance(backer.publicKey);
  //   console.log(`Backer new balance: ${balance / anchor.web3.LAMPORTS_PER_SOL} SOL`);
  // });

  it('create pool', async () => {
    const liquidity = new BN(100_000_000_000); // example liquidity
    const sqrtPrice = new BN(1000);

    let capturedEvent: any = null;

    const listener = await program.addEventListener('coinLaunched', (event, slot) => {
      capturedEvent = event;
    });

    const DAMM_V2_PROGRAM_ID = new anchor.web3.PublicKey('cpamdpZCGKUy5JxQXB4dcpGPiikHawvSWAd6mEn1sGG');
    function minKey(key1, key2) {
      return key1.toBuffer().compare(key2.toBuffer()) < 0 ? key1 : key2;
    }
    
    function maxKey(key1, key2) {
      return key1.toBuffer().compare(key2.toBuffer()) > 0 ? key1 : key2;
    }

    const CONFIG = new anchor.web3.PublicKey('8CNy9goNQNLM4wtgRw528tUQGMKD3vSuFRZY2gLGLLvF');
    const WSOL_MINT = new anchor.web3.PublicKey('So11111111111111111111111111111111111111112');


    const [poolAuthority, poolAuthorityBump] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("pool_authority")],
      program.programId
    );
    
    // Mints
    const tokenBMint = WSOL_MINT;
    
    // Derive pool PDA
    const [pool, poolBump] = anchor.web3.PublicKey.findProgramAddressSync(
      [
        Buffer.from("pool"),
        CONFIG.toBuffer(),
        maxKey(mint.publicKey, tokenBMint).toBuffer(),
        minKey(mint.publicKey, tokenBMint).toBuffer(),
      ],
      program.programId
    );
    
    // Position NFT Mint (must be a real Keypair, not PDA)
    const positionNftMint = anchor.web3.Keypair.generate();
    
    // Derive position account PDA
    const [position, positionBump] = anchor.web3.PublicKey.findProgramAddressSync(
      [
        Buffer.from("position"),
        positionNftMint.publicKey.toBuffer(),
      ],
      program.programId
    );
    
    // Derive position NFT token account (PDA, seeded)
    const [positionNftAccount, positionNftAccountBump] = anchor.web3.PublicKey.findProgramAddressSync(
      [
        Buffer.from("position_nft_account"), // replace with your `POSITION_NFT_ACCOUNT_PREFIX`
        positionNftMint.publicKey.toBuffer(),
      ],
      program.programId
    );
    
    const dammEventAuthority = anchor.web3.Keypair.generate();

    // Derive token vaults
    const [tokenAVault, tokenAVaultBump] = anchor.web3.PublicKey.findProgramAddressSync(
      [
        Buffer.from("token_vault"),
        mint.publicKey.toBuffer(),
        pool.toBuffer(),
      ],
      program.programId
    );
    
    const [tokenBVault, tokenBVaultBump] = anchor.web3.PublicKey.findProgramAddressSync(
      [
        Buffer.from("token_vault"),
        tokenBMint.toBuffer(),
        pool.toBuffer(),
      ],
      program.programId
    );

    const [quoteVault] = await anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("quote_vault"), maker.publicKey.toBuffer(), WSOL_MINT.toBuffer()],
      program.programId
    );

    const [baseVault] = await anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("base_vault"), maker.publicKey.toBuffer(), mint.publicKey.toBuffer()],
      program.programId
    );

    const tx = await program.methods
      .createPool(liquidity, sqrtPrice, poolAuthorityBump)
      .accountsPartial({
        proposal,
        maker: maker.publicKey,
        tokenVault: vault,
        wsolVault: wsol_vault,
        makerTokenAccount: baseVault,
        poolAuthority,
        poolConfig: CONFIG,
        pool,
        firstPositionNftMint: positionNftMint.publicKey,
        firstPositionNftAccount: positionNftAccount,
        firstPosition: position,
        ammProgram: DAMM_V2_PROGRAM_ID,
        baseMint: mint.publicKey,
        quoteMint: WSOL_MINT,
        tokenAVault,
        tokenBVault,
        baseVault: baseVault,
        quoteVault: quoteVault,
        payer: maker.publicKey,
        tokenBaseProgram: TOKEN_PROGRAM_ID,
        tokenQuoteProgram: TOKEN_PROGRAM_ID,
        token2022Program: TOKEN_2022_PROGRAM_ID,
        dammEventAuthority: dammEventAuthority.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })      
      .signers([maker])
      .rpc()
      .then(confirm);

    await new Promise((resolve) => setTimeout(resolve, 5000));
    await program.removeEventListener(listener);

    expect(capturedEvent).to.not.be.null;
    expect(capturedEvent.proposalAddress.toBase58()).to.equal(proposal.toBase58());
    expect(capturedEvent.mintAccount.toBase58()).to.equal(mint.publicKey.toBase58());

    console.log('\nPool created successfully');
    console.log('Transaction:', tx);
  });

});