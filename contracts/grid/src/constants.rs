use near_sdk::{Balance, Gas};

/// Attach no deposit.
pub const NO_DEPOSIT: Balance = 0;
pub const ONE_YOCTO: Balance = 1;

pub const GAS_FOR_FT_TRANSFER: Gas = Gas(Gas::ONE_TERA.0 * 10);
pub const GAS_FOR_AFTER_FT_TRANSFER: Gas = Gas(Gas::ONE_TERA.0 * 20);

/// slippage denominator
pub const SLIPPAGE_DENOMINATOR: u16 = 10000;

/// grid rate denominator
pub const GRID_RATE_DENOMINATOR: u16 = 10000;

pub const DEFAULT_PROTOCOL_FEE: u128 = 10000;
pub const MAX_PROTOCOL_FEE: u128 = 100000;
/// protocol fee denominator
pub const PROTOCOL_FEE_DENOMINATOR: u128 = 1000000;

pub const STORAGE_FEE: u128 = 10_000_000_000_000_000_000_000; // 0.01Near

/// self.order_map[bot_id][FORWARD_ORDERS_INDEX]
pub const FORWARD_ORDERS_INDEX: u64 = 0;
/// self.order_map[bot_id][REVERSE_ORDERS_INDEX]
pub const REVERSE_ORDERS_INDEX: u64 = 1;
/// Forward and Reverse
pub const ORDER_POSITION_SIZE: u64 = 2;

