use anchor_lang::prelude::*;
use crate::state::{Vault, VaultStatus, User, UserType};
use crate::constant::{VAULT_ALIEN_USER_SEED, VAULT_CTZN_USER_SEED};

#[derive(Accounts)]
#[instruction(user_type: u8)]
pub struct CreateUser<'info> {
  // authority
  #[account(mut)]
  authority: Signer<'info>,

  // vault
  #[account(
    mut,
    constraint = vault.status == VaultStatus::Initialized
  )]
  vault: Account<'info, Vault>,

  // user 
  #[account(
    init,
    payer = authority,
    space = User::LEN + 8,
    seeds = [
      match user_type { 
        0 => VAULT_CTZN_USER_SEED, 
        _ => VAULT_ALIEN_USER_SEED
      }.as_bytes(),
      vault.key().as_ref(), 
      authority.key.as_ref()
    ], 
    bump
  )]
  user: Account<'info, User>,

  system_program: Program<'info, System>,
}

pub fn create_user(ctx: Context<CreateUser>, user_type: u8) -> Result<()> {
  let user = &mut ctx.accounts.user;
  user.vault = *ctx.accounts.vault.to_account_info().key;
  user.key = *ctx.accounts.authority.key;
  user.user_type = match user_type {
    0 => UserType::Ctzn,
    _ => UserType::Alien,
  };
  user.items_count = 0;
  user.items = vec![];

  Ok(())
}
