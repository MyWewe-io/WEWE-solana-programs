import * as anchor from '@coral-xyz/anchor';
import { Program } from '@coral-xyz/anchor';
import type NodeWallet from '@coral-xyz/anchor/dist/cjs/nodewallet';
import BN from 'bn.js';

import {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  TOKEN_PROGRAM_ID,
  getAssociatedTokenAddressSync,
  getOrCreateAssociatedTokenAccount,
} from '@solana/spl-token';
import type { WeweTokenLaunchPad } from '../target/types/wewe_token_launch_pad.ts';

describe('wewe_token_launch_pad', () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.WeweTokenLaunchPad as Program<WeweTokenLaunchPad>;

  const maker = anchor.web3.Keypair.generate();

  let contributorATA: anchor.web3.PublicKey;

  let makerATA: anchor.web3.PublicKey;

  const wallet = provider.wallet as NodeWallet;

  const proposal = anchor.web3.PublicKey.findProgramAddressSync([Buffer.from('proposer'), maker.publicKey.toBuffer()], program.programId)[0];

  const backer = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from('backer'), proposal.toBuffer(), provider.publicKey.toBuffer()],
    program.programId,
  )[0];

  const [mint] = anchor.web3.PublicKey.findProgramAddressSync([Buffer.from('mint'), maker.publicKey.toBuffer()], program.programId);

  const metadata = {
    name: 'Solana Gold',
    symbol: 'GOLDSOL',
    uri: 'https://raw.githubusercontent.com/solana-developers/program-examples/new-examples/tokens/tokens/.assets/spl-token.json',
  };

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

  });

  it('Create proposal', async () => {
    const vault = getAssociatedTokenAddressSync(mint, proposal, true);
    const amount_to_raise = new BN(50000000);
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

    console.log('\nInitialized proposal Account');
    console.log('Your transaction signature', tx);

  });

  // it('Contribute to proposal', async () => {
  //   const vault = getAssociatedTokenAddressSync(mint, proposal, true);

  //   const tx = await program.methods
  //     .contribute(new anchor.BN(1000000))
  //     .accountsPartial({
  //       contributor: provider.publicKey,
  //       proposal,
  //       contributorAccount: contributor,
  //       contributorAta: contributorATA,
  //       vault,
  //       tokenProgram: TOKEN_PROGRAM_ID,
  //     })
  //     .rpc()
  //     .then(confirm);

  //   console.log('\nContributed to proposal', tx);
  //   console.log('Your transaction signature', tx);
  //   console.log('Vault balance', (await provider.connection.getTokenAccountBalance(vault)).value.amount);

  //   const contributorAccount = await program.account.contributor.fetch(contributor);
  //   console.log('Contributor balance', contributorAccount.amount.toString());
  // });
  // it('Contribute to proposal', async () => {
  //   const vault = getAssociatedTokenAddressSync(mint, proposal, true);

  //   const tx = await program.methods
  //     .contribute(new anchor.BN(1000000))
  //     .accountsPartial({
  //       contributor: provider.publicKey,
  //       proposal,
  //       contributorAccount: contributor,
  //       contributorAta: contributorATA,
  //       vault,
  //       tokenProgram: TOKEN_PROGRAM_ID,
  //     })
  //     .rpc()
  //     .then(confirm);

  //   console.log('\nContributed to proposal', tx);
  //   console.log('Your transaction signature', tx);
  //   console.log('Vault balance', (await provider.connection.getTokenAccountBalance(vault)).value.amount);

  //   const contributorAccount = await program.account.contributor.fetch(contributor);
  //   console.log('Contributor balance', contributorAccount.amount.toString());
  // });

  // it('Contribute to proposal - Robustness Test', async () => {
  //   try {
  //     const vault = getAssociatedTokenAddressSync(mint, proposal, true);

  //     const tx = await program.methods
  //       .contribute(new anchor.BN(2000000))
  //       .accountsPartial({
  //         contributor: provider.publicKey,
  //         proposal,
  //         contributorAccount: contributor,
  //         contributorAta: contributorATA,
  //         vault,
  //         tokenProgram: TOKEN_PROGRAM_ID,
  //       })
  //       .rpc()
  //       .then(confirm);

  //     console.log('\nContributed to proposal', tx);
  //     console.log('Your transaction signature', tx);
  //     console.log('Vault balance', (await provider.connection.getTokenAccountBalance(vault)).value.amount);
  //   } catch (error) {
  //     console.log('\nError contributing to proposal');
  //     console.log(error.msg);
  //   }
  // });

  // it('Check contributions - Robustness Test', async () => {
  //   try {
  //     const vault = getAssociatedTokenAddressSync(mint, proposal, true);

  //     const tx = await program.methods
  //       .checkContributions()
  //       .accountsPartial({
  //         maker: maker.publicKey,
  //         mintToRaise: mint,
  //         proposal,
  //         makerAta: makerATA,
  //         vault,
  //         tokenProgram: TOKEN_PROGRAM_ID,
  //       })
  //       .signers([maker])
  //       .rpc()
  //       .then(confirm);

  //     console.log('\nChecked contributions');
  //     console.log('Your transaction signature', tx);
  //     console.log('Vault balance', (await provider.connection.getTokenAccountBalance(vault)).value.amount);
  //   } catch (error) {
  //     console.log('\nError checking contributions');
  //     console.log(error.msg);
  //   }
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