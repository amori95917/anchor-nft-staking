mod constant;
mod instructions;
mod state;
mod utils;

use anchor_lang::prelude::*;
use instructions::*;

declare_id!("HES9CZTGAyJvpyHaVEAVxjfSHNw1wY27eeMZJBefFKgk");

#[program]
pub mod nft_staking {
    use super::*;

    pub fn create_vault(
        ctx: Context<CreateVault>,
        ctzns_pool_bump: u8,
        aliens_pool_bump: u8,
        gods_pool_bump: u8,
    ) -> Result<()> {
        create_vault::create_vault(ctx, ctzns_pool_bump, aliens_pool_bump, gods_pool_bump)
    }

    pub fn create_user(ctx: Context<CreateUser>, user_type: u8) -> Result<()> {
        create_user::create_user(ctx, user_type)
    }

    pub fn fund(ctx: Context<Fund>, amount: u64) -> Result<()> {
        fund::fund(ctx, amount)
    }

    pub fn stake(ctx: Context<Stake>, item_type: u8) -> Result<()> {
        stake::stake(ctx, item_type)
    }

    pub fn unstake(ctx: Context<Unstake>, vault_stake_bump: u8, manually: bool) -> Result<()> {
        unstake::unstake(ctx, vault_stake_bump, manually)
    }

    // pub fn unstake_manually(ctx: Context<UnstakeManually>, vault_stake_bump: u8) -> Result<()> {
    //     unstake_manually::unstake_manually(ctx, vault_stake_bump)
    // }

    pub fn claim(ctx: Context<Claim>, user_type: u8) -> Result<()> {
        if user_type == 0 {
            claim::claim_ctzn(ctx)
        } else {
            claim::claim_alien(ctx)
        }
    }

    pub fn withdraw_ctzns_pool(ctx: Context<WithdrawCtznsPool>, amount: u64) -> Result<()> {
        withdraw_ctzns_pool::withdraw_ctzns_pool(ctx, amount)
    }

    pub fn withdraw_aliens_pool(ctx: Context<WithdrawAliensPool>, amount: u64) -> Result<()> {
        withdraw_aliens_pool::withdraw_aliens_pool(ctx, amount)
    }

    pub fn withdraw_gods_pool(ctx: Context<WithdrawGodsPool>, amount: u64) -> Result<()> {
        withdraw_gods_pool::withdraw_gods_pool(ctx, amount)
    }
}

