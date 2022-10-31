use anchor_lang::prelude::*;
use crate::state::{Vault, VaultStatus, User, ErrorCode, ItemType, StakeItem};
use anchor_spl::token::{Token, TokenAccount};
use crate::constant::{VAULT_STAKE_SEED};
use crate::utils::get_now_timestamp;
use spl_token::instruction::AuthorityType::AccountOwner;
// use metaplex_token_metadata::state::{Metadata};

#[derive(Accounts)]
pub struct Stake<'info> {
  // authority
  #[account(mut)]
  staker: Signer<'info>,
  // vault
  #[account(
    mut,
    constraint = vault.status == VaultStatus::Initialized,
  )]
  vault: Account<'info, Vault>,
  // stake account
  #[account(
    mut,
    constraint = !user.items.iter().any(|x| x.mint_account == *stake_account.to_account_info().key),
    constraint = stake_account.amount > 0
  )]
  stake_account: Account<'info, TokenAccount>,
  //stake mint
  /// CHECK:
  stake_mint: AccountInfo<'info>,
  //nft metadata account
  /// CHECK:
  //metadata_info: AccountInfo<'info>,
  // user
  #[account(
    mut,
    constraint = user.vault == *vault.to_account_info().key,
    constraint = user.key == *staker.key,
  )]
  user: Account<'info, User>,
  //token program
  #[account(address = spl_token::id())]
  token_program: Program<'info, Token>,
  // system program
  system_program: Program<'info, System>,

}

pub fn stake(ctx: Context<Stake>, item_type: u8) -> Result<()> {
  let vault = &mut ctx.accounts.vault;
  if vault.status != VaultStatus::Initialized {
    return Err(ErrorCode::VaultNotInitialized.into());
  }
  // let metadata = Metadata::from_account_info(&ctx.accounts.metadata_info.to_account_info())?;
  

  let user = &mut ctx.accounts.user;

  let stake_account = &mut ctx.accounts.stake_account;

  if user.items.iter().any(|x| x.mint_account == stake_account.key()) {
    return Err(ErrorCode::AlreadyStakedAccount.into());
  }
  
  user.items_count = user.items_count.checked_add(1).unwrap();
  let item_type = match item_type {
    0 => ItemType::NormalCTZN,
    1 => {
      vault.normal_aliens_count = vault.normal_aliens_count.checked_add(1).unwrap();
      ItemType::NormalAlien
    },
    2 => {
      vault.alpha_aliens_count = vault.alpha_aliens_count.checked_add(1).unwrap();
      ItemType::AlphaAlien
    },
    _ => ItemType::AlienGod,
  };

  let item = StakeItem {
    mint: ctx.accounts.stake_mint.key(),
    mint_account: stake_account.key(),
    item_type: item_type,
    first_staked_time: get_now_timestamp(),
    last_claimed_time: get_now_timestamp(),
    earned_reward: 0,
  };
  user.items.push(item.clone());

  if item.item_type == ItemType::NormalAlien || item.item_type == ItemType::AlphaAlien {
    vault.aliens.push(item);
  }
  
  // transfer token authority
  let (vault_pda, _vault_bump) = Pubkey::find_program_address(
    &[
      VAULT_STAKE_SEED.as_bytes(),
      vault.key().as_ref(),
      ctx.accounts.staker.key().as_ref(),
      stake_account.key().as_ref()
    ],
    ctx.program_id,
  );

  let cpi_context = CpiContext::new(
    ctx.accounts.token_program.to_account_info(),
    anchor_spl::token::SetAuthority {
      current_authority: ctx.accounts.staker.to_account_info().clone(),
      account_or_mint: stake_account.to_account_info().clone(),
    },
  );

  anchor_spl::token::set_authority(cpi_context, AccountOwner, Some(vault_pda))?;

  Ok(())
}