use std::ops::{Add, Div, Mul, Sub};
use crate::*;
use near_sdk::{env, require};
use crate::{GridBotContract, SLIPPAGE_DENOMINATOR};
use crate::big_decimal::BigDecimal;
use crate::entity::GridType;
use crate::entity::GridType::EqOffset;

impl GridBotContract {

    pub fn internal_get_and_use_next_bot_id(&mut self) -> u128 {
        let next_id = self.next_bot_id.clone();

        require!(self.next_bot_id.checked_add(1) != None, INVALID_NEXT_BOT_ID);

        self.next_bot_id += 1;

        return next_id;
    }

    pub fn internal_init_bot_status(&self, bot: &mut GridBot, entry_price: U128C) {
        if bot.trigger_price == U128C::from(0) {
            bot.active = true;
            return;
        }
        if entry_price >= bot.trigger_price {
            bot.trigger_price_above_or_below = false;
        } else {
            bot.trigger_price_above_or_below = true;
        }
    }

    pub fn internal_check_oracle_price(&self, entry_price: U128C, pair_id: String, slippage: u16) -> bool {
        if !self.oracle_price_map.contains_key(&pair_id) {
            return false;
        }
        let price_info = self.oracle_price_map.get(&pair_id).unwrap();
        if price_info.valid_timestamp < env::block_timestamp() {
            // oracle price expired
            return false;
        }

        let recorded_price = price_info.price;
        if entry_price >= recorded_price {
            return (entry_price - recorded_price) / entry_price * SLIPPAGE_DENOMINATOR <= U128C::from(slippage);
        } else {
            return (recorded_price - entry_price) / entry_price * SLIPPAGE_DENOMINATOR <= U128C::from(slippage);
        }
    }

    pub fn internal_check_bot_close_permission(&self, user: &AccountId, bot: &GridBot) -> bool {
        if user == &(bot.user) {
            return true;
        }
        let oracle_price = self.internal_get_oracle_price(bot.pair_id.clone());
        if oracle_price >= bot.take_profit_price {
            return true;
        }
        if oracle_price <= bot.stop_loss_price {
            return true;
        }
        return false;
    }

    pub fn internal_get_first_forward_order(grid_bot: GridBot, pair: Pair, level: usize) -> Order {
        let mut order = Order{
            order_id: level.to_string(),
            token_sell: pair.base_token.clone(),
            token_buy: pair.quote_token.clone(),
            amount_sell: U128C::from(0),
            amount_buy: U128C::from(0),
            fill_buy_or_sell: false,
            filled: U128C::from(0),
        };
        let grid_rate_denominator_128 = U128C::from(GRID_RATE_DENOMINATOR);
        if grid_bot.grid_buy_count > (level.clone() as u16) {
            // buy grid
            order.token_sell = pair.quote_token.clone();
            order.token_buy = pair.base_token.clone();
            order.fill_buy_or_sell = grid_bot.fill_base_or_quote.clone();
            if grid_bot.fill_base_or_quote {
                // fixed base
                order.amount_buy = grid_bot.first_base_amount.clone();
                order.amount_sell = if grid_bot.grid_type == EqOffset {
                    // arithmetic grid
                    grid_bot.first_quote_amount.clone() + grid_bot.grid_offset * U128C::from(level.clone() as u16)
                } else {
                    // proportional grid
                    grid_bot.first_quote_amount.clone() * (grid_rate_denominator_128 + U128C::from(grid_bot.grid_rate)).pow(U128C::from(level.clone() as u16)) / grid_rate_denominator_128.pow(U128C::from(level.clone() as u16))
                };
            } else {
                // fixed quote
                order.amount_sell = grid_bot.first_quote_amount.clone();
                order.amount_buy = if grid_bot.grid_type == EqOffset {
                    // arithmetic grid
                    grid_bot.first_base_amount.clone() - grid_bot.grid_offset * U128C::from(level.clone() as u16)
                } else {
                    // proportional grid
                    grid_bot.first_base_amount.clone() * (grid_rate_denominator_128 - U128C::from(grid_bot.grid_rate)).pow(U128C::from(level.clone() as u16)) / grid_rate_denominator_128.pow(U128C::from(level.clone() as u16))
                };
            }
        } else {
            // sell grid
            order.token_sell = pair.base_token.clone();
            order.token_buy = pair.quote_token.clone();
            order.fill_buy_or_sell = !grid_bot.fill_base_or_quote.clone();
            let coefficient = U128C::from(grid_bot.grid_buy_count.clone() + grid_bot.grid_sell_count.clone() - 1 - level.clone() as u16);
            if grid_bot.fill_base_or_quote {
                // fixed base
                order.amount_sell = grid_bot.last_base_amount.clone();
                order.amount_buy = if grid_bot.grid_type == EqOffset {
                    grid_bot.last_quote_amount.clone() - grid_bot.grid_offset * coefficient.clone()
                } else {
                    grid_bot.last_quote_amount.clone() * (grid_rate_denominator_128 - U128C::from(grid_bot.grid_rate)).pow(coefficient.clone()) / grid_rate_denominator_128.pow(coefficient.clone())
                };
            } else {
                // fixed quote
                order.amount_buy = grid_bot.last_quote_amount.clone();
                order.amount_sell = if grid_bot.grid_type == EqOffset {
                    grid_bot.last_base_amount.clone() + grid_bot.grid_offset * coefficient.clone()
                } else {
                    grid_bot.last_base_amount.clone() * (grid_rate_denominator_128 + U128C::from(grid_bot.grid_rate)).pow(coefficient.clone()) / grid_rate_denominator_128.pow(coefficient.clone())
                };
            }
        }
        return order;
    }

    pub fn internal_calculate_bot_assets(first_quote_amount: U128C, last_base_amount: U128C, grid_sell_count: u16, grid_buy_count: u16,
                                         grid_type: GridType, grid_rate: u16, grid_offset: U128C, fill_base_or_quote: bool) -> (U128C, U128C) {
        // calculate quote
        let grid_buy_count_u128 = U128C::from(grid_buy_count);
        let quote_amount_buy = if grid_buy_count == 0 {
            U128C::from(0)
        } else if fill_base_or_quote {
            if grid_type == EqOffset {
                first_quote_amount * grid_buy_count_u128.clone() + grid_offset * (grid_buy_count_u128.clone() - U128C::from(1)) * grid_buy_count_u128.clone() / U128C::from(2)
            } else {
                let geometric_series_sum = GridBotContract::private_calculate_rate_bot_geometric_series_sum(grid_buy_count.clone() as u64, grid_rate.clone() as u64);
                U128C::from(BigDecimal::from(first_quote_amount.clone().as_u128()).mul(geometric_series_sum).div(BigDecimal::from(GRID_RATE_DENOMINATOR as u64)).round_down_u128())
            }
        } else {
            first_quote_amount * grid_buy_count_u128.clone()
        };

        // calculate base
        let grid_sell_count_u128 = U128C::from(grid_sell_count);
        let base_amount_sell = if grid_sell_count == 0 {
            U128C::from(0)
        } else if fill_base_or_quote {
            last_base_amount * grid_sell_count_u128.clone()
        } else {
            if grid_type == EqOffset {
                last_base_amount * grid_sell_count_u128.clone() + grid_offset * (grid_sell_count_u128.clone() - U128C::from(1)) * grid_sell_count_u128.clone() / U128C::from(2)
            } else {
                let geometric_series_sum = GridBotContract::private_calculate_rate_bot_geometric_series_sum(grid_sell_count.clone() as u64, grid_rate.clone() as u64);
                U128C::from(BigDecimal::from(last_base_amount.clone().as_u128()).mul(geometric_series_sum).div(BigDecimal::from(GRID_RATE_DENOMINATOR as u64)).round_down_u128())
            }
        };
        return (base_amount_sell, quote_amount_buy);
    }

    fn private_calculate_rate_bot_geometric_series_sum(n: u64, delta_r: u64) -> BigDecimal {
        let scale = BigDecimal::from(GRID_RATE_DENOMINATOR as u64);
        let a = scale;   // 1.0 * scale
        let r = BigDecimal::from(delta_r).add(BigDecimal::from(GRID_RATE_DENOMINATOR as u64));
        let sum = a.mul(scale.sub(r.pow(n))).div(scale.sub(r));
        return sum;
    }
}
