/// Excel serial date/time.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ExcelDateTime {
    serial: f64,
    is_1904: bool,
}

impl ExcelDateTime {
    pub fn new(serial: f64, is_1904: bool) -> Self { Self { serial, is_1904 } }
    pub fn serial(&self) -> f64 { self.serial }

    /// Create from year/month/day.
    pub fn from_ymd(year: i32, month: u32, day: u32) -> Option<Self> {
        Self::from_ymd_hms(year, month, day, 0, 0, 0)
    }

    /// Create from year/month/day/hour/minute/second.
    pub fn from_ymd_hms(year: i32, month: u32, day: u32, hour: u32, min: u32, sec: u32) -> Option<Self> {
        if month < 1 || month > 12 || day < 1 || day > 31 { return None; }
        let serial = ymd_to_serial(year, month, day)?;
        let time_frac = (hour as f64 * 3600.0 + min as f64 * 60.0 + sec as f64) / 86400.0;
        Some(Self { serial: serial as f64 + time_frac, is_1904: false })
    }

    /// Parse "2024-01-15" or "2024-01-15T10:30:00".
    pub fn parse(s: &str) -> Option<Self> {
        let s = s.trim();
        if s.len() < 10 { return None; }
        let year: i32 = s[0..4].parse().ok()?;
        if s.as_bytes()[4] != b'-' { return None; }
        let month: u32 = s[5..7].parse().ok()?;
        if s.as_bytes()[7] != b'-' { return None; }
        let day: u32 = s[8..10].parse().ok()?;

        if s.len() == 10 {
            return Self::from_ymd(year, month, day);
        }
        if s.len() >= 19 && (s.as_bytes()[10] == b'T' || s.as_bytes()[10] == b' ') {
            let hour: u32 = s[11..13].parse().ok()?;
            let min: u32 = s[14..16].parse().ok()?;
            let sec: u32 = s[17..19].parse().ok()?;
            return Self::from_ymd_hms(year, month, day, hour, min, sec);
        }
        None
    }

    /// Convert serial date to (year, month, day, hour, min, sec).
    pub fn to_ymd_hms(&self) -> (i32, u32, u32, u32, u32, u32) {
        let base = if self.is_1904 { self.serial + 1462.0 } else { self.serial };
        serial_to_ymd_hms(base)
    }

    /// Format as ISO 8601 string.
    pub fn to_iso_string(&self) -> String {
        let (y, m, d, h, mi, s) = self.to_ymd_hms();
        if h == 0 && mi == 0 && s == 0 {
            format!("{y:04}-{m:02}-{d:02}")
        } else {
            format!("{y:04}-{m:02}-{d:02}T{h:02}:{mi:02}:{s:02}")
        }
    }
}

/// Convert y/m/d to Excel serial date (1900 epoch).
fn ymd_to_serial(year: i32, month: u32, day: u32) -> Option<i64> {
    if year < 1900 || month < 1 || month > 12 || day < 1 || day > 31 { return None; }

    // Count days from 1900-01-01 (which is serial date 1)
    let mut days: i64 = 0;

    // Add days for complete years
    for y in 1900..year {
        days += if is_leap(y) { 366 } else { 365 };
    }

    // Add days for complete months in the target year
    let month_days: [u32; 12] = if is_leap(year) {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };
    for m in 0..(month as usize - 1) {
        days += month_days[m] as i64;
    }

    // Add the day
    days += day as i64;

    // Excel serial: 1900-01-01 = 1
    // Excel has a bug: it thinks 1900 is a leap year, so Feb 29, 1900 = serial 60
    // All dates after Feb 28, 1900 (serial 59) need +1 to account for this phantom day
    if days >= 60 {
        days += 1; // account for the phantom Feb 29, 1900
    }

    Some(days)
}

fn is_leap(y: i32) -> bool {
    (y % 4 == 0 && y % 100 != 0) || y % 400 == 0
}

/// Convert Excel serial date to y/m/d/h/m/s.
fn serial_to_ymd_hms(serial: f64) -> (i32, u32, u32, u32, u32, u32) {
    let mut day_num = serial.floor() as i64;
    let time_frac = serial - serial.floor();

    if day_num <= 0 { return (1900, 1, 1, 0, 0, 0); }
    if day_num == 60 { return (1900, 2, 29, 0, 0, 0); } // Excel's phantom leap day

    // Undo the phantom leap day adjustment
    if day_num > 60 { day_num -= 1; }

    // Now day_num is days since 1900-01-01 where 1900-01-01 = 1
    day_num -= 1; // make 0-based (1900-01-01 = 0)

    let mut year = 1900i32;
    loop {
        let days_in_year = if is_leap(year) { 366 } else { 365 };
        if day_num < days_in_year {
            break;
        }
        day_num -= days_in_year;
        year += 1;
    }

    let month_days: [i64; 12] = if is_leap(year) {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };

    let mut month = 1u32;
    for &md in &month_days {
        if day_num < md { break; }
        day_num -= md;
        month += 1;
    }
    let day = day_num as u32 + 1;

    let total_secs = (time_frac * 86400.0).round() as u32;
    let hour = total_secs / 3600;
    let min = (total_secs % 3600) / 60;
    let sec = total_secs % 60;

    (year, month, day, hour, min, sec)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_dates() {
        // Jan 1, 1900 = serial 1
        let dt = ExcelDateTime::from_ymd(1900, 1, 1).unwrap();
        assert_eq!(dt.serial() as i64, 1);

        // Jan 1, 2024 = serial 45292
        let dt = ExcelDateTime::from_ymd(2024, 1, 1).unwrap();
        assert_eq!(dt.serial() as i64, 45292);
    }

    #[test]
    fn roundtrip() {
        for &(y, m, d) in &[(2024, 1, 15), (1900, 3, 1), (2000, 12, 31), (2099, 6, 15)] {
            let dt = ExcelDateTime::from_ymd(y, m, d).unwrap();
            let (ry, rm, rd, _, _, _) = dt.to_ymd_hms();
            assert_eq!((ry, rm, rd), (y, m, d), "failed for {y}-{m}-{d}");
        }
    }

    #[test]
    fn with_time() {
        let dt = ExcelDateTime::from_ymd_hms(2024, 1, 15, 10, 30, 0).unwrap();
        let (y, m, d, h, mi, s) = dt.to_ymd_hms();
        assert_eq!((y, m, d, h, mi, s), (2024, 1, 15, 10, 30, 0));
    }

    #[test]
    fn parse_iso() {
        let dt = ExcelDateTime::parse("2024-01-15").unwrap();
        assert_eq!(dt.to_iso_string(), "2024-01-15");

        let dt = ExcelDateTime::parse("2024-01-15T10:30:00").unwrap();
        assert_eq!(dt.to_iso_string(), "2024-01-15T10:30:00");
    }
}
