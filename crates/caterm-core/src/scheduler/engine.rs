//! Engine scheduling loop & schedule evaluation (REQ-38).

use chrono::{DateTime, Duration, Local, NaiveTime, TimeZone, Utc};
use std::str::FromStr;

/// Parse schedule expression to calculate next run time.
/// Supports:
/// 1. "every Nh" -> every N hours (e.g. "every 6h", "every 1h")
/// 2. "every Nm" -> every N minutes (e.g. "every 30m", "every 5m")
/// 3. "every Nd" -> every N days (e.g. "every 1d")
/// 4. "at HH:MM" -> daily at specific 24h time (e.g. "at 02:00")
/// 5. Cron syntax: "min hour day_of_month month day_of_week" (e.g. "0 2 * * *")
pub fn calculate_next_run(
    schedule_expr: &str,
    after: Option<DateTime<Utc>>,
) -> Result<DateTime<Utc>, crate::error::CatermError> {
    let now = after.unwrap_or_else(Utc::now);
    let expr = schedule_expr.trim();

    if expr.is_empty() {
        return Err(crate::error::ValidationError::Generic("Schedule expression cannot be empty".to_string()).into());
    }

    // 1. Interval: "every <N><unit>"
    if let Some(rest) = expr.strip_prefix("every ") {
        let trimmed = rest.trim();
        if let Some(num_str) = trimmed.strip_suffix('h') {
            let hours: i64 = num_str
                .trim()
                .parse()
                .map_err(|_| crate::error::ValidationError::Generic(format!("Invalid hours in schedule: '{expr}'")))?;
            if hours <= 0 {
                return Err(crate::error::ValidationError::Generic("Interval hours must be > 0".to_string()).into());
            }
            return Ok(now + Duration::hours(hours));
        } else if let Some(num_str) = trimmed.strip_suffix('m') {
            let minutes: i64 = num_str
                .trim()
                .parse()
                .map_err(|_| crate::error::ValidationError::Generic(format!("Invalid minutes in schedule: '{expr}'")))?;
            if minutes <= 0 {
                return Err(crate::error::ValidationError::Generic("Interval minutes must be > 0".to_string()).into());
            }
            return Ok(now + Duration::minutes(minutes));
        } else if let Some(num_str) = trimmed.strip_suffix('d') {
            let days: i64 = num_str
                .trim()
                .parse()
                .map_err(|_| crate::error::ValidationError::Generic(format!("Invalid days in schedule: '{expr}'")))?;
            if days <= 0 {
                return Err(crate::error::ValidationError::Generic("Interval days must be > 0".to_string()).into());
            }
            return Ok(now + Duration::days(days));
        }
    }

    // 2. Specific daily time: "at HH:MM"
    if let Some(time_str) = expr.strip_prefix("at ") {
        let time = NaiveTime::from_str(time_str.trim())
            .map_err(|_| crate::error::ValidationError::Generic(format!("Invalid time format in schedule (expected HH:MM): '{expr}'")))?;

        let local_now = Local::now();
        let mut target_local_date = local_now.date_naive();
        let target_naive_dt = target_local_date.and_time(time);
        let target_local_dt = Local
            .from_local_datetime(&target_naive_dt)
            .single()
            .ok_or_else(|| crate::error::ValidationError::Generic("Ambiguous or invalid local time".to_string()))?;

        let target_utc = target_local_dt.with_timezone(&Utc);
        if target_utc <= now {
            // Schedule for next day
            target_local_date += chrono::Duration::days(1);
            let next_dt = target_local_date.and_time(time);
            let next_local = Local
                .from_local_datetime(&next_dt)
                .single()
                .ok_or_else(|| crate::error::ValidationError::Generic("Ambiguous or invalid local time".to_string()))?;
            return Ok(next_local.with_timezone(&Utc));
        } else {
            return Ok(target_utc);
        }
    }

    // 3. 5-part cron syntax: "min hour dom month dow"
    // e.g. "0 2 * * *"
    if let Ok(next) = parse_cron_5_part(expr, now) {
        return Ok(next);
    }

    Err(crate::error::ValidationError::Generic(format!("Unsupported schedule expression format: '{expr}'")).into())
}

/// A lightweight 5-part cron evaluator for standard expressions like "0 2 * * *"
fn parse_cron_5_part(expr: &str, now: DateTime<Utc>) -> Result<DateTime<Utc>, String> {
    let parts: Vec<&str> = expr.split_whitespace().collect();
    if parts.len() != 5 {
        return Err("Cron must have 5 fields (min hour dom month dow)".to_string());
    }

    let min_part = parts[0];
    let hour_part = parts[1];
    let _dom_part = parts[2];
    let _mon_part = parts[3];
    let _dow_part = parts[4];

    // Simple matching for common forms:
    // e.g. "0 2 * * *" (Every day at 02:00 UTC/Local)
    let min: u32 = if min_part == "*" {
        0
    } else {
        min_part
            .parse()
            .map_err(|_| "Invalid cron minute".to_string())?
    };

    let hour: u32 = if hour_part == "*" {
        0
    } else {
        hour_part
            .parse()
            .map_err(|_| "Invalid cron hour".to_string())?
    };

    if min >= 60 || hour >= 24 {
        return Err("Cron time out of range".to_string());
    }

    let target_time =
        NaiveTime::from_hms_opt(hour, min, 0).ok_or_else(|| "Invalid time in cron".to_string())?;

    // Look for the next occurrence starting from `now`
    let mut candidate_date = now.date_naive();
    let candidate_dt = candidate_date.and_time(target_time);
    let mut candidate_utc = Utc.from_utc_datetime(&candidate_dt);

    if candidate_utc <= now {
        candidate_date += chrono::Duration::days(1);
        let next_dt = candidate_date.and_time(target_time);
        candidate_utc = Utc.from_utc_datetime(&next_dt);
    }

    Ok(candidate_utc)
}
