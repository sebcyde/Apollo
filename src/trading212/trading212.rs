pub mod index {
    use std::time::SystemTime;

    use chrono::{offset::LocalResult, DateTime, Datelike, NaiveDateTime, TimeZone, Timelike, Utc};

    fn is_after_start_time(start_time: &str, order_date: &str) -> Option<bool> {
        let format: &str = "%d-%m %H:%M:%S";
        let naive_dt = NaiveDateTime::parse_from_str(start_time, format).unwrap();
        let current_year: i32 = Utc::now().year();
        let local_cycle_start_time = Utc.with_ymd_and_hms(
            current_year,
            naive_dt.month(),
            naive_dt.day(),
            naive_dt.hour(),
            naive_dt.minute(),
            naive_dt.second(),
        );
        /////
        let parsed_order_date = DateTime::parse_from_rfc3339(order_date)
            .expect("S: Failed to parse order date.")
            .with_timezone(&Utc);

        let cycle_start_time: Option<DateTime<Utc>> = match local_cycle_start_time {
            LocalResult::None => {
                println!("S: Local start time conversion failed.");
                None
            }
            LocalResult::Single(datetime) => Some(datetime),
            LocalResult::Ambiguous(datetime1, datetime2) => {
                println!(
                    "S: Conversion Ambigous Results: {:?} // {:?}",
                    &datetime1, &datetime2
                );
                println!("S: Using the first possible value.");
                Some(datetime1)
            }
        };

        if cycle_start_time.is_some() {
            return Some(parsed_order_date > cycle_start_time.unwrap());
        }

        return None;
    }
}
