import * as anchor from "@project-serum/anchor";
import {
  TOKEN_PROGRAM_ID,
  ASSOCIATED_TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import {
  PublicKey,
  Keypair,
  TransactionSignature,
  SYSVAR_RENT_PUBKEY,
  SystemProgram,
} from "@solana/web3.js";
import { Mint } from "./mint";
import { 
  getRewardAddress, 
  getUserAddress,
  getStakeAddress,
  spawnMoney,
  getBlockTime,
} from "./lib";
import { TokenAccount } from "./token-account";
import { NftStaking } from "../../target/types/nft_staking";

const VAULT_STAKE_SEED = "vault_stake";
export class Vault {
  constructor(
    public program: anchor.Program<NftStaking>,
    public key: PublicKey,
    public mint: Mint,
    public ctznsPool: PublicKey,
    public aliensPool: PublicKey,
    public godsPool: PublicKey,
    public ctznsPoolAccount: PublicKey,
    public aliensPoolAccount: PublicKey,
    public godsPoolAccount: PublicKey,
    public ctznsPoolAmount: number,
    public aliensPoolAmount: number,
    public godsPoolAmount: number,
  ) {}

  async fetch(): Promise<VaultData | null> {
    return (await this.program.account.vault.fetchNullable(
      this.key
    )) as VaultData | null;
  }

  async fetchUser(userAddress: PublicKey): Promise<UserData | null> {
    return (await this.program.account.user.fetchNullable(
      userAddress
    )) as UserData | null;
  }

  static async create({
    authority = Keypair.generate(),
    vaultKey = Keypair.generate(),
    program,
    mint,
  }: {
    authority?: Keypair;
    vaultKey?: Keypair;
    program: anchor.Program<NftStaking>;
    mint: Mint;
  }): Promise<{
    authority: Keypair;
    vault: Vault;
    sig: TransactionSignature;
  }> {
    await spawnMoney(program, authority.publicKey, 10);

    const [ctznsPool, ctzns_pool_bump] = await getRewardAddress(
      vaultKey.publicKey,
      program,
      0
    );

    const [aliensPool, aliens_pool_bump] = await getRewardAddress(
      vaultKey.publicKey,
      program,
      1
    );

    const [godsPool, gods_pool_bump] = await getRewardAddress(
      vaultKey.publicKey,
      program,
      2
    );

    const ctznsPoolAccount = await mint.getAssociatedTokenAddress(ctznsPool);
    const aliensPoolAccount = await mint.getAssociatedTokenAddress(aliensPool);
    const godsPoolAccount = await mint.getAssociatedTokenAddress(godsPool);

    const txSignature = await program.rpc.createVault(
      ctzns_pool_bump,
      aliens_pool_bump,
      gods_pool_bump,
      {
        accounts: {
          authority: authority.publicKey,
          vault: vaultKey.publicKey,
          rewardMint: mint.key,
          ctznsPool,
          aliensPool,
          godsPool,
          ctznsPoolAccount,
          aliensPoolAccount,
          godsPoolAccount,
          rent: SYSVAR_RENT_PUBKEY,
          tokenProgram: TOKEN_PROGRAM_ID,
          associatedToken: ASSOCIATED_TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        },
        signers: [authority, vaultKey],
        options: {
          commitment: "confirmed",
        },
      }
    );
    return {
      authority,
      vault: new Vault(
        program,
        vaultKey.publicKey,
        mint,
        ctznsPool,
        aliensPool,
        godsPool,
        ctznsPoolAccount,
        aliensPoolAccount,
        godsPoolAccount,
        0,
        0,
        0,
      ),
      sig: txSignature,
    };
  }

  async createUser({
    authority = Keypair.generate(), 
    userType
  }: {
    authority?: Keypair,
    userType: number
  }): Promise<{
    authority: Keypair;
    user: PublicKey;
    sig: TransactionSignature;
  }> {
    await spawnMoney(this.program, authority.publicKey, 10);
    const [userAddress] = await getUserAddress(
      this.key,
      authority.publicKey,
      this.program,
      userType
    );

    const txSignature = await this.program.rpc.createUser(userType, {
      accounts: {
        authority: authority.publicKey,
        vault: this.key,
        user: userAddress,
        systemProgram: SystemProgram.programId,
      },
      signers: [authority],
      options: {
        commitment: "confirmed",
      },
    });

    return {
      authority,
      user: userAddress,
      sig: txSignature,
    };
  }

  async fund({
    authority,
    funder,
    funderAccount,
    amount,
  }: {
    authority: Keypair;
    funder: Keypair;
    funderAccount: PublicKey;
    amount: anchor.BN;
  }): Promise<{
    sig: TransactionSignature;
  }> {
    const txSignature = await this.program.rpc.fund(amount, {
      accounts: {
        funder: funder.publicKey,
        vault: this.key,
        ctznsPoolAccount: this.ctznsPoolAccount,
        funderAccount,
        tokenProgram: TOKEN_PROGRAM_ID,
      },
      signers: [funder],
      options: {
        commitment: "confirmed",
      },
    });
    return {
      sig: txSignature,
    };
  }

  async stake(
    itemType: number,
    curAuthoriy?: Keypair,
    curUser?: PublicKey,
  ): Promise<{
    userAuthority: Keypair;
    user: PublicKey;
    stakeAccount: TokenAccount<PublicKey>;
    stakeMint: Mint;
  }> {
    let userAuthority: Keypair;
    let user: PublicKey;

    if (!curUser) {
      // create user
      const { authority, user: created } = await this.createUser({ userType: itemType == 0 ? 0 : 1 })
      userAuthority = authority;
      user = created;
    } else {
      userAuthority = curAuthoriy;
      user = curUser;
    }

    // create a token to be staked and its account of userAuthority
    const stakeMint = await Mint.create(this.program);
    const stakeAccount = await stakeMint.createAssociatedAccount(
      userAuthority.publicKey
    );
    await stakeMint.mintTokens(stakeAccount, 1);

    // stake
    await this.program.rpc.stake(itemType, {
      accounts: {
        staker: userAuthority.publicKey,
        vault: this.key,
        stakeAccount: stakeAccount.key,
        stakeMint: stakeMint.key,
        user,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      },
      signers: [userAuthority],
      options: { commitment: "confirmed" },
    });

    return { userAuthority, user, stakeAccount, stakeMint };
  }

  async unstake(
    authority: Keypair,
    user: PublicKey,
    stakeAccount: TokenAccount<PublicKey>,
  ): Promise<boolean> {
    const [vaultPda, vaultStakeBump] = await getStakeAddress(
      this.key, 
      authority.publicKey, 
      stakeAccount.key, 
      this.program
    );

    await this.program.rpc.unstake(vaultStakeBump, {
      accounts: {
        staker: authority.publicKey,
        vault: this.key,
        unstakeAccount: stakeAccount.key,
        vaultPda,
        user,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      },
      signers: [authority],
      options: { commitment: "confirmed" },
    });
    return true;
  }

  async unstake_manually(
    authority: Keypair,
    staker: PublicKey,
    user: PublicKey,
    stakeAccount: TokenAccount<PublicKey>,
  ): Promise<boolean> {
    const [vaultPda, vaultStakeBump] = await getStakeAddress(
      this.key, 
      staker, 
      stakeAccount.key, 
      this.program
    );

    await this.program.rpc.unstake(vaultStakeBump, true, {
      accounts: {
        // payer: authority.publicKey,
        staker,
        vault: this.key,
        unstakeAccount: stakeAccount.key,
        vaultPda,
        user,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      },
      // signers: [authority],
      options: { commitment: "confirmed" },
    });
    return true;
  }

  async getRewardAmount(
    user: PublicKey,
  ): Promise<number> {
    const now = await getBlockTime(this.program);
    const userData = await this.fetchUser(user);
    let total = 0;
    userData.items.forEach(item => {
      total = total + item.earnedReward.toNumber() + (now - item.lastClaimedTime.toNumber()) * 36;
      // console.log(item.lastClaimedTime.toNumber());
    });
    return total;
  }

  async claim(claimer: Keypair, user: PublicKey, userType: number) {
    const claimerAccount = await this.mint.getAssociatedTokenAddress(
      claimer.publicKey
    );
    const [ctznsPool] = await getRewardAddress(
      this.key,
      this.program,
      0
    );

    const [aliensPool] = await getRewardAddress(
      this.key,
      this.program,
      1
    );

    const [godsPool] = await getRewardAddress(
      this.key,
      this.program,
      2
    );

    const ctznsPoolAccount = await this.mint.getAssociatedTokenAddress(ctznsPool);
    const aliensPoolAccount = await this.mint.getAssociatedTokenAddress(aliensPool);
    const godsPoolAccount = await this.mint.getAssociatedTokenAddress(godsPool);
    await this.program.rpc.claim(userType, {
      accounts: {
        claimer: claimer.publicKey,
        vault: this.key,
        ctznsPool,
        aliensPool,
        godsPool,
        rewardMint: this.mint.key,
        ctznsPoolAccount,
        aliensPoolAccount,
        godsPoolAccount,
        claimerAccount,
        user,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        rent: SYSVAR_RENT_PUBKEY,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      },
      signers: [claimer],
      options: { commitment: "confirmed" },
    });
  }

  async withdraw(claimer: Keypair, amount: anchor.BN) {
    const claimerAccount = await this.mint.getAssociatedTokenAddress(
      claimer.publicKey
    );
    const [ctznsPool] = await getRewardAddress(
      this.key,
      this.program,
      0
    );
    const ctznsPoolAccount = await this.mint.getAssociatedTokenAddress(ctznsPool);
    await this.program.rpc.withdrawCtznsPool(amount, {
      accounts: {
        claimer: claimer.publicKey,
        vault: this.key,
        ctznsPool,
        rewardMint: this.mint.key,
        ctznsPoolAccount,
        claimerAccount,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        rent: SYSVAR_RENT_PUBKEY,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      },
      signers: [claimer],
      options: { commitment: "confirmed" },
    });
  }
}

export type VaultStatus = {
  none?: {};
  initialized?: {};
};

export type VaultData = {
  authority: PublicKey;
  status: VaultStatus;
  rewardMint: PublicKey;
  ctznsPoolBump: number;
  ctznsPoolAccount: PublicKey;
  aliensPoolBump: number;
  aliensPoolAccount: PublicKey;
  godsPoolBump: number;
  godsPoolAccount: PublicKey;
  ctznsPoolAmount: anchor.BN;
  aliensPoolAmount: anchor.BN;
  godsPoolAmount: anchor.BN;
  alphaAliensCount: number;
  normalAliensCount: number;
};


export type UserData = {
  vault: PublicKey;
  key: PublicKey;
  userType: UserType;
  itemsCount: number;
  items: StakeItemData[];
};

export type ItemType = {
  normalCtzn?: {};
  normalAlien?: {};
  alphaAlien?: {};
  alienGod?: {};
};

export type UserType = {
  ctzn?: {};
  alien?: {};
};


export type StakeItemData = {
  mint: PublicKey;
  mintAccount: PublicKey;
  itemType: ItemType;
  firstStakedTime: anchor.BN;
  lastClaimedTime: anchor.BN;
  earnedReward: anchor.BN;
} 

