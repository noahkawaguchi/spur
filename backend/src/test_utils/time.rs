use chrono::{DateTime, Utc};

/// Finds whether two `DateTime<Utc>`s are within 1000ms of each other.
pub fn within_one_second(dt1: DateTime<Utc>, dt2: DateTime<Utc>) -> bool {
    (dt1 - dt2).num_milliseconds().abs() < 1000
}

/// Finds whether two `Option<DateTime<Utc>>`s are either both `None` or within 1000ms of each
/// other.
pub fn both_none_or_within_one_second(
    maybe_dt1: Option<DateTime<Utc>>,
    maybe_dt2: Option<DateTime<Utc>>,
) -> bool {
    (maybe_dt1.is_none() && maybe_dt2.is_none())
        || (maybe_dt1.is_some_and(|dt1| maybe_dt2.is_some_and(|dt2| within_one_second(dt1, dt2))))
}
