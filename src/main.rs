mod control;
mod file_control;
mod finnhub;
mod helpers;
mod trading212;

use std::sync::mpsc;
use std::thread::{self, JoinHandle};
use std::time::Duration;

use dotenv::dotenv;
use file_control::read::read::{get_instruments_from_file, get_positions_from_file};
use file_control::write::write::{write_instruments_to_file, write_positions_to_file};
use helpers::helpers::helpers::{print_message, THREAD};
use lazy_static::lazy_static;
use std::sync::{Arc, Mutex};
use trading212::helpers::helpers::{get_account_balance, get_all_positions, get_instruments};
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
    print_message(THREAD::MAIN, "Starting Apollo...");
    dotenv().ok();

    // --------------------- Global Variables --------------------- //
    let balance_arc: Arc<Mutex<BalanceObject>> = Arc::new(Mutex::new(BalanceObject::default()));
    let c_balance_arc: Arc<Mutex<BalanceObject>> = Arc::clone(&balance_arc);
    // let b_balance_arc: Arc<Mutex<BalanceObject>> = Arc::clone(&balance_arc);
    // let s_balance_arc: Arc<Mutex<BalanceObject>> = Arc::clone(&balance_arc);

    //
    // --------------------- Trading212 Stock List Data Collection Thread --------------------- //
    print_message(THREAD::MAIN, "Creating 212 stock data collection thread...");
    let trading212_stock_list_collection_handle: JoinHandle<()> = thread::Builder::new()
        .name("Trading212_Stock_List_Collection".to_string())
        .spawn(move || {
            loop {
                // Check if current data is already from today
                print_message(
                    THREAD::COLLECTION,
                    "Checking Trading212 Stock List Data Validity...",
                );
                if get_instruments_from_file().is_none() {
                    print_message(
                        THREAD::COLLECTION,
                        "Starting Trading212 Stock List Data Collection...",
                    );
                    let all_trading212_stocks_data: Vec<Instrument> = get_instruments();
                    write_instruments_to_file(all_trading212_stocks_data); //"src/data/instruments.json"
                    print_message(THREAD::COLLECTION, "Trading212 Stock List Data Updated.");
                }

                thread::sleep(Duration::from_secs(60 * 60)); // Every Hour
            }
        })
        .expect("[Main Thread] Failed to spawn Stock List Data Collection thread");

    print_message(
        THREAD::MAIN,
        "Creating 212 Account data collection thread...",
    );
    let trading212_account_data_collection_handle: JoinHandle<()> = thread::Builder::new()
        .name("Trading212_Account_Data_Collection".to_string())
        .spawn(move || {
            loop {
                // Check if current data is already from today
                print_message(THREAD::COLLECTION, "Getting Trading212 Account Data");
                thread::sleep(Duration::from_secs(60 * 60)); // Every Hour
            }
        })
        .expect("[Main Thread] Failed to spawn Stock List Data Collection thread");

    // --------------------- Trading 212 Account Information  --------------------- //

    // let current_positions_raw_data: Vec<Position> =
    //     get_all_positions().expect("Failed to get positions from API.");
    // if current_positions_raw_data.len() > 0 {
    //     write_positions_to_file(current_positions_raw_data); // "src/data/current_positions.json"
    //     println!("\nCurrent positions retreived and written to storage.\n");
    // }

    // let current_positions: Vec<Position> =
    //     get_positions_from_file().expect("Failed to pull positions data from file");

    // // For Debugging
    // for current_position in current_positions {
    //     println!("\nPosition: {:?}", &current_position);
    // }

    // let account_balance: Result<BalanceObject, serde_json::Error> = get_account_balance();
    // println!("\nAccount Balance: {:?}", account_balance);

    // let current_positions: Vec<Position> =
    //     get_positions_from_file().expect("Failed to pull positions data from file");

    _ = trading212_stock_list_collection_handle.join();
    _ = trading212_account_data_collection_handle.join();
    // _ = sell_handle.join();
}
