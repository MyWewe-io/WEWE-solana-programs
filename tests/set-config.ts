// // tests/set-config.ts
// import * as anchor from '@coral-xyz/anchor';
// import { Program } from '@coral-xyz/anchor';
// import BN from 'bn.js';
// import { assert, expect } from 'chai';
// import * as fs from 'fs';
// import * as path from 'path';
// import * as os from 'os';
// import type { WeweTokenLaunchPad } from '../target/types/wewe_token_launch_pad';
// import {
//   confirm,
//   findConfigPDA,
//   findProposalPDA,
//   findMakerAccountPDA,
//   getTokenVaultAddress,
//   getMetadata,
//   generateKeypairs,
//   findMintAccount,
//   findUserAta,
// } from './utils';
// import {
//   TOKEN_PROGRAM_ID,
//   ASSOCIATED_TOKEN_PROGRAM_ID,
//   getAssociatedTokenAddressSync,
// } from '@solana/spl-token';

// // Helper function to wait for a specific event
// const waitForEvent = async (program: Program<any>, eventName: string): Promise<any> => {
//   return new Promise((resolve) => {
//     const listener = program.addEventListener(eventName, (event: any) => {
//       resolve(event);
//       program.removeEventListener(listener);
//     });
//   });
// };

// // Retry helper with exponential backoff for rate limits
// async function retryWithBackoff<T>(
//   fn: () => Promise<T>,
//   maxRetries = 5,
//   baseDelay = 2000
// ): Promise<T> {
//   for (let i = 0; i < maxRetries; i++) {
//     try {
//       return await fn();
//     } catch (err: any) {
//       const isRateLimit = 
//         err?.message?.includes('429') ||
//         err?.message?.includes('too many requests') ||
//         err?.message?.includes('rate limit') ||
//         err?.code === 429 ||
//         err?.logs?.some((log: string) => log.includes('429') || log.includes('rate limit'));
      
//       if (isRateLimit && i < maxRetries - 1) {
//         const delay = baseDelay * Math.pow(2, i);
//         console.log(`⚠️  Rate limit hit, retrying in ${delay}ms... (attempt ${i + 1}/${maxRetries})`);
//         await new Promise(resolve => setTimeout(resolve, delay));
//         continue;
//       }
//       throw err;
//     }
//   }
//   throw new Error('Max retries exceeded');
// }

// describe('Set Config Test', () => {
//   const provider = anchor.AnchorProvider.env();
//   anchor.setProvider(provider);

//   const program = anchor.workspace.WeweTokenLaunchPad as Program<WeweTokenLaunchPad>;
//   // Use the wallet from ANCHOR_WALLET or default Solana wallet
//   const walletPath = process.env.ANCHOR_WALLET || path.join(os.homedir(), '.config', 'solana', 'id.json');
//   const walletKeypair = JSON.parse(fs.readFileSync(walletPath, 'utf8'));
//   const authority = anchor.web3.Keypair.fromSecretKey(Uint8Array.from(walletKeypair));
  
//   // Generate test keypairs
//   const { maker } = generateKeypairs();
//   const mint = anchor.web3.Keypair.generate();
  
//   // Setup PDAs and accounts
//   const configStruct = findConfigPDA(program.programId, authority.publicKey);
//   const proposalIndex = new BN(0);
//   const proposal = findProposalPDA(program.programId, maker.publicKey, proposalIndex);
//   const makerAccount = findMakerAccountPDA(program.programId, maker.publicKey);
  
//   const [vaultAuthority] = anchor.web3.PublicKey.findProgramAddressSync(
//     [Buffer.from("vault_authority")],
//     program.programId
//   );
  
//   const [vault] = getTokenVaultAddress(vaultAuthority, mint.publicKey, program.programId);
//   const metadata = getMetadata();
  
//   // Soulbound mint and token account (required for createProposal)
//   const soulboundMint = findMintAccount(program.programId);
//   const makerSoulboundTokenAccount = findUserAta(maker.publicKey, soulboundMint);
  
//   // Mint authority PDA
//   const [mintAuthority] = anchor.web3.PublicKey.findProgramAddressSync(
//     [Buffer.from('mint_authority')],
//     program.programId
//   );
  
//   // Token Metadata Program ID
//   const TOKEN_METADATA_PROGRAM_ID = new anchor.web3.PublicKey('metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s');
  
//   // Metadata account PDA
//   const [metadataAccount] = anchor.web3.PublicKey.findProgramAddressSync(
//     [
//       Buffer.from('metadata'),
//       TOKEN_METADATA_PROGRAM_ID.toBuffer(),
//       mint.publicKey.toBuffer(),
//     ],
//     TOKEN_METADATA_PROGRAM_ID
//   );

//   it('1. Sets constant values', async () => {
//     // Check if config account exists
//     const accountInfo = await provider.connection.getAccountInfo(configStruct);
//     if (accountInfo) {
//       console.log(`✅ Config account already exists (${accountInfo.lamports / 1e9} SOL, ${accountInfo.data.length} bytes)`);
//     } else {
//       console.log(`ℹ️  Config account does not exist - will be created`);
//     }
    
//     const amountToRaisePerUser = new BN(10_000_000); // 0.1 SOL
//     const totalMint = new BN(1_000_000_000);
//     const totalPoolTokens = new BN(150_000_000);
//     const makerTokenAmount = new BN(10_000_000);
//     const totalAirdropAmountPerMilestone = new BN(140_000_000);
//     const minBackers = new BN(1);
//     const maxBackedProposals = new BN(3);
//     const refundFeeBps = new BN(1000); // 100 BPS = 1%
    
//     // Derive config PDA with bump to ensure it matches
//     const [derivedConfigPDA, bump] = anchor.web3.PublicKey.findProgramAddressSync(
//       [Buffer.from('config')],
//       program.programId
//     );
    
//     console.log(`📍 Config PDA: ${derivedConfigPDA.toBase58()}, bump: ${bump}`);
    
//     // Use .accounts() with explicit config PDA - this ensures init_if_needed works correctly
//     // TypeScript types might complain, but this is the correct way for init_if_needed with PDAs
//     const txPromise = (program.methods as any)
//       .setConfig(
//         amountToRaisePerUser,
//         totalMint,
//         totalPoolTokens,
//         makerTokenAmount,
//         totalAirdropAmountPerMilestone,
//         minBackers,
//         maxBackedProposals,
//         refundFeeBps,
//       )
//       .accounts({
//         authority: authority.publicKey,
//         config: derivedConfigPDA,
//         systemProgram: anchor.web3.SystemProgram.programId,
//       })
//       .signers([authority])
//       .rpc();

//     const txSignature = await confirm(txPromise);
//     console.log(`✅ Transaction confirmed: ${txSignature}`);
    
//     // Verify the config was set correctly
//     const configAccount = await program.account.configs.fetch(derivedConfigPDA);
//     assert(configAccount.amountToRaisePerUser.eq(amountToRaisePerUser), 'amountToRaisePerUser mismatch');
//     console.log('✅ Config set successfully');
//   });

//   it('2. Mints soulbound token to maker', async () => {
//     // Check if soulbound mint already exists
//     const soulboundMintInfo = await provider.connection.getAccountInfo(soulboundMint);
//     if (soulboundMintInfo) {
//       console.log(`✅ Soulbound mint already exists: ${soulboundMint.toBase58()}`);
//     } else {
//       console.log(`ℹ️  Soulbound mint does not exist - will be created`);
//     }
    
//     // Check if maker already has token
//     const makerTokenInfo = await provider.connection.getAccountInfo(makerSoulboundTokenAccount);
//     if (makerTokenInfo) {
//       console.log(`✅ Maker already has soulbound token`);
//       return; // Skip if already has token
//     }
    
//     console.log(`📤 Minting soulbound token to maker: ${maker.publicKey.toBase58()}`);
    
//     const txPromise = program.methods
//       .mintSoulboundToUser()
//       .accountsPartial({
//         payer: authority.publicKey,
//         user: maker.publicKey,
//         mintAuthority,
//         userTokenAccount: makerSoulboundTokenAccount,
//         associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
//         tokenProgram: TOKEN_PROGRAM_ID,
//         systemProgram: anchor.web3.SystemProgram.programId,
//       })
//       .signers([authority])
//       .rpc();

//     await confirm(txPromise);
//     console.log('✅ Soulbound token minted to maker');
//   });

//   it('3. Creates first proposal', async () => {
//     // Check if soulbound mint exists and maker has token
//     const soulboundMintInfo = await provider.connection.getAccountInfo(soulboundMint);
//     if (!soulboundMintInfo) {
//       throw new Error(`Soulbound mint ${soulboundMint.toBase58()} does not exist. You need to mint soulbound tokens first.`);
//     }
    
//     const makerTokenInfo = await provider.connection.getAccountInfo(makerSoulboundTokenAccount);
//     if (!makerTokenInfo) {
//       throw new Error(`Maker soulbound token account ${makerSoulboundTokenAccount.toBase58()} does not exist. Maker needs a soulbound token to create proposals.`);
//     }
    
//     console.log('📤 Creating proposal...');
//     console.log(`   Maker: ${maker.publicKey.toBase58()}`);
//     console.log(`   Proposal: ${proposal.toBase58()}`);
//     console.log(`   Mint: ${mint.publicKey.toBase58()}`);
    
//     try {
//       const txPromise = program.methods
//         .createProposal(metadata.name, metadata.symbol, metadata.uri)
//         .accountsPartial({
//           payer: authority.publicKey,
//           maker: maker.publicKey,
//           makerAccount,
//           vaultAuthority,
//           proposal,
//           mintAccount: mint.publicKey,
//           tokenVault: vault,
//           mint: soulboundMint,
//           userTokenAccount: makerSoulboundTokenAccount,
//           metadataAccount,
//           tokenProgram: TOKEN_PROGRAM_ID,
//           tokenMetadataProgram: TOKEN_METADATA_PROGRAM_ID,
//           systemProgram: anchor.web3.SystemProgram.programId,
//           rent: anchor.web3.SYSVAR_RENT_PUBKEY,
//           config: configStruct
//         })
//         .signers([authority, mint, maker])
//         .rpc();

//       const txSignature = await confirm(txPromise);
//       console.log(`✅ Transaction confirmed: ${txSignature}`);

//       // Verify proposal was created by fetching it directly (more reliable than events)
//       await new Promise(resolve => setTimeout(resolve, 2000)); // Wait a bit for account to be available
      
//       const proposalAccount = await program.account.proposal.fetch(proposal);
//       console.log('✅ Proposal account verified:');
//       console.log(`   Maker: ${proposalAccount.maker.toBase58()}`);
//       console.log(`   Mint: ${proposalAccount.mintAccount.toBase58()}`);
//       console.log(`   Proposal ID: ${proposalAccount.proposalId.toString()}`);
      
//       // Verify it matches expected values
//       expect(proposalAccount.maker.toBase58()).to.equal(maker.publicKey.toBase58());
//       expect(proposalAccount.mintAccount.toBase58()).to.equal(mint.publicKey.toBase58());
      
//       console.log('✅ Proposal created successfully');
//     } catch (err: any) {
//       console.error('❌ Error creating proposal:', err.message);
//       if (err.logs) {
//         console.error('Transaction logs:', err.logs);
//       }
//       throw err;
//     }
//   });
// });
