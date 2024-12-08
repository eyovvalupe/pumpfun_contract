use crate::consts::INITIAL_LAMPORTS_FOR_POOL;
use crate::consts::INITIAL_PRICE_DIVIDER;
use crate::consts::PROPORTION;
use crate::errors::CustomError;
use anchor_lang::prelude::*;
use anchor_lang::system_program;
use anchor_spl::token::{self, Mint, Token, TokenAccount};

#[account]
pub struct CurveConfiguration {
    pub fees: f64,
}

impl CurveConfiguration {
    pub const SEED: &'static str = "CurveConfiguration";

    // Discriminator (8) + f64 (8)
    pub const ACCOUNT_SIZE: usize = 8 + 32 + 8;

    pub fn new(fees: f64) -> Self {
        Self { fees }
    }
}

#[account]
pub struct LiquidityProvider {
    pub shares: u64, // The number of shares this provider holds in the liquidity pool ( didnt add to contract now )
}

impl LiquidityProvider {
    pub const SEED_PREFIX: &'static str = "LiqudityProvider"; // Prefix for generating PDAs

    // Discriminator (8) + f64 (8)
    pub const ACCOUNT_SIZE: usize = 8 + 8;
}

#[account]
pub struct LiquidityPool {
    pub creator: Pubkey,
    pub token: Pubkey,          // Regular token mint
    pub exchange_token: Pubkey, // Exchange token mint (replacing SOL)
    pub total_supply: u64,
    pub reserve_token: u64,     // Reserve amount of regular token
    pub reserve_exchange: u64,  // Reserve amount of exchange token (replacing reserve_exchange)
    pub bump: u8,
}

impl LiquidityPool {
    pub const POOL_SEED_PREFIX: &'static str = "liquidity_pool";

    // Discriminator (8) + Pubkey (32) + Pubkey (32) + totalsupply (8)
    // + reserve one (8) + reserve two (8) + Bump (1)
    pub const ACCOUNT_SIZE: usize = 8 + 32 + 32 + 8 + 8 + 8 + 1;

    // Constructor to initialize a LiquidityPool with two tokens and a bump for the PDA
    pub fn new(creator: Pubkey, token: Pubkey, exchange_token: Pubkey, bump: u8) -> Self {
        Self {
            creator,
            token,
            exchange_token,
            total_supply: 0_u64,
            reserve_token: 0_u64,
            reserve_exchange: 0_u64,
            bump,
        }
    }
}

pub trait LiquidityPoolAccount<'info> {
    // Updates the token reserves in the liquidity pool
    fn update_reserves(&mut self, reserve_token: u64, reserve_exchange: u64) -> Result<()>;

    // Allows adding liquidity by depositing an amount of two tokens and getting back pool shares
    fn add_liquidity(
        &mut self,
        token_accounts: (
            &mut Account<'info, Mint>,    // token_mint
            &mut Account<'info, TokenAccount>, // pool_token_account
            &mut Account<'info, TokenAccount>, // user_token_account
            &mut Account<'info, Mint>,    // exchange_token_mint
            &mut Account<'info, TokenAccount>, // pool_exchange_token_account
            &mut Account<'info, TokenAccount>, // user_exchange_token_account
        ),
        authority: &Signer<'info>,
        token_program: &Program<'info, Token>,
    ) -> Result<()>;

    // Allows removing liquidity by burning pool shares and receiving back a proportionate amount of tokens
    fn remove_liquidity(
        &mut self,
        token_accounts: (
            &mut Account<'info, Mint>,
            &mut Account<'info, TokenAccount>,
            &mut Account<'info, TokenAccount>,
            &mut Account<'info, Mint>,
            &mut Account<'info, TokenAccount>,
            &mut Account<'info, TokenAccount>,
        ),
        authority: &Signer<'info>,
        bump: u8,
        token_program: &Program<'info, Token>,
    ) -> Result<()>;

    fn buy(
        &mut self,
        // bonding_configuration_account: &Account<'info, CurveConfiguration>,
        token_accounts: (
            &mut Account<'info, Mint>,
            &mut Account<'info, TokenAccount>,
            &mut Account<'info, TokenAccount>,
            &mut Account<'info, Mint>,
            &mut Account<'info, TokenAccount>,
            &mut Account<'info, TokenAccount>,
        ),
        amount: u64,
        authority: &Signer<'info>,
        token_program: &Program<'info, Token>,
    ) -> Result<()>;

    fn sell(
        &mut self,
        // bonding_configuration_account: &Account<'info, CurveConfiguration>,
        token_accounts: (
            &mut Account<'info, Mint>,
            &mut Account<'info, TokenAccount>,
            &mut Account<'info, TokenAccount>,
            &mut Account<'info, Mint>,
            &mut Account<'info, TokenAccount>,
            &mut Account<'info, TokenAccount>,
        ),
        amount: u64,
        authority: &Signer<'info>,
        bump: u8,
        token_program: &Program<'info, Token>,
    ) -> Result<()>;

    fn transfer_token_from_pool(
        &self,
        from: &Account<'info, TokenAccount>,
        to: &Account<'info, TokenAccount>,
        amount: u64,
        token_program: &Program<'info, Token>,
    ) -> Result<()>;

    fn transfer_token_to_pool(
        &self,
        from: &Account<'info, TokenAccount>,
        to: &Account<'info, TokenAccount>,
        amount: u64,
        authority: &Signer<'info>,
        token_program: &Program<'info, Token>,
    ) -> Result<()>;
}

