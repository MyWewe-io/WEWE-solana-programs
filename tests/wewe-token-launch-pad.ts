import * as anchor from '@coral-xyz/anchor';
import { Program } from '@coral-xyz/anchor';
import BN from 'bn.js';

import {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  TOKEN_PROGRAM_ID,
  getAssociatedTokenAddressSync,
} from '@solana/spl-token';
import type { WeweTokenLaunchPad } from '../target/types/wewe_token_launch_pad.ts';
import { expect } from 'chai';

describe('wewe_token_launch_pad', () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.WeweTokenLaunchPad as Program<WeweTokenLaunchPad>;

  const maker = anchor.web3.Keypair.generate();

  const backer = anchor.web3.Keypair.generate();

  const proposalCount = new BN(0);
  const proposal = anchor.web3.PublicKey.findProgramAddressSync([Buffer.from('proposer'), maker.publicKey.toBuffer(), proposalCount.toArrayLike(Buffer, "le", 8)], program.programId)[0];

  const backer_account = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from('backer'), proposal.toBuffer(), backer.publicKey.toBuffer()],
    program.programId,
  )[0];

  const maker_account = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from('maker'), maker.publicKey.toBuffer()],
    program.programId,
  )[0];

  const [mint] = anchor.web3.PublicKey.findProgramAddressSync([Buffer.from('mint'), maker.publicKey.toBuffer(), proposalCount.toArrayLike(Buffer, "le", 8),], program.programId);

  const metadata = {
    name: 'Solana Gold',
    symbol: 'GOLDSOL',
    uri: 'https://raw.githubusercontent.com/solana-developers/program-examples/new-examples/tokens/tokens/.assets/spl-token.json',
  };

  const vault = getAssociatedTokenAddressSync(mint, proposal, true);

  const authority = anchor.web3.Keypair.fromSecretKey(Uint8Array.from([42,132,54,48,86,137,10,155,254,103,140,97,104,8,197,48,55,71,171,157,247,159,233,130,100,213,107,236,96,40,175,164,179,49,15,185,22,130,249,11,142,174,6,253,52,133,167,81,80,179,15,199,164,252,14,233,42,74,178,20,71,62,139,21])
  );

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

    const airdrop_backer = await provider.connection.requestAirdrop(backer.publicKey, 1 * anchor.web3.LAMPORTS_PER_SOL).then(confirm);
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
      .createProposal(9, amount_to_raise, metadata.name, metadata.symbol, metadata.uri, 100)
      .accountsPartial({
        maker: maker.publicKey,
        makerAccount: maker_account,
        proposal,
        mintAccount: mint,
        tokenVault: vault,
        systemProgram: anchor.web3.SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
      })
      .signers([maker])
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
    console.log('\nInitialized proposal Account');
    console.log('Your transaction signature', tx);
  });


  it('back a proposal', async () => {
    const tx = await program.methods
      .depositSol(new BN(0), new BN(5))
      .accountsPartial({
        backer: backer.publicKey,
        proposal,
        backerAccount: backer_account,
      })
      .signers([backer])
      .rpc()
      .then(confirm);

    console.log('\nContributed to proposal', tx);
    console.log('Your transaction signature', tx);

    const contributorAccount = await program.account.backers.fetch(backer_account);
    console.log('Contributor balance', contributorAccount.amount.toString());
  });

  it('back to proposal again with same account', async () => {
    const tx = await program.methods
      .depositSol(new BN(0), new BN(5))
      .accountsPartial({
        backer: backer.publicKey,
        proposal,
        backerAccount: backer_account,
      })
      .signers([backer])
      .rpc()
      .then(confirm);

    console.log('\nContributed to proposal', tx);
    console.log('Your transaction signature', tx);

    const contributorAccount = await program.account.backers.fetch(backer_account);
    console.log('Contributor balance', contributorAccount.amount.toString());
  });

  it('reject a proposal from authority', async () => {
    let proposal_index = new BN(0);
    const tx = await program.methods
      .rejectProposal(proposal_index)
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

  it('Refund Backing', async () => {
    let proposal_index = new BN(0);
    const tx = await program.methods
      .refund(proposal_index)
      .accountsPartial({
        recepient: backer.publicKey,
        maker: maker.publicKey,
        proposal,
        backerAccount: backer_account,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc()
      .then(confirm);
    console.log('\nRefunded contributions', tx);
    console.log('Your transaction signature', tx);
    console.log('proposal balance', (await provider.connection.getBalance(proposal)));
  });
});