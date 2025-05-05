pub mod filtering {
    use crate::{helpers::types::types::FullCompanyInfo, FILTER_STRICTNESS, MINIMUM_MARKET_CAP};

    // Could add in sentiment creation based on recent company news

    pub fn stock_passes_filters(company: &FullCompanyInfo) -> bool {
        println!("\nST: --- Applying filters...");

        if !filter_market_cap(company) {
            println!("ST: --- Filtering FAILED\n");
            return false;
        }

        if !filter_volume(company) {
            println!("ST: --- Filtering FAILED\n");
            return false;
        }
        if !filter_volatility(company) {
            println!("ST: --- Filtering FAILED\n");
            return false;
        }
        if !filter_daily_price_performance(company) {
            println!("ST: --- Filtering FAILED\n");
            return false;
        }

        match *FILTER_STRICTNESS {
            crate::FILTERING_STRICTNESS::STRONG => {
                if !filter_insider_activity(company) {
                    println!("ST: --- Filtering FAILED\n");
                    return false;
                }
            }
            crate::FILTERING_STRICTNESS::LIGHT => {}
        };

        println!("ST: --- Filtering complete.\n");
        return true;
    }

    fn filter_market_cap(company: &FullCompanyInfo) -> bool {
        let min_mkt_cap: f64 = match *FILTER_STRICTNESS {
            crate::FILTERING_STRICTNESS::STRONG => *MINIMUM_MARKET_CAP,
            crate::FILTERING_STRICTNESS::LIGHT => *MINIMUM_MARKET_CAP / 2.0,
        };

        if company.company_info.market_capitalization < min_mkt_cap {
            println!(
                "ST: FAILED - Market cap of {}m is below acceptable parameters.",
                company.company_info.market_capitalization.trunc() as i64
            );
            return false;
        }
        println!(
            "ST: PASSED - Market cap of {}m is within acceptable parameters.",
            company.company_info.market_capitalization.trunc() as i64
        );
        return true;
    }

    fn filter_daily_price_performance(company: &FullCompanyInfo) -> bool {
        if company.company_stock_quote.current_price > company.company_stock_quote.todays_open {
            println!("ST: PASSED - Daily price performance is acceptable.");
            return true;
        }
        println!("ST: FAILED - Downard trending price performance detected.");
        return false;
    }

    fn filter_recent_price_performance(company: &FullCompanyInfo) -> bool {
        if company
            .company_financials
            .metric
            .week_price_return_daily_52
            .is_some_and(|price| price > 10.0)
        {
            println!(
                "ST: PASSED - 52 week daily price return of {} is acceptable.",
                company
                    .company_financials
                    .metric
                    .week_price_return_daily_52
                    .unwrap()
            );
            return true;
        }
        println!("ST: FAILED - Downard trending recent price performance detected.");
        return false;
    }

    fn filter_volume(company: &FullCompanyInfo) -> bool {
        if company
            .company_financials
            .metric
            .avg_trading_volume_10_day
            .is_none()
        {
            println!("ST: Unable to check volume. No metric recieved.");
            return false;
        }

        let volume_to_mcap_ratio: f64 = (company
            .company_financials
            .metric
            .avg_trading_volume_10_day
            .unwrap()
            * company.company_stock_quote.current_price)
            / company.company_info.market_capitalization
            * 100.0;

        if volume_to_mcap_ratio > 0.1 {
            println!(
                "ST: PASSED - Trading volume of {} is within acceptable parameters.",
                format!("{:.3}", volume_to_mcap_ratio)
            );
            return true;
        }

        println!(
            "ST: FAILED - Trading volume of {} is below acceptable paremeters.",
            format!("{:.3}", volume_to_mcap_ratio)
        );
        return false;
    }

    fn filter_volatility(company: &FullCompanyInfo) -> bool {
        let vol_metric: f64 = company.company_financials.metric.beta;

        if vol_metric < 0.5 {
            println!(
                "ST: FAILED - Volatility of {} is outside acceptable parameters.",
                format!("{:.3}", vol_metric)
            );
            return false;
        }

        println!(
            "ST: PASSED - Volatility of {} is within acceptable parameters.",
            format!("{:.3}", vol_metric)
        );

        return true;
    }

    // fn filter_moving_average(company: &FullCompanyInfo) -> bool {
    //     println!("ST: PASSED - Moving average is acceptable.");
    //     return true;
    // }

    // fn filter_relative_strength_index(company: &FullCompanyInfo) -> bool {
    //     println!("ST: PASSED - Relative strength is acceptable.");
    //     return true;
    // }

    // fn filter_revenue_growth(company: &FullCompanyInfo) -> bool {
    //     println!("ST: PASSED - Revenue growth is acceptable.");
    //     return true;
    // }

    // fn filter_earnings_per_share(company: &FullCompanyInfo) -> bool {
    //     println!("ST: PASSED - EPS is acceptable.");
    //     return true;
    // }

    // fn filter_sector_performance(company: &FullCompanyInfo) -> bool {
    //     println!("ST: PASSED - Sector performance is acceptable.");
    //     return true;
    // }

    fn filter_liquidity(company: &FullCompanyInfo) -> bool {
        if company
            .company_financials
            .metric
            .avg_trading_volume_10_day
            .is_none()
        {
            println!("ST: Unable to check liquidity. No metric recieved.");
            return false;
        }

        let turnover_ratio: f64 = company
            .company_financials
            .metric
            .avg_trading_volume_10_day
            .unwrap()
            / company.company_info.market_capitalization;

        // 0.5% - more than 1% is high liquidity
        if turnover_ratio > 0.005 {
            println!(
                "ST: PASSED - Liquidity of {} is acceptable.",
                turnover_ratio
            );
            return true;
        }

        println!(
            "ST: FAILED - Liquidity of {} is outside of acceptable parameters.",
            turnover_ratio
        );
        return false;
    }

    // fn filter_time_of_day(company: &FullCompanyInfo) -> bool {
    //     println!("ST: PASSED - Time of day is acceptable.");
    //     return true;
    // }

    fn filter_insider_activity(company: &FullCompanyInfo) -> bool {
        if company.insider_transactions.is_none() {
            println!("ST: PASSED - No insider activity detected.");
            return true;
        }

        let total_change: i64 = company
            .insider_transactions
            .as_ref()
            .unwrap()
            .iter()
            .map(|t| t.change)
            .sum();

        if total_change > 0 {
            println!("ST: PASSED - Insider activity has strong buying trend.");
            return true;
        }

        println!("ST: FAILED - Insider activity has strong selling trend.");
        return false;
    }

    // fn filter_short_interest(company: &FullCompanyInfo) -> bool {
    //     println!("ST: PASSED - Short interest is acceptable.");
    //     return true;
    // }
}
