pub const INITIAL_PRICE_DIVIDER: u64 = 800_000;       // lamports per one token (without decimal)
pub const INITIAL_LAMPORTS_FOR_POOL: u64 = 10_000_000;   // 0.01SOL
pub const TOKEN_SELL_LIMIT_PERCENT: u64 = 8000;     //  80%
pub const PROPORTION: u64 = 1280;      //  800M token is sold on 500SOL ===> (500 * 2 / 800) = 1.25 ===> 800 : 1.25 = 640 ====> 640 * 2 = 1280
pub const TOTAL_SUPPLY: u64 = 1000000000 * 10u64.pow(TOKEN_DECIMAL as u32);
pub const TOKEN_DECIMAL: u8 = 6;
