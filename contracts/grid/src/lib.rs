use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
#[allow(unused_imports)]
use near_sdk::{env, near_bindgen, AccountId, Balance, PanicOnDefault, PromiseOrValue};
use near_sdk::collections::{LookupMap, Vector};

mod utils;
mod constants;
mod errors;
mod entity;
mod grid_bot;
mod orderbook;
mod grid_bot_internal;
mod token;
mod orderbook_internal;
mod grid_bot_views;
mod orderbook_views;
mod big_decimal;
mod events;
mod grid_bot_private;
mod grid_bot_get_set;
mod grid_bot_asset;
mod owner;

pub use crate::constants::*;
pub use crate::errors::*;
pub use crate::utils::*;
pub use crate::entity::*;

// near_sdk::setup_alloc!();
// near_sdk::wee_alloc!();

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct GridBotContract {
    pub owner_id: AccountId,
    pub status: GridStatus,
    /// real_protocol_fee = protocol_fee / 1000000
    pub protocol_fee_rate: u128,
    /// bot_map[bot_id] = bot
    /// bot_id = GRID:index
    pub bot_map: LookupMap<String, GridBot>,
    /// order_map[bot_id][0][0] = first forward order; order_map[bot_id][1][0] = first reverse order;
    pub order_map: LookupMap<String, Vector<Vector<Order>>>,
    /// start from 0, used from 1
    pub next_bot_id: u128,
    /// oracle_price_map[pair_id] = OraclePrice
    pub oracle_price_map: LookupMap<String, OraclePrice>,
    /// pair_map[base_token_addr+":"+quote_token_addr] = Pair
    pub pair_map: LookupMap<String, Pair>,
    pub protocol_fee_map: LookupMap<AccountId, U128C>,
    pub global_balances_map: LookupMap<AccountId, U128C>,
    pub user_balances_map: LookupMap<AccountId, LookupMap<AccountId, U128C>>,
    pub user_locked_balances_map: LookupMap<AccountId, LookupMap<AccountId, U128C>>,
}

#[near_bindgen]
impl GridBotContract {
    #[init]
    pub fn new(owner_id: AccountId) -> Self {
        GridBotContract {
            owner_id: owner_id.clone(),
            status: GridStatus::Running,
            // 1%
            protocol_fee_rate: DEFAULT_PROTOCOL_FEE,
            bot_map: LookupMap::new(b"bots".to_vec()),
            order_map: LookupMap::new(b"orders".to_vec()),
            next_bot_id: 0,
            oracle_price_map: LookupMap::new(b"oracle".to_vec()),
            pair_map: LookupMap::new(b"pairs".to_vec()),
            protocol_fee_map: LookupMap::new(b"protocol".to_vec()),
            global_balances_map: LookupMap::new(b"global".to_vec()),
            user_balances_map: LookupMap::new(StorageKey::UserBalanceMainKey),
            user_locked_balances_map: LookupMap::new(StorageKey::UserLockedBalanceMainKey),
        }
    }
}
