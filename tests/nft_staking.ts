import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { expect } from "chai";
import { NftStaking } from "../target/types/nft_staking";
// import { PublicKey } from '@solana/web3.js';
import {
  checkTokenAccounts,
  createVault, 
  getRewardAddress, 
  getTokenAmounts,
  getStakeAddress,
  sleep,
  spawnMoney,
} from "./fixtures/lib";
import { UserData, VaultData } from "./fixtures/vault";
import { Keypair } from '@solana/web3.js';


describe("nft_staking", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const program = anchor.workspace.NftStaking as Program<NftStaking>;
  
/*
  it("Create Vault", async () => {
    const { vault, authority } = await createVault(program);

    // fetch vault data
    const vaultData: VaultData = await vault.fetch();

    // check the result
    expect(vaultData.authority.toString()).to.equal(
      authority.publicKey.toString()
    );
    expect(vaultData.ctznsPoolAccount.toString()).to.equal(
      vault.ctznsPoolAccount.toString()
    );
    expect(vaultData.aliensPoolAccount.toString()).to.equal(
      vault.aliensPoolAccount.toString()
    );
    expect(vaultData.godsPoolAccount.toString()).to.equal(
      vault.godsPoolAccount.toString()
    );
    expect(vaultData.ctznsPoolAmount.toNumber()).to.equal(0);
    expect(vaultData.aliensPoolAmount.toNumber()).to.equal(0);
    expect(vaultData.godsPoolAmount.toNumber()).to.equal(0);
    expect(vaultData.alphaAliensCount).to.equal(0);
    expect(vaultData.normalAliensCount).to.equal(0);
    expect(vaultData.status.initialized !== null).to.be.true;
    // console.log(vaultData);
  });
  
  it("Create User", async () => {
    const { vault } = await createVault(program);

    // create user
    const { authority: userAuthrity, user } = await vault.createUser({userType: 0});
    const userData: UserData = await vault.fetchUser(user);
    // console.log(userData);
    // const vaultData = await vault.fetch();

    expect(userData.vault.toString()).to.equal(vault.key.toString());
    expect(userData.items.length).to.equal(0);
    expect(userData.key.toString()).to.equal(
      userAuthrity.publicKey.toString()
    );
    expect(userData.userType.ctzn !== null).to.be.true;
    expect(userData.itemsCount).to.equal(0);
  });

  it("Fund Amount", async () => {
    const { authority, vault, mint } = await createVault(program);

    // add funder
    const funder = Keypair.generate();
    const funderAccount = await mint.createAssociatedAccount(
      funder.publicKey
    );

    const amount = new anchor.BN("1000000");
    await mint.mintTokens(funderAccount, amount.toNumber());

    // fund
    await vault.fund({ 
      authority, 
      funder, 
      funderAccount: funderAccount.key, 
      amount,
    });

    let vaultData = await vault.fetch();
    const after_pool_amount = vaultData.ctznsPoolAmount;
    const after_amount = await getTokenAmounts(program, vault.ctznsPool, vault.ctznsPoolAccount);
    expect(after_amount.toString()).to.equal(amount.toString());
    expect(after_pool_amount.toString()).to.equal(amount.toString());
  });
*/
  it("Stake a CTZN and Unstake", async () => {
    let userData: UserData;
    let vaultData: VaultData;

    // create vault
    const { mint, authority, vault } = await createVault(program);
  
    // add funder
    const funder = Keypair.generate();
    const funderAccount = await mint.createAssociatedAccount(
      funder.publicKey
    );
    
    const amount = new anchor.BN("1000000");
    await mint.mintTokens(funderAccount, amount.toNumber());

    // fund
    await vault.fund({
      authority,
      funder,
      funderAccount: funderAccount.key,
      amount,
    });

    // create ctzn user and stake
    const { userAuthority, user, stakeAccount } = await vault.stake(0);

    // get user and vault data
    vaultData = await vault.fetch();
    userData = await vault.fetchUser(user);

    // check staked account is not owned by user anymore
    let stakeAccountOwned = await checkTokenAccounts(
      program,
      userAuthority.publicKey,
      stakeAccount.key,
    );

    expect(!stakeAccountOwned).to.be.true;

    // check vault pda account owned  
    const [vaultPda] = await getStakeAddress(
      vault.key, 
      userAuthority.publicKey, 
      stakeAccount.key, 
      program
    );
    stakeAccountOwned = await checkTokenAccounts(
      program,
      vaultPda,
      stakeAccount.key,
    );

    expect(stakeAccountOwned).to.be.true;

    // check user data and vault data
    expect(userData.itemsCount).to.equal(1);
    expect(userData.items.length).to.equal(1);
    expect(userData.items[0].mintAccount.toString()).to.equal(stakeAccount.key.toString());
    expect(userData.items[0].earnedReward.toNumber()).to.equal(0);
    expect(userData.items[0].itemType.normalCtzn !== null).to.be.true;
    expect(userData.userType.ctzn !== null).to.be.true;
    // unstake 
    await vault.unstake_manually(userAuthority, userAuthority.publicKey, user, stakeAccount);

    // check staked account owned back to user
    stakeAccountOwned = await checkTokenAccounts(
      program,
      userAuthority.publicKey,
      stakeAccount.key,
    );
    expect(stakeAccountOwned).to.be.true;
    
    stakeAccountOwned = await checkTokenAccounts(
      program,
      vaultPda,
      stakeAccount.key,
    );
    expect(!stakeAccountOwned).to.be.true;
    // // check user and vault data
    userData = await vault.fetchUser(user);
    vaultData = await vault.fetch();

    expect(userData.itemsCount).to.equal(0);
    expect(userData.items.length).to.equal(0);


  });
/*
  it("Stake an Alien and Unstake", async () => {
    let userData: UserData;
    let vaultData: VaultData;

    // create vault
    const { mint, authority, vault } = await createVault(program);
  
    // add funder
    const funder = Keypair.generate();
    const funderAccount = await mint.createAssociatedAccount(
      funder.publicKey
    );
    
    const amount = new anchor.BN("1000000");
    await mint.mintTokens(funderAccount, amount.toNumber());

    // fund
    await vault.fund({
      authority,
      funder,
      funderAccount: funderAccount.key,
      amount,
    });

    // create alien user and stake
    const { userAuthority, user, stakeAccount } = await vault.stake(1);

    // get user and vault data
    vaultData = await vault.fetch();
    userData = await vault.fetchUser(user);

    // check staked account is not owned by user anymore
    let stakeAccountOwned = await checkTokenAccounts(
      program,
      userAuthority.publicKey,
      stakeAccount.key,
    );

    expect(!stakeAccountOwned).to.be.true;

    // check vault pda account owned  
    const [vaultPda] = await getStakeAddress(
      vault.key, 
      userAuthority.publicKey, 
      stakeAccount.key, 
      program
    );
    stakeAccountOwned = await checkTokenAccounts(
      program,
      vaultPda,
      stakeAccount.key,
    );

    expect(stakeAccountOwned).to.be.true;

    // check user data and vault data
    expect(userData.itemsCount).to.equal(1);
    expect(userData.items.length).to.equal(1);
    expect(userData.items[0].mintAccount.toString()).to.equal(stakeAccount.key.toString());
    expect(userData.items[0].earnedReward.toNumber()).to.equal(0);
    expect(userData.items[0].itemType.normalCtzn !== null).to.be.true;
    expect(userData.userType.ctzn !== null).to.be.true;
    // unstake 
    try {
      await vault.unstake(userAuthority, user, stakeAccount);
    } catch (error) {
      const expectErrorMessage = 'Cannot Unstake Alien untill 2 days reward accrued';
      expect(error.error.errorMessage).to.equal(expectErrorMessage);
    }

    // check staked account owned back to user
    stakeAccountOwned = await checkTokenAccounts(
      program,
      userAuthority.publicKey,
      stakeAccount.key,
    );
    expect(!stakeAccountOwned).to.be.true;
    
    stakeAccountOwned = await checkTokenAccounts(
      program,
      vaultPda,
      stakeAccount.key,
    );
    expect(stakeAccountOwned).to.be.true;
    // // check user and vault data
    userData = await vault.fetchUser(user);
    vaultData = await vault.fetch();

    expect(userData.itemsCount).to.equal(1);
    expect(userData.items.length).to.equal(1);
  });

  it("Get ctzn user's reward amount",async () => {
    let userData: UserData;
    let vaultData: VaultData;

    // create vault
    const { mint, authority, vault } = await createVault(program);
  
    // add funder
    const funder = Keypair.generate();
    const funderAccount = await mint.createAssociatedAccount(
      funder.publicKey
    );
    
    const amount = new anchor.BN("1000000");
    await mint.mintTokens(funderAccount, amount.toNumber());

    // fund
    await vault.fund({
      authority,
      funder,
      funderAccount: funderAccount.key,
      amount,
    });

    // create ctzn user and stake
    const { user } = await vault.stake(0);
    await sleep(10000);
    const reward_earned = await vault.getRewardAmount(user);
    
    // console.log(reward_earned);
  });

  it("Claim CTZN",async () => {
    let userData: UserData;
    let vaultData: VaultData;

    // create vault
    const { mint, authority, vault } = await createVault(program);
  
    // add funder
    const funder = Keypair.generate();
    const funderAccount = await mint.createAssociatedAccount(
      funder.publicKey
    );
    
    const amount = new anchor.BN("1000000");
    await mint.mintTokens(funderAccount, amount.toNumber());

    // fund
    await vault.fund({
      authority,
      funder,
      funderAccount: funderAccount.key,
      amount,
    });

    // create ctzn user and stake
    const { user, userAuthority } = await vault.stake(0);
    await spawnMoney(program, userAuthority.publicKey, 10);
    await sleep(5000);
    userData = await vault.fetchUser(user);
    console.log(userData);
    await vault.claim(userAuthority, user, 0);
    
    userData = await vault.fetchUser(user);
    vaultData = await vault.fetch();

    console.log(userData, userData.items[0].lastClaimedTime.toNumber(), userData.items[0].firstStakedTime.toNumber());
  });

  it("Withdraw", async () => {
    const { mint, authority, vault } = await createVault(program);
  
    // add funder
    const funder = Keypair.generate();
    const funderAccount = await mint.createAssociatedAccount(
      funder.publicKey
    );
    
    const amount = new anchor.BN("1000000");
    await mint.mintTokens(funderAccount, amount.toNumber());
    // console.log(await getTokenAmounts(program, funder.publicKey, funderAccount.key));

    // fund
    await vault.fund({
      authority,
      funder,
      funderAccount: funderAccount.key,
      amount,
    });

    // console.log(await getTokenAmounts(program, funder.publicKey, funderAccount.key));
    await vault.withdraw(funder, amount);
    // console.log(await getTokenAmounts(program, funder.publicKey, funderAccount.key));
  });*/
});
