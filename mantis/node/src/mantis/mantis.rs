use cosmrs::tendermint::block::Height;
use cw_mantis_order::{OrderItem, OrderSolution, OrderSubMsg};

use crate::{
    prelude::*,
    solver::{orderbook::OrderList, solution::Solution, types::OrderType},
};

use super::cosmos::client::timeout;

pub type SolutionsPerPair = Vec<(Vec<OrderSolution>, (u64, u64))>;

pub fn do_cows(all_orders: Vec<OrderItem>) -> SolutionsPerPair {
    let all_orders = all_orders.into_iter().group_by(|x| {
        let mut ab = [x.given.denom.clone(), x.msg.wants.denom.clone()];
        ab.sort();
        (ab[0].clone(), ab[1].clone())
    });
    let mut cows_per_pair = vec![];
    for ((a, b), orders) in all_orders.into_iter() {
        let orders = orders.collect::<Vec<_>>();
        use crate::solver::solver::*;
        use crate::solver::types::*;
        let orders = orders.iter().map(|x| {
            let side = if x.given.denom == a {
                OrderType::Buy
            } else {
                OrderType::Sell
            };

            crate::solver::types::Order::new_integer(
                x.given.amount.u128(),
                x.msg.wants.amount.u128(),
                side,
                x.order_id,
            )
        });
        let orders = OrderList {
            value: orders.collect(),
        };
        orders.print();
        let optimal_price = orders.compute_optimal_price(1000);
        println!("optimal_price: {:?}", optimal_price);
        let mut solution = Solution::new(orders.value.clone());
        solution = solution.match_orders(optimal_price);
        solution.print();
        let cows = solution
            .orders
            .value
            .into_iter()
            .filter(|x| x.amount_out > <_>::default())
            .map(|x| {
                let filled = x.amount_out.to_u128().expect("u128");
                OrderSolution {
                    order_id: x.id,
                    cow_amount: filled.into(),
                    cross_chain: 0u128.into(),
                }
            })
            .collect::<Vec<_>>();
        println!("optimal price {:?}", optimal_price);
        let optimal_price = decimal_to_fraction(optimal_price.0);
        println!("cows: {:?}", cows);
        if !cows.is_empty() {
            cows_per_pair.push((cows, optimal_price));
        }
    }
    cows_per_pair
}

/// convert decimal to normalized fraction
fn decimal_to_fraction(amount: Decimal) -> (u64, u64) {
    let decimal_string = amount.to_string();
    let decimal_string: Vec<_> = decimal_string.split('.').collect();
    if decimal_string.len() == 1 {
        (decimal_string[0].parse().expect("in range"), 1)
    } else {
        let digits_after_decimal = decimal_string[1].len() as u32;
        let denominator = 10_u128.pow(digits_after_decimal) as u64;
        let numerator = (amount * Decimal::from(denominator))
            .to_u64()
            .expect("integer");
        let fraction = fraction::Fraction::new(numerator, denominator);

        (
            *fraction.numer().expect("num"),
            *fraction.denom().expect("denom"),
        )
    }
}

pub fn randomize_order(
    coins_pair: String,
    tip: Height,
) -> (cw_mantis_order::ExecMsg, cosmrs::Coin) {
    let coins: Vec<_> = coins_pair
        .split(',')
        .map(|x| cosmwasm_std::Coin::from_str(x).expect("coin"))
        .collect();

    let coins = if rand::random::<bool>() {
        (coins[0].clone(), coins[1].clone())
    } else {
        (coins[1].clone(), coins[0].clone())
    };
    let coin_0_random = randomize_coin(coins.0.amount.u128());
    let coin_1_random = randomize_coin(coins.1.amount.u128());

    let msg = cw_mantis_order::ExecMsg::Order {
        msg: OrderSubMsg {
            wants: cosmwasm_std::Coin {
                amount: coin_0_random.into(),
                denom: coins.0.denom.clone(),
            },
            transfer: None,
            timeout: timeout(tip, 100),
            min_fill: None,
        },
    };
    let fund = cosmrs::Coin {
        amount: coin_1_random.into(),
        denom: cosmrs::Denom::from_str(&coins.1.denom).expect("denom"),
    };
    (msg, fund)
}

fn randomize_coin(coin_0_amount: u128) -> u128 {
    let delta_0 = 1.max(coin_0_amount / 10);
    let coin_0_random = rand_distr::Uniform::new(coin_0_amount - delta_0, coin_0_amount + delta_0);
    let coin_0_random: u128 = coin_0_random.sample(&mut rand::thread_rng());
    coin_0_random
}
