//! Date functions: TODAY, NOW, DATE, YEAR, MONTH, DAY, EDATE, EOMONTH, NETWORKDAYS.
//!
//! Excel serial dates: day 1 = January 1, 1900 (serial number 1).
//! We implement date arithmetic without external crates using simple algorithms.

use crate::formula_engine::evaluator::{ErrorKind, Value};

// ---------------------------------------------------------------------------
// Date conversion helpers
// ---------------------------------------------------------------------------

/// Convert (year, month, day) to Excel serial date number.
/// Handles the Excel Lotus 1-2-3 bug: Feb 29, 1900 is treated as valid (serial 60).
fn date_to_serial(year: i32, month: i32, day: i32) -> Option<f64> {
    // Normalize month/day with rollover
    let (y, m) = normalize_year_month(year, month);
    let d = day;

    // Calculate the actual date after day rollover
    let jdn = to_jdn(y, m, d);
    // Excel epoch: Jan 1, 1900 = serial 1
    // JDN of Dec 31, 1899 = 2415020
    let excel_epoch_jdn = 2415020;
    let serial = jdn - excel_epoch_jdn;

    if serial < 1 {
        return None;
    }

    // Excel bug: serial 60 = Feb 29, 1900 (doesn't exist)
    // So dates after Feb 28, 1900 (serial 59) are off by 1
    let adjusted = if serial >= 60 { serial + 1 } else { serial };

    Some(adjusted as f64)
}

/// Convert Excel serial date to (year, month, day).
fn serial_to_date(serial: f64) -> Option<(i32, i32, i32)> {
    let mut s = serial as i64;
    if s < 1 {
        return None;
    }

    // Handle Excel's Lotus bug: serial 60 = Feb 29, 1900
    if s == 60 {
        return Some((1900, 2, 29));
    }
    // Adjust for the phantom Feb 29, 1900
    if s > 60 {
        s -= 1;
    }

    let excel_epoch_jdn = 2415020i64;
    let jdn = s + excel_epoch_jdn;
    from_jdn(jdn as i32)
}

/// Normalize year and month so month is in 1..12, rolling over as needed.
fn normalize_year_month(year: i32, month: i32) -> (i32, i32) {
    let m0 = month - 1; // 0-based
    let y = year + m0.div_euclid(12);
    let m = m0.rem_euclid(12) + 1;
    (y, m)
}

/// Julian Day Number from (year, month, day).
fn to_jdn(year: i32, month: i32, day: i32) -> i32 {
    let a = (14 - month) / 12;
    let y = year + 4800 - a;
    let m = month + 12 * a - 3;
    day + (153 * m + 2) / 5 + 365 * y + y / 4 - y / 100 + y / 400 - 32045
}

/// (year, month, day) from Julian Day Number.
fn from_jdn(jdn: i32) -> Option<(i32, i32, i32)> {
    let a = jdn + 32044;
    let b = (4 * a + 3) / 146097;
    let c = a - (146097 * b) / 4;
    let d = (4 * c + 3) / 1461;
    let e = c - (1461 * d) / 4;
    let m = (5 * e + 2) / 153;
    let day = e - (153 * m + 2) / 5 + 1;
    let month = m + 3 - 12 * (m / 10);
    let year = 100 * b + d - 4800 + m / 10;
    Some((year, month, day))
}

/// Check if a year is a leap year.
fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

/// Days in a given month.
fn days_in_month(year: i32, month: i32) -> i32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => if is_leap_year(year) { 29 } else { 28 },
        _ => 30,
    }
}

/// Day of week: 0=Monday, 1=Tuesday, ..., 4=Friday, 5=Saturday, 6=Sunday
fn day_of_week(year: i32, month: i32, day: i32) -> i32 {
    let jdn = to_jdn(year, month, day);
    // JDN % 7: 0=Mon, 1=Tue, 2=Wed, 3=Thu, 4=Fri, 5=Sat, 6=Sun
    jdn.rem_euclid(7)
}

// ---------------------------------------------------------------------------
// TODAY
// ---------------------------------------------------------------------------

/// TODAY()
/// Returns the current date as an Excel serial number.
pub fn fn_today(_args: &[Value]) -> Value {
    // Use system time
    #[cfg(not(test))]
    {
        use std::time::{SystemTime, UNIX_EPOCH};
        let secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        // Unix epoch = Jan 1, 1970 = Excel serial 25569
        let days = (secs / 86400) as f64;
        Value::Number(days + 25569.0)
    }
    #[cfg(test)]
    {
        // Fixed date for testing: Jan 1, 2024 = serial 45292
        Value::Number(45292.0)
    }
}

// ---------------------------------------------------------------------------
// NOW
// ---------------------------------------------------------------------------

/// NOW()
/// Returns the current date and time as an Excel serial number.
pub fn fn_now(_args: &[Value]) -> Value {
    #[cfg(not(test))]
    {
        use std::time::{SystemTime, UNIX_EPOCH};
        let secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs_f64();
        let days = secs / 86400.0;
        Value::Number(days + 25569.0)
    }
    #[cfg(test)]
    {
        // Fixed: Jan 1, 2024 12:00:00 = 45292.5
        Value::Number(45292.5)
    }
}

// ---------------------------------------------------------------------------
// DATE
// ---------------------------------------------------------------------------

/// DATE(year, month, day)
/// Returns the serial number for a date. Handles rollover for out-of-range values.
pub fn fn_date(args: &[Value]) -> Value {
    if args.len() != 3 {
        return Value::Error(ErrorKind::Value);
    }
    let year = match args[0].to_number() {
        Ok(n) => {
            let y = n as i32;
            // Excel: 0-29 → 2000-2029, 30-99 → 1930-1999, else as-is
            if (0..30).contains(&y) {
                y + 2000
            } else if (30..100).contains(&y) {
                y + 1900
            } else {
                y
            }
        }
        Err(e) => return Value::Error(e),
    };
    let month = match args[1].to_number() {
        Ok(n) => n as i32,
        Err(e) => return Value::Error(e),
    };
    let day = match args[2].to_number() {
        Ok(n) => n as i32,
        Err(e) => return Value::Error(e),
    };

    match date_to_serial(year, month, day) {
        Some(serial) => Value::Number(serial),
        None => Value::Error(ErrorKind::Num),
    }
}

// ---------------------------------------------------------------------------
// YEAR, MONTH, DAY
// ---------------------------------------------------------------------------

/// YEAR(serial_number)
pub fn fn_year(args: &[Value]) -> Value {
    if args.len() != 1 {
        return Value::Error(ErrorKind::Value);
    }
    let serial = match args[0].to_number() {
        Ok(n) => n,
        Err(e) => return Value::Error(e),
    };
    match serial_to_date(serial) {
        Some((y, _, _)) => Value::Number(y as f64),
        None => Value::Error(ErrorKind::Num),
    }
}

/// MONTH(serial_number)
pub fn fn_month(args: &[Value]) -> Value {
    if args.len() != 1 {
        return Value::Error(ErrorKind::Value);
    }
    let serial = match args[0].to_number() {
        Ok(n) => n,
        Err(e) => return Value::Error(e),
    };
    match serial_to_date(serial) {
        Some((_, m, _)) => Value::Number(m as f64),
        None => Value::Error(ErrorKind::Num),
    }
}

/// DAY(serial_number)
pub fn fn_day(args: &[Value]) -> Value {
    if args.len() != 1 {
        return Value::Error(ErrorKind::Value);
    }
    let serial = match args[0].to_number() {
        Ok(n) => n,
        Err(e) => return Value::Error(e),
    };
    match serial_to_date(serial) {
        Some((_, _, d)) => Value::Number(d as f64),
        None => Value::Error(ErrorKind::Num),
    }
}

// ---------------------------------------------------------------------------
// EDATE
// ---------------------------------------------------------------------------

/// EDATE(start_date, months)
/// Returns the serial number of the date that is the indicated number of months
/// before or after the start date.
pub fn fn_edate(args: &[Value]) -> Value {
    if args.len() != 2 {
        return Value::Error(ErrorKind::Value);
    }
    let serial = match args[0].to_number() {
        Ok(n) => n,
        Err(e) => return Value::Error(e),
    };
    let months = match args[1].to_number() {
        Ok(n) => n as i32,
        Err(e) => return Value::Error(e),
    };

    let (y, m, d) = match serial_to_date(serial) {
        Some(ymd) => ymd,
        None => return Value::Error(ErrorKind::Num),
    };

    let (new_y, new_m) = normalize_year_month(y, m + months);
    let max_day = days_in_month(new_y, new_m);
    let new_d = d.min(max_day);

    match date_to_serial(new_y, new_m, new_d) {
        Some(s) => Value::Number(s),
        None => Value::Error(ErrorKind::Num),
    }
}

// ---------------------------------------------------------------------------
// EOMONTH
// ---------------------------------------------------------------------------

/// EOMONTH(start_date, months)
/// Returns the serial number of the last day of the month that is the indicated
/// number of months before or after start_date.
pub fn fn_eomonth(args: &[Value]) -> Value {
    if args.len() != 2 {
        return Value::Error(ErrorKind::Value);
    }
    let serial = match args[0].to_number() {
        Ok(n) => n,
        Err(e) => return Value::Error(e),
    };
    let months = match args[1].to_number() {
        Ok(n) => n as i32,
        Err(e) => return Value::Error(e),
    };

    let (y, m, _) = match serial_to_date(serial) {
        Some(ymd) => ymd,
        None => return Value::Error(ErrorKind::Num),
    };

    let (new_y, new_m) = normalize_year_month(y, m + months);
    let last_day = days_in_month(new_y, new_m);

    match date_to_serial(new_y, new_m, last_day) {
        Some(s) => Value::Number(s),
        None => Value::Error(ErrorKind::Num),
    }
}

// ---------------------------------------------------------------------------
// NETWORKDAYS
// ---------------------------------------------------------------------------

