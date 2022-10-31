import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { NftStaking } from "../../target/types/nft_staking";
import { Keypair, PublicKey } from "@solana/web3.js";
import { TOKEN_PROGRAM_ID } from "@solana/spl-token";
import { Mint } from "./mint";
import { ItemType, Vault } from "./vault";

const VAULT_CTZN_REWARD_SEED = "vault_ctzn_reward";
const VAULT_ALIEN_REWARD_SEED = "vault_alien_reward";
const VAULT_GOD_REWARD_SEED = "vault_god_reward";
const VAULT_CTZN_USER_SEED = "vault_ctzn_user";
const VAULT_ALIEN_USER_SEED = "vault_alien_user";
const VAULT_STAKE_SEED = "vault_stake";

export function toPublicKey<T extends PublicKey | Keypair>(val: T): PublicKey {
  if ("publicKey" in val) {
    return val.publicKey;
  } else {
    return val;
  }
}

export async function getRewardAddress(
  source: PublicKey,
  program: Program<NftStaking>,
  userType: number
): Promise<[PublicKey, number]> {
  let seed_prefix;
  switch (userType) {
    case 0: seed_prefix = VAULT_CTZN_REWARD_SEED; break;
    case 1: seed_prefix = VAULT_ALIEN_REWARD_SEED; break;
    case 2: seed_prefix = VAULT_GOD_REWARD_SEED; break;
    default: break;
  }
  return await PublicKey.findProgramAddress(
    [
      Buffer.from(seed_prefix), 
      source.toBuffer()
    ],
    program.programId
  );
}

export async function getUserAddress(
  vault: PublicKey,
  authority: PublicKey,
  program: Program<NftStaking>,
  userType: number
): Promise<[PublicKey, number]> {
  return await PublicKey.findProgramAddress(
    [
      Buffer.from(userType === 0 ? VAULT_CTZN_USER_SEED : VAULT_ALIEN_USER_SEED), 
      vault.toBuffer(), 
      authority.toBuffer()
    ],
    program.programId
  );
}

export async function getStakeAddress(
  vault: PublicKey,
  authority: PublicKey,
  stakeAccount: PublicKey,
  program: Program<NftStaking>,
): Promise<[PublicKey, number]> {
  return await PublicKey.findProgramAddress(
    [
      Buffer.from(VAULT_STAKE_SEED), 
      vault.toBuffer(), 
      authority.toBuffer(),
      stakeAccount.toBuffer(),
    ],
    program.programId
  );
}

export async function getBlockTime(program: Program<NftStaking>): Promise<number> {
  const slot = await program.provider.connection.getSlot();
  const timestamp = await program.provider.connection.getBlockTime(slot);
  return timestamp;
}

export async function spawnMoney(
  program: anchor.Program<NftStaking>,
  to: PublicKey,
  sol: number
): Promise<anchor.web3.TransactionSignature> {
  const lamports = sol * anchor.web3.LAMPORTS_PER_SOL;
  const transaction = new anchor.web3.Transaction();
  transaction.add(
    anchor.web3.SystemProgram.transfer({
      fromPubkey: program.provider.wallet.publicKey,
      lamports,
      toPubkey: to,
    })
  );
  return await program.provider.sendAndConfirm(transaction, [], {
    commitment: "confirmed",
  });
}

export function sleep(ms: number) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

export async function getTokenAmounts(
  program: Program<NftStaking>,
  owner: PublicKey,
  tokenAccount: PublicKey
): Promise<number> {
  const { value: accounts } =
    await program.provider.connection.getParsedTokenAccountsByOwner(owner, {
      programId: new PublicKey(TOKEN_PROGRAM_ID),
    });

  const checkedAccounts = accounts.filter(
    (t) => t.pubkey.toString() === tokenAccount.toString()
  );

  if (checkedAccounts.length > 0) {
    // console.log(checkedAccounts[0].account.data.parsed.info.tokenAmount);
    return checkedAccounts[0].account.data.parsed.info.tokenAmount.amount as number;
  }

  return 0;
}

export async function checkTokenAccounts(
  program: Program<NftStaking>,
  owner: PublicKey,
  tokenAccount: PublicKey
): Promise<boolean> {
  const { value: accounts } =
    await program.provider.connection.getParsedTokenAccountsByOwner(owner, {
      programId: new PublicKey(TOKEN_PROGRAM_ID),
    });

  const checkedAccounts = accounts.filter(
    (t) => t.pubkey.toString() === tokenAccount.toString()
  );

  return checkedAccounts.length > 0;
}

export async function createVault(program: Program<NftStaking>): Promise<{
  mint: Mint;
  authority: Keypair;
  vault: Vault;
}> {
  // create reward token
  const mint = await Mint.create(program);

  // create vault
  const { authority, vault } = await Vault.create({
    program,
    mint,
  });

  return {
    mint,
    authority,
    vault,
  };
}