pub mod seed_data;
pub mod temp_db;

use chrono::{DateTime, Utc};

/// Finds whether two `DateTime`s are within 1000ms of each other.
pub fn within_one_second(d1: DateTime<Utc>, d2: DateTime<Utc>) -> bool {
    (d1 - d2).num_milliseconds().abs() < 1000
}
