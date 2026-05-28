use std::fmt;
use std::time::SystemTime;

const WEEKDAY_NAMES: &[&str] = &["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
const MONTH_NAMES: &[&str] = &[
    "Jan", "Feb", "Mar", "Apr", "May", "Jun",
    "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

pub fn fmt_http_date(w: &mut fmt::Formatter<'_>, time: SystemTime) -> fmt::Result {
    let dur = time.duration_since(SystemTime::UNIX_EPOCH).unwrap_or_default();
    let secs = dur.as_secs();
    let days = secs / 86400;
    let remaining = secs % 86400;
    let hours = remaining / 3600;
    let minutes = (remaining % 3600) / 60;
    let seconds = remaining % 60;

    let weekday_idx = ((days as i64 + 4).rem_euclid(7)) as usize;
    let (y, m, d) = civil_from_days(days as i64);

    write!(
        w,
        "{}, {:02} {} {:04} {:02}:{:02}:{:02} GMT",
        WEEKDAY_NAMES[weekday_idx],
        d,
        MONTH_NAMES[(m - 1) as usize],
        y,
        hours,
        minutes,
        seconds
    )
}

fn civil_from_days(days: i64) -> (i64, i64, i64) {
    let z = days + 719468;
    let era = if z >= 0 { z } else { z - 146096 };
    let doe = era.rem_euclid(146097);
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era / 146097 * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let dd = doy - (153 * mp + 2) / 5 + 1;
    let mm = if mp < 10 { mp + 3 } else { mp - 9 };
    let yy = if mm <= 2 { y + 1 } else { y };
    (if yy <= 0 { yy - 1 } else { yy }, mm, dd)
}
