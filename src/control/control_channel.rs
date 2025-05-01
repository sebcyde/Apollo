pub mod control_channel {
    use std::sync::{mpsc, MutexGuard};
    use std::thread;
    use std::time::Duration;

    use crate::file_control::read::read::get_instruments_from_file;
    use crate::file_control::write::write::write_instruments_to_file;
    use crate::finnhub::utils::helpers::is_market_open;
    use crate::helpers::helpers::helpers::{shuffle_instruments, sleep_thread};
    use crate::trading212::helpers::helpers::{
        get_account_balance, get_all_orders_request, get_all_positions, get_instruments,
    };
    use crate::trading212::types::types::{BalanceObject, Instrument, LimitOrder, Position};
    use crate::ChannelParam;

    enum ReceiverType {
        BUY,
        SELL,
    }

    fn notify_thread(sender: mpsc::Sender<bool>, receiver: ReceiverType) {
        match sender.send(true) {
            Ok(_) => match receiver {
                ReceiverType::BUY => println!("CT: Signalled Buy thread."),
                ReceiverType::SELL => println!("CT: Signalled Sell thread."),
            },
            Err(e) => match receiver {
                ReceiverType::BUY => println!("CT: Failed to signal Buy thread."),
                ReceiverType::SELL => println!("CT: Failed to signal Sell thread."),
            },
        }
    }

    pub fn start_control(
        ctrl_to_sell_tx: mpsc::Sender<bool>,
        ctrl_to_buy_tx: mpsc::Sender<bool>,
        sell_to_ctrl_rx: mpsc::Receiver<bool>,
        data: ChannelParam,
    ) {
        let mut should_get_new_instruments: bool = true;
        let mut is_first_iteration: bool = true;

        // Data is pulled in loops so that if there is ever an error we can essentially retry until the data is susccessfully fetched

        // Periodic data update - Every 5 mins
        loop {
            if !is_market_open() {
                // Sleep 1 hour
                println!("\n--------------------- CT: MARKET CLOSED ---------------------\n");
                sleep_thread(3600);
                continue;
            }

            if !is_first_iteration {
                // Wait for trigger from sell thread
                println!("CT: Data updated. Waiting for Sell trigger to continue.");

                let res = sell_to_ctrl_rx.recv();
                match res {
                    Err(e) => {
                        println!("\n\nCT: Sell receiver error: {:?}", e);
                    }
                    Ok(_) => println!("\nCT: No error in sell receiver"),
                }

                println!("CT: Received signal from Sell thread to update data.");
            }

            println!("CT: Populating data...");

            // Updating balances data
            'balance: loop {
                println!("CT: Updating balance data...");
                let raw_data: Result<BalanceObject, serde_json::Error> = get_account_balance();
                match raw_data {
                    Err(e) => {
                        println!("CT: Error fetching balance data. Sleeping and retrying...");
                        println!("CT: Error: {:?}", e);
                        sleep_thread(20);
                    }
                    Ok(balance_data) => {
                        let mut balance = data.arc_balance_value.lock().unwrap();
                        balance.blocked = balance_data.blocked;
                        balance.invested = balance.invested;
                        balance.pie_cash = balance.pie_cash;
                        balance.free = balance_data.free;
                        balance.result = balance.result;
                        balance.total = balance.total;
                        balance.ppl = balance.ppl;

                        println!("CT: Balance data updated.\n");
                        break 'balance;
                    }
                }
            }

            'instruments: loop {
                // TODO - Check error handling
                // Instruments update - in case of too many requests, keep trying until it gets data
                println!("CT: Updating instruments data...");

                let mut instrument_list: Vec<Instrument> = if should_get_new_instruments {
                    println!("CT: Fetching new instruments data.");
                    let mut fetched_instruments: Vec<Instrument> = get_instruments();
                    fetched_instruments = shuffle_instruments(fetched_instruments);
                    write_instruments_to_file(fetched_instruments.clone());
                    should_get_new_instruments = !should_get_new_instruments;
                    fetched_instruments
                } else {
                    println!("CT: Fetching instruments data from file.");
                    let fetched_instruments: Option<Vec<Instrument>> = get_instruments_from_file();
                    match fetched_instruments {
                        Some(instrument_data) => instrument_data,
                        None => {
                            println!("CT: Failed to fetch instruments data from file.\nFetching new data...");
                            let mut fetched_instruments: Vec<Instrument> = get_instruments();
                            fetched_instruments = shuffle_instruments(fetched_instruments);
                            write_instruments_to_file(fetched_instruments.clone());
                            fetched_instruments
                        }
                    }
                };

                let mut instruments: MutexGuard<Vec<Instrument>> =
                    data.arc_instruments_value.lock().unwrap();
                instruments.clear();
                instruments.append(&mut instrument_list);

                println!("CT: Instruments data updated.\n");
                break 'instruments;
            }

            'positions: loop {
                // instruments update - in case of too many requests, keep trying until it gets data
                println!("CT: Updating positions data...");

                sleep_thread(10);
                let initial_positions_data: Option<Vec<Position>> = get_all_positions();

                if initial_positions_data.is_none() {
                    println!("CT: Position data returned null. Retrying...");
                    sleep_thread(10);
                    continue;
                }

                let mut initial_positions = initial_positions_data.unwrap();

                let mut positions = data.arc_positions_value.lock().unwrap();
                positions.clear();
                positions.append(&mut initial_positions);

                println!("CT: Positions data updated.\n");
                break 'positions;
            }

            'limits: loop {
                // Limit orders update - in case of too many requests, keep trying until it gets data
                println!("CT: Updating instruments data...");
                let raw_data: Option<Vec<LimitOrder>> = get_all_orders_request();

                match raw_data {
                    None => {
                        println!(
                            "CT: Error fetching limit orders data. Sleeping and retrying...\n"
                        );
                        sleep_thread(60);
                    }

                    Some(mut limit_orders) => {
                        let mut limits = data.arc_limits_value.lock().unwrap();

                        limits.clear();
                        limits.append(&mut limit_orders);

                        println!("CT: Limit orders data updated.\n");
                        break 'limits;
                    }
                }
            }

            // Notify threads of new data availability
            println!("CT: Data Updated. Notifying threads...\n");
            is_first_iteration = false;

            notify_thread(ctrl_to_buy_tx.clone(), ReceiverType::BUY);
            notify_thread(ctrl_to_sell_tx.clone(), ReceiverType::SELL);

            // sleep_thread(600); // 10 Minutes
        }
    }
}