impl<'info> LiquidityPoolAccount<'info> for Account<'info, LiquidityPool> {
    fn update_reserves(&mut self, reserve_token: u64, reserve_exchange: u64) -> Result<()> {
        self.reserve_token = reserve_token;
        self.reserve_exchange = reserve_exchange;
        Ok(())
    }

    fn add_liquidity(
        &mut self,
        token_accounts: (
            &mut Account<'info, Mint>,    // token_mint
            &mut Account<'info, TokenAccount>, // pool_token_account
            &mut Account<'info, TokenAccount>, // user_token_account
            &mut Account<'info, Mint>,    // exchange_token_mint
            &mut Account<'info, TokenAccount>, // pool_exchange_token_account
            &mut Account<'info, TokenAccount>, // user_exchange_token_account
        ),
        authority: &Signer<'info>,
        token_program: &Program<'info, Token>,
    ) -> Result<()> {
        // Transfer regular token to pool
        self.transfer_token_to_pool(
            token_accounts.2,  // user_token_account
            token_accounts.1,  // pool_token_account
            token_accounts.0.supply,  // amount of regular token
            authority,
            token_program,
        )?;

        // Transfer exchange token to pool
        self.transfer_token_to_pool(
            token_accounts.5,  // user_exchange_token_account
            token_accounts.4,  // pool_exchange_token_account
            token_accounts.3.supply,  // amount of exchange token
            authority,
            token_program,
        )?;

        // Update pool state
        self.total_supply = 1_000_000_000_000_000_000;
        self.update_reserves(token_accounts.0.supply, token_accounts.3.supply)?;

        Ok(())
    }

    fn remove_liquidity(
        &mut self,
        token_accounts: (
            &mut Account<'info, Mint>,
            &mut Account<'info, TokenAccount>,
            &mut Account<'info, TokenAccount>,
            &mut Account<'info, Mint>,
            &mut Account<'info, TokenAccount>,
            &mut Account<'info, TokenAccount>,
        ),
        authority: &Signer<'info>,
        _bump: u8,
        token_program: &Program<'info, Token>,
    ) -> Result<()> {
        // Transfer all regular tokens from pool to user
        self.transfer_token_from_pool(
            token_accounts.1,  // pool_token_account
            token_accounts.2,  // user_token_account
            token_accounts.1.amount,
            token_program,
        )?;

        // Transfer all exchange tokens from pool to user
        self.transfer_token_from_pool(
            token_accounts.4,  // pool_exchange_token_account
            token_accounts.5,  // user_exchange_token_account
            token_accounts.4.amount,
            token_program,
        )?;

        // Update pool state
        self.update_reserves(0, 0)?;
        self.total_supply = 0;

        Ok(())
    }

    fn buy(
        &mut self,
        token_accounts: (
            &mut Account<'info, Mint>,          // Project token mint
            &mut Account<'info, TokenAccount>,  // Project token pool account
            &mut Account<'info, TokenAccount>,  // User's project token account
            &mut Account<'info, Mint>,          // Project token mint
            &mut Account<'info, TokenAccount>,  // Exchange token pool account
            &mut Account<'info, TokenAccount>,  // User's exchange token account
        ),
        amount: u64,
        authority: &Signer<'info>,
        token_program: &Program<'info, Token>,
    ) -> Result<()> {
        if amount == 0 {
            return err!(CustomError::InvalidAmount);
        }
    
        msg!("Trying to buy from the pool");
    
        let bought_amount = (self.total_supply as f64 - self.reserve_token as f64) / 1_000_000.0 / 1_000_000_000.0;
        msg!("bought_amount {}", bought_amount);
    
        let root_val = (PROPORTION as f64 * amount as f64 / 1_000_000_000.0 + bought_amount * bought_amount).sqrt();
        let amount_out_f64 = (root_val - bought_amount as f64) * 1_000_000.0 * 1_000_000_000.0;
        let amount_out = amount_out_f64.round() as u64;
    
        if amount_out > self.reserve_token {
            return err!(CustomError::NotEnoughTokenInVault);
        }
    
        // Transfer exchange tokens from user to pool
        self.transfer_token_to_pool(
            token_accounts.5,
            token_accounts.4,
            amount,
            authority,
            token_program,
        )?;
    
        // Transfer project tokens from pool to user
        self.transfer_token_from_pool(
            token_accounts.1,
            token_accounts.2,
            amount_out,
            token_program,
        )?;
    
        self.reserve_exchange += amount;      // This becomes exchange token reserve
        self.reserve_token -= amount_out;
    
        Ok(())
    }
    
    fn sell(
        &mut self,
        token_accounts: (
            &mut Account<'info, Mint>,          // Project token mint
            &mut Account<'info, TokenAccount>,  // Project token pool account
            &mut Account<'info, TokenAccount>,  // User's project token account
            &mut Account<'info, Mint>,          // Project token mint
            &mut Account<'info, TokenAccount>,  // Exchange token pool account
            &mut Account<'info, TokenAccount>,  // User's exchange token account
        ),
        amount: u64,
        authority: &Signer<'info>,
        bump: u8,
        token_program: &Program<'info, Token>,
    ) -> Result<()> {
        if amount == 0 {
            return err!(CustomError::InvalidAmount);
        }
    
        if self.reserve_token < amount {
            return err!(CustomError::TokenAmountToSellTooBig);
        }
    
        let bought_amount = (self.total_supply as f64 - self.reserve_token as f64) / 1_000_000.0 / 1_000_000_000.0;
        let result_amount = (self.total_supply as f64 - self.reserve_token as f64 - amount as f64) / 1_000_000.0 / 1_000_000_000.0;
        let amount_out_f64 = (bought_amount * bought_amount - result_amount * result_amount) / PROPORTION as f64 * 1_000_000_000.0;
        let amount_out = amount_out_f64.round() as u64;
    
        if self.reserve_exchange < amount_out {  // This checks exchange token reserve
            return err!(CustomError::NotEnoughExchangeTokenInVault);
        }
    
        // Transfer project tokens from user to pool
        self.transfer_token_to_pool(
            token_accounts.2,
            token_accounts.1,
            amount,
            authority,
            token_program,
        )?;
    
        // Transfer exchange tokens from pool to user
        self.transfer_token_from_pool(
            token_accounts.4,
            token_accounts.5,
            amount_out,
            token_program,
        )?;
    
        self.reserve_token += amount;
        self.reserve_exchange -= amount_out;  // This becomes exchange token reserve
    
        Ok(())
    }

    fn transfer_token_from_pool(
        &self,
        from: &Account<'info, TokenAccount>,
        to: &Account<'info, TokenAccount>,
        amount: u64,
        token_program: &Program<'info, Token>,
    ) -> Result<()> {
        token::transfer(
            CpiContext::new_with_signer(
                token_program.to_account_info(),
                token::Transfer {
                    from: from.to_account_info(),
                    to: to.to_account_info(),
                    authority: self.to_account_info(),
                },
                &[&[
                    LiquidityPool::POOL_SEED_PREFIX.as_bytes(),
                    self.token.key().as_ref(),
                    self.exchange_token.key().as_ref(),
                    &[self.bump],
                ]],
            ),
            amount,
        )?;
        Ok(())
    }

    fn transfer_token_to_pool(
        &self,
        from: &Account<'info, TokenAccount>,
        to: &Account<'info, TokenAccount>,
        amount: u64,
        authority: &Signer<'info>,
        token_program: &Program<'info, Token>,
    ) -> Result<()> {
        token::transfer(
            CpiContext::new(
                token_program.to_account_info(),
                token::Transfer {
                    from: from.to_account_info(),
                    to: to.to_account_info(),
                    authority: authority.to_account_info(),
                },
            ),
            amount,
        )?;
        Ok(())
    }
}

fn calculate_amount_out(reserve_token_with_decimal: u64, amount_with_decimal: u64) -> Result<u64> {
    // Convert to f64 for decimal calculations
    let reserve_token = (reserve_token_with_decimal as f64) / 1_000_000_000.0;
    let amount = (amount_with_decimal as f64) / 1_000_000_000.0;

    msg!(
        "Starting calculation with reserve_token: {}, amount: {}",
        reserve_token,
        amount
    );

    let two_reserve_token = reserve_token * 2.0;
    msg!("two_reserve_token: {}", two_reserve_token);

    let one_added = two_reserve_token + 1.0;
    msg!("one_added: {}", one_added);

    let squared = one_added * one_added;
    msg!("squared: {}", squared);

    // Use `amount` directly as it's already a decimal in f64
    let amount_added = squared + amount * 8.0;
    msg!("amount_added: {}", amount_added);

    // Square root calculation
    let sqrt_result = amount_added.sqrt();
    msg!("sqrt_result: {}", sqrt_result);

    // Check if sqrt_result is valid
    if sqrt_result < 0.0 {
        msg!("Error: Negative sqrt_result");
        return err!(CustomError::NegativeNumber);
    }

    let subtract_one = sqrt_result - one_added;
    msg!("subtract_one: {}", subtract_one);

    let amount_out = subtract_one / 2.0;
    msg!("amount_out: {}", amount_out);

    // Convert the final result back to u64 with appropriate scaling
    let amount_out_decimal =
        (amount_out * 1_000_000_000.0 * INITIAL_PRICE_DIVIDER as f64).round() as u64;
    msg!("amount_out_decimal: {}", amount_out_decimal);

    Ok(amount_out_decimal)
}

///////////////////////////////////////////////////////////////
///////////////////////////////////////////////////////////////
//
//              Linear bonding curve swap
//
/////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////
//
//  Linear bonding curve : S = T * P ( here, p is constant that show initial price )
//  SOL amount => S
//  Token amount => T
//  Initial Price => P
//
//  SOL amount to buy Token a => S_a = ((T_a  + 1) * T_a / 2) * P
//  SOL amount to buy Token b => S_b = ((T_b + 1) * T_b / 2) * P
//
//  If amount a of token sold, and x (x = b - a) amount of token is bought (b > a)
//  S = S_a - S_b = ((T_b + T_a + 1) * (T_b - T_a) / 2) * P
//
//
// let s = amount;
// let T_a = reserve_token - amount;
// let T_b = reserve_token;
// let P = INITIAL_PRICE_DIVIDER;

// let amount_inc = self
//     .reserve_token
//     .checked_mul(2)
//     .ok_or(CustomError::OverflowOrUnderflowOccurred)?
//     .checked_add(amount)
//     .ok_or(CustomError::OverflowOrUnderflowOccurred)?
//     .checked_add(1)
//     .ok_or(CustomError::OverflowOrUnderflowOccurred)?;

// let multiplier = amount
//     .checked_div(2)
//     .ok_or(CustomError::OverflowOrUnderflowOccurred)?;

// msg!("multiplier : {}", 200);
// let amount_out = amount_inc
//     .checked_mul(multiplier)
//     .ok_or(CustomError::OverflowOrUnderflowOccurred)?
//     .checked_mul(INITIAL_PRICE_DIVIDER)
//     .ok_or(CustomError::OverflowOrUnderflowOccurred)?;

// let amount_in_float = convert_to_float(amount, token_accounts.0.decimals);

// // Convert the input amount to float with decimals considered
// let amount_float = convert_to_float(amount, token_accounts.0.decimals);

// Apply fees
// let adjusted_amount_in_float = amount_float
//     .div(100_f64)
//     .mul(100_f64.sub(bonding_configuration_account.fees));

// let adjusted_amount =
//     convert_from_float(adjusted_amount_in_float, token_accounts.0.decimals);

// Linear bonding curve calculations
// let p = 1 / INITIAL_PRICE_DIVIDER;
// let t_a = convert_to_float(self.reserve_token, token_accounts.0.decimals);
// let t_b = t_a + adjusted_amount_in_float;

// let s_a = ((t_a + 1.0) * t_a / 2.0) * p;
// let s_b = ((t_b + 1.0) * t_b / 2.0) * p;

// let s = s_b - s_a;

// let amount_out = convert_from_float(s, sol_token_accounts.0.decimals);

// let new_reserves_one = self
//     .reserve_token
//     .checked_add(amount)
//     .ok_or(CustomError::OverflowOrUnderflowOccurred)?;
// msg!("new_reserves_one : {}", );
// let new_reserves_two = self
//     .reserve_exchange
//     .checked_sub(amount_out)
//     .ok_or(CustomError::OverflowOrUnderflowOccurred)?;

// msg!("new_reserves_two : {}", );
// self.update_reserves(new_reserves_one, new_reserves_two)?;

// let adjusted_amount_in_float = convert_to_float(amount, token_accounts.0.decimals)
//     .div(100_f64)
//     .mul(100_f64.sub(bonding_configuration_account.fees));

// let adjusted_amount =
//     convert_from_float(adjusted_amount_in_float, token_accounts.0.decimals);

