//! Real system-date helpers shared by the generator and the audit exporter.

/// Today's date as `YYYY-MM-DD`, read from the system clock.
pub fn real_date_yyyymmdd() -> String {
    let days = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() / 86400)
        .unwrap_or(0);
    let (y, m, d) = civil_from_days(days as i64);
    format!("{:04}-{:02}-{:02}", y, m, d)
}

/// Gregorian date from days since the Unix epoch (Howard Hinnant's algorithm).
pub fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = (if mp < 10 { mp + 3 } else { mp - 9 }) as u32;
    (if m <= 2 { y + 1 } else { y }, m, d)
}

/// Three-letter weekday name for a Gregorian date.
pub fn day_of_week(y: i64, m: u32, d: u32) -> &'static str {
    static OFFSETS: [i64; 12] = [0, 3, 2, 5, 0, 3, 5, 1, 4, 6, 2, 4];
    let mut yy = y;
    if m < 3 {
        yy -= 1;
    }
    let dow = (yy + yy / 4 - yy / 100 + yy / 400 + OFFSETS[(m as usize) - 1] + d as i64) % 7;
    ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"][dow as usize]
}

/// Three-letter month names, used by the RSS date format.
pub const MONTHS: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];
