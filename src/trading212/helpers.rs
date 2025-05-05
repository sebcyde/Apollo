pub mod helpers {

    use serde::{Deserialize, Serialize};

    use std::{env, time::SystemTime};

    use crate::{
        file_control::types::types::SaleResult,
        helpers::{
            helpers::helpers::{
                calc_perc_change, calculate_amount_spent_per_ticker, chop_two_after_dec,
                print_message, round_down, shuffle_positions, sleep_thread, THREAD,
            },
            types::types::FullCompanyInfo,
        },
        trading212::types::types::{
            BalanceObject, HistoricalOrder, Instrument, LimitOrder, MarketOrder, Position,
            SystemLimitOrder, MOVEMENT_DIRECTION,
        },
        SELL_PERCENT_DOWN, SELL_PERCENT_UP, STOCK_VERSION, VERSION,
    };

    use reqwest::{header, Body, Client, RequestBuilder, Response};
    use serde_json::{from_value, json, Value};
    use tokio;

    ////////////////////////// PIES //////////////////////////////////

    pub fn get_single_pie(pie_id: i32) -> Value {
        let query: String = format!(
            "https://demo.trading212.com/api/v0/equity/pies/{}",
            pie_id.to_string(),
        );

        let raw_data: Option<serde_json::Value> = make_request(query);
        let data: Value = raw_data.unwrap();

        // println!("Pie Data: {:?}", &data);
        return data;
    }

    pub fn get_all_pies() -> Vec<Value> {
        let query: String = match *STOCK_VERSION {
            VERSION::DEMO => String::from("https://demo.trading212.com/api/v0/equity/pies"),
            VERSION::LIVE => String::from("https://live.trading212.com/api/v0/equity/pies"),
        };

        let raw_data: Option<serde_json::Value> = make_request(query);
        let data: Vec<Value> = raw_data.unwrap().as_array().unwrap().to_owned();

        println!("\nST: All Pies Data: {:?}", &data);
        return data;
    }

    ////////////////////////// ACCOUNT BALANCE //////////////////////////////////

    pub fn get_account_balance() -> Result<BalanceObject, serde_json::Error> {
        println!("\nCT: Fetching account data...");

        let query: String = match *STOCK_VERSION {
            VERSION::DEMO => String::from("https://demo.trading212.com/api/v0/equity/account/cash"),
            VERSION::LIVE => String::from("https://live.trading212.com/api/v0/equity/account/cash"),
        };

        let raw_data: Option<serde_json::Value> = make_request(query);

        let balance: Result<BalanceObject, serde_json::Error> =
            serde_json::from_value(raw_data.unwrap());

        println!("ST: Done");
        return balance;
    }

    ////////////////////////// DIVIDENDS //////////////////////////////////

    pub fn get_paid_dividends() -> Value {
        let query: String = String::from("https://demo.trading212.com/api/v0/history/dividends");

        let raw_data: Option<serde_json::Value> = make_request(query);
        let data: Value = raw_data.unwrap();

        println!("Dividend Data: {:?}", &data);
        return data;
    }

    ////////////////////////// TRANSACTIONS //////////////////////////////////

    pub fn get_transaction_list() -> Value {
        let query: String = String::from("https://demo.trading212.com/api/v0/history/transactions");

        let raw_data: Option<serde_json::Value> = make_request(query);

        let data: Value = raw_data.unwrap();

        println!("Transaction Data: {:?}", &data);
        return data;
    }

    ////////////////////////// POSITIONS //////////////////////////////////

    pub fn get_all_positions() -> Option<Vec<Position>> {
        let query: &str = match *STOCK_VERSION {
            VERSION::DEMO => "https://demo.trading212.com/api/v0/equity/portfolio",
            VERSION::LIVE => "https://live.trading212.com/api/v0/equity/portfolio",
        };

        let raw_data: Option<serde_json::Value> = make_request(query.to_owned());

        let data_option: Option<Value> = match raw_data {
            None => None,
            Some(data) => Some(data),
        };

        if data_option.is_none() {
            println!("Failed to retreive position data.");
            return None;
        }

        let data: Value = data_option.unwrap();

        let mut positions: Vec<Position> =
            serde_json::from_value(data).expect("Failed to deserialise positions data.");

        positions = shuffle_positions(positions);

        return Some(positions);
    }

    pub fn get_single_position(ticker: &str) -> Option<Position> {
        let query: String = match *STOCK_VERSION {
            VERSION::DEMO => format!(
                "https://demo.trading212.com/api/v0/equity/portfolio/{}",
                ticker
            ),
            VERSION::LIVE => format!(
                "https://live.trading212.com/api/v0/equity/portfolio/{}",
                ticker
            ),
        };

        let raw_data: Option<serde_json::Value> = make_request(query);
        if raw_data.is_none() {
            return None;
        }

        let position: Position = serde_json::from_value(raw_data.unwrap())
            .expect("Failed to deserialise positions data.");

        return Some(position);
    }

    ////////////////////////// ORDERS //////////////////////////////////

    #[tokio::main]
    pub async fn create_market_order(ticker: String, quantity: f64) -> Option<MarketOrder> {
        let auth_token: String = match *STOCK_VERSION {
            VERSION::DEMO => env::var("DEMO_API_KEY").expect("DEMO_API_KEY must be set"),
            VERSION::LIVE => env::var("LIVE_API_KEY").expect("LIVE_API_KEY must be set"),
        };

        let endpoint: &str = match *STOCK_VERSION {
            VERSION::DEMO => "https://demo.trading212.com/api/v0/equity/orders/market",
            VERSION::LIVE => "https://live.trading212.com/api/v0/equity/orders/market",
        };

        let json_body: Value = json!({
            "quantity": quantity,
            "ticker": ticker
        });

        let client: Client = Client::new();

        println!("ST: Market Order Payload: {:?}", &json_body);

        let response: Response = client
            .post(endpoint)
            .header(header::AUTHORIZATION, auth_token)
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(serde_json::to_string(&json_body).unwrap()))
            .send()
            .await
            .unwrap();

        if response.status().is_success() {
            let body: String = response.text().await.unwrap();
            let result: Result<Value, serde_json::Error> = serde_json::from_str(&body);
            println!("ST: Market order successful.");

            let trade_type: &str = match quantity {
                x if x > 0.0 => "Bought",
                x if x < 0.0 => "Sold",
                _ => "Held",
            };

            match result {
                Ok(json) => {
                    // println!("Order JSON: {:?}", &json);
                    let content: String =
                        format!("{} {} shares of {}", trade_type, quantity.abs(), ticker);
                    println!("ST: {}\n", &content);

                    let res: Result<MarketOrder, serde_json::Error> = serde_json::from_value(json);

                    match res {
                        Ok(market_orders) => {
                            return Some(market_orders);
                        }
                        Err(e) => {
                            println!(
                                "ST: Failed to deserialize into MarketOrder vector: {:?}\n",
                                e
                            );
                            return None;
                        }
                    }
                }
                Err(e) => {
                    println!("Failed to deserialize JSON: {:?}", e);
                    return None;
                }
            }
        } else {
            println!("ST: Market order failed. Code: {}\n", &response.status());
            return None;
        }
    }

    pub enum TradeDirection {
        BUY,
        SELL,
    }

    #[tokio::main]
    pub async fn create_limit_order(
        ticker: String,
        price: f64,
        quantity: f64,
        trade_direction: TradeDirection,
    ) -> Option<LimitOrder> {
        let auth_token: String = match *STOCK_VERSION {
            VERSION::DEMO => env::var("DEMO_API_KEY").expect("DEMO_API_KEY must be set"),
            VERSION::LIVE => env::var("LIVE_API_KEY").expect("LIVE_API_KEY must be set"),
        };

        if ticker.to_ascii_lowercase().contains("vusa") {
            println!("Ignoring VUSA limit request.");
            return None;
        }

        let endpoint: &str = match *STOCK_VERSION {
            VERSION::DEMO => "https://demo.trading212.com/api/v0/equity/orders/limit",
            VERSION::LIVE => "https://live.trading212.com/api/v0/equity/orders/limit",
        };

        let json_body: Value = json!({
            "quantity": quantity,
            "ticker": ticker,
            "limitPrice": price,
            "timeValidity": "DAY"
        });

        let client: Client = Client::new();

        let thread_name: &str = match trade_direction {
            TradeDirection::BUY => "BT",
            TradeDirection::SELL => "ST",
        };

        // println!("\n{}: Limit Order Payload: {:?}", thread_name, &json_body);

        let resp = client
            .post(endpoint)
            .header(header::AUTHORIZATION, auth_token)
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(serde_json::to_string(&json_body).unwrap()))
            .send()
            .await;

        if resp.is_err() {
            println!("{}: Error in limit order response.\n", thread_name);
            return None;
        }

        let response: Response = resp.unwrap();

        // println!("\n{}: Pure Res: {:?}\n", thread_name, response);

        if response.status().is_success() {
            let body: String = response.text().await.unwrap();
            let result: Result<Value, serde_json::Error> = serde_json::from_str(&body);
            // println!("{}: Limit order creation successful.", thread_name);

            let trade_type: &str = match trade_direction {
                TradeDirection::BUY => "buy",
                TradeDirection::SELL => "sell",
            };

            match result {
                Ok(json) => {
                    // println!("Order JSON: {:?}", &json);
                    let content: String = format!(
                        "{}: Created limit order to {} {} shares of {} at {}",
                        thread_name,
                        trade_type,
                        quantity.abs(),
                        ticker,
                        price
                    );
                    println!("{}", &content);

                    let res: Result<LimitOrder, serde_json::Error> = serde_json::from_value(json);

                    match res {
                        Ok(limit_orders) => {
                            return Some(limit_orders);
                        }
                        Err(e) => {
                            println!(
                                "{}: Failed to deserialize into LimitOrder vector: {:?}",
                                thread_name, e
                            );
                            return None;
                        }
                    }
                }
                Err(e) => {
                    println!("Failed to deserialize JSON: {:?}", e);
                    return None;
                }
            }
        } else {
            println!(
                "{}: Limit order failed. Code: {:?}",
                thread_name,
                &response.status()
            );
            return None;
        }
    }

    #[tokio::main]
    pub async fn get_single_order(id: u64) -> Option<MarketOrder> {
        let auth_token: String = match *STOCK_VERSION {
            VERSION::DEMO => env::var("DEMO_API_KEY").expect("DEMO_API_KEY must be set"),
            VERSION::LIVE => env::var("LIVE_API_KEY").expect("LIVE_API_KEY must be set"),
        };

        let endpoint: &str = match *STOCK_VERSION {
            VERSION::DEMO => &format!("https://demo.trading212.com/api/v0/equity/orders/{}", id),
            VERSION::LIVE => &format!("https://live.trading212.com/api/v0/equity/orders/{}", id),
        };

        let client: Client = Client::new();

        let request: RequestBuilder = client
            .get(endpoint)
            .header(header::AUTHORIZATION, auth_token);

        let resp: Result<Response, reqwest::Error> = request.send().await;

        if resp.is_err() {
            println!("Error fetching single order.\n");
            return None;
        }

        let response: Response = resp.unwrap();

        if response.status().is_success() {
            let body: String = response.text().await.unwrap();
            let result: Result<Value, serde_json::Error> = serde_json::from_str(&body);

            match result {
                Ok(json) => {
                    let res: Result<MarketOrder, serde_json::Error> = serde_json::from_value(json);
                    match res {
                        Ok(market_order) => {
                            return Some(market_order);
                        }
                        Err(e) => {
                            println!("\nST: Failed to deserialize into MarketOrder: {:?}", e);
                            return None;
                        }
                    }
                }
                Err(e) => {
                    println!("\nST: Failed to deserialize JSON: {:?}", e);
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

    ////////////////////////////

    #[tokio::main]
    pub async fn get_all_historical_orders() -> Option<Vec<HistoricalOrder>> {
        let auth_token: String = match *STOCK_VERSION {
            VERSION::DEMO => env::var("DEMO_API_KEY").expect("DEMO_API_KEY must be set"),
            VERSION::LIVE => env::var("LIVE_API_KEY").expect("LIVE_API_KEY must be set"),
        };

        let endpoint: &str = match *STOCK_VERSION {
            VERSION::DEMO => "https://demo.trading212.com/api/v0/equity/history/orders?limit=50",
            VERSION::LIVE => "https://live.trading212.com/api/v0/equity/history/orders?limit=50",
        };

        let client: Client = Client::new();

        let request: RequestBuilder = client
            .get(endpoint)
            .header(header::AUTHORIZATION, auth_token);

        let response: Response = request.send().await.unwrap();

        if response.status().is_success() {
            let body: String = response.text().await.unwrap();
            let result: Result<Value, serde_json::Error> = serde_json::from_str(&body);

            match result {
                Ok(json) => {
                    let res: Result<Vec<HistoricalOrder>, serde_json::Error> =
                        serde_json::from_value(json);

                    match res {
                        Ok(historical_orders) => {
                            return Some(historical_orders);
                        }
                        Err(e) => {
                            println!(
                                "\nST: Failed to deserialize into HistoricalOrder vector: {:?}",
                                e
                            );
                            return None;
                        }
                    }
                }
                Err(e) => {
                    println!("\nST: Failed to deserialize HistoricalOrder JSON: {:?}", e);
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

    ////////////////////////////

    #[tokio::main]
    pub async fn get_all_orders_request() -> Option<Vec<LimitOrder>> {
        let auth_token: String = match *STOCK_VERSION {
            VERSION::DEMO => env::var("DEMO_API_KEY").expect("DEMO_API_KEY must be set"),
            VERSION::LIVE => env::var("LIVE_API_KEY").expect("LIVE_API_KEY must be set"),
        };

        let endpoint: &str = match *STOCK_VERSION {
            VERSION::DEMO => "https://demo.trading212.com/api/v0/equity/orders",
            VERSION::LIVE => "https://live.trading212.com/api/v0/equity/orders",
        };

        let client: Client = Client::new();

        let request: RequestBuilder = client
            .get(endpoint)
            .header(header::AUTHORIZATION, auth_token);

        let resp: Result<Response, reqwest::Error> = request.send().await;

        if resp.is_err() {
            println!("ST: Error fetching orders.\n");
            return None;
        }

        let response: Response = resp.unwrap();

        if response.status().is_success() {
            let body: String = response.text().await.unwrap();
            let result: Result<Value, serde_json::Error> = serde_json::from_str(&body);

            match result {
                Ok(json) => {
                    let res: Result<Vec<LimitOrder>, serde_json::Error> =
                        serde_json::from_value(json);

                    match res {
                        Ok(market_orders) => {
                            return Some(market_orders);
                        }
                        Err(e) => {
                            println!(
                                "\nST: Failed to deserialize into MarketOrder vector: {:?}",
                                e
                            );
                            return None;
                        }
                    }
                }
                Err(e) => {
                    println!("\nST: Failed to deserialize JSON: {:?}", e);
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
    pub async fn cancel_order(id: u64) -> Option<bool> {
        let auth_token: String = match *STOCK_VERSION {
            VERSION::DEMO => env::var("DEMO_API_KEY").expect("DEMO_API_KEY must be set"),
            VERSION::LIVE => env::var("LIVE_API_KEY").expect("LIVE_API_KEY must be set"),
        };

        let endpoint: String = match *STOCK_VERSION {
            VERSION::DEMO => {
                format!("https://demo.trading212.com/api/v0/equity/orders/{}", id)
            }
            VERSION::LIVE => {
                format!("https://live.trading212.com/api/v0/equity/orders/{}", id)
            }
        };

        let client: Client = Client::new();

        let resp = client
            .delete(endpoint)
            .header(header::AUTHORIZATION, auth_token)
            .send()
            .await;

        if resp.is_err() {
            println!("ST: Error cancelling limit order response.\n");
            return None;
        }

        let response: Response = resp.unwrap();

        if response.status().is_success() {
            Some(true)
        } else {
            println!("Order cancel failed. Code: {}\n", response.status());
            return None;
        }
    }

    // ////////////////////////// GENERAL //////////////////////////////////

    pub fn get_perc_increase(current_price: f64, perc_inc: f64) -> f64 {
        let percentage_as_decimal: f64 = perc_inc / 100.0;
        let increase: f64 = current_price * percentage_as_decimal;
        current_price + increase
    }

    pub fn get_perc_decrease(current_price: f64, perc_dec: f64) -> f64 {
        let percentage_as_decimal = perc_dec / 100.0;
        let decrease: f64 = current_price * percentage_as_decimal;
        current_price - decrease
    }

    pub fn get_sell_quant(position: &Position) -> f64 {
        let final_quant: f64;

        if position.quantity > position.max_sell {
            final_quant = position.max_sell
        } else {
            final_quant = position.quantity
        }

        return final_quant;
    }

    pub fn get_buy_quantity(company: &FullCompanyInfo, balance_data: &BalanceObject) -> f64 {
        // Calculating quantity to buy - rounded down to 2 decimal places
        let amount_to_spend: f64 = calculate_amount_spent_per_ticker(balance_data);
        let quant_to_buy: f64 = amount_to_spend / company.company_stock_quote.current_price;
        let rounded_quant: f64 = round_down(quant_to_buy, 2);
        let final_quant: f64;

        if rounded_quant > company.instrument.max_open_quantity {
            final_quant = company.instrument.max_open_quantity
        } else {
            final_quant = rounded_quant
        }

        final_quant
    }

    pub fn get_movement_direction(position: &Position) -> MOVEMENT_DIRECTION {
        return match position.average_price - position.current_price >= 0.0 {
            true => MOVEMENT_DIRECTION::DOWN,
            false => MOVEMENT_DIRECTION::UP,
        };
    }

    pub fn is_valid_perc_change(position: &Position) -> bool {
        let percentage_change: f64 = round_down(
            calc_perc_change(position.average_price, position.current_price),
            2,
        );

        return percentage_change > *SELL_PERCENT_UP || percentage_change < *SELL_PERCENT_DOWN;
    }

    // pub fn create_sale_object(sell_order: &LimitOrder, position: &Position) -> SaleResult {
    //     println!("\nCREATING SALE OBJECT ----",);

    //     let sale_rest: SaleResult = SaleResult {
    //         limit_order_id: sell_order.id,
    //         buy_price: chop_two_after_dec(position.average_price),
    //         sale_price: sell_order.limit_price.unwrap(),
    //         quantity_sold: transform_f64_to_pos(sell_order.quantity),
    //         ticker: sell_order.ticker.clone(),
    //         profit: chop_two_after_dec(position.average_price) - sell_order.limit_price.unwrap(),
    //         ppl: position.ppl,
    //     };

    //     println!("{:?}\n", sale_rest);
    //     sale_rest
    // }

    pub fn transform_f64_to_pos(number: f64) -> f64 {
        let positive_value: f64 = if number < 0.0 { number.abs() } else { number };
        return positive_value;
    }

    pub fn get_instruments() -> Vec<Instrument> {
        print_message(THREAD::COLLECTION, "Fetching new instruments data...");

        let query: String = match *STOCK_VERSION {
            VERSION::DEMO => {
                String::from("https://demo.trading212.com/api/v0/equity/metadata/instruments")
            }
            VERSION::LIVE => {
                String::from("https://live.trading212.com/api/v0/equity/metadata/instruments")
            }
        };

        let raw_data: Option<serde_json::Value> = make_request(query);
        let data: Value = raw_data.unwrap();
        print_message(THREAD::COLLECTION, "Fetch Successful.");

        return from_value(data).unwrap();
    }

    #[tokio::main]
    pub async fn make_request(endpoint: String) -> Option<Value> {
        let auth_token: String = match *STOCK_VERSION {
            VERSION::DEMO => env::var("DEMO_API_KEY").expect("DEMO_API_KEY must be set"),
            VERSION::LIVE => env::var("LIVE_API_KEY").expect("LIVE_API_KEY must be set"),
        };

        let client: Client = Client::new();

        let request: RequestBuilder = client
            .get(&endpoint)
            .header(header::AUTHORIZATION, auth_token);

        let resp: Result<Response, reqwest::Error> = request.send().await;

        if resp.is_err() {
            println!("Error making request.\n");
            return None;
        }

        let response: Response = resp.unwrap();

        if response.status().is_success() {
            let body: String = response.text().await.unwrap();
            let result: Result<Value, serde_json::Error> = serde_json::from_str(&body);

            match result {
                Ok(json) => {
                    // println!("Deserialized JSON: {:?}", json);
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

    pub fn killswitch() {
        println!("\n-------------- KILLSWITCH TRIGGERED --------------");

        let all_orders_res: Vec<LimitOrder> = get_all_orders_request().unwrap();
        println!("\nCancelling current orders...\n");

        for order in all_orders_res {
            println!("Cancelling Order: {:?} - {:?}", &order.id, &order.ticker);
            _ = cancel_order(order.id);
            sleep_thread(3);
        }

        println!("\nSelling current positions...\n");
        let raw_positions: Option<Vec<Position>> = get_all_positions();

        if raw_positions.is_none() {
            println!("\n\nKillswitch failed. Data was null.\n\n");
        }

        for position in raw_positions.unwrap() {
            println!(
                "Selling {:?} shares of {:?}...",
                position.quantity, position.ticker
            );
            let sell_quantity: f64 = -position.quantity;
            _ = create_market_order(position.ticker, sell_quantity);
            std::thread::sleep(std::time::Duration::from_secs(2))
        }

        println!("-------------- KILLSWITCH COMPLETE --------------\n");
    }
}
