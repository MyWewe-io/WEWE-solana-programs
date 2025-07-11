// utils/helpers.ts
import * as anchor from '@coral-xyz/anchor';
import { ASSOCIATED_TOKEN_PROGRAM_ID, getAssociatedTokenAddressSync } from '@solana/spl-token';

export const WSOL_MINT = new anchor.web3.PublicKey("So11111111111111111111111111111111111111112");

export const confirm = async (promise: Promise<string>, provider = anchor.AnchorProvider.env()): Promise<string> => {
  const signature = await promise;
  const block = await provider.connection.getLatestBlockhash();
  await provider.connection.confirmTransaction({ signature, ...block });
  return signature;
};

export const generateKeypairs = () => ({
  maker: anchor.web3.Keypair.generate(),
  backer: anchor.web3.Keypair.generate(),
  authority: anchor.web3.Keypair.fromSecretKey(Uint8Array.from([42, 132, 54, 48, 86, 137, 10, 155, 254, 103, 140, 97, 104, 8, 197, 48, 55, 71, 171, 157, 247, 159, 233, 130, 100, 213, 107, 236, 96, 40, 175, 164, 179, 49, 15, 185, 22, 130, 249, 11, 142, 174, 6, 253, 52, 133, 167, 81, 80, 179, 15, 199, 164, 252, 14, 233, 42, 74, 178, 20, 71, 62, 139, 21])
  ),
  mint: anchor.web3.Keypair.generate(),
  mint2: anchor.web3.Keypair.generate(),
});

export const getMetadata = () => ({
  name: 'Solana Gold',
  symbol: 'GOLDSOL',
  uri: 'https://raw.githubusercontent.com/solana-developers/program-examples/new-examples/tokens/tokens/.assets/spl-token.json',
});

export const findProposalPDA = (programId: anchor.web3.PublicKey, maker: anchor.web3.PublicKey, index: anchor.BN) =>
  anchor.web3.PublicKey.findProgramAddressSync([
    Buffer.from('proposal'),
    maker.toBuffer(),
    index.toArrayLike(Buffer, "le", 8),
  ], programId)[0];

export const findBackerAccountPDA = (programId: anchor.web3.PublicKey, proposal: anchor.web3.PublicKey, backer: anchor.web3.PublicKey) =>
  anchor.web3.PublicKey.findProgramAddressSync([
    Buffer.from('backer'),
    proposal.toBuffer(),
    backer.toBuffer(),
  ], programId)[0];

export const findMakerAccountPDA = (programId: anchor.web3.PublicKey, maker: anchor.web3.PublicKey) =>
  anchor.web3.PublicKey.findProgramAddressSync([
    Buffer.from('maker'),
    maker.toBuffer(),
  ], programId)[0];

export const getTokenVaultAddress = (
  vaultAuthority: anchor.web3.PublicKey,
  mint: anchor.web3.PublicKey,
  programId: anchor.web3.PublicKey
): [anchor.web3.PublicKey, number] => {
  return anchor.web3.PublicKey.findProgramAddressSync(
    [
      Buffer.from("token_vault"),
      vaultAuthority.toBuffer(),
      mint.toBuffer(),
    ],
    programId
  );
};

export function findMintAccount(programId: anchor.web3.PublicKey): anchor.web3.PublicKey {
  const [mintAccount] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("mint_soulbound")],
    programId
  );
  return mintAccount;
}

export function findFreezeAuthority(programId: anchor.web3.PublicKey): anchor.web3.PublicKey {
  const [freezeAuthority] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("freeze_authority")],
    programId
  );
  return freezeAuthority;
}

export function findMintAuthority(programId: anchor.web3.PublicKey): anchor.web3.PublicKey {
  const [freezeAuthority] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("mint_authority")],
    programId
  );
  return freezeAuthority;
}

export function findUserAta(user: anchor.web3.PublicKey, mint: anchor.web3.PublicKey): anchor.web3.PublicKey {
  return getAssociatedTokenAddressSync(mint, user, false, undefined, undefined);
}

export const derivePoolPDAs = (
  programId: anchor.web3.PublicKey,
  cpAmmProgramId: anchor.web3.PublicKey,
  baseMint: anchor.web3.PublicKey,
  quoteMint: anchor.web3.PublicKey,
  maker: anchor.web3.PublicKey,
  config: anchor.web3.PublicKey
) => {
  const minKey = (a: anchor.web3.PublicKey, b: anchor.web3.PublicKey) =>
    a.toBuffer().compare(b.toBuffer()) < 0 ? a : b;
  const maxKey = (a: anchor.web3.PublicKey, b: anchor.web3.PublicKey) =>
    a.toBuffer().compare(b.toBuffer()) > 0 ? a : b;

  const [poolAuthority, poolAuthorityBump] = anchor.web3.PublicKey.findProgramAddressSync([
    Buffer.from("pool_authority"),
  ], cpAmmProgramId);

  const [pool] = anchor.web3.PublicKey.findProgramAddressSync([
    Buffer.from("pool"),
    config.toBuffer(),
    maxKey(baseMint, quoteMint).toBuffer(),
    minKey(baseMint, quoteMint).toBuffer(),
  ], cpAmmProgramId);

  const positionNftMint = anchor.web3.Keypair.generate();

  const [position] = anchor.web3.PublicKey.findProgramAddressSync([
    Buffer.from("position"),
    positionNftMint.publicKey.toBuffer(),
  ], cpAmmProgramId);

  const [positionNftAccount] = anchor.web3.PublicKey.findProgramAddressSync([
    Buffer.from("position_nft_account"),
    positionNftMint.publicKey.toBuffer(),
  ], cpAmmProgramId);

  const [dammEventAuthority] = anchor.web3.PublicKey.findProgramAddressSync([
    Buffer.from("__event_authority"),
  ], cpAmmProgramId);

  const [tokenAVault] = anchor.web3.PublicKey.findProgramAddressSync([
    Buffer.from("token_vault"),
    baseMint.toBuffer(),
    pool.toBuffer(),
  ], cpAmmProgramId);

  const [tokenBVault] = anchor.web3.PublicKey.findProgramAddressSync([
    Buffer.from("token_vault"),
    quoteMint.toBuffer(),
    pool.toBuffer(),
  ], cpAmmProgramId);

  const [quoteVault] = anchor.web3.PublicKey.findProgramAddressSync([
    maker.toBuffer(),
    quoteMint.toBuffer(),
  ], programId);

  const [baseVault] = anchor.web3.PublicKey.findProgramAddressSync([
    maker.toBuffer(),
    baseMint.toBuffer(),
  ], programId);

  const makerTokenAccount = getAssociatedTokenAddressSync(
    baseMint,
    maker,
    false,
    programId,
    ASSOCIATED_TOKEN_PROGRAM_ID
  );

  return {
    poolAuthority,
    poolAuthorityBump,
    pool,
    positionNftMint,
    position,
    positionNftAccount,
    tokenAVault,
    tokenBVault,
    quoteVault,
    baseVault,
    dammEventAuthority,
    makerTokenAccount
  };
};