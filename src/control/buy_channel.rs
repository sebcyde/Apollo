pub mod buy_channel {
    use std::sync::{mpsc, MutexGuard};
    use std::time::{Duration, Instant};

    use crate::file_control::read::read::get_buy_list_from_file;
    use crate::file_control::write::write::write_buy_list_to_file;
    use crate::helpers::filters::filtering::stock_passes_filters;
    use crate::helpers::helpers::helpers::{get_full_company_info, sleep_thread};
    use crate::helpers::types::types::FullCompanyInfo;
    use crate::trading212::helpers::helpers::{
        create_limit_order, get_buy_quantity, get_perc_increase, TradeDirection,
    };
    use crate::trading212::types::types::{BalanceObject, Instrument, LimitOrder};
    use crate::{ChannelParam, MINIMUM_BUYS, SHOPPING_TIME};

    pub fn start_buying(
        ctrl_to_buy_rx: mpsc::Receiver<bool>,
        sell_to_buy_rx: mpsc::Receiver<bool>,
        buy_to_sell_tx: mpsc::Sender<bool>,
        data: ChannelParam,
    ) {
        loop {
            // Wait for start signal from Control thread - Executes every 5 minutes or so
            _ = ctrl_to_buy_rx.recv().unwrap();
            println!("BT: Received start signal from Control.");

            // Populate shopping list

            let buy_list_file_data: Option<Vec<FullCompanyInfo>> = get_buy_list_from_file();
            let buy_list_file_data_exists: bool = buy_list_file_data.as_ref().is_some();
            let mut buy_list: Vec<FullCompanyInfo> = Vec::new();
            let start: Instant = Instant::now();

            let instruments: MutexGuard<Vec<Instrument>> =
                data.arc_instruments_value.lock().unwrap();

            for instrument in &*instruments {
                if start.elapsed() >= Duration::from_secs(*SHOPPING_TIME) {
                    // If its been 5 minutes break
                    println!("BT: Buy cycle time limit elapsed.");
                    break;
                }

                let raw_full_company_info: Option<FullCompanyInfo> =
                    get_full_company_info(instrument.clone());

                let company_info: FullCompanyInfo = match raw_full_company_info {
                    Some(company_info) => company_info,
                    None => {
                        println!("BT: Company info collection failed. Skipping...\n");
                        sleep_thread(5);
                        continue;
                    }
                };

                if !stock_passes_filters(&company_info) {
                    sleep_thread(5);
                    continue;
                }

                if company_info.company_stock_quote.current_price == 0.0 {
                    println!("BT: Fetched a zero value price. Skipping...\n");
                    sleep_thread(5);
                    continue;
                }

                if !buy_list_file_data_exists
                    || buy_list_file_data.as_ref().is_some_and(|buy_list| {
                        !buy_list.iter().any(|list_item: &FullCompanyInfo| {
                            list_item
                                .instrument
                                .ticker
                                .eq_ignore_ascii_case(&instrument.ticker)
                        })
                    })
                {
                    println!("BT: Adding instrument to buy list...");
                    buy_list.push(company_info);
                }

                if buy_list.len() == *MINIMUM_BUYS {
                    println!("BT: Buy list complete.");
                    break;
                }

                println!("BT: Current buy list size: {}", buy_list.len());
            }

            if buy_list.len() == 0 {
                println!("BT: No buys executed this cycle.\nBT: Waiting for sell thread...");

                _ = sell_to_buy_rx.recv().unwrap();
                println!("BT: Received signal from Sell thread.");

                // Send sell thread trigger
                buy_to_sell_tx.send(true).unwrap();
                return;
            }

            // Update data on file
            write_buy_list_to_file(&buy_list);

            _ = sell_to_buy_rx.recv().unwrap();
            println!("BT: Received signal from Sell thread. Executing buys...\n");

            // Execute buys

            let balance: MutexGuard<BalanceObject> = data.arc_balance_value.lock().unwrap();

            let mut successful_buys: u32 = 0;
            for company in &buy_list {
                let buy_quantity: f64 = get_buy_quantity(&company, &balance);

                // Execute market order
                let new_tick: String = company.instrument.ticker.clone();
                println!("BT: Creating buy limit order for {}", new_tick);

                // buy .1% below current price - attempting to maximise margins for sale
                let stock_price: f64 = company.company_stock_quote.current_price;
                let buy_price: f64 = get_perc_increase(stock_price, 0.1);

                let order_result: Option<LimitOrder> = create_limit_order(
                    new_tick.clone(),
                    buy_price,
                    buy_quantity,
                    TradeDirection::BUY,
                );

                match order_result {
                    Some(_) => {
                        println!("BT: Order creation successful.\n");
                        successful_buys += 1;
                        sleep_thread(5);
                    }
                    None => {
                        println!("BT: Failed to create buy limit order. Retrying...\n");
                        sleep_thread(5);

                        let retry: Option<LimitOrder> = create_limit_order(
                            new_tick,
                            buy_price + 0.01,
                            buy_quantity,
                            TradeDirection::BUY,
                        );

                        match retry {
                            Some(_) => {
                                println!("BT: Retry order creation successful.\n");
                                successful_buys += 1;
                                sleep_thread(5);
                            }
                            None => {
                                println!("BT: Failed second buy attempt. Skipping...\n");
                                sleep_thread(5);
                                continue;
                            }
                        }
                        continue;
                    }
                }
            }

            println!(
                "\nBT: {}/{} buys complete.",
                successful_buys,
                buy_list.len()
            );

            println!("BT: Triggering sell thread");
            // Send sell thread trigger
            buy_to_sell_tx.send(true).unwrap();
        }
    }
}
