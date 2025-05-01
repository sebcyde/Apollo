mod control;
mod file_control;
mod finnhub;
mod helpers;
mod trading212;

use std::sync::mpsc;
use std::thread::{self};

use control::buy_channel::buy_channel::start_buying;
use control::control_channel::control_channel::start_control;
use control::sell_channel::sell_channel::start_selling;
use dotenv::dotenv;
use lazy_static::lazy_static;
use std::sync::{Arc, Mutex};
use trading212::types::types::{BalanceObject, Instrument};
use trading212::types::types::{LimitOrder, Position};

#[derive(Debug)]
pub enum VERSION {
    LIVE,
    DEMO,
}

#[derive(Debug)]
pub enum FILTERING_STRICTNESS {
    STRONG,
    LIGHT,
}

lazy_static! {
    pub static ref STOCK_VERSION: VERSION = VERSION::DEMO; // Live or testing on practise account

    // Minimum amount of seconds to await limit orders to hit
    // 300 = 5 minutes
    // 180 = 3 minutes
    pub static ref LIMIT_WAIT_TIME: u64 = 60;

    // Filtering Config
    pub static ref FILTER_STRICTNESS: FILTERING_STRICTNESS = FILTERING_STRICTNESS::LIGHT;
    pub static ref MINIMUM_MARKET_CAP: f64 = 2000.0; // Minimum market cap - 2bil?

    // Buy Config
    pub static ref MINIMUM_BALANCE: f64 = 15000.0; // Minimum acceptable balance to execute buys
    pub static ref AMOUNT_OF_TICKERS_TO_BUY: usize = 10; // Amount of tickers to buy per cycle
    pub static ref SPEND_PERC: f64 = 0.05; // % of balance to spend per cycle 0.1 = 10%
    pub static ref SHOPPING_TIME: u64 = 300; // Time per shopping cycle - 5 minutes
    pub static ref MINIMUM_BUYS: usize = 3; // Minimum amount of buys per cycle

    // Sell Config
    pub static ref SELL_PERCENT_DOWN: f64 = -0.025;
    pub static ref SELL_PERCENT_UP: f64 = 0.05;

}

// Data to be sent across threads during execution
#[derive(Debug)]
pub struct ChannelParam {
    pub arc_instruments_value: Arc<Mutex<Vec<Instrument>>>,
    pub arc_positions_value: Arc<Mutex<Vec<Position>>>,
    pub arc_limits_value: Arc<Mutex<Vec<LimitOrder>>>,
    pub arc_balance_value: Arc<Mutex<BalanceObject>>,
}

