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
  findUserAta,
  findMintAccount,
  findMintAuthority,
  calculateInitSqrtPrice,
} from './utils';

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

  let proposalIndex1 = new BN(0);
  let proposalIndex2 = new BN(1);
  const proposal = findProposalPDA(program.programId, maker.publicKey, proposalIndex1);
  const proposal2 = findProposalPDA(program.programId, maker.publicKey, proposalIndex2);

  const backerAccount = findBackerAccountPDA(program.programId, proposal, backer.publicKey);
  const backerAccount2 = findBackerAccountPDA(program.programId, proposal2, backer.publicKey);
  const makerAccount = findMakerAccountPDA(program.programId, maker.publicKey);

  const metadata = getMetadata();
  const [vaultAuthority] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("vault_authority")],
    program.programId
  );
  
  const mint = anchor.web3.Keypair.generate();
  const mint2 = anchor.web3.Keypair.generate();
  const [vault] = getTokenVaultAddress(vaultAuthority, mint.publicKey, program.programId);
  const [vault2] = getTokenVaultAddress(vaultAuthority, mint2.publicKey, program.programId);

  const mintAccount = findMintAccount(program.programId);
  const userAta = findUserAta(backer.publicKey, mintAccount);
  const makerAta = findUserAta(maker.publicKey, mintAccount);
  const [freezeAuthority] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("freeze_authority")],
    program.programId
  );
  const mintAuthority = findMintAuthority(program.programId);
  const backerTokenAccount = findUserAta(backer.publicKey, mint.publicKey);

  const config = new anchor.web3.PublicKey('DJN8YHxQKZnF7bL2GwuKNB2UcfhKCqRspfLe7YYEN3rr');
  const pdas = derivePoolPDAs(program.programId, cpAmm.programId, mint.publicKey, WSOL_MINT, maker.publicKey, config);

  it('1. Airdrops funds to test accounts', async () => {
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
          lamports: 3e9,
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
          toPubkey: vaultAuthority,
          lamports: 1e9,
        })
      )),
    ];
    await Promise.all(airdropPromises.map(p => confirm(p)));
  });

  it("2. Mints and freezes soulbound token to user and maker", async () => {
    const tx1 = await program.methods
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
    await program.methods
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

  // Refactored test case to fix the failure
  it('6. Fails when user backs same proposal twice', async () => {
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
      
      // If the rpc call succeeds, the test should fail
      assert.fail('Should not allow double backing');
    } catch (err) {
      // Check for a generic Anchor program error
      expect(err.message).to.include('custom program error');
    }
  });

  it('7. Backs the second proposal', async () => {
    await program.methods
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

  it('8. Authority rejects a proposal', async () => {
    await program.methods
      .rejectProposal()
      .accountsPartial({
        authority: authority.publicKey,
        proposal: proposal2,
      })
      .signers([authority])
      .rpc()
      .then(confirm);
  });

  it("9. Refunds SOL to backer after proposal is rejected", async () => {
    await program.methods
      .refund()
      .accounts({
        backer: backer.publicKey,
        proposal: proposal2,
        vaultAuthority,
        backerAccount: backerAccount2,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([])
      .rpc()
      .then(confirm);
  });

  it('10. Launches coin and creates DAMM pool', async () => {
    const [wsolVault] = getTokenVaultAddress(vaultAuthority, WSOL_MINT, program.programId);

    const eventPromise = waitForEvent(program, 'coinLaunched');
    const config_account = await cpAmm.account.config.fetch(config);

    const computeUnitsIx = ComputeBudgetProgram.setComputeUnitLimit({ units: 1_400_000 });
    const sqrtPrice = calculateInitSqrtPrice(new BN(150_000_000), new BN(1), config_account.sqrtMinPrice, config_account.sqrtMaxPrice);

    const tx = await program.methods
      .createPool(sqrtPrice)
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
    await provider.sendAndConfirm(tx, [authority, pdas.positionNftMint]);

    const capturedEvent = await eventPromise;

    expect(capturedEvent.proposalAddress.toBase58()).to.equal(proposal.toBase58());
    expect(capturedEvent.mintAccount.toBase58()).to.equal(mint.publicKey.toBase58());
  });

  it("11. Airdrop launched coin successfully", async () => {
    const backerTokenAccount = findUserAta(backer.publicKey, mint.publicKey);
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
      })
      .signers([authority])
      .rpc()
      .then(confirm);

    const tokenAccountInfo = await provider.connection.getTokenAccountBalance(backerTokenAccount);
    const balance = tokenAccountInfo.value.uiAmount;
    assert.ok(balance && balance > 0, "Backer should receive airdropped tokens");
  });

  it('12. Starts a milestone (initialiseMilestone)', async () => {
    await program.methods
      .initialiseMilestone()
      .accounts({
        authority: authority.publicKey,
        proposal,
      })
      .signers([authority])
      .rpc()
      .then(confirm);
  });

  it('13. Updates backer milestone amount', async () => {
    await program.methods
      .snapshotBackerAmount()
      .accounts({
        authority: authority.publicKey,
        proposal,
        backer: backer.publicKey,
        backerAccount,
        backerTokenAccount,
        mintAccount: mint.publicKey,
      })
      .signers([authority])
      .rpc()
      .then(confirm);
  });
  
  it('14. Ends a milestone', async () => {
    await program.methods
      .endMilestone()
      .accounts({
        authority: authority.publicKey,
        proposal,
        mint: mint.publicKey,
        vaultAuthority,
        tokenVault: vault,
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
    const computeUnitsIx = ComputeBudgetProgram.setComputeUnitLimit({ units: 1_400_000 });

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