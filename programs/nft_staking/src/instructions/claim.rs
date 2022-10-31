use anchor_lang::prelude::*;
use crate::{
  state::{Vault, VaultStatus, User, ErrorCode, ItemType},
  constant::{
    VAULT_ALIEN_REWARD_SEED,
    VAULT_CTZN_REWARD_SEED, 
    VAULT_GOD_REWARD_SEED,
    ONE_DAY_TO_SECOND
  },
  utils::{get_now_timestamp, get_random},
};
use anchor_spl::token::{Token};
use anchor_spl::associated_token::{AssociatedToken, create, Create};

#[derive(Accounts)]
pub struct Claim<'info> {
  // claimer
  #[account(mut)]
  claimer: Signer<'info>,
  // vault
  #[account(
    mut,
    has_one = ctzns_pool_account,
    has_one = aliens_pool_account,
    has_one = gods_pool_account,
    has_one = reward_mint,
    constraint = vault.status == VaultStatus::Initialized
  )]
  vault: Account<'info, Vault>,
  // reward pda account
  /// CHECK:
  #[account(
    mut,
    seeds = [VAULT_CTZN_REWARD_SEED.as_bytes(), vault.to_account_info().key.as_ref()],
    bump = vault.ctzns_pool_bump
  )]
  ctzns_pool: AccountInfo<'info>,
  /// CHECK:
  #[account(
    mut,
    seeds = [VAULT_ALIEN_REWARD_SEED.as_bytes(), vault.to_account_info().key.as_ref()],
    bump = vault.aliens_pool_bump
  )]
  aliens_pool: AccountInfo<'info>,
  /// CHECK:
  #[account(
    mut,
    seeds = [VAULT_GOD_REWARD_SEED.as_bytes(), vault.to_account_info().key.as_ref()],
    bump = vault.gods_pool_bump
  )]
  gods_pool: AccountInfo<'info>,
  // reward mint
  /// CHECK:
  reward_mint: AccountInfo<'info>,
  // vault ctzns reward associated token account
  /// CHECK:
  #[account(mut)]
  ctzns_pool_account: AccountInfo<'info>,
  // vault ctzns reward associated token account
  /// CHECK:
  #[account(mut)]
  aliens_pool_account: AccountInfo<'info>,
  // vault ctzns reward associated token account
  /// CHECK:
  #[account(mut)]
  gods_pool_account: AccountInfo<'info>,
  // claimer reward account
  /// CHECK:
  #[account(mut)]
  claimer_account: AccountInfo<'info>,
  //user
  #[account(
    mut,
    constraint = user.vault == *vault.to_account_info().key,
    constraint = user.key == *claimer.key
  )]
  user: Account<'info, User>,
  // associated token program 
  #[account(address = anchor_spl::associated_token::ID)]
  associated_token_program: Program<'info, AssociatedToken>,
  // rent
  rent: Sysvar<'info, Rent>,
  // token program
  #[account(address = spl_token::id())]
  token_program: Program<'info, Token>,
  // system program
  system_program: Program<'info, System>,
}

pub fn claim_ctzn(ctx: Context<Claim>) -> Result<()> {
  let vault = &mut ctx.accounts.vault;
  if vault.status != VaultStatus::Initialized {
    return Err(ErrorCode::VaultNotInitialized.into());
  }
  
  let user = &mut ctx.accounts.user;
  let now = get_now_timestamp();
  let mut ctzns_reward: u64 = 0;
  let mut aliens_reward: u64 = 0;
  for item in &mut user.items {
    let time_diff = std::cmp::max(
      now.checked_sub(item.last_claimed_time).unwrap(), 0 as u64
    );
    // let accured_day = time_diff.checked_div(ONE_DAY_TO_SECOND).unwrap();
    item.earned_reward = time_diff.checked_mul(1250).unwrap().checked_div(3).unwrap();

    let time_diff = std::cmp::max(
      now.checked_sub(item.first_staked_time).unwrap(), 0 as u64
    );
    let staked_day = time_diff.checked_div(ONE_DAY_TO_SECOND).unwrap();
    let mut risk_bound: u64 = 240;
    risk_bound = risk_bound
      .checked_sub(
        std::cmp::min(staked_day, 15 as u64)
          .checked_mul(10).unwrap()
      ).unwrap();
    
    let rand = get_random().checked_rem(300).unwrap();
    
    if u64::from(rand) < risk_bound {
      aliens_reward = aliens_reward.checked_add(item.earned_reward).unwrap();
      item.earned_reward = 0;
    } else {
      aliens_reward = aliens_reward.checked_add(
        item.earned_reward.checked_div(5).unwrap()
      ).unwrap();
      item.earned_reward = item.earned_reward
        .checked_div(5).unwrap()
        .checked_mul(4).unwrap();
    }
    ctzns_reward = ctzns_reward.checked_add(item.earned_reward).unwrap();
    item.last_claimed_time = now;
  }
  let mut burned = aliens_reward.checked_div(4).unwrap();
  aliens_reward = aliens_reward
    .checked_div(4).unwrap()
    .checked_mul(3).unwrap();

  let total = vault.normal_aliens_count.checked_mul(5).unwrap()
    .checked_add(
      vault.alpha_aliens_count.checked_mul(6).unwrap()
    ).unwrap();

  for item in &mut vault.aliens {
    if item.item_type == ItemType::NormalAlien {
      item.earned_reward = item.earned_reward.checked_add(
        aliens_reward.checked_mul(5).unwrap()
          .checked_div(total.into()).unwrap()
      ).unwrap();
    } else {
      item.earned_reward = item.earned_reward.checked_add(
        aliens_reward.checked_mul(6).unwrap()
          .checked_div(total.into()).unwrap()
      ).unwrap();
    }
  }

  if vault.ctzns_pool_amount <= ctzns_reward {
    ctzns_reward = vault.ctzns_pool_amount;
  }
  if vault.ctzns_pool_amount <= ctzns_reward + aliens_reward {
    aliens_reward = 0;
  }
  if vault.ctzns_pool_amount <= ctzns_reward + aliens_reward + burned {
    burned = 0;
  }

  if ctx.accounts.claimer_account.owner == &System::id() {
    let cpi_context = Create {
      payer: ctx.accounts.claimer.to_account_info(),
      associated_token: ctx.accounts.claimer_account.to_account_info(),
      authority: ctx.accounts.claimer.to_account_info(),
      mint: ctx.accounts.reward_mint.clone(),
      rent: ctx.accounts.rent.to_account_info(),
      token_program: ctx.accounts.token_program.to_account_info(),
      system_program: ctx.accounts.system_program.to_account_info(),
    };
    let create_tx = CpiContext::new(
      ctx.accounts.associated_token_program.to_account_info(),
      cpi_context,
    );
    create(create_tx).unwrap();
  }
  let vault_address = vault.key().clone();
  let ctzns_seeds = [
    VAULT_CTZN_REWARD_SEED.as_bytes(),
    vault_address.as_ref(),
    &[vault.ctzns_pool_bump],
  ];

  if ctzns_reward > 0 {
    let cpi_context = CpiContext::new(
      ctx.accounts.token_program.to_account_info().clone(),
      anchor_spl::token::Transfer {
        from: ctx.accounts.ctzns_pool_account.to_account_info().clone(),
        to: ctx.accounts.claimer_account.to_account_info().clone(),
        authority: ctx.accounts.ctzns_pool.to_account_info().clone(),
      },
    );
    anchor_spl::token::transfer(cpi_context.with_signer(&[&ctzns_seeds[..]]), ctzns_reward)?;
  }

  if aliens_reward > 0 {
    let cpi_context = CpiContext::new(
      ctx.accounts.token_program.to_account_info().clone(),
      anchor_spl::token::Transfer {
        from: ctx.accounts.ctzns_pool_account.to_account_info().clone(),
        to: ctx.accounts.aliens_pool_account.to_account_info().clone(),
        authority: ctx.accounts.ctzns_pool.to_account_info().clone(),
      },
    );
    anchor_spl::token::transfer(cpi_context.with_signer(&[&ctzns_seeds[..]]), aliens_reward)?;
  }

  if burned > 0 {
    let cpi_context = CpiContext::new(
      ctx.accounts.token_program.to_account_info().clone(),
      anchor_spl::token::Transfer {
        from: ctx.accounts.ctzns_pool_account.to_account_info().clone(),
        to: ctx.accounts.gods_pool_account.to_account_info().clone(),
        authority: ctx.accounts.ctzns_pool.to_account_info().clone(),
      },
    );
    anchor_spl::token::transfer(cpi_context.with_signer(&[&ctzns_seeds[..]]), burned)?;
  }

  vault.ctzns_pool_amount = vault.ctzns_pool_amount
    .checked_sub(ctzns_reward).unwrap()
    .checked_sub(aliens_reward).unwrap()
    .checked_sub(burned).unwrap();
  
  vault.aliens_pool_amount = vault.aliens_pool_amount
    .checked_add(aliens_reward).unwrap();
  
  vault.gods_pool_amount = vault.gods_pool_amount
    .checked_add(burned).unwrap();

  Ok(())
}

pub fn claim_alien(ctx: Context<Claim>) -> Result<()> {
  let vault = &mut ctx.accounts.vault;
  if vault.status != VaultStatus::Initialized {
    return Err(ErrorCode::VaultNotInitialized.into());
  }

  let user = &mut ctx.accounts.user;
  let now = get_now_timestamp();
  let mut aliens_reward: u64 = 0;
  for item in &mut user.items {
    let index = vault.aliens.iter().position(|x| x.mint_account == item.mint_account);
    if let Some(index) = index {
      let alien_item = &mut vault.aliens[index];
      aliens_reward = aliens_reward.checked_add(alien_item.earned_reward).unwrap();
      alien_item.earned_reward = 0;
      alien_item.last_claimed_time = now;
    }
  }

  let vault_address = vault.key().clone();
  let aliens_seeds = [
    VAULT_ALIEN_REWARD_SEED.as_bytes(),
    vault_address.as_ref(),
    &[vault.aliens_pool_bump],
  ];
  let cpi_context = CpiContext::new(
    ctx.accounts.token_program.to_account_info().clone(),
    anchor_spl::token::Transfer {
      from: ctx.accounts.aliens_pool_account.to_account_info().clone(),
      to: ctx.accounts.claimer_account.to_account_info().clone(),
      authority: ctx.accounts.aliens_pool.to_account_info().clone(),
    },
  );
  anchor_spl::token::transfer(cpi_context.with_signer(&[&aliens_seeds[..]]), aliens_reward)?;

  vault.aliens_pool_amount = vault.aliens_pool_amount.checked_sub(aliens_reward).unwrap();
  Ok(())
}