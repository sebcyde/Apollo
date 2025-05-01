pub mod helpers {
    use reqwest::{header, Body, Client, RequestBuilder, Response};
    use serde::{Deserialize, Serialize};
    use serde_json::Value;
    use std::env;
    use tokio;

    use crate::helpers::helpers::helpers::sleep_thread;

    #[derive(Debug, Serialize, Deserialize)]
    pub struct FHStockData {
        #[serde(rename = "c")]
        pub current_price: f64, // Current price
        #[serde(rename = "h")]
        pub todays_high: f64, // High price
        #[serde(rename = "l")]
        pub todays_low: f64, // Low price
        #[serde(rename = "o")]
        pub todays_open: f64, // Open price
        #[serde(rename = "pc")]
        pub previous_close: f64, // Previous close price
        #[serde(rename = "t")]
        pub timestamp: i64, // Timestamp (in seconds since epoch)
    }

    pub fn convert_to_fh_ticker(ticker: &String) -> String {
        let fh_ticker: String;

        if ticker.contains("_US_EQ") {
            fh_ticker = ticker.replace("_US_EQ", "")
        } else if ticker.contains("_EQ") {
            fh_ticker = ticker.replace("_EQ", "")
        } else {
            fh_ticker = ticker.clone()
        }

        return fh_ticker.to_ascii_uppercase();
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct CompanyInfo {
        pub country: String,
        pub currency: String,
        pub exchange: String,
        pub ipo: String,
        #[serde(rename = "marketCapitalization")]
        pub market_capitalization: f64,
        pub name: String,
        pub phone: String,
        #[serde(rename = "shareOutstanding")]
        pub share_outstanding: f64,
        pub ticker: String,
        pub weburl: String,
        pub logo: String,
        #[serde(rename = "finnhubIndustry")]
        pub finnhub_industry: String,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct MarketStatus {
        pub exchange: String,        // The exchange (e.g., "US")
        pub holiday: Option<String>, // Holiday status (nullable)
        #[serde(rename = "isOpen")]
        pub is_open: bool, // Indicates if the market is open
        pub session: String,         // Market session (e.g., "pre-market")
        pub timezone: String,        // Timezone (e.g., "America/New_York")
        pub t: u64,                  // Timestamp
    }

    pub fn is_market_open() -> bool {
        let raw_market_data: Result<MarketStatus, serde_json::Error> =
            get_market_data().expect("Failed to get price");
        match raw_market_data {
            Ok(market_data) => market_data.is_open,
            Err(e) => {
                println!("Failed to fetch market data. {:?}\nRetrying...", e);

                let raw_market_data: Result<MarketStatus, serde_json::Error> =
                    get_market_data().expect("Failed to get price");
                match raw_market_data {
                    Ok(market_data) => market_data.is_open,
                    Err(e) => {
                        println!("Failed second attempt to fetch market data. {:?}\n", e);
                        false
                    }
                }
            }
        }
    }

    #[tokio::main]
    pub async fn get_market_data() -> Option<Result<MarketStatus, serde_json::Error>> {
        println!("CT: Fetching US market data...");

        let api_key: String = env::var("FH_API_KEY").expect("FH_API_KEY must be set");

        let endpoint: String = format!(
            "https://finnhub.io/api/v1/stock/market-status?exchange=US&token={}",
            api_key
        );

        let client: Client = Client::new();

        let request: RequestBuilder = client.get(&endpoint).header(header::AUTHORIZATION, api_key);
        let response: Response = request.send().await.unwrap();

        if response.status().is_success() {
            let body: String = response.text().await.unwrap();
            let result: Result<Value, serde_json::Error> = serde_json::from_str(&body);

            match result {
                Ok(json) => {
                    let res: Result<MarketStatus, serde_json::Error> = serde_json::from_value(json);
                    println!("CT: Done.");

                    return Some(res);
                }
                Err(e) => {
                    println!("CT: Failed to deserialize Market Data JSON: {:?}", e);
                    return None;
                }
            }
        } else {
            println!(
                "CT: Request to {} failed. Code: {}",
                &endpoint,
                response.status()
            );
            return None;
        }
    }

    #[tokio::main]
    pub async fn get_company_data(
        ticker: &String,
    ) -> Option<Result<CompanyInfo, serde_json::Error>> {
        println!("BT: Fetching company data for: {}", &ticker);

        let api_key: String = env::var("FH_API_KEY").expect("FH_API_KEY must be set");

        let endpoint: String = format!(
            "https://finnhub.io/api/v1/stock/profile2?symbol={}&token={}",
            ticker, api_key
        );

        let client: Client = Client::new();

        let request: RequestBuilder = client.get(&endpoint).header(header::AUTHORIZATION, api_key);
        let res = request.send().await;

        if res.is_err() {
            return None;
        }

        let response = res.unwrap();
        if response.status().is_success() {
            let body: String = response.text().await.unwrap();
            let result: Result<Value, serde_json::Error> = serde_json::from_str(&body);

            match result {
                Ok(json) => {
                    let res: Result<CompanyInfo, serde_json::Error> = serde_json::from_value(json);
                    return Some(res);
                }
                Err(e) => {
                    println!("BT: Failed to deserialize company info JSON: {:?}", e);
                    return None;
                }
            }
        } else {
            println!(
                "BT: Request to {} failed. Code: {}",
                &endpoint,
                response.status()
            );
            return None;
        }
    }

    #[tokio::main]
    pub async fn get_stock_price(
        ticker: &String,
    ) -> Option<Result<FHStockData, serde_json::Error>> {
        println!("BT: Fetching price for: {}", &ticker);

        let api_key: String = env::var("FH_API_KEY").expect("DEMO_API_KEY must be set");

        let endpoint: String = format!(
            "https://finnhub.io/api/v1/quote?symbol={}&token={}",
            ticker, api_key
        );

        let client: Client = Client::new();

        let request: RequestBuilder = client.get(&endpoint).header(header::AUTHORIZATION, api_key);
        let response: Response = request.send().await.unwrap();

        if response.status().is_success() {
            let body: String = response.text().await.unwrap();
            let result: Result<Value, serde_json::Error> = serde_json::from_str(&body);

            match result {
                Ok(json) => {
                    let res: Result<FHStockData, serde_json::Error> = serde_json::from_value(json);
                    return Some(res);
                }
                Err(e) => {
                    println!("Failed to deserialize JSON: {:?}", e);
                    return None;
                }
            }
        } else {
            println!(
                "Request to {} failed. Code: {}",
                &endpoint,
                response.status()
            );
            return None;
        }
    }

    #[tokio::main]
    pub async fn make_fh_request(mut endpoint: String) -> Option<Value> {
        let api_key: String = env::var("FH_API_KEY").expect("DEMO_API_KEY must be set");
        endpoint = format!("{}&token={}", endpoint, api_key);

        let client: Client = Client::new();
        let request: RequestBuilder = client.get(&endpoint).header(header::AUTHORIZATION, api_key);
        let res: Result<Response, reqwest::Error> = request.send().await;

        if res.is_err() {
            return None;
        }

        let response: Response = res.unwrap();

        // println!("Request sent. Throttling thread...");
        sleep_thread(3);
        // println!("Throttle complete.");

        if response.status().is_success() {
            let body: String = response.text().await.unwrap();
            let result: Result<Value, serde_json::Error> = serde_json::from_str(&body);

            match result {
                Ok(json) => {
                    return Some(json);
                }
                Err(e) => {
                    println!("Failed to deserialize JSON: {:?}", e);
                    return None;
                }
            }
        } else {
            println!(
                "Request to {} failed. Code: {}",
                &endpoint,
                response.status()
            );
            return None;
        }
    }
}
