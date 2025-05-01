pub mod sell_channel {
    use std::sync::{mpsc, MutexGuard};
    use std::time::{Duration, SystemTime};

    use crate::helpers::helpers::helpers::sleep_thread;
    use crate::trading212::helpers::helpers::{
        cancel_order, create_limit_order, get_all_orders_request, get_movement_direction,
        get_perc_decrease, get_perc_increase, get_sell_quant, TradeDirection,
    };
    use crate::trading212::types::types::{
        HistoricalOrder, LimitOrder, Position, SystemLimitOrder, MOVEMENT_DIRECTION,
    };
    use crate::{ChannelParam, LIMIT_WAIT_TIME};

    fn cancel_existing_sell_orders() {
        println!("ST: Cancelling all existing limit orders...");
        let orders_data: Option<Vec<LimitOrder>> = get_all_orders_request();
        let orders: Vec<LimitOrder> = match orders_data {
            Some(orders) => orders,
            None => {
                println!("ST: Failed to retreive order data... Skipping cycle.");
                Vec::new()
            }
        };

        if orders.is_empty() {
            println!("ST: Order list empty.");
            return;
        }

        for order in orders {
            if order.quantity > 0.0 {
                // Buy order, not a sell order. Not to be cancelled
                continue;
            }

            println!("\nST: Cancelling {} order...", order.ticker);
            let res: Option<bool> = cancel_order(order.id);
            match res {
                Some(_) => {
                    println!("ST: Cancel successful.");
                }
                None => {
                    println!("ST: Cancel Failed. Retrying...");
                    sleep_thread(5);
                    let res_two: Option<bool> = cancel_order(order.id);
                    match res_two {
                        None => {
                            println!("ST: Failed second cancel attempt. Skipping.");
                        }
                        Some(_) => {
                            println!("ST: Cancel successful.");
                        }
                    }
                }
            }
            sleep_thread(10);
        }
    }

    pub fn start_selling(
        ctrl_to_sell_rx: mpsc::Receiver<bool>,
        sell_to_ctrl_tx: mpsc::Sender<bool>,
        sell_to_buy_tx: mpsc::Sender<bool>,
        buy_to_sell_rx: mpsc::Receiver<bool>,
        data: ChannelParam,
    ) {
        let mut system_limit_orders: Vec<SystemLimitOrder> = Vec::new();
        let mut sale_results: Vec<HistoricalOrder> = Vec::new(); // TODO

        loop {
            // Wait for start signal from Control thread
            _ = ctrl_to_sell_rx.recv().unwrap();
            println!("ST: Received start signal from Control.");

            sleep_thread(20);
            cancel_existing_sell_orders();

            // Create initial sell limit order for each position
            let positions: MutexGuard<Vec<Position>> = data.arc_positions_value.lock().unwrap();
            'initial_limit_creation_loop: for position in &*positions {
                let movement_direction: MOVEMENT_DIRECTION = get_movement_direction(&position);
                let sale_price: f64 = get_perc_increase(position.current_price, 1.0);
                let sell_quant: f64 = get_sell_quant(&position);

                println!("ST: Creating limit order for {}", position.ticker);

                match create_limit_order(
                    position.ticker.clone(),
                    sale_price,
                    -sell_quant,
                    TradeDirection::SELL,
                ) {
                    None => {
                        println!("ST: Failed to create initial sell order. Skipping...\n");
                    }
                    Some(limit_order) => {
                        let system_limit_order: SystemLimitOrder = SystemLimitOrder {
                            creation_time: SystemTime::now(),
                            movement_direction,
                            sell_attempts: 0,
                            limit_order,
                        };

                        println!(
                            "ST: Created system limit order for {}.\n",
                            &system_limit_order.limit_order.ticker
                        );
                        system_limit_orders.push(system_limit_order);
                    }
                }
                sleep_thread(10);
            }
            drop(positions);

            // Wait three minutes before cancelling and updating sell orders
            sleep_thread(180);
            cancel_existing_sell_orders();
            println!("\nST: All existing orders cancelled.");

            // wait for buys to complete
            println!("ST: Sending trigger to buy thread.\n");
            sell_to_buy_tx.send(true).unwrap();
            _ = buy_to_sell_rx.recv().unwrap();
            println!("ST: Received response from buy thread.\nST: Creating fresh sell orders.");
            sleep_thread(180);

            // Check if limit orders have hit. Change asking amount - 6 checks
            'order_loop: for i in 0..6 {
                // Create a new set of limit orders for each existing position
                let positions: MutexGuard<Vec<Position>> = data.arc_positions_value.lock().unwrap();

                // DEBUG
                println!("\nST: Existing limit orders to update:");
                for o in &system_limit_orders {
                    for position in &*positions {
                        if position.ticker.eq_ignore_ascii_case(&o.limit_order.ticker) {
                            println!(
                                "{}: Current Price: {} - Limit Price: {}",
                                o.limit_order.ticker,
                                position.current_price,
                                o.limit_order.limit_price.unwrap()
                            );
                        }
                    }
                }
                println!(" ");

                cancel_existing_sell_orders();
                sleep_thread(30);

                'inner_order_loop: for system_limit_order in &mut system_limit_orders {
                    let positions: Vec<&Position> = (*positions
                        .iter()
                        .filter(|position| {
                            position
                                .ticker
                                .eq_ignore_ascii_case(&system_limit_order.limit_order.ticker)
                        })
                        .collect::<Vec<&Position>>())
                    .to_vec();

                    let position: &Position = match positions.len() {
                        1 => positions.first().unwrap(),
                        _ => continue 'inner_order_loop,
                    };

                    // if !is_valid_perc_change(&position) {
                    //     println!("S: Sell conditions for {} not met.\n", position.ticker);
                    //     continue 'initial_limit_loop;
                    // }

                    match get_movement_direction(position) {
                        MOVEMENT_DIRECTION::DOWN => {
                            // For downward trending tickers - priority = minimise losses
                            let quantity: f64 = position.quantity.clone();
                            let ticker: String = position.ticker.clone();

                            match system_limit_order.sell_attempts {
                                0 | 1 | 2 => {
                                    println!("\nST: Downward Trending. Creating emergency Break Even limit order for {}...", position.ticker);
                                    // 3 attempts to sell at break even
                                    create_limit_order(
                                        ticker,
                                        position.average_price,
                                        -quantity,
                                        TradeDirection::SELL,
                                    );
                                }
                                3 | 4 => {
                                    println!("\nST: Downward Trending. Creating -0.01% emergency limit order for {}...", position.ticker);
                                    // 3 attempts to sell 0.01% below current price
                                    create_limit_order(
                                        ticker,
                                        get_perc_decrease(position.current_price, 0.01),
                                        -quantity,
                                        TradeDirection::SELL,
                                    );
                                }
                                _ => {
                                    println!("\nST: Downward Trending. Creating -0.05% emergency limit order for {}...", position.ticker);
                                    // Sell 0.05% below current price
                                    create_limit_order(
                                        ticker,
                                        get_perc_decrease(position.current_price, 0.05),
                                        -quantity,
                                        TradeDirection::SELL,
                                    );
                                }
                            };
                        }
                        MOVEMENT_DIRECTION::UP => {
                            // For upward trending tickers - priority = maximise gains
                            let quantity: f64 = position.quantity.clone();
                            let ticker: String = position.ticker.clone();

                            match system_limit_order.sell_attempts {
                                0 | 1 => {
                                    // Sell at .5% above
                                    println!("\nST: Creating .5% limit order...");

                                    create_limit_order(
                                        ticker,
                                        get_perc_increase(position.current_price, 0.5),
                                        -quantity,
                                        TradeDirection::SELL,
                                    );
                                }
                                2 | 3 => {
                                    // Sell at .25% above
                                    println!("\nST: Creating .25% limit order...");

                                    create_limit_order(
                                        ticker,
                                        get_perc_increase(position.current_price, 0.25),
                                        -quantity,
                                        TradeDirection::SELL,
                                    );
                                }
                                4 | 5 => {
                                    // Sell at .1% above
                                    println!("\nST: Creating .1% limit order...");

                                    create_limit_order(
                                        ticker,
                                        get_perc_increase(position.current_price, 0.1),
                                        -quantity,
                                        TradeDirection::SELL,
                                    );
                                }
                                _ => {
                                    // Sell at current price
                                    println!("\nST: Creating limit sell for current price...");

                                    create_limit_order(
                                        ticker,
                                        get_perc_increase(position.current_price, 0.0),
                                        -quantity,
                                        TradeDirection::SELL,
                                    );
                                }
                            }
                        }
                    }

                    system_limit_order.sell_attempts += 1;
                    sleep_thread(20);
                }
                println!("\nST: Order Loop iteration {} complete. Sleeping...\n", i);
                // sleep_thread(300);
            }

            println!("ST: New sell orders complete. Signalling Control.");
            sell_to_ctrl_tx.send(true).unwrap();

            // TODO
            // // Cycle through closed orders since cycle start time and populate sale result vector
            // let historical_orders_data: Option<Vec<HistoricalOrder>> = get_all_historical_orders();

            // if historical_orders_data.is_none() {
            //     println!("S: Historical orders returned a NONE value.");
            // } else {
            //     let historical_orders: Vec<HistoricalOrder> = historical_orders_data.unwrap();

            //     // Filter so its only filled sales
            //     let historical_sells: Vec<&HistoricalOrder> = historical_orders
            //         .iter()
            //         .filter(|order| {
            //             order.status.eq_ignore_ascii_case("FILLED")
            //                 && order.filled_quantity < 0.0
            //                 && order.ordered_quantity < 0.0
            //             // && is_after_start_time(start_time, &order.date_modified)
            //             //     .is_some_and(|res| res == true)
            //         })
            //         .collect();

            //     for historical_sell in historical_sells {
            //         sale_results.push(historical_sell.clone())
            //     }
            // }

            // // sale_results
        }
    }
}