/// NETWORKDAYS(start_date, end_date, [holidays])
/// Returns the number of whole working days between two dates (excluding weekends).
/// Holidays array is optional.
pub fn fn_networkdays(args: &[Value]) -> Value {
    if args.is_empty() || args.len() > 3 {
        return Value::Error(ErrorKind::Value);
    }
    let start_serial = match args[0].to_number() {
        Ok(n) => n as i64,
        Err(e) => return Value::Error(e),
    };
    let end_serial = match args[1].to_number() {
        Ok(n) => n as i64,
        Err(e) => return Value::Error(e),
    };

    // Collect holidays
    let mut holidays = std::collections::HashSet::new();
    if args.len() == 3 {
        match &args[2] {
            Value::Array(rows) => {
                for row in rows {
                    for cell in row {
                        match cell.to_number() {
                            Ok(n) => { holidays.insert(n as i64); }
                            Err(_) => {} // skip non-numeric
                        }
                    }
                }
            }
            Value::Number(n) => { holidays.insert(*n as i64); }
            _ => {}
        }
    }

    let (from, to, sign) = if start_serial <= end_serial {
        (start_serial, end_serial, 1i64)
    } else {
        (end_serial, start_serial, -1i64)
    };

    let mut count = 0i64;
    for serial in from..=to {
        if holidays.contains(&serial) {
            continue;
        }
        // Check if it's a weekday
        if let Some((y, m, d)) = serial_to_date(serial as f64) {
            let dow = day_of_week(y, m, d); // 0=Mon..6=Sun
            if dow < 5 {
                // Monday-Friday
                count += 1;
            }
        }
    }

    Value::Number((count * sign) as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn num(n: f64) -> Value {
        Value::Number(n)
    }

    #[test]
    fn test_date_basic() {
        // Jan 1, 1900 = serial 1
        assert_eq!(fn_date(&[num(1900.0), num(1.0), num(1.0)]), num(1.0));
        // Jan 1, 2024 = serial 45292
        assert_eq!(fn_date(&[num(2024.0), num(1.0), num(1.0)]), num(45292.0));
    }

    #[test]
    fn test_date_rollover() {
        // Month 13 = January of next year
        assert_eq!(
            fn_date(&[num(2023.0), num(13.0), num(1.0)]),
            fn_date(&[num(2024.0), num(1.0), num(1.0)])
        );
        // Month 0 = December of previous year
        assert_eq!(
            fn_date(&[num(2024.0), num(0.0), num(1.0)]),
            fn_date(&[num(2023.0), num(12.0), num(1.0)])
        );
    }

    #[test]
    fn test_year_month_day() {
        let serial = fn_date(&[num(2024.0), num(3.0), num(15.0)]);
        if let Value::Number(s) = serial {
            assert_eq!(fn_year(&[num(s)]), num(2024.0));
            assert_eq!(fn_month(&[num(s)]), num(3.0));
            assert_eq!(fn_day(&[num(s)]), num(15.0));
        } else {
            panic!("Expected number from DATE");
        }
    }

    #[test]
    fn test_year_month_day_epoch() {
        // Serial 1 = Jan 1, 1900
        assert_eq!(fn_year(&[num(1.0)]), num(1900.0));
        assert_eq!(fn_month(&[num(1.0)]), num(1.0));
        assert_eq!(fn_day(&[num(1.0)]), num(1.0));
    }

    #[test]
    fn test_edate() {
        let jan15 = fn_date(&[num(2024.0), num(1.0), num(15.0)]);
        if let Value::Number(s) = jan15 {
            let result = fn_edate(&[num(s), num(1.0)]);
            // Should be Feb 15, 2024
            if let Value::Number(r) = result {
                assert_eq!(fn_month(&[num(r)]), num(2.0));
                assert_eq!(fn_day(&[num(r)]), num(15.0));
            } else {
                panic!("Expected number from EDATE");
            }
        }
    }

    #[test]
    fn test_edate_end_of_month() {
        // Jan 31 + 1 month = Feb 29 (2024 is leap year)
        let jan31 = fn_date(&[num(2024.0), num(1.0), num(31.0)]);
        if let Value::Number(s) = jan31 {
            let result = fn_edate(&[num(s), num(1.0)]);
            if let Value::Number(r) = result {
                assert_eq!(fn_month(&[num(r)]), num(2.0));
                assert_eq!(fn_day(&[num(r)]), num(29.0));
            }
        }
    }

    #[test]
    fn test_eomonth() {
        let jan15 = fn_date(&[num(2024.0), num(1.0), num(15.0)]);
        if let Value::Number(s) = jan15 {
            let result = fn_eomonth(&[num(s), num(0.0)]);
            // End of January = Jan 31
            if let Value::Number(r) = result {
                assert_eq!(fn_day(&[num(r)]), num(31.0));
                assert_eq!(fn_month(&[num(r)]), num(1.0));
            }
            // End of February 2024 (leap year) = Feb 29
            let result2 = fn_eomonth(&[num(s), num(1.0)]);
            if let Value::Number(r) = result2 {
                assert_eq!(fn_day(&[num(r)]), num(29.0));
                assert_eq!(fn_month(&[num(r)]), num(2.0));
            }
        }
    }

    #[test]
    fn test_networkdays() {
        // Mon Jan 1, 2024 to Fri Jan 5, 2024 = 5 working days
        let start = fn_date(&[num(2024.0), num(1.0), num(1.0)]);
        let end = fn_date(&[num(2024.0), num(1.0), num(5.0)]);
        if let (Value::Number(s), Value::Number(e)) = (start, end) {
            assert_eq!(fn_networkdays(&[num(s), num(e)]), num(5.0));
        }
    }

    #[test]
    fn test_today_now() {
        // In test mode, these return fixed values
        if let Value::Number(t) = fn_today(&[]) {
            assert_eq!(t, 45292.0);
        }
        if let Value::Number(n) = fn_now(&[]) {
            assert_eq!(n, 45292.5);
        }
    }

    #[test]
    fn test_date_two_digit_year() {
        // 0-29 → 2000-2029
        assert_eq!(
            fn_date(&[num(24.0), num(1.0), num(1.0)]),
            fn_date(&[num(2024.0), num(1.0), num(1.0)])
        );
        // 30-99 → 1930-1999
        assert_eq!(
            fn_date(&[num(99.0), num(1.0), num(1.0)]),
            fn_date(&[num(1999.0), num(1.0), num(1.0)])
        );
    }
}
