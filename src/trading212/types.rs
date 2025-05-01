pub mod types {
    use std::time::SystemTime;

    use chrono::{DateTime, Utc};
    use serde::{Deserialize, Deserializer, Serialize};
    use serde_json::Value;

    #[derive(Debug)]
    pub struct SystemLimitOrder {
        pub movement_direction: MOVEMENT_DIRECTION,
        pub creation_time: SystemTime,
        pub limit_order: LimitOrder,
        pub sell_attempts: i32,
    }

    #[derive(Debug)]
    pub enum MOVEMENT_DIRECTION {
        UP,
        DOWN,
    }

    #[derive(Debug, Serialize, Deserialize, Default)]
    pub struct BalanceObject {
        pub blocked: Option<f64>,
        pub free: f64,
        pub invested: f64,
        #[serde(rename = "pieCash")]
        pub pie_cash: f64,
        pub ppl: f64,
        pub result: f64,
        pub total: f64,
    }

    #[derive(Debug, Serialize, Deserialize, Clone)]
    pub struct Position {
        #[serde(rename = "averagePrice")]
        pub average_price: f64,
        #[serde(rename = "currentPrice")]
        pub current_price: f64,
        pub frontend: String,
        #[serde(rename = "fxPpl")]
        pub fx_ppl: Option<f64>, // Use Option<f64> to handle potential null values
        #[serde(rename = "initialFillDate")]
        pub initial_fill_date: String,
        #[serde(rename = "maxBuy")]
        pub max_buy: f64,
        #[serde(rename = "maxSell")]
        pub max_sell: f64,
        #[serde(rename = "pieQuantity")]
        pub pie_quantity: f64,
        pub ppl: f64,
        pub quantity: f64,
        pub ticker: String,
    }

    #[derive(Debug, Serialize, Deserialize, Clone)]
    pub struct Instrument {
        #[serde(rename = "addedOn")]
        pub added_on: String,
        #[serde(rename = "currencyCode")]
        pub currency_code: String,
        pub isin: String,
        #[serde(rename = "maxOpenQuantity")]
        pub max_open_quantity: f64,
        #[serde(rename = "minTradeQuantity")]
        pub min_trade_quantity: f64,
        pub name: String,
        pub shortname: Option<String>,
        pub ticker: String,
        #[serde(rename = "type")]
        pub asset_type: String,
        #[serde(rename = "workingScheduleId")]
        pub working_schedule_id: i64,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct MarketOrder {
        #[serde(rename = "creationTime")]
        pub creation_time: String,

        #[serde(rename = "filledQuantity")]
        pub filled_quantity: f64,

        #[serde(rename = "filledValue")]
        pub filled_value: Option<f64>,

        #[serde(rename = "id")]
        pub id: u64,

        #[serde(rename = "limitPrice")]
        pub limit_price: Option<f64>,

        #[serde(rename = "quantity")]
        pub quantity: f64,

        #[serde(rename = "status")]
        pub status: String,

        #[serde(rename = "stopPrice")]
        pub stop_price: Option<f64>,

        #[serde(rename = "strategy")]
        pub strategy: String,

        #[serde(rename = "ticker")]
        pub ticker: String,

        #[serde(rename = "type")]
        pub order_type: String,

        #[serde(rename = "value")]
        pub value: Option<f64>,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct LimitOrder {
        #[serde(rename = "creationTime")]
        pub creation_time: String,

        #[serde(rename = "filledQuantity")]
        pub filled_quantity: f64,

        #[serde(rename = "filledValue")]
        pub filled_value: Option<f64>,

        #[serde(rename = "id")]
        pub id: u64,

        #[serde(rename = "limitPrice")]
        pub limit_price: Option<f64>,

        #[serde(rename = "quantity")]
        pub quantity: f64,

        #[serde(rename = "status")]
        pub status: String,

        #[serde(rename = "stopPrice")]
        pub stop_price: Option<f64>,

        #[serde(rename = "strategy")]
        pub strategy: String,

        #[serde(rename = "ticker")]
        pub ticker: String,

        #[serde(rename = "type")]
        pub order_type: String,

        #[serde(rename = "value")]
        pub value: Option<f64>,
    }

    fn deserialize_datetime_to_string<'de, D>(deserializer: D) -> Result<String, D::Error>
    where
        D: Deserializer<'de>,
    {
        let datetime_str: &str = Deserialize::deserialize(deserializer)?;
        let datetime: DateTime<Utc> = DateTime::parse_from_rfc3339(datetime_str)
            .map_err(serde::de::Error::custom)?
            .with_timezone(&Utc);
        Ok(datetime.to_rfc3339())
    }

    fn deserialize_optional_datetime_to_string<'de, D>(
        deserializer: D,
    ) -> Result<Option<String>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let option: Option<&str> = Deserialize::deserialize(deserializer)?;
        if let Some(datetime_str) = option {
            let datetime: DateTime<Utc> = DateTime::parse_from_rfc3339(datetime_str)
                .map_err(serde::de::Error::custom)?
                .with_timezone(&Utc);
            Ok(Some(datetime.to_rfc3339()))
        } else {
            Ok(None)
        }
    }

    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub struct HistoricalOrder {
        #[serde(rename = "type")]
        pub order_type: String,
        pub id: u64,
        pub fill_id: u64,
        pub parent_order: u64,
        pub ticker: String,
        pub ordered_quantity: f64,
        pub filled_quantity: f64,
        pub limit_price: f64,
        pub stop_price: Option<f64>,
        pub time_validity: Option<String>,
        pub ordered_value: Option<f64>,
        pub filled_value: Option<f64>,
        pub executor: String,
        pub date_modified: String,
        pub date_executed: Option<String>,
        pub date_created: String,
        pub fill_result: Option<String>,
        pub fill_price: f64,
        pub fill_cost: Option<f64>,
        pub taxes: Vec<HistoricalTaxItem>,
        pub fill_type: String,
        pub status: String,
    }

    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub struct HistoricalTaxItem {
        pub fill_id: String,
        pub name: String,
        pub quantity: f64,
        pub time_charged: String,
    }
}
