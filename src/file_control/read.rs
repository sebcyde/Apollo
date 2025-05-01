pub mod read {
    use std::path::PathBuf;

    use crate::{
        file_control::types::types::file_instrument_data,
        helpers::{
            helpers::helpers::{get_current_date, get_current_hours, is_first_date_before},
            types::types::{DateType, FullCompanyInfo},
        },
        trading212::types::types::Instrument,
    };

    pub fn get_dir_path() -> PathBuf {
        let mut path: PathBuf = dirs::config_dir().unwrap();
        path.push("Friday");
        path
    }

    pub fn get_instruments_from_file() -> Option<Vec<Instrument>> {
        println!("\nST: Reading instrument data from file...");

        let read_instruments: String =
            match std::fs::read_to_string("src/data/instruments.json").ok() {
                None => {
                    println!("ST: None found.");
                    return None;
                }
                Some(read_instruments) => read_instruments,
            };

        if read_instruments.is_empty() {
            println!("ST: None found.");
            return None;
        }

        match serde_json::from_str::<file_instrument_data>(&read_instruments) {
            Ok(instrument_data) => {
                println!("ST: Checking validity of data...");

                let current_date: String = get_current_date(DateType::DMY);
                if is_first_date_before(&instrument_data.creation_date, &current_date) {
                    println!("ST: Data out of date.");
                    return None;
                };

                // TODO
                // let current_hours: String = get_current_hours();
                // if is_first_hours_before(&instrument_data.creation_date, &current_hours) {
                //     println!("ST: Data out of date.");
                //     return None;
                // }

                println!("ST: Done.");
                return Some(instrument_data.instruments);
            }
            Err(_) => {
                println!("ST: None found.");
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
