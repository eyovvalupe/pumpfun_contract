use anchor_lang::prelude::*;

pub mod errors;
pub mod utils;
pub mod instructions;
pub mod state;
pub mod consts;

use crate::instructions::*;

declare_id!("5EKkfU7mvHHeD6ABq2Ep2ztMKHUfaSfyqmwRDNswuM2T");

#[program]
pub mod pump {
    use super::*;

    pub fn initialize(ctx: Context<InitializeCurveConfiguration>, fee: f64) -> Result<()> {
        instructions::initialize(ctx, fee)
    }

    pub fn create_pool(ctx: Context<CreateLiquidityPool>) -> Result<()> {
        instructions::create_pool(ctx)
    }

    pub fn add_liquidity(
        ctx: Context<AddLiquidity>,
    ) -> Result<()> {
        instructions::add_liquidity(ctx)
    }

    pub fn remove_liquidity(ctx: Context<RemoveLiquidity>, bump: u8) -> Result<()> {
        instructions::remove_liquidity(ctx, bump)
    }

    pub fn buy(ctx: Context<Buy>, amount: u64) -> Result<()> {
        instructions::buy(ctx, amount)
    }

    pub fn sell(ctx: Context<Sell>, amount: u64, bump: u8) -> Result<()> {
        instructions::sell(ctx, amount, bump)
    }

    pub fn raydium_initialize(ctx: Context<RaydiumInitialize>, init_amount_0: u64, init_amount_1: u64, open_time: u64) -> Result<()> {
        instructions::raydium_initialize(ctx, init_amount_0, init_amount_1, open_time)
    }
    
}