// let denominator_sum = self
//     .reserve_token
//     .checked_add(adjusted_amount)
//     .ok_or(CustomError::OverflowOrUnderflowOccurred)?;

// let numerator_mul = self
//     .reserve_exchange
//     .checked_mul(adjusted_amount)
//     .ok_or(CustomError::OverflowOrUnderflowOccurred)?;

// let amount_out = numerator_mul
//     .checked_div(denominator_sum)
//     .ok_or(CustomError::OverflowOrUnderflowOccurred)?;

// let new_reserves_one = self
//     .reserve_token
//     .checked_add(amount)
//     .ok_or(CustomError::OverflowOrUnderflowOccurred)?;
// let new_reserves_two = self
//     .reserve_exchange
//     .checked_sub(amount_out)
//     .ok_or(CustomError::OverflowOrUnderflowOccurred)?;

// self.update_reserves(new_reserves_one, new_reserves_two)?;
// let amount_out = amount.checked_div(2)

// self.transfer_token_to_pool(
//     token_accounts.2,
//     token_accounts.1,
//     1000 as u64,
//     authority,
//     token_program,
// )?;

// self.transfer_token_from_pool(
//     sol_token_accounts.1,
// sol_token_accounts.2,
//     1000 as u64,
//     token_program,
// )?;

// let amount_out: u64 = 1000000000000;
// let amount_out = ((((2 * self.reserve_token + 1) * (2 * self.reserve_token + 1) + amount) as f64).sqrt() as u64 - ( 2 * self.reserve_token + 1)) / 2;

// let token_sold = match self.total_supply.checked_sub(self.reserve_token) {
//     Some(value) if value == 0 => 1_000_000_000,
//     Some(value) => value,
//     None => return err!(CustomError::OverflowOrUnderflowOccurred),
// };

// msg!("token_sold: {}", token_sold);

// let amount_out: u64 = calculate_amount_out(token_sold, amount)?;
// msg!("amount_out: {}", amount_out);

// if self.reserve_token < amount_out {
//     return err!(CustomError::InvalidAmount);
// }
// self.reserve_exchange += amount;
// self.reserve_token -= amount_out;

// Function to perform the calculation with error handling

// fn calculate_amount_out(reserve_token_decimal: u64, amount_decimal: u64) -> Result<u64> {
//     let reserve_token = reserve_token_decimal.checked_div(1000000000).ok_or(CustomError::OverflowOrUnderflowOccurred)?;
//     let amount = amount_decimal.checked_div(1000000000).ok_or(CustomError::OverflowOrUnderflowOccurred)?;
//     msg!("Starting calculation with reserve_token: {}, amount: {}", reserve_token, amount);
//     let two_reserve_token = reserve_token.checked_mul(2).ok_or(CustomError::OverflowOrUnderflowOccurred)?;
//     msg!("two_reserve_token: {}", two_reserve_token);

//     let one_added = two_reserve_token.checked_add(1).ok_or(CustomError::OverflowOrUnderflowOccurred)?;
//     msg!("one_added: {}", one_added);

//     let squared = one_added.checked_mul(one_added).ok_or(CustomError::OverflowOrUnderflowOccurred)?;
//     msg!("squared: {}", squared);

//     let amount_divided = amount.checked_mul(INITIAL_PRICE_DIVIDER).ok_or(CustomError::OverflowOrUnderflowOccurred)?;
//     msg!("amount_divided: {}", amount_divided);

//     let amount_added = squared.checked_add(amount_divided).ok_or(CustomError::OverflowOrUnderflowOccurred)?;
//     msg!("amount_added: {}", amount_added);

//     // Convert to f64 for square root calculation
//     let sqrt_result = (amount_added as f64).sqrt();
//     msg!("sqrt_result: {}", sqrt_result);

//     // Check if sqrt_result can be converted back to u64 safely
//     if sqrt_result < 0.0 {
//         msg!("Error: Negative sqrt_result");
//         return err!(CustomError::NegativeNumber);
//     }

//     let sqrt_u64 = sqrt_result as u64;
//     msg!("sqrt_u64: {}", sqrt_u64);

//     let subtract_one = sqrt_u64.checked_sub(one_added).ok_or(CustomError::OverflowOrUnderflowOccurred)?;
//     msg!("subtract_one: {}", subtract_one);

//     let amount_out = subtract_one.checked_div(2).ok_or(CustomError::OverflowOrUnderflowOccurred)?;
//     msg!("amount_out: {}", amount_out);
//     let amount_out_decimal = amount_out.checked_mul(1000000000)
//     Ok(amount_out)
// }
