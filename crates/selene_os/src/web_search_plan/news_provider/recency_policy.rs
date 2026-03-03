#![forbid(unsafe_code)]

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImportanceTier {
    Low,
    Medium,
    High,
}

impl ImportanceTier {
    pub fn parse(raw: &str) -> Option<Self> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "low" => Some(Self::Low),
            "medium" => Some(Self::Medium),
            "high" => Some(Self::High),
            _ => None,
        }
    }
}

pub fn recency_window_days(tier: ImportanceTier) -> i64 {
    match tier {
        ImportanceTier::Low => 30,
        ImportanceTier::Medium => 14,
        ImportanceTier::High => 7,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NormalizedPublishedAt {
    pub epoch_ms: i64,
    pub utc_rfc3339: String,
}

pub fn normalize_published_at(raw: &str) -> Option<NormalizedPublishedAt> {
    parse_rfc3339_like(raw).map(|epoch_ms| NormalizedPublishedAt {
        epoch_ms,
        utc_rfc3339: epoch_ms_to_utc_rfc3339(epoch_ms),
    })
}

pub fn freshness_score(published_at_ms: Option<i64>, now_ms: i64, window_days: i64) -> f64 {
    let Some(published) = published_at_ms else {
        return 0.25;
    };

    let age_ms = now_ms.saturating_sub(published).max(0);
    let window_ms = window_days.saturating_mul(86_400_000).max(1);

    if age_ms > window_ms {
        0.0
    } else {
        let remaining = window_ms.saturating_sub(age_ms);
        (remaining as f64 / window_ms as f64).max(0.01)
    }
}

pub fn within_recency_window(published_at_ms: Option<i64>, now_ms: i64, window_days: i64) -> bool {
    let Some(published) = published_at_ms else {
        return true;
    };
    let age_ms = now_ms.saturating_sub(published).max(0);
    age_ms <= window_days.saturating_mul(86_400_000)
}

fn parse_rfc3339_like(raw: &str) -> Option<i64> {
    let input = raw.trim();
    if input.is_empty() {
        return None;
    }

    if let Some(parsed) = parse_compact_utc(input) {
        return Some(parsed);
    }

    parse_iso_utc_or_offset(input)
}

fn parse_compact_utc(input: &str) -> Option<i64> {
    if input.len() != 16 || !input.ends_with('Z') {
        return None;
    }
    let bytes = input.as_bytes();
    if bytes[8] != b'T' {
        return None;
    }

    let year = parse_i32(&input[0..4])?;
    let month = parse_u32(&input[4..6])?;
    let day = parse_u32(&input[6..8])?;
    let hour = parse_u32(&input[9..11])?;
    let minute = parse_u32(&input[11..13])?;
    let second = parse_u32(&input[13..15])?;

    build_epoch_ms(year, month, day, hour, minute, second, 0)
}

fn parse_iso_utc_or_offset(input: &str) -> Option<i64> {
    if input.len() < 20 {
        return None;
    }

    let year = parse_i32(input.get(0..4)?)?;
    if input.get(4..5)? != "-" {
        return None;
    }
    let month = parse_u32(input.get(5..7)?)?;
    if input.get(7..8)? != "-" {
        return None;
    }
    let day = parse_u32(input.get(8..10)?)?;
    if input.get(10..11)? != "T" {
        return None;
    }
    let hour = parse_u32(input.get(11..13)?)?;
    if input.get(13..14)? != ":" {
        return None;
    }
    let minute = parse_u32(input.get(14..16)?)?;
    if input.get(16..17)? != ":" {
        return None;
    }
    let second = parse_u32(input.get(17..19)?)?;

    let tz_start = find_timezone_start(input)?;
    let tz_part = input.get(tz_start..)?;

    let offset_seconds = if tz_part == "Z" {
        0
    } else {
        parse_offset_seconds(tz_part)?
    };

    build_epoch_ms(year, month, day, hour, minute, second, offset_seconds)
}

fn find_timezone_start(input: &str) -> Option<usize> {
    if input.ends_with('Z') {
        return Some(input.len() - 1);
    }

    for (idx, ch) in input.char_indices().rev() {
        if (ch == '+' || ch == '-') && idx >= 19 {
            return Some(idx);
        }
    }
    None
}

fn parse_offset_seconds(offset: &str) -> Option<i32> {
    if offset.len() != 6 {
        return None;
    }
    let sign = match &offset[0..1] {
        "+" => 1,
        "-" => -1,
        _ => return None,
    };
    if &offset[3..4] != ":" {
        return None;
    }
    let hours = parse_i32(&offset[1..3])?;
    let minutes = parse_i32(&offset[4..6])?;
    Some(sign * (hours * 3600 + minutes * 60))
}

fn build_epoch_ms(
    year: i32,
    month: u32,
    day: u32,
    hour: u32,
    minute: u32,
    second: u32,
    offset_seconds: i32,
) -> Option<i64> {
    if !(1..=12).contains(&month)
        || !(1..=31).contains(&day)
        || hour > 23
        || minute > 59
        || second > 59
    {
        return None;
    }

    let days = days_from_civil(year, month, day)?;
    let local_seconds = days
        .saturating_mul(86_400)
        .saturating_add(hour as i64 * 3_600)
        .saturating_add(minute as i64 * 60)
        .saturating_add(second as i64);
    let utc_seconds = local_seconds.saturating_sub(offset_seconds as i64);
    Some(utc_seconds.saturating_mul(1_000))
}

fn days_from_civil(year: i32, month: u32, day: u32) -> Option<i64> {
    let y = year - if month <= 2 { 1 } else { 0 };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;
    let m = month as i32;
    let d = day as i32;
    let doy = (153 * (m + if m > 2 { -3 } else { 9 }) + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    Some(era as i64 * 146_097 + doe as i64 - 719_468)
}

fn epoch_ms_to_utc_rfc3339(epoch_ms: i64) -> String {
    let seconds = epoch_ms.div_euclid(1_000);
    let days = seconds.div_euclid(86_400);
    let seconds_of_day = seconds.rem_euclid(86_400);

    let (year, month, day) = civil_from_days(days);
    let hour = seconds_of_day / 3_600;
    let minute = (seconds_of_day % 3_600) / 60;
    let second = seconds_of_day % 60;

    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        year, month, day, hour, minute, second
    )
}

fn civil_from_days(days_since_epoch: i64) -> (i64, i64, i64) {
    let z = days_since_epoch + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = mp + if mp < 10 { 3 } else { -9 };
    let year = y + if month <= 2 { 1 } else { 0 };
    (year, month, day)
}

fn parse_i32(raw: &str) -> Option<i32> {
    raw.parse::<i32>().ok()
}

fn parse_u32(raw: &str) -> Option<u32> {
    raw.parse::<u32>().ok()
}
