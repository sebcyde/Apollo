pub mod helpers {

    use chrono::{DateTime, Duration, Local, Month, NaiveDate, Utc};
    use rand::seq::SliceRandom;
    use rand::thread_rng;
    use serde_json::Error;
    use serde_json::{self, Value};
    use std::time::Instant;
    use std::{fs, path::Path};

    use crate::finnhub::utils::helpers::make_fh_request;
    use crate::helpers::types::types::{EarningsCalendar, FullInsiderTransaction, RawSentData};
    use crate::{
        finnhub::utils::helpers::{
            convert_to_fh_ticker, get_company_data, get_stock_price, CompanyInfo, FHStockData,
        },
        helpers::types::types::{
            CompanyFinancials, DateType, EarningsRelease, FullCompanyInfo, InsiderTransaction,
            NewsArticle, SentimentData,
        },
        trading212::types::types::{BalanceObject, Instrument, Position},
    };
    use crate::{AMOUNT_OF_TICKERS_TO_BUY, MINIMUM_MARKET_CAP, SPEND_PERC};

    pub fn sleep_thread(seconds: u64) {
        std::thread::sleep(std::time::Duration::from_secs(seconds));
    }

    pub fn get_time() -> Instant {
        let start_time = Instant::now();
        start_time
    }

    pub fn get_current_time() -> String {
        let current_time = Local::now();
        let current_time_str: String = current_time.format("%d-%m %H:%M:%S").to_string();
        current_time_str
    }

    pub fn get_current_hours() -> String {
        let current_time = Local::now();
        let current_time_str: String = current_time.format("%H:%M").to_string();
        current_time_str
    }

    pub fn round_down(original_number: f64, decimals: u32) -> f64 {
        let factor: f64 = 10f64.powi(decimals as i32);
        let rounded: f64 = (original_number * factor).floor() / factor;
        rounded
    }

    pub fn chop_two_after_dec(num: f64) -> f64 {
        let num_string = format!("{:.10}", num); // Convert to string with a sufficient number of decimal places
        let parts: Vec<&str> = num_string.split('.').collect(); // Split into integer and fractional parts
        if parts.len() > 1 {
            let truncated_fraction = &parts[1][..2.min(parts[1].len())]; // Take only the first two digits of the fractional part
            format!("{}.{}", parts[0], truncated_fraction)
                .parse()
                .unwrap() // Concatenate and convert back to f64
        } else {
            num
        }
    }

    pub fn calc_perc_change(bought_at: f64, current: f64) -> f64 {
        if bought_at == 0.0 {
            return f64::NAN; // Not a Number (NaN) for cases where bought_at is zero
        }

        ((current - bought_at) / bought_at) * 100.0
    }

    pub fn parse_date(date_input: &str) -> String {
        let parsed_date_time = DateTime::parse_from_rfc3339(date_input)
            .expect("Failed to parse date-time")
            .with_timezone(&Utc);

        return parsed_date_time.format("%d/%m/%y").to_string();
    }

    pub fn is_first_date_before(first_date_str: &str, second_date_str: &str) -> bool {
        let format = "%d-%m-%Y";
        let first_date =
            NaiveDate::parse_from_str(first_date_str, format).expect("Failed to parse first date");
        let second_date = NaiveDate::parse_from_str(second_date_str, format)
            .expect("Failed to parse second date");

        first_date < second_date
    }

    pub fn shuffle_instruments(mut instruments: Vec<Instrument>) -> Vec<Instrument> {
        println!("\nST: Shuffling instrument list...");
        let mut rng = thread_rng();
        instruments.shuffle(&mut rng);
        println!("ST: Done.");
        return instruments;
    }

    pub fn shuffle_positions(mut positions: Vec<Position>) -> Vec<Position> {
        println!("\nST: Shuffling positions list...\n");
        let mut rng = thread_rng();
        positions.shuffle(&mut rng);
        return positions;
    }

    pub fn calculate_amount_spent_per_ticker(balance_data: &BalanceObject) -> f64 {
        let available_for_trading: f64 = balance_data.free * (1.0 - (SPEND_PERC.clamp(0.0, 1.0)));

        let amount_per_ticker: f64 = available_for_trading / *AMOUNT_OF_TICKERS_TO_BUY as f64;
        println!("BT: Spending {:?} per ticker.\n", amount_per_ticker);

        amount_per_ticker
    }

    pub fn get_current_date(datetype: DateType) -> String {
        let date: NaiveDate = Local::now().date_naive();

        match datetype {
            DateType::D => return format!("{}", date.format("%d")),
            DateType::Y => return format!("{}", date.format("%Y")),
            DateType::DMY => return format!("{}", date.format("%d-%m-%Y")),
            DateType::MY => return format!("{}", date.format("%B-%Y")),
            DateType::M => return format!("{}", date.format("%B")),
        }
    }

    pub fn get_full_company_info(instrument: Instrument) -> Option<FullCompanyInfo> {
        let fh_ticker: &String = &convert_to_fh_ticker(&instrument.ticker);

        println!("\n--- Starting info collection for {}...", fh_ticker);

        let company_info: CompanyInfo = match get_company_data(fh_ticker) {
            Some(raw_company_info) => match raw_company_info {
                Ok(company_info) => company_info,
                Err(e) => return None,
            },
            None => return None,
        };

        if company_info.market_capitalization < *MINIMUM_MARKET_CAP {
            println!(
                "BT: Market Cap of {}m is below acceptable parameters.",
                company_info.market_capitalization.trunc() as i64
            );
            return None;
        }

        let company_quote_res: Result<FHStockData, Error> = match get_stock_price(fh_ticker) {
            None => return None,
            Some(quote_res) => quote_res,
        };

        let company_quote: FHStockData = match company_quote_res {
            Err(e) => {
                println!("Error fetching quote: {:?}", e);
                return None;
            }
            Ok(company_quote) => company_quote,
        };

        let insider_data: Option<Vec<InsiderTransaction>> = get_insider_transactions(fh_ticker);

        let news_data: Vec<NewsArticle> = match get_news_articles(fh_ticker) {
            Some(news_data) => news_data,
            None => return None,
        };

        let peers_data: Option<Vec<String>> = get_company_peers(fh_ticker);

        let financials_data: CompanyFinancials = match get_company_financials(fh_ticker) {
            Some(financials_data) => financials_data,
            None => return None,
        };

        let sentiment_data: Vec<SentimentData> = match get_company_sentiment(fh_ticker) {
            Some(sentiment_data) => sentiment_data,
            None => return None,
        };

        let earnings_calendar: Option<Vec<EarningsRelease>> = get_earnings_calendar(fh_ticker);

        let full_company_data: FullCompanyInfo = FullCompanyInfo {
            instrument,
            company_info,
            company_stock_quote: company_quote,
            company_news: news_data,
            company_peers: peers_data,
            company_financials: financials_data,
            company_sentiment: sentiment_data,
            company_earnings_calendar: earnings_calendar,
            insider_transactions: insider_data,
        };

        println!("--- Info collection complete.");

        Some(full_company_data)
    }

    fn get_insider_transactions(fh_ticker: &String) -> Option<Vec<InsiderTransaction>> {
        println!("Fetching insider transactions...");

        let endpoint: String = format!(
            "https://finnhub.io/api/v1/stock/insider-transactions?symbol={}",
            fh_ticker
        );
        let raw_data: Option<Value> = make_fh_request(endpoint);

        if raw_data.is_none() {
            println!("Transaction data was null.\n");
            return None;
        }

        let raw_full_transaction_data: Option<FullInsiderTransaction> =
            match serde_json::from_value(raw_data.unwrap()) {
                Ok(raw_full_transaction_data) => raw_full_transaction_data,
                Err(e) => {
                    println!("Error deserialising full insider transactions: {:?}\n", e);
                    None
                }
            };

        if raw_full_transaction_data.is_some() {
            let full_transaction_data: FullInsiderTransaction = raw_full_transaction_data.unwrap();
            return Some(full_transaction_data.transactions);
        }

        None
    }

    fn get_news_articles(fh_ticker: &String) -> Option<Vec<NewsArticle>> {
        let today = Utc::now();
        let start_date = today - Duration::days(5);
        let start_date_str = start_date.format("%Y-%m-%d").to_string();
        let end_date_str = today.format("%Y-%m-%d").to_string();
        let final_string = format!("{}&to={}", start_date_str, end_date_str);

        println!(
            "Fetching company news from {} to {}...",
            &start_date_str, &end_date_str
        );

        let endpoint: String = format!(
            "https://finnhub.io/api/v1/company-news?symbol={}&from={}",
            fh_ticker, final_string
        );
        let data: Option<Value> = make_fh_request(endpoint);
        if data.is_none() {
            println!("News data was null.\n");
            return None;
        }

        match serde_json::from_value(data.unwrap()) {
            Ok(news_articles) => news_articles,
            Err(e) => {
                println!("Error fetching news data: {:?}", e);
                None
            }
        }
    }

    fn get_company_peers(fh_ticker: &String) -> Option<Vec<String>> {
        println!("Fetching company peers...");

        let endpoint: String =
            format!("https://finnhub.io/api/v1/stock/peers?symbol={}", fh_ticker);
        let data: Option<Value> = make_fh_request(endpoint);

        if data.is_none() {
            println!("Peers data was null.\n");
            return None;
        }

        match serde_json::from_value(data.unwrap()) {
            Ok(peers) => peers,
            Err(e) => {
                println!("Error fetching company peers: {:?}\n", e);
                None
            }
        }
    }

    fn get_company_financials(fh_ticker: &String) -> Option<CompanyFinancials> {
        println!("Fetching company financials...");

        let endpoint: String = format!(
            "https://finnhub.io/api/v1/stock/metric?symbol={}&metric=all",
            fh_ticker
        );
        let data: Option<Value> = make_fh_request(endpoint);

        if data.is_none() {
            println!("Financials data was null.\n");
            return None;
        }

        match serde_json::from_value(data.unwrap()) {
            Ok(financials) => financials,
            Err(e) => {
                println!("Error fetching company financials: {:?}\n", e);
                None
            }
        }
    }

    fn get_company_sentiment(fh_ticker: &String) -> Option<Vec<SentimentData>> {
        let today = Utc::now();
        // 62 dats = 2 months?
        let start_date = today - Duration::days(62);
        let start_date_str = start_date.format("%Y-%m-%d").to_string();
        let end_date_str = today.format("%Y-%m-%d").to_string();
        let final_string = format!("{}&to={}", start_date_str, end_date_str);

        println!(
            "Fetching company sentiment from {} to {}...",
            &start_date_str, &end_date_str
        );

        let endpoint: String = format!(
            "https://finnhub.io/api/v1/stock/insider-sentiment?symbol={}&from={}",
            fh_ticker, final_string
        );
        let data: Option<Value> = make_fh_request(endpoint);

        if data.is_none() {
            println!("Sentiment data was null.\n");
            return None;
        }

        let raw_data: RawSentData = match serde_json::from_value(data.unwrap()) {
            Ok(raw_sentiment) => raw_sentiment,
            Err(e) => {
                println!("Error fetching company sentiment: {:?}\n", e);
                return None;
            }
        };

        Some(raw_data.data)
    }

    fn get_earnings_calendar(fh_ticker: &String) -> Option<Vec<EarningsRelease>> {
        let today = Utc::now();
        // from the past year
        let start_date = today - Duration::days(365);
        let start_date_str = start_date.format("%Y-%m-%d").to_string();
        let end_date_str = today.format("%Y-%m-%d").to_string();
        let final_string = format!("{}&to={}", start_date_str, end_date_str);

        println!(
            "BT: Fetching company earnings from {} to {}...",
            &start_date_str, &end_date_str
        );

        let endpoint: String = format!(
            "https://finnhub.io/api/v1/calendar/earnings?from={}&symbol={}",
            final_string, fh_ticker
        );
        let data: Option<Value> = make_fh_request(endpoint);

        if data.is_none() {
            println!("Earnings calendar data was null.\n");
            return None;
        }

        let calendar: EarningsCalendar = match serde_json::from_value(data.unwrap()) {
            Ok(calendar) => calendar,
            Err(e) => {
                return None;
            }
        };

        if !calendar.earnings_calendar.len() > 0 {
            return None;
        }

        Some(calendar.earnings_calendar)
    }
}
