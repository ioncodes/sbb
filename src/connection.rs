use chrono::{NaiveTime, Duration};

pub struct Connection {
    pub departure_name: String,
    pub departure_date: NaiveTime,
    pub arrival_name: String,
    pub arrival_date: NaiveTime,
    pub duration: Duration,
    pub platform: String,
}