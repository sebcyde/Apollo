pub mod types {
    use serde::{Deserialize, Serialize};

    use crate::{
        finnhub::utils::helpers::{CompanyInfo, FHStockData},
        trading212::types::types::Instrument,
    };

    pub enum DateType {
        D,
        Y,
        M,
        DMY,
        MY,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct FullCompanyInfo {
        pub instrument: Instrument,
        //
        // https://finnhub.io/api/v1/stock/profile2?symbol={}&token={}
        pub company_info: CompanyInfo,
        //
        // https://finnhub.io/docs/api/quote
        pub company_stock_quote: FHStockData,
        //
        // https://finnhub.io/docs/api/insider-transactions
        pub insider_transactions: Option<Vec<InsiderTransaction>>,
        //
        // https://finnhub.io/docs/api/company-news
        pub company_news: Vec<NewsArticle>,
        //
        // https://finnhub.io/docs/api/company-peers
        pub company_peers: Option<Vec<String>>,
        //
        // https://finnhub.io/docs/api/company-basic-financials
        pub company_financials: CompanyFinancials,
        //
        // https://finnhub.io/docs/api/insider-sentiment
        pub company_sentiment: Vec<SentimentData>,
        //
        // // https://finnhub.io/docs/api/company-earningss
        // pub company_past_earnings: Vec<EarningsReport>,
        //
        // https://finnhub.io/docs/api/earnings-calendar
        pub company_earnings_calendar: Option<Vec<EarningsRelease>>,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct FullInsiderTransaction {
        #[serde(rename = "data")]
        pub transactions: Vec<InsiderTransaction>,
        pub symbol: String,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct InsiderTransaction {
        pub name: String,
        pub share: i64,
        pub change: i64,
        #[serde(rename = "filingDate")]
        pub filing_date: String,
        #[serde(rename = "transactionDate")]
        pub transaction_date: String,
        #[serde(rename = "transactionCode")]
        pub transaction_code: String,
        #[serde(rename = "transactionPrice")]
        pub transaction_price: f64,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct NewsArticle {
        pub category: String,
        pub datetime: i64,
        pub headline: String,
        pub id: i32,
        pub image: String,
        pub related: String,
        pub source: String,
        pub summary: String,
        pub url: String,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct DataPoint {
        pub period: String,
        pub v: f64,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct AnnualSeries {
        #[serde(rename = "currentRatio")]
        pub current_ratio: Option<Vec<DataPoint>>,
        #[serde(rename = "salesPerShare")]
        pub sales_per_share: Option<Vec<DataPoint>>,
        #[serde(rename = "netMargin")]
        pub net_margin: Option<Vec<DataPoint>>,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct Series {
        pub annual: AnnualSeries,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct Metric {
        #[serde(rename = "10DayAverageTradingVolume")]
        pub avg_trading_volume_10_day: Option<f64>,
        #[serde(rename = "52WeekHigh")]
        pub week_high_52: f64,
        #[serde(rename = "52WeekLow")]
        pub week_low_52: f64,
        #[serde(rename = "52WeekLowDate")]
        pub week_low_date_52: String,
        #[serde(rename = "52WeekPriceReturnDaily")]
        pub week_price_return_daily_52: Option<f64>,
        pub beta: f64,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct CompanyFinancials {
        pub series: Series,
        pub metric: Metric,
        #[serde(rename = "metricType")]
        pub metric_type: String,
        pub symbol: String,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct SentimentData {
        pub symbol: String, // Symbol of the company
        pub year: u32,      // Year of the transaction
        pub month: u32,     // Month of the transaction
        pub change: i32,    // Net buying/selling from all insiders' transactions
        pub mspr: f64,      // Monthly share purchase ratio
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct RawSentData {
        pub data: Vec<SentimentData>,
        pub symbol: String,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct EarningsReport {
        pub symbol: String, // Company symbol
        pub year: u32,      // Fiscal year
        pub quarter: u32,   // Fiscal quarter
        pub period: String, // Reported period (e.g., "2023-03-31")
        pub actual: f64,    // Actual earning result
        pub estimate: f64,  // Estimated earning
        pub surprise: f64,  // Surprise - The difference between actual and estimate
        #[serde(rename = "surprisePercent")]
        pub surprise_percent: f64, // Surprise percent
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct EarningsCalendar {
        #[serde(rename = "earningsCalendar")]
        pub earnings_calendar: Vec<EarningsRelease>,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct EarningsRelease {
        pub date: String, // Date of the earnings release
        #[serde(rename = "epsActual")]
        pub eps_actual: f64, // Actual EPS (Earnings Per Share)
        #[serde(rename = "epsEstimate")]
        pub eps_estimate: f64, // Estimated EPS
        pub hour: String, // Time of the earnings release (bmo, amc, or dmh)
        pub quarter: u32, // Fiscal quarter
        #[serde(rename = "revenueActual")]
        pub revenue_actual: u64, // Actual revenue
        #[serde(rename = "revenueEstimate")]
        pub revenue_estimate: u64, // Estimated revenue
        pub symbol: String, // Company symbol
        pub year: u32,    // Fiscal year
    }
}
