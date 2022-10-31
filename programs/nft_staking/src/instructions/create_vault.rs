use anchor_lang::prelude::*;
use anchor_spl::token::Token;
use anchor_spl::associated_token::{create, AssociatedToken, Create};
use crate::constant::{VAULT_CTZN_REWARD_SEED, VAULT_ALIEN_REWARD_SEED, VAULT_GOD_REWARD_SEED};
use crate::state::{ErrorCode, Vault, VaultStatus};

#[derive(Accounts)]
#[instruction(ctzns_pool_bump: u8, aliens_pool_bump: u8, gods_pool_bump: u8)]
pub struct CreateVault<'info> {
  // the vault athority
  #[account(mut)]
  authority: Signer<'info>,

  // vault account to be created
  #[account(init, payer = authority, space = Vault::LEN + 8)]
  vault: Account<'info, Vault>,

  // reward token mint
  /// CHECK:
  reward_mint: AccountInfo<'info>,

  // ctzns pool pda account
  #[account(seeds = [VAULT_CTZN_REWARD_SEED.as_bytes(), vault.key().as_ref()], bump = ctzns_pool_bump)]
  ctzns_pool: SystemAccount<'info>,

  // aliens pool pda account
  #[account(seeds = [VAULT_ALIEN_REWARD_SEED.as_bytes(), vault.key().as_ref()], bump = aliens_pool_bump)]
  aliens_pool: SystemAccount<'info>,

  // gods pool pda account
  #[account(seeds = [VAULT_GOD_REWARD_SEED.as_bytes(), vault.key().as_ref()], bump = gods_pool_bump)]
  gods_pool: SystemAccount<'info>,

  // ctzns pool account to be created, owned by vault
  /// CHECK:
  #[account(mut)]
  ctzns_pool_account: UncheckedAccount<'info>,

  // aliens pool account to be created, owned by vault
  /// CHECK:
  #[account(mut)]
  aliens_pool_account: UncheckedAccount<'info>,

  // ctzn pool account to be created, owned by vault
  /// CHECK:
  #[account(mut)]
  gods_pool_account: UncheckedAccount<'info>,

  rent: Sysvar<'info, Rent>,

  #[account(address = anchor_spl::associated_token::ID)]
  associated_token: Program<'info, AssociatedToken>,

  #[account(address = spl_token::id())]
  token_program: Program<'info, Token>,

  system_program: Program<'info, System>,
}

pub fn create_vault(
  ctx: Context<CreateVault>,
    ctzns_pool_bump: u8,
    aliens_pool_bump: u8,
    gods_pool_bump: u8
) -> Result<()> {

  // set vault
  let vault = &mut ctx.accounts.vault;

  // check vault status
  if vault.status != VaultStatus::None {
    return Err(ErrorCode::VaultAlreadyCreated.into());
  }

  // create ctzns pool token account
  if ctx.accounts.ctzns_pool.owner == &System::id() {
    let cpi_context = Create {
      payer: ctx.accounts.authority.to_account_info(),
      associated_token: ctx.accounts.ctzns_pool_account.to_account_info(),
      authority: ctx.accounts.ctzns_pool.to_account_info(),
      mint: ctx.accounts.reward_mint.to_account_info(),
      rent: ctx.accounts.rent.to_account_info(),
      token_program: ctx.accounts.token_program.to_account_info(),
      system_program: ctx.accounts.system_program.to_account_info(),
    };
    let create_ctx =
      CpiContext::new(ctx.accounts.associated_token.to_account_info(), cpi_context);
    create(create_ctx)?;
  }

   // create aliens pool token account
   if ctx.accounts.aliens_pool.owner == &System::id() {
    let cpi_context = Create {
      payer: ctx.accounts.authority.to_account_info(),
      associated_token: ctx.accounts.aliens_pool_account.to_account_info(),
      authority: ctx.accounts.aliens_pool.to_account_info(),
      mint: ctx.accounts.reward_mint.to_account_info(),
      rent: ctx.accounts.rent.to_account_info(),
      token_program: ctx.accounts.token_program.to_account_info(),
      system_program: ctx.accounts.system_program.to_account_info(),
    };
    let create_ctx =
      CpiContext::new(ctx.accounts.associated_token.to_account_info(), cpi_context);
    create(create_ctx)?;
  }

   // create gods pool token account
   if ctx.accounts.gods_pool.owner == &System::id() {
    let cpi_context = Create {
      payer: ctx.accounts.authority.to_account_info(),
      associated_token: ctx.accounts.gods_pool_account.to_account_info(),
      authority: ctx.accounts.gods_pool.to_account_info(),
      mint: ctx.accounts.reward_mint.to_account_info(),
      rent: ctx.accounts.rent.to_account_info(),
      token_program: ctx.accounts.token_program.to_account_info(),
      system_program: ctx.accounts.system_program.to_account_info(),
    };
    let create_ctx =
      CpiContext::new(ctx.accounts.associated_token.to_account_info(), cpi_context);
    create(create_ctx)?;
  }
  vault.status = VaultStatus::Initialized;
  vault.authority = *ctx.accounts.authority.key;
  vault.reward_mint = *ctx.accounts.reward_mint.to_account_info().key;
  vault.ctzns_pool_account = ctx.accounts.ctzns_pool_account.key();
  vault.aliens_pool_account = ctx.accounts.aliens_pool_account.key();
  vault.gods_pool_account = ctx.accounts.gods_pool_account.key();
  vault.ctzns_pool_bump = ctzns_pool_bump;
  vault.aliens_pool_bump = aliens_pool_bump;
  vault.gods_pool_bump = gods_pool_bump;
  vault.ctzns_pool_amount = 0;
  vault.aliens_pool_amount = 0;
  vault.gods_pool_amount = 0;
  vault.alpha_aliens_count = 0;
  vault.normal_aliens_count = 0;
  vault.aliens = vec![];
  Ok(())
}
