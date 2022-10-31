use anchor_lang::prelude::*;
use crate::state::{Vault, VaultStatus, ErrorCode, User, UserType, ItemType};
use anchor_spl::token::{TokenAccount, Token};
use crate::constant::{VAULT_STAKE_SEED, ONE_DAY_TO_SECOND};
use spl_token::instruction::AuthorityType::AccountOwner;
use crate::utils::{get_now_timestamp};

#[derive(Accounts)]
#[instruction(vault_stake_bump: u8)]
pub struct Unstake<'info> {
  // authority
  /// CHECK:
  #[account(mut)]
  staker: AccountInfo<'info>,
  // vault
  #[account(mut)]
  vault: Account<'info, Vault>,
  //unstake mint account
  #[account(
    mut,
    constraint = user.items.iter().any(|x| x.mint_account == *unstake_account.to_account_info().key),
    constraint = unstake_account.amount > 0,
  )]
  unstake_account: Account<'info, TokenAccount>,
  /// CHECK:
  // vault pda
  #[account(
    mut,
    seeds = [
      VAULT_STAKE_SEED.as_bytes(), 
      vault.key().as_ref(), 
      staker.key().as_ref(),
      unstake_account.key().as_ref(),
    ],
    bump = vault_stake_bump,
  )]
  vault_pda: AccountInfo<'info>,
  // user account
  #[account(
    mut,
    constraint = user.vault == *vault.to_account_info().key,
  )]
  user: Account<'info, User>,
  // token program
  #[account(address = spl_token::id())]
  token_program: Program<'info, Token>,
  system_program: Program<'info, System>,
}

pub fn unstake(ctx: Context<Unstake>, vault_stake_bump: u8, manually: bool) -> Result<()> {
  let vault = &mut ctx.accounts.vault;
  if vault.status != VaultStatus::Initialized {
    return Err(ErrorCode::VaultNotInitialized.into());
  }

  // update
  let user = &mut ctx.accounts.user;
  let unstake_account = &mut ctx.accounts.unstake_account;

  if !user.items.iter().any(|x| x.mint_account == unstake_account.key()) {
    return Err(ErrorCode::StakedAccountDoesNotExist.into());
  }

  user.items_count = user.items_count.checked_sub(1).unwrap();
  
  let index = user.items.iter().position(|x| x.mint_account == unstake_account.key());

  if let Some(existed) = index {
    let item = &user.items[existed];
    if manually == false && user.user_type == UserType::Alien && 
      get_now_timestamp() < (item.last_claimed_time + 2 * ONE_DAY_TO_SECOND) {
      
      return Err(ErrorCode::CannotUnstakeAlien.into());
    }
    
    if item.item_type == ItemType::AlphaAlien {
      vault.alpha_aliens_count = vault.alpha_aliens_count.checked_sub(1).unwrap();
    } 
    if item.item_type == ItemType::NormalAlien {
      vault.normal_aliens_count = vault.normal_aliens_count.checked_sub(1).unwrap();
    }

    if item.item_type == ItemType::AlphaAlien || item.item_type == ItemType::NormalAlien {
      let index = vault.aliens.iter().position(|x| x.mint_account == unstake_account.key());
      if let Some(index) = index {
        vault.aliens.remove(index);
      }
    }

    user.items.remove(existed);
  }

  // transfer token authority
  let vault_address = ctx.accounts.vault.key().clone();
  let staker_address = ctx.accounts.staker.key().clone();
  let unstake_account_address = unstake_account.key().clone();
  // let (_vault_pda, vault_bump) = Pubkey::find_program_address(
  //   &[
  //     VAULT_STAKE_SEED.as_bytes(),
  //     vault_address.as_ref(), 
  //     staker_address.as_ref()
  //   ],
  //   ctx.program_id,
  // );

  let seeds = &[
    VAULT_STAKE_SEED.as_bytes(),
    vault_address.as_ref(),
    staker_address.as_ref(),
    unstake_account_address.as_ref(),
    &[vault_stake_bump],
  ]; // need this to sign the pda, match the authority

  let cpi_context = CpiContext::new(
    ctx.accounts.token_program.to_account_info(),
    anchor_spl::token::SetAuthority {
      current_authority: ctx.accounts.vault_pda.to_account_info().clone(),
      account_or_mint: unstake_account.to_account_info().clone(),
    },
  );

  anchor_spl::token::set_authority(
    cpi_context.with_signer(&[&seeds[..]]),
    AccountOwner,
    Some(ctx.accounts.staker.key()), 
  )?;
  Ok(())
}