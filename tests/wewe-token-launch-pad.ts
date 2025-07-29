// tests/wewe_token_launch_pad.spec.ts
import * as anchor from '@coral-xyz/anchor';
import { Program } from '@coral-xyz/anchor';
import BN from 'bn.js';
import { assert, expect } from 'chai';
import {
  TOKEN_PROGRAM_ID,
  TOKEN_2022_PROGRAM_ID,
  ASSOCIATED_TOKEN_PROGRAM_ID,
  getAssociatedTokenAddressSync,
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
  findFreezeAuthority,
  findUserAta,
  findMintAccount,
  findMintAuthority,
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
  const backerAccount2 = findBackerAccountPDA(program.programId, proposal2, backer.publicKey);
  const makerAccount = findMakerAccountPDA(program.programId, maker.publicKey);

  const metadata = getMetadata();
  const [vaultAuthority] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("vault_authority")],
    program.programId
  );
  const [vault] = getTokenVaultAddress(vaultAuthority, mint.publicKey, program.programId);
  const [vault2] = getTokenVaultAddress(vaultAuthority, mint2.publicKey, program.programId);

  const mintAccount = findMintAccount(program.programId);
  const userAta = findUserAta(backer.publicKey, mintAccount);
  const makerAta = findUserAta(maker.publicKey, mintAccount);
  const freezeAuthority = findFreezeAuthority(program.programId);
  const mintAuthority = findMintAuthority(program.programId);

  const config = new anchor.web3.PublicKey('8CNy9goNQNLM4wtgRw528tUQGMKD3vSuFRZY2gLGLLvF');
  
    const pdas = derivePoolPDAs(program.programId, cpAmm.programId, mint.publicKey, WSOL_MINT, maker.publicKey, config);

  it('Airdrops funds to test accounts', async () => {
    await confirm(provider.connection.requestAirdrop(provider.wallet.publicKey, 5e9)); // 5 SOL total

    // Transfer 1 SOL to maker
    await provider.sendAndConfirm(new anchor.web3.Transaction().add(
      anchor.web3.SystemProgram.transfer({
        fromPubkey: provider.wallet.publicKey,
        toPubkey: maker.publicKey,
        lamports: 1e9,
      })
    ));

    // Transfer 3 SOL to backer
    await provider.sendAndConfirm(new anchor.web3.Transaction().add(
      anchor.web3.SystemProgram.transfer({
        fromPubkey: provider.wallet.publicKey,
        toPubkey: backer.publicKey,
        lamports: 3e9,
      })
    ));

    // Transfer 1 SOL to authority
    await provider.sendAndConfirm(new anchor.web3.Transaction().add(
      anchor.web3.SystemProgram.transfer({
        fromPubkey: provider.wallet.publicKey,
        toPubkey: authority.publicKey,
        lamports: 1e9,
      })
    ));

    await provider.sendAndConfirm(new anchor.web3.Transaction().add(
      anchor.web3.SystemProgram.transfer({
        fromPubkey: provider.wallet.publicKey,
        toPubkey: vaultAuthority,
        lamports: 1e9,
      })
    ));
  });

  it("mints and freezes nft to user", async () => {
    const tx = await program.methods
      .mintSoulboundToUser()
      .accounts({
        payer: authority.publicKey,
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
        payer: authority.publicKey,
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
  });

  it('Creates first proposal', async () => {
    let capturedEvent: any = null;
    const listener = await program.addEventListener('proposalCreated', (event) => capturedEvent = event);

    const tx = await program.methods
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
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([authority, mint, maker])
      .rpc()
      .then(confirm);

    // Wait for the event to be captured
    await new Promise((resolve) => setTimeout(resolve, 5000));

    // Remove the listener
    await program.removeEventListener(listener);
    const expectedEvent = {
      maker: maker.publicKey.toBase58(),
      proposalAddress: proposal.toBase58(),
    };

    // Assert event fields
    expect(capturedEvent.maker.toBase58()).to.equal(expectedEvent.maker);
    expect(capturedEvent.proposalAddress.toBase58()).to.equal(expectedEvent.proposalAddress);
  });

  it('Creates second proposal with same maker', async () => {
    let capturedEvent: any = null;
    const listener = await program.addEventListener('proposalCreated', (event) => capturedEvent = event);

    const tx = await program.methods
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
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([authority, mint2, maker])
      .rpc()
      .then(confirm);

    await program.removeEventListener(listener);
    const expectedEvent = {
      maker: maker.publicKey.toBase58(),
      proposalAddress: proposal2.toBase58(),
    };

    // Assert event fields
    expect(capturedEvent.maker.toBase58()).to.equal(expectedEvent.maker);
    expect(capturedEvent.proposalAddress.toBase58()).to.equal(expectedEvent.proposalAddress);
  });

  it('Backs the first proposal with SOL', async () => {
    const tx = await program.methods
      .depositSol()
      .accountsPartial({
        backer: backer.publicKey,
        mint: mintAccount,
        userTokenAccount: userAta,
        proposal,
        backerAccount,
        vaultAuthority,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([backer])
      .rpc()
      .then(confirm);
  });

  it('Fails when user backs same proposal twice', async () => {
    try {
      await program.methods
        .depositSol()
        .accountsPartial({
          backer: backer.publicKey,
          proposal,
          vaultAuthority,
          backerAccount,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .signers([backer])
        .rpc();

      assert.fail('Should not allow double backing');
    } catch (err) {
      expect(err.message).to.match(/custom program error/);
    }
  });

  it('Backs the second proposal', async () => {
    const tx = await program.methods
      .depositSol()
      .accountsPartial({
        backer: backer.publicKey,
        proposal: proposal2,
        backerAccount: backerAccount2,
        vaultAuthority,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([backer])
      .rpc()
      .then(confirm);
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
  });

  it("Refunds SOL to backer after proposal ends or is rejected", async () => {
    const tx = await program.methods
      .refund()
      .accounts({
        backer: backer.publicKey,
        proposal: proposal2,
        vaultAuthority,
        backerAccount: backerAccount2,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([backer])
      .rpc()
      .then(confirm);
  });

  it('Launches coin and creates DAMM pool', async () => {
    const [wsolVault] = getTokenVaultAddress(vaultAuthority, WSOL_MINT, program.programId);

    let capturedEvent: any = null;
    const listener = await program.addEventListener('coinLaunched', (event) => capturedEvent = event);

    const computeUnitsIx = ComputeBudgetProgram.setComputeUnitLimit({ units: 1_400_000 });

    const tx = await program.methods
      .createPool()
      .accountsPartial({
        proposal,
        vaultAuthority,
        maker: maker.publicKey,
        tokenVault: vault,
        wsolVault,
        poolAuthority: pdas.poolAuthority,
        dammPoolAuthority: pdas.poolAuthority,
        poolConfig: config,
        pool: pdas.pool,
        positionNftMint: pdas.positionNftMint.publicKey,
        positionNftAccount: pdas.positionNftAccount,
        position: pdas.position,
        ammProgram: cpAmm.programId,
        baseMint: mint.publicKey,
        makerTokenAccount: pdas.makerTokenAccount,
        quoteMint: WSOL_MINT,
        tokenAVault: pdas.tokenAVault,
        tokenBVault: pdas.tokenBVault,
        payer: authority.publicKey,
        tokenBaseProgram: TOKEN_PROGRAM_ID,
        tokenQuoteProgram: TOKEN_PROGRAM_ID,
        token2022Program: TOKEN_2022_PROGRAM_ID,
        dammEventAuthority: pdas.dammEventAuthority,
        systemProgram: anchor.web3.SystemProgram.programId,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
      })
      .signers([authority, pdas.positionNftMint])
      .transaction();
    
    tx.instructions.unshift(computeUnitsIx); 
    const signature = await provider.sendAndConfirm(tx, [authority, pdas.positionNftMint]);

    await program.removeEventListener(listener);

    expect(capturedEvent.proposalAddress.toBase58()).to.equal(proposal.toBase58());
    expect(capturedEvent.mintAccount.toBase58()).to.equal(mint.publicKey.toBase58());
  });

  it('Burns tokens from vault', async () => {
    const burnAmount = 2;
  
    // Fetch current proposal state
    const proposalAccountBefore = await program.account.proposal.fetch(proposal);
    const currentCycleBefore = proposalAccountBefore.currentAirdropCycle;
  
    const [vaultAuthority] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from('vault_authority')],
      program.programId
    );
  
    const [tokenVault] = getTokenVaultAddress(vaultAuthority, mint.publicKey, program.programId);
  
    // Burn the tokens
    const tx = await program.methods
      .burn(new BN(burnAmount))
      .accounts({
        authority: authority.publicKey,
        proposal,
        vaultAuthority,
        tokenVault,
        mint: mint.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([authority])
      .rpc()
      .then(confirm);
  
    // Fetch updated proposal account
    const proposalAccountAfter = await program.account.proposal.fetch(proposal);
    const currentCycleAfter = proposalAccountAfter.currentAirdropCycle;
  
    // Assert airdrop cycle increment
    expect(currentCycleAfter).to.equal(currentCycleBefore + 1);
  });

  it('Updates backer airdrop claim amount', async () => {
    const claimAmount = new BN(200); // 200 tokens 
  
    // Fetch proposal and backer account before update
    const proposalAccount = await program.account.proposal.fetch(proposal);
    const backerAccountBefore = await program.account.backers.fetch(backerAccount);
  
    const currentCycle = proposalAccount.currentAirdropCycle;
    const alreadyUpdated = backerAccountBefore.amountUpdatedUptoCycle;
  
    expect(currentCycle).to.be.greaterThan(alreadyUpdated);

    // Update the backer's airdrop claim
    const tx = await program.methods
      .updateAirdropAmount(claimAmount)
      .accounts({
        authority: authority.publicKey,
        proposal,
        backerAccount,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([authority])
      .rpc()
      .then(confirm);
  
    // Fetch updated backer account
    const backerAccountAfter = await program.account.backers.fetch(backerAccount);
  
    // Assert claim amount increased by 200 tokens
    const claimed = new BN(backerAccountAfter.claimAmount.toString());
    expect(200).to.equal(claimed.toNumber());
  
    // Ensure the cycle was updated
    expect(backerAccountAfter.amountUpdatedUptoCycle).to.equal(currentCycle);
  });  

  it("Backer claims airdrop successfully", async () => {
    // Assumes proposal, mint, vault, and backer account are already setup
    const backerTokenAccount = findUserAta(backer.publicKey, mint.publicKey);
  
    const tx = await program.methods
      .claim()
      .accounts({
        backer: backer.publicKey,
        maker: maker.publicKey,
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
  
    // Validate the token balance has increased
    const tokenAccountInfo = await provider.connection.getTokenAccountBalance(backerTokenAccount);
    const balance = tokenAccountInfo.value.uiAmount;
  
    assert.ok(balance && balance > 0, "Backer should receive airdropped tokens");
  
    // Verify claim_amount reset
    const backerData = await program.account.backers.fetch(backerAccount);
    assert.strictEqual(backerData.claimAmount.toNumber(), 0, "Claim amount should be reset to zero");
  });
  
  it('Claims position fee and distributes tokens', async () => {
    const userTokenAmount = new BN(10e9); // 10 tokens
    const userWsolAmount = new BN(1e9); // 1 WSOL (in lamports)
    const weweTreasury = new anchor.web3.PublicKey("76U9hvHNUNn7YV5FekSzDHzqnHETsUpDKq4cMj2dMxNi");
  
    const [vaultAuthority] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from('vault_authority')],
      program.programId
    );
  
    const weweWsolAccount = getAssociatedTokenAddressSync(WSOL_MINT, weweTreasury, true, undefined, undefined);
    const weweTokenAccount = getAssociatedTokenAddressSync(mint.publicKey, weweTreasury, true, undefined, undefined);
    const makerWsolAccount = getAssociatedTokenAddressSync(WSOL_MINT, maker.publicKey, true, undefined, undefined);
    const [wsolVault] = getTokenVaultAddress(vaultAuthority, WSOL_MINT, program.programId);

    const computeUnitsIx = ComputeBudgetProgram.setComputeUnitLimit({ units: 1_400_000 });

    const tx = await program.methods
      .claimPoolFee(userWsolAmount, userTokenAmount)
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
        tokenAProgram: TOKEN_PROGRAM_ID,
        tokenBProgram: TOKEN_PROGRAM_ID,
        ammProgram: cpAmm.programId,
        eventAuthority: pdas.dammEventAuthority,
        systemProgram: anchor.web3.SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
      })
      .signers([authority])
      .preInstructions([computeUnitsIx])
      .rpc()
      .then(confirm);
  
  });  
});