fn main() {
    println!("\nStarting Friday...");
    dotenv().ok();

    // --------------------- VARIABLES --------------------- //

    // Account Balance
    let balance_arc: Arc<Mutex<BalanceObject>> = Arc::new(Mutex::new(BalanceObject::default()));
    let c_balance_arc: Arc<Mutex<BalanceObject>> = Arc::clone(&balance_arc);
    let b_balance_arc: Arc<Mutex<BalanceObject>> = Arc::clone(&balance_arc);
    let s_balance_arc: Arc<Mutex<BalanceObject>> = Arc::clone(&balance_arc);

    // Instruments
    let instruments_arc: Arc<Mutex<Vec<Instrument>>> = Arc::new(Mutex::new(Vec::new()));
    let c_instruments_arc: Arc<Mutex<Vec<Instrument>>> = Arc::clone(&instruments_arc);
    let b_instruments_arc: Arc<Mutex<Vec<Instrument>>> = Arc::clone(&instruments_arc);
    let s_instruments_arc: Arc<Mutex<Vec<Instrument>>> = Arc::clone(&instruments_arc);

    // Current Positions
    let positions_arc: Arc<Mutex<Vec<Position>>> = Arc::new(Mutex::new(Vec::new()));
    let c_positions_arc: Arc<Mutex<Vec<Position>>> = Arc::clone(&positions_arc);
    let b_positions_arc: Arc<Mutex<Vec<Position>>> = Arc::clone(&positions_arc);
    let s_positions_arc: Arc<Mutex<Vec<Position>>> = Arc::clone(&positions_arc);

    // Limit Orders
    let limit_order_arc: Arc<Mutex<Vec<LimitOrder>>> = Arc::new(Mutex::new(Vec::new()));
    let c_limit_order_arc: Arc<Mutex<Vec<LimitOrder>>> = Arc::clone(&limit_order_arc);
    let b_limit_order_arc: Arc<Mutex<Vec<LimitOrder>>> = Arc::clone(&limit_order_arc);
    let s_limit_order_arc: Arc<Mutex<Vec<LimitOrder>>> = Arc::clone(&limit_order_arc);

    //
    //

    // --------------------- CHANNELS --------------------- //
    println!("\nCreating channels...");

    // For signalling data updates from Control thread
    let (ctrl_to_buy_tx, ctrl_to_buy_rx) = mpsc::channel::<bool>();
    let (ctrl_to_sell_tx, ctrl_to_sell_rx) = mpsc::channel::<bool>();

    // Sell to Buy
    let (sell_to_buy_tx, sell_to_buy_rx) = mpsc::channel::<bool>();

    // Sell to control
    let (sell_to_ctrl_tx, sell_to_ctrl_rx) = mpsc::channel::<bool>();

    // Buy to Sell
    let (buy_to_sell_tx, buy_to_sell_rx) = mpsc::channel::<bool>();

    println!("Done.");

    //
    //

    // --------------------- THREADS --------------------- //
    println!("\nCreating control thread...");

    let control_start_params: ChannelParam = ChannelParam {
        arc_instruments_value: c_instruments_arc,
        arc_positions_value: c_positions_arc,
        arc_limits_value: c_limit_order_arc,
        arc_balance_value: c_balance_arc,
    };

    let control_handle = thread::Builder::new()
        .name("Control".to_string())
        .spawn(move || {
            start_control(
                ctrl_to_sell_tx,
                ctrl_to_buy_tx,
                sell_to_ctrl_rx,
                control_start_params,
            );
        })
        .expect("Failed to spawn Control thread");

    println!("Done.");

    //

    println!("\nCreating buying thread...");

    let buy_start_params: ChannelParam = ChannelParam {
        arc_instruments_value: b_instruments_arc,
        arc_positions_value: b_positions_arc,
        arc_limits_value: b_limit_order_arc,
        arc_balance_value: b_balance_arc,
    };

    // Buy Thread
    let buy_handle = thread::Builder::new()
        .name("Buy".to_string())
        .spawn(move || {
            start_buying(
                ctrl_to_buy_rx,
                sell_to_buy_rx,
                buy_to_sell_tx,
                buy_start_params,
            );
        })
        .expect("Failed to spawn Buy thread");

    println!("Done.");

    //

    println!("\nCreating selling thread...");

    let sell_start_params: ChannelParam = ChannelParam {
        arc_instruments_value: s_instruments_arc,
        arc_positions_value: s_positions_arc,
        arc_limits_value: s_limit_order_arc,
        arc_balance_value: s_balance_arc,
    };

    // Sell Thread
    let sell_handle = thread::Builder::new()
        .name("Sell".to_string())
        .spawn(move || {
            start_selling(
                ctrl_to_sell_rx,
                sell_to_ctrl_tx,
                sell_to_buy_tx,
                buy_to_sell_rx,
                sell_start_params,
            );
        })
        .expect("Failed to spawn Sell thread");

    println!("Done.");
    // ------------------- END OF THREADS ------------------- //

    //
    //

    // --------------------- EXECUTION --------------------- //
    println!("\nStarting...\n");

    _ = control_handle.join();
    _ = buy_handle.join();
    _ = sell_handle.join();

    println!("\nDone.");
    // ------------------- END OF EXECUTION ------------------- //

    // TODO - create an arc mutex to hold commonly accessed data
    // ex balance, current positions, limit orders etv

    // TODO - some kind of lockon pulling account balance data to avoid rate limit collisions

    // let buy_thread_handle = thread::Builder::new()
    //     .name(format!("Buying-Thread"))
    //     .spawn(|| loop {
    //         println!("\n--------------------- STARTING BUY CYCLE ---------------------\n");

    //         if !is_market_open() {
    //             // Sleep 1 hour then restart buy cycle
    //             println!("\n--------------------- B: MARKET CLOSED ---------------------\n");
    //             sleep_thread(3600);
    //             continue;
    //         }

    //         let raw_balance_data: Result< BalanceObject, serde_json::Error> = get_account_balance();
    //         if raw_balance_data.is_err() {
    //             sleep_thread(180);
    //             continue;
    //         }

    //         let balance_data: BalanceObject = raw_balance_data.unwrap();

    //         if balance_data.free < *MINIMUM_BALANCE {
    //             // Sleep until sell cycle re-triggers to free up funds
    //             println!("\n--------------------- LOW BALANCE - SKIPPING BUY CYCLE ---------------------\n");
    //             sleep_thread(*LIMIT_WAIT_SECONDS);
    //             continue;
    //         }

    //         let instruments: Vec<Instrument> = match get_instruments_from_file() {
    //             Some(existing_instruments) => shuffle_instruments(existing_instruments),
    //             None => {
    //                 // If no instruments exist, fetch new ones and save them to file
    //                 let mut fetched_instruments: Vec<Instrument> = get_instruments();
    //                 fetched_instruments = shuffle_instruments(fetched_instruments);
    //                 write_instruments_to_file(fetched_instruments.clone());
    //                 fetched_instruments
    //             }
    //         };

    //         println!("\nB: Fetched {} Instruments.", instruments.len());
    //         println!("B: Available Account Balance: {:?}\n", balance_data.free);

    //         println!("\n--------------------- EXECUTING BUYS ---------------------\n");
    //         execute_buys(instruments, &balance_data);
    //         println!("\n--------------------- BUYS COMPLETE ---------------------\n");

    //         sleep_thread(*LIMIT_WAIT_SECONDS);
    //     })
    //     .expect("Failed to activate buying thread");

    // let sell_thread_handle = thread::Builder::new()
    //     .name(format!("Selling-Thread"))
    //     .spawn(|| loop {
    //         if !is_market_open() {
    //             // Sleep 1 hour then restart sell cycle
    //             println!("\n--------------------- S: MARKET CLOSED ---------------------\n");
    //             sleep_thread(3600);
    //             continue;
    //         }

    //         sleep_thread(180);

    //         println!("\n--------------------- STARTING SELL CYCLE ---------------------\n");

    //         let raw_balance_data = get_account_balance();
    //         if raw_balance_data.is_err() {
    //             sleep_thread(180);
    //             continue;
    //         }

    //         let balance_data: BalanceObject = raw_balance_data.unwrap();

    //         let start_time: String = get_current_time();
    //         let results: Vec<HistoricalOrder> = execute_sells(&start_time);

    //         println!("\n--------------------- UPDATING SELL LOGS ---------------------\n");
    //         sleep_thread(30);

    //         let new_raw_balance_data = get_account_balance();
    //         if new_raw_balance_data.is_err() {
    //             sleep_thread(180);
    //             continue;
    //         }

    //         let new_balance_data: BalanceObject = new_raw_balance_data.unwrap();

    //         log_cycle_result(start_time, results, new_balance_data.ppl - balance_data.ppl);

    //         println!("\n--------------------- SELL CYCLE COMPLETE ---------------------\n");

    //         sleep_thread(*LIMIT_WAIT_SECONDS);
    //     })
    //     .expect("Failed to activate selling thread");

    // _ = sell_thread_handle.join();
    // _ = buy_thread_handle.join();
}
