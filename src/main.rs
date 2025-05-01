use governor::{Quota, RateLimiter};
use std::num::NonZeroU32;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() {
    println!("Starting Apollo...");

    // Pull full list of stock tickers

    // Randomly choose 300 tickers and note the names/tickers

    // Three seperate threads, go through the 300. 100 each pulling data

    //

    // Allow 30 requests per second globally
    let quota = Quota::per_second(NonZeroU32::new(30).unwrap());
    let limiter = Arc::new(RateLimiter::direct(quota));

    let mut handles = vec![];

    for i in 0..3 {
        let limiter = Arc::clone(&limiter);
        let handle = tokio::spawn(async move {
            for j in 0..600 {
                limiter.until_ready().await; // this waits until allowed
                println!("Thread {} - making request {}", i, j);
                // your API call would go here
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await.unwrap();
    }
}

// im working on a stock trading bot called Apollo. Well its mainly a screener / shortlister, ill add the buying and selling functionality later. The api im getting the stock data from has a rate limit
