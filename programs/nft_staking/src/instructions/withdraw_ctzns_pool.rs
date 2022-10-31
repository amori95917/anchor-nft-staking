use anchor_lang::prelude::*;
use crate::{
  state::{Vault, VaultStatus},
  constant::{VAULT_CTZN_REWARD_SEED}
};
use anchor_spl::token::{Token};
use anchor_spl::associated_token::{AssociatedToken, create, Create};

#[derive(Accounts)]
pub struct WithdrawCtznsPool<'info> {
  // claimer
  #[account(mut)]
  pub claimer: Signer<'info>,
  // vault
  #[account(
    mut,
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
  // vault ctzns reward associated token account
  /// CHECK:
  #[account(mut)]
  ctzns_pool_account: AccountInfo<'info>,
  // reward mint
  /// CHECK:
  reward_mint: AccountInfo<'info>,
  /// CHECK:
  #[account(mut)]
  claimer_account: AccountInfo<'info>,
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

pub fn withdraw_ctzns_pool(ctx: Context<WithdrawCtznsPool>, amount: u64) -> Result<()> {
  let vault = &mut ctx.accounts.vault;
  let vault_address = vault.key().clone();

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

  let ctzns_seeds = [
    VAULT_CTZN_REWARD_SEED.as_bytes(),
    vault_address.as_ref(),
    &[vault.ctzns_pool_bump],
  ];

  let cpi_context = CpiContext::new(
    ctx.accounts.token_program.to_account_info().clone(),
    anchor_spl::token::Transfer {
      from: ctx.accounts.ctzns_pool_account.to_account_info().clone(),
      to: ctx.accounts.claimer_account.to_account_info().clone(),
      authority: ctx.accounts.ctzns_pool.to_account_info().clone(),
    },
  );
  anchor_spl::token::transfer(cpi_context.with_signer(&[&ctzns_seeds[..]]), amount)?;

  Ok(())
}