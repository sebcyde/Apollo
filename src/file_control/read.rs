pub mod read {
    use std::path::PathBuf;

    use crate::{
        file_control::types::types::{
            file_current_trading212_positions_data, file_instrument_data,
        },
        helpers::{
            helpers::helpers::{
                get_current_date, get_current_hours, is_before_today, is_first_date_before,
                print_message, THREAD,
            },
            types::types::{DateType, FullCompanyInfo},
        },
        trading212::types::types::{Instrument, Position},
    };

    pub fn get_dir_path() -> PathBuf {
        let mut path: PathBuf = dirs::config_dir().unwrap();
        path.push("Friday");
        path
    }

    pub fn get_positions_from_file() -> Option<Vec<Position>> {
        println!("\nST: Reading position data from file...");

        let read_positions: String =
            match std::fs::read_to_string("src/data/current_positions.json").ok() {
                None => {
                    println!("ST: None found.");
                    return None;
                }
                Some(read_positions) => read_positions,
            };

        if read_positions.is_empty() {
            println!("ST: None found.");
            return None;
        }

        match serde_json::from_str::<file_current_trading212_positions_data>(&read_positions) {
            Ok(positions_data) => {
                return Some(positions_data.positions);
            }
            Err(_) => {
                println!("ST: None found.");
                return None;
            }
        }
    }

    pub fn get_instruments_from_file() -> Option<Vec<Instrument>> {
        print_message(THREAD::FILE, "Reading instrument data from file...");

        let read_instruments: String =
            match std::fs::read_to_string("src/data/instruments.json").ok() {
                None => {
                    print_message(THREAD::FILE, "None found.");
                    return None;
                }
                Some(read_instruments) => read_instruments,
            };

        if read_instruments.is_empty() {
            print_message(THREAD::FILE, "None found.");
            return None;
        }

        match serde_json::from_str::<file_instrument_data>(&read_instruments) {
            Ok(instrument_data) => {
                if is_before_today(&instrument_data.creation_date) {
                    return None;
                }
                return Some(instrument_data.instruments);
            }
            Err(_) => {
                print_message(THREAD::FILE, "None found.");
                return None;
            }
        }
    }

    pub fn get_buy_list_from_file() -> Option<Vec<FullCompanyInfo>> {
        println!("\nBT: Reading buy list data from file...");

        let mut buy_list_path: PathBuf = get_dir_path();
        buy_list_path.push("Lists");
        buy_list_path.push("buy_list.json");

        let read_buy_list_data: String = match std::fs::read_to_string(buy_list_path).ok() {
            None => {
                println!("BT: None found.");
                return None;
            }
            Some(read_instruments) => read_instruments,
        };

        if read_buy_list_data.is_empty() {
            println!("BT: None found.");
            return None;
        }

        match serde_json::from_str::<Vec<FullCompanyInfo>>(&read_buy_list_data) {
            Ok(list_data) => {
                println!("BT: Done.");
                return Some(list_data);
            }
            Err(_) => {
                println!("BT: None found.");
                return None;
            }
        }
    }

    pub fn get_docs_dir_path() -> PathBuf {
        let mut path: PathBuf = dirs::document_dir().unwrap();
        path.push("Friday");
        path
    }

    pub fn add_stock_to_personal_list(company: FullCompanyInfo) {
        let dir: PathBuf = get_docs_dir_path();
    }
}
