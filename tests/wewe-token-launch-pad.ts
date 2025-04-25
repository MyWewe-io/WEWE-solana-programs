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

  const proposal = anchor.web3.PublicKey.findProgramAddressSync([Buffer.from('proposer'), maker.publicKey.toBuffer()], program.programId)[0];

  const backer_account = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from('backer'), proposal.toBuffer(), backer.publicKey.toBuffer()],
    program.programId,
  )[0];

  const [mint] = anchor.web3.PublicKey.findProgramAddressSync([Buffer.from('mint'), maker.publicKey.toBuffer()], program.programId);

  const metadata = {
    name: 'Solana Gold',
    symbol: 'GOLDSOL',
    uri: 'https://raw.githubusercontent.com/solana-developers/program-examples/new-examples/tokens/tokens/.assets/spl-token.json',
  };

  const vault = getAssociatedTokenAddressSync(mint, proposal, true);

  const confirm = async (signature: string): Promise<string> => {
    const block = await provider.connection.getLatestBlockhash();
    await provider.connection.confirmTransaction({
      signature,
      ...block,
    });
    return signature;
  };

  it('Test Preparation', async () => {
    const airdrop = await provider.connection.requestAirdrop(maker.publicKey, 1 * anchor.web3.LAMPORTS_PER_SOL).then(confirm);
    console.log('\nAirdropped 1 SOL to maker', airdrop);

    const airdrop_backer = await provider.connection.requestAirdrop(backer.publicKey, 1 * anchor.web3.LAMPORTS_PER_SOL).then(confirm);
    console.log('\nAirdropped 1 SOL to backer', airdrop_backer);

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
      .depositSol(new BN(5))
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
      .depositSol(new BN(5))
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

  // it('reject a proposal from authority', async () => {
  //   let authority = anchor.web3.Keypair.fromSecretKey(secretKey);
  //   const tx = await program.methods
  //     .rejectProposal()
  //     .accountsPartial({
  //       authority.publicKey,
  //       proposal,
  //       systemProgram: anchor.web3.SystemProgram.programId,
  //     })
  //     .signers([authority])
  //     .rpc()
  //     .then(confirm);
  //
  //   console.log('\nContributed to proposal', tx);
  //   console.log('Your transaction signature', tx);
  // });

  // it('Refund Contributions', async () => {
  //   const vault = getAssociatedTokenAddressSync(mint, proposal, true);

  //   const contributorAccount = await program.account.contributor.fetch(contributor);
  //   console.log('\nContributor balance', contributorAccount.amount.toString());

  //   const tx = await program.methods
  //     .refund()
  //     .accountsPartial({
  //       contributor: provider.publicKey,
  //       maker: maker.publicKey,
  //       mintToRaise: mint,
  //       proposal,
  //       contributorAccount: contributor,
  //       contributorAta: contributorATA,
  //       vault,
  //       tokenProgram: TOKEN_PROGRAM_ID,
  //       systemProgram: anchor.web3.SystemProgram.programId,
  //     })
  //     .rpc()
  //     .then(confirm);

  //   console.log('\nRefunded contributions', tx);
  //   console.log('Your transaction signature', tx);
  //   console.log('Vault balance', (await provider.connection.getTokenAccountBalance(vault)).value.amount);
  // });
});