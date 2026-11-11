use anyhow::{anyhow, Result};
use chrono::{DateTime, Datelike, NaiveTime, TimeZone, Weekday};
use chrono_tz::Tz;

pub fn generate_current_title(template: &str, target_due: &DateTime<Tz>) -> String {
    let iso_week = target_due.iso_week().week();
    let kw = format!("KW{:02}", iso_week);
    template.replace("{calender_week}", &kw)
}

pub fn generate_regex_pattern(template: &str) -> String {
    let escaped = regex::escape(template);
    let replaced = escaped.replace("\\{calender_week\\}", r"KW\d{2}");
    format!("^{}$", replaced)
}

/// Finds the very next occurrence of the specified day/time from `now`.
pub fn next_occurrence(
    tz: &Tz,
    day_str: &str,
    time_str: &str,
    now: DateTime<Tz>,
) -> Result<DateTime<Tz>> {
    let weekday = parse_weekday(day_str)?;
    let time = NaiveTime::parse_from_str(time_str, "%H:%M")
        .map_err(|_| anyhow!("Invalid time format, use HH:MM"))?;

    let mut candidate_naive = now.date_naive().and_time(time);
    let days_offset = (weekday.num_days_from_monday() as i64)
        - (candidate_naive.weekday().num_days_from_monday() as i64);
    candidate_naive += chrono::Duration::days(days_offset);

    let mut candidate = bind_timezone(tz, candidate_naive);

    // If it has already passed this week, roll to next week
    if candidate < now {
        candidate_naive += chrono::Duration::days(7);
        candidate = bind_timezone(tz, candidate_naive);
    }

    Ok(candidate)
}

pub fn add_repetition(date: &DateTime<Tz>, repetition: &str) -> Result<DateTime<Tz>> {
    let parts: Vec<&str> = repetition.trim().split_whitespace().collect();
    if parts.len() != 2 {
        return Err(anyhow!("Repetition must be '<number> <unit>'"));
    }

    let amount: i64 = parts[0].parse().map_err(|_| anyhow!("Invalid number"))?;
    let unit = parts[1].to_lowercase();

    let new_date = match unit.as_str() {
        "day" | "days" => *date + chrono::Duration::days(amount),
        "week" | "weeks" => *date + chrono::Duration::weeks(amount),
        "month" | "months" => date
            .checked_add_months(chrono::Months::new(amount as u32))
            .unwrap_or(*date),
        _ => return Err(anyhow!("Unsupported repetition unit: {}", unit)),
    };

    Ok(new_date)
}

fn parse_weekday(day: &str) -> Result<Weekday> {
    match day.to_lowercase().as_str() {
        "monday" => Ok(Weekday::Mon),
        "tuesday" => Ok(Weekday::Tue),
        "wednesday" => Ok(Weekday::Wed),
        "thursday" => Ok(Weekday::Thu),
        "friday" => Ok(Weekday::Fri),
        "saturday" => Ok(Weekday::Sat),
        "sunday" => Ok(Weekday::Sun),
        _ => Err(anyhow!("Invalid weekday literal")),
    }
}

fn bind_timezone(tz: &Tz, naive: chrono::NaiveDateTime) -> DateTime<Tz> {
    match tz.from_local_datetime(&naive) {
        chrono::LocalResult::Single(dt) => dt,
        chrono::LocalResult::Ambiguous(dt, _) => dt,
        chrono::LocalResult::None => tz.from_utc_datetime(&naive),
    }
}