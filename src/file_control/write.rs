pub mod write {
    use std::{
        fs::{File, OpenOptions},
        io::{Read, Write},
        path::{Path, PathBuf},
    };

    use serde_json::{from_str, to_string_pretty};

    use crate::{
        file_control::types::types::{file_instrument_data, CycleResult, SaleResult},
        helpers::{
            helpers::helpers::{get_current_date, get_current_time},
            types::types::{DateType, FullCompanyInfo},
        },
        trading212::types::types::{BalanceObject, HistoricalOrder, Instrument},
    };

    ////////////////////////// Trading 212 //////////////////////////////////

    pub fn write_instruments_to_file(instruments: Vec<Instrument>) {
        println!("\nST: Writing instrument list to file...");

        let instrument_data: file_instrument_data = file_instrument_data {
            creation_date: get_current_date(DateType::DMY),
            instruments,
        };

        let instrument_list: String = serde_json::to_string_pretty(&instrument_data)
            .expect("Instruments serialization failed");

        std::fs::write("src/data/instruments.json", instrument_list)
            .expect("ST: Failed to write instruments to file.");
        println!("ST: Done.");
    }

    pub fn write_buy_list_to_file(instruments: &Vec<FullCompanyInfo>) {
        println!("\nBT: Writing buy list to file...");

        let instrument_list: String =
            serde_json::to_string_pretty(instruments).expect("Instruments serialization failed");

        std::fs::write("src/data/buy_list.json", instrument_list)
            .expect("BT: Failed to write buy list to file.");
        println!("BT: Done.");
    }

    pub fn log_cycle_result(start_time: String, results: Vec<HistoricalOrder>, total_profit: f64) {
        println!(
            "\n--------------------- S: PROFIT THIS CYCLE: {} ---------------------\n",
            total_profit
        );

        let current_result: CycleResult = CycleResult {
            end_time: get_current_time(),
            sales: results,
            total_profit,
            start_time,
        };

        // Pull down data, append to sales vector, and write back to json file

        let file_path: &str = "src/data/result_list.json";

        let mut data: Vec<CycleResult> = if Path::new(file_path).exists() {
            let mut file = File::open(file_path).expect("Failed to open results file.");
            let mut contents = String::new();
            file.read_to_string(&mut contents)
                .expect("Failed to read results file");

            from_str(&contents).expect("Failed to deserialize results JSON")
        } else {
            Vec::new()
        };

        data.push(current_result);

        let updated_json: String =
            to_string_pretty(&data).expect("Failed to serialize results to JSON");

        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(file_path)
            .expect("Failed to truncate to results file.");

        file.write_all(updated_json.as_bytes())
            .expect("Failed to write to results file.");
    }
}
