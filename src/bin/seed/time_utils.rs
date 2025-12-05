use anyhow::{Context, Result};
use chrono::{DateTime, FixedOffset, NaiveDate, TimeDelta, Utc};

/// Adds the provided days, hours, and minutes to the anchor `DateTime`, 2019-04-30 15:00:00 UTC
/// (the beginning of the Reiwa era, somewhat arbitrarily chosen for a recent "time 0").
pub fn anchor_offset(days: u32, hours: u32, minutes: u32) -> Result<DateTime<Utc>> {
    let reiwa = reiwa_start_utc().context("failed to create anchor date")?;

    let offset = TimeDelta::try_days(days.into())
        .and_then(|d| Some(d + TimeDelta::try_hours(hours.into())?))
        .and_then(|dh| Some(dh + TimeDelta::try_minutes(minutes.into())?))
        .context("failed to create offset TimeDelta")?;

    Ok(reiwa + offset)
}

/// Converts the beginning of the Reiwa era (2019-05-01 00:00:00 JST/UTC+9) to UTC (2019-04-30
/// 15:00:00 UTC).
fn reiwa_start_utc() -> Option<DateTime<Utc>> {
    NaiveDate::from_ymd_opt(2019, 5, 1)?
        .and_hms_opt(0, 0, 0)?
        .and_local_timezone(FixedOffset::east_opt(9 * 3600)?)
        .single()
        .map(|reiwa_start| reiwa_start.with_timezone(&Utc))
}
