use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount},
};

use crate::state::{LiquidityPool, LiquidityPoolAccount};

pub fn add_liquidity(ctx: Context<AddLiquidity>) -> Result<()> {
    let pool = &mut ctx.accounts.pool;

    let token_accounts = (
        &mut *ctx.accounts.token_mint,
        &mut *ctx.accounts.pool_token_account,
        &mut *ctx.accounts.user_token_account,
        &mut *ctx.accounts.exchange_token_mint,
        &mut *ctx.accounts.pool_exchange_token_account,
        &mut *ctx.accounts.user_exchange_token_account,
    );

    pool.add_liquidity(
        token_accounts,
        &ctx.accounts.user,
        &ctx.accounts.token_program,
    )?;
    Ok(())
}

#[derive(Accounts)]
pub struct AddLiquidity<'info> {
    #[account(
        mut,
        seeds = [
            LiquidityPool::POOL_SEED_PREFIX.as_bytes(), 
            token_mint.key().as_ref(),
            exchange_token_mint.key().as_ref()
        ],
        bump
    )]
    pub pool: Account<'info, LiquidityPool>,

    #[account(mut)]
    pub token_mint: Box<Account<'info, Mint>>,

    #[account(mut)]
    pub exchange_token_mint: Box<Account<'info, Mint>>,

    #[account(
        mut,
        associated_token::mint = token_mint,
        associated_token::authority = pool
    )]
    pub pool_token_account: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        associated_token::mint = exchange_token_mint,
        associated_token::authority = pool
    )]
    pub pool_exchange_token_account: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        associated_token::mint = token_mint,
        associated_token::authority = user,
    )]
    pub user_token_account: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        associated_token::mint = exchange_token_mint,
        associated_token::authority = user,
    )]
    pub user_exchange_token_account: Box<Account<'info, TokenAccount>>,

    #[account(mut)]
    pub user: Signer<'info>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}