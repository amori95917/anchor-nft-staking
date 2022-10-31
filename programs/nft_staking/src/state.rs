use anchor_lang::prelude::*;

#[derive(Debug, AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum VaultStatus {
    None,
    Initialized,
}

impl Default for VaultStatus {
  fn default() -> Self {
      Self::None
  }
}

#[account]
#[derive(Default)]
pub struct Vault {
    // authority
    pub authority: Pubkey,
    // state
    pub status: VaultStatus,
    // reward token mint
    pub reward_mint: Pubkey,
    // ctzns pool account bump
    pub ctzns_pool_bump: u8,
    // ctzns pool account
    pub ctzns_pool_account: Pubkey,
    // aliens pool account bump
    pub aliens_pool_bump: u8,
    // aliens pool account
    pub aliens_pool_account: Pubkey,
    // alien gods pool account bump,
    pub gods_pool_bump: u8,
    // alien gods pool account
    pub gods_pool_account: Pubkey,
    // reward token amount in ctzns pool
    pub ctzns_pool_amount: u64,
    // reward token amount in aliens pool
    pub aliens_pool_amount: u64,
    // reward token amount in alien gods pool
    pub gods_pool_amount: u64,
    // alpha aliens count
    pub alpha_aliens_count: u8,
    // normal aliens count
    pub normal_aliens_count: u8,
    // aliens 
    pub aliens: Vec<StakeItem>,
}

impl Vault {
  pub const LEN: usize = 32 + 1 + 32 + 1 + 32 + 1 + 32 + 1 + 32 + 8 + 8 + 8 + 1 + 1 + StakeItem::LEN * 77;
}


#[derive(Debug, AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum ItemType {
    NormalCTZN,
    NormalAlien,
    AlphaAlien,
    AlienGod
}

impl Default for ItemType {
  fn default() -> Self {
      Self::NormalCTZN
  }
}

#[derive(Debug, AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum UserType {
    Ctzn,
    Alien,
}

impl Default for UserType {
  fn default() -> Self {
      Self::Ctzn
  }
}

#[account]
#[derive(Default)]
pub struct User {
    // vault
    pub vault: Pubkey,
    // user type
    pub user_type: UserType,
    // user pub key
    pub key: Pubkey,
    // number of staked nfts
    pub items_count: u32,
    // staked items
    pub items: Vec<StakeItem>,
}

impl User {
  pub const LEN: usize = 32 + 1 + 32 + 4 + StakeItem::LEN * 100;
}

#[derive(Debug, AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub struct StakeItem {
  // mint key
  pub mint: Pubkey,
  // mint account
  pub mint_account: Pubkey,
  // type of nft item
  pub item_type: ItemType,
  // first_stake_time
  pub first_staked_time: u64,
  // last claimed time
  pub last_claimed_time: u64,
  // earned_reward
  pub earned_reward: u64,
}

impl StakeItem {
  pub const LEN: usize = 32 + 32 + 1 + 8 + 8 + 8;
}

#[error_code]
pub enum ErrorCode {
  #[msg("Vault already created")]
  VaultAlreadyCreated,
  #[msg("Vault didn't initialized")]
  VaultNotInitialized,
  #[msg("User already created")]
  UserAlreadyCreated,
  #[msg("Already staked item")]
  AlreadyStakedAccount,
  #[msg("Stake acount does not exist")]
  StakedAccountDoesNotExist,
  #[msg("Cannot Unstake Alien untill 2 days reward accrued")]
  CannotUnstakeAlien,
}