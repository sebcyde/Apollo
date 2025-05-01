pub mod types {
    use serde::{Deserialize, Serialize};

    use crate::trading212::types::types::{HistoricalOrder, Instrument};

    #[derive(Debug, Serialize, Deserialize)]
    pub struct file_instrument_data {
        pub creation_date: String,
        pub instruments: Vec<Instrument>,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct CycleResult {
        pub start_time: String,
        pub end_time: String,
        pub sales: Vec<HistoricalOrder>,
        pub total_profit: f64,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct SaleResult {
        pub sale_time: String,
        pub sale_price: f64,
        pub quantity_sold: f64,
        pub ticker: String,
        pub profit: f64,
    }
}
