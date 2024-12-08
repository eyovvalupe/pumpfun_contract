use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount},
};

use crate::state::{CurveConfiguration, LiquidityPool, LiquidityPoolAccount};

pub fn buy(ctx: Context<Buy>, amount: u64) -> Result<()> {
    let pool = &mut ctx.accounts.pool;

    let token_accounts = (
        &mut *ctx.accounts.output_token_mint,
        &mut *ctx.accounts.pool_output_token_account,
        &mut *ctx.accounts.user_output_token_account,
        &mut *ctx.accounts.input_token_mint,
        &mut *ctx.accounts.pool_input_token_account,
        &mut *ctx.accounts.user_input_token_account,
    );

    pool.buy(
        token_accounts,
        amount,
        &ctx.accounts.user,
        &ctx.accounts.token_program,
    )?;
    Ok(())
}

#[derive(Accounts)]
pub struct Buy<'info> {
    #[account(
        mut,
        seeds = [CurveConfiguration::SEED.as_bytes()],
        bump,
    )]
    pub dex_configuration_account: Box<Account<'info, CurveConfiguration>>,

    #[account(
        mut,
        seeds = [LiquidityPool::POOL_SEED_PREFIX.as_bytes(), output_token_mint.key().as_ref(), input_token_mint.key().as_ref()],
        bump = pool.bump
    )]
    pub pool: Box<Account<'info, LiquidityPool>>,

    #[account(mut)]
    pub output_token_mint: Box<Account<'info, Mint>>,

    #[account(
        mut,
        associated_token::mint = output_token_mint,
        associated_token::authority = pool
    )]
    pub pool_output_token_account: Box<Account<'info, TokenAccount>>,

    /// CHECK:
    #[account(
        init_if_needed,
        payer = user,
        associated_token::mint = output_token_mint,
        associated_token::authority = user,
    )]
    pub user_output_token_account: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        associated_token::mint = input_token_mint,
        associated_token::authority = pool
    )]
    pub pool_input_token_account: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        associated_token::mint = input_token_mint,
        associated_token::authority = user
    )]
    pub user_input_token_account: Box<Account<'info, TokenAccount>>,

    #[account(mut)]
    pub input_token_mint: Box<Account<'info, Mint>>,

    #[account(mut)]
    pub user: Signer<'info>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}
