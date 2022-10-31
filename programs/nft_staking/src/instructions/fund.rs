use anchor_lang::prelude::*;
use crate::{
  state::{ Vault, VaultStatus },
};
use anchor_spl::token::TokenAccount;

#[derive(Accounts)]
#[instruction(amount: u64)]
pub struct Fund<'info> {
  // funder
  /// CHECK:
  #[account(signer)]
  funder: AccountInfo<'info>,

  // vault 
  #[account(
    mut,
    constraint = vault.status == VaultStatus::Initialized,
    constraint = vault.ctzns_pool_account == * ctzns_pool_account.to_account_info().key
  )]
  vault: Account<'info, Vault>,

  // reward account
  #[account(mut)]
  ctzns_pool_account: Account<'info, TokenAccount>,

  // funder account
  #[account(
    mut,
    constraint = funder_account.amount >= amount
  )]
  funder_account: Account<'info, TokenAccount>,

  // token program
  /// CHECK:
  #[account(address = spl_token::id())]
  token_program: AccountInfo<'info>,
}

pub fn fund(ctx: Context<Fund>, amount: u64) -> Result<()> {
  let vault = &mut ctx.accounts.vault;
  vault.ctzns_pool_amount = vault.ctzns_pool_amount.checked_add(amount).unwrap();
  // transfer token
  let cpi_context = CpiContext::new(
    ctx.accounts.token_program.to_account_info(),
    anchor_spl::token::Transfer {
      from: ctx.accounts.funder_account.to_account_info(),
      to: ctx.accounts.ctzns_pool_account.to_account_info(),
      authority: ctx.accounts.funder.to_account_info(),
    },
  );

  anchor_spl::token::transfer(cpi_context, amount)?;
  Ok(())
}