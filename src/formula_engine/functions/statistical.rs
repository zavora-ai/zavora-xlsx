//! Statistical functions: STDEV, VAR, MEDIAN, PERCENTILE, RANK, COUNTIF, SUMIF.

use crate::formula_engine::evaluator::{ErrorKind, Value};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Extract numeric values from args, skipping non-numeric in arrays.
fn collect_numbers(args: &[Value]) -> Result<Vec<f64>, ErrorKind> {
    let mut nums = Vec::new();
    for arg in args {
        match arg {
            Value::Number(n) => nums.push(*n),
            Value::Array(rows) => {
                for row in rows {
                    for cell in row {
                        match cell {
                            Value::Number(n) => nums.push(*n),
                            Value::Error(e) => return Err(*e),
                            _ => {} // skip non-numeric in ranges
                        }
                    }
                }
            }
            Value::Bool(b) => nums.push(if *b { 1.0 } else { 0.0 }),
            Value::Empty => {}
            Value::String(s) => {
                if let Ok(n) = s.parse::<f64>() {
                    nums.push(n);
                }
            }
            Value::Error(e) => return Err(*e),
        }
    }
    Ok(nums)
}

// ---------------------------------------------------------------------------
// STDEV (sample standard deviation)
// ---------------------------------------------------------------------------

/// STDEV(number1, [number2], ...)
/// Returns the sample standard deviation. Requires at least 2 data points.
pub fn fn_stdev(args: &[Value]) -> Value {
    let nums = match collect_numbers(args) {
        Ok(v) => v,
        Err(e) => return Value::Error(e),
    };
    if nums.len() < 2 {
        return Value::Error(ErrorKind::Div0);
    }
    let mean = nums.iter().sum::<f64>() / nums.len() as f64;
    let variance = nums.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / (nums.len() - 1) as f64;
    Value::Number(variance.sqrt())
}

// ---------------------------------------------------------------------------
// VAR (sample variance)
// ---------------------------------------------------------------------------

/// VAR(number1, [number2], ...)
/// Returns the sample variance. Requires at least 2 data points.
pub fn fn_var(args: &[Value]) -> Value {
    let nums = match collect_numbers(args) {
        Ok(v) => v,
        Err(e) => return Value::Error(e),
    };
    if nums.len() < 2 {
        return Value::Error(ErrorKind::Div0);
    }
    let mean = nums.iter().sum::<f64>() / nums.len() as f64;
    let variance = nums.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / (nums.len() - 1) as f64;
    Value::Number(variance)
}

// ---------------------------------------------------------------------------
// MEDIAN
// ---------------------------------------------------------------------------

/// MEDIAN(number1, [number2], ...)
/// Returns the median of the given numbers.
pub fn fn_median(args: &[Value]) -> Value {
    let mut nums = match collect_numbers(args) {
        Ok(v) => v,
        Err(e) => return Value::Error(e),
    };
    if nums.is_empty() {
        return Value::Error(ErrorKind::Num);
    }
    nums.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let len = nums.len();
    if len % 2 == 0 {
        Value::Number((nums[len / 2 - 1] + nums[len / 2]) / 2.0)
    } else {
        Value::Number(nums[len / 2])
    }
}

// ---------------------------------------------------------------------------
// PERCENTILE
// ---------------------------------------------------------------------------

/// PERCENTILE(array, k)
/// Returns the k-th percentile of values in a range. k is between 0 and 1.
pub fn fn_percentile(args: &[Value]) -> Value {
    if args.len() != 2 {
        return Value::Error(ErrorKind::Value);
    }
    let mut nums = match collect_numbers(&args[0..1]) {
        Ok(v) => v,
        Err(e) => return Value::Error(e),
    };
    if nums.is_empty() {
        return Value::Error(ErrorKind::Num);
    }
    let k = match args[1].to_number() {
        Ok(n) => n,
        Err(e) => return Value::Error(e),
    };
    if !(0.0..=1.0).contains(&k) {
        return Value::Error(ErrorKind::Num);
    }
    nums.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let n = nums.len() as f64;
    let rank = k * (n - 1.0);
    let lower = rank.floor() as usize;
    let upper = rank.ceil() as usize;
    if lower == upper || upper >= nums.len() {
        Value::Number(nums[lower])
    } else {
        let frac = rank - lower as f64;
        Value::Number(nums[lower] + frac * (nums[upper] - nums[lower]))
    }
}

// ---------------------------------------------------------------------------
// RANK
// ---------------------------------------------------------------------------

/// RANK(number, ref, [order])
/// Returns the rank of a number in a list. order=0 (default) descending, order=1 ascending.
pub fn fn_rank(args: &[Value]) -> Value {
    if args.len() < 2 || args.len() > 3 {
        return Value::Error(ErrorKind::Value);
    }
    let number = match args[0].to_number() {
        Ok(n) => n,
        Err(e) => return Value::Error(e),
    };
    let nums = match collect_numbers(&args[1..2]) {
        Ok(v) => v,
        Err(e) => return Value::Error(e),
    };
    let order = if args.len() == 3 {
        match args[2].to_number() {
            Ok(n) => n as i32,
            Err(e) => return Value::Error(e),
        }
    } else {
        0
    };

    // Check if number exists in the list
    if !nums.iter().any(|&x| (x - number).abs() < f64::EPSILON) {
        return Value::Error(ErrorKind::Na);
    }

    let rank = if order == 0 {
        // Descending: count how many are greater + 1
        nums.iter().filter(|&&x| x > number + f64::EPSILON).count() + 1
    } else {
        // Ascending: count how many are smaller + 1
        nums.iter().filter(|&&x| x < number - f64::EPSILON).count() + 1
    };

    Value::Number(rank as f64)
}

// ---------------------------------------------------------------------------
// COUNTIF
// ---------------------------------------------------------------------------

/// COUNTIF(range, criteria)
/// Counts the number of cells that meet a criteria.
pub fn fn_countif(args: &[Value]) -> Value {
    if args.len() != 2 {
        return Value::Error(ErrorKind::Value);
    }
    let range = flatten_values(&args[0]);
    let criteria = parse_criteria(&args[1]);

    let count = range.iter().filter(|v| matches_criteria(v, &criteria)).count();
    Value::Number(count as f64)
}

// ---------------------------------------------------------------------------
// SUMIF
// ---------------------------------------------------------------------------

/// SUMIF(range, criteria, [sum_range])
/// Sums cells that meet a criteria. If sum_range is omitted, sums the range itself.
pub fn fn_sumif(args: &[Value]) -> Value {
    if args.is_empty() || args.len() > 3 {
        return Value::Error(ErrorKind::Value);
    }
    let range = flatten_values(&args[0]);
    let criteria = parse_criteria(&args[1]);
    let sum_range = if args.len() == 3 {
        flatten_values(&args[2])
    } else {
        range.clone()
    };

    let mut total = 0.0;
    for (i, val) in range.iter().enumerate() {
        if matches_criteria(val, &criteria) {
            if let Some(sum_val) = sum_range.get(i) {
                if let Ok(n) = sum_val.to_number() {
                    total += n;
                }
            }
        }
    }
    Value::Number(total)
}

// ---------------------------------------------------------------------------
// Criteria parsing
// ---------------------------------------------------------------------------

#[derive(Debug)]
enum Criteria {
    Equal(Value),
    NotEqual(Value),
    GreaterThan(f64),
    GreaterEqual(f64),
    LessThan(f64),
    LessEqual(f64),
    Wildcard(String), // simplified: * and ? patterns
}

fn parse_criteria(val: &Value) -> Criteria {
    match val {
        Value::String(s) => {
            let trimmed = s.trim();
            if let Some(rest) = trimmed.strip_prefix(">=") {
                if let Ok(n) = rest.trim().parse::<f64>() {
                    return Criteria::GreaterEqual(n);
                }
            }
            if let Some(rest) = trimmed.strip_prefix("<=") {
                if let Ok(n) = rest.trim().parse::<f64>() {
                    return Criteria::LessEqual(n);
                }
            }
            if let Some(rest) = trimmed.strip_prefix("<>") {
                let cmp_val = parse_criteria_value(rest.trim());
                return Criteria::NotEqual(cmp_val);
            }
            if let Some(rest) = trimmed.strip_prefix('>') {
                if let Ok(n) = rest.trim().parse::<f64>() {
                    return Criteria::GreaterThan(n);
                }
            }
            if let Some(rest) = trimmed.strip_prefix('<') {
                if let Ok(n) = rest.trim().parse::<f64>() {
                    return Criteria::LessThan(n);
                }
            }
            if let Some(rest) = trimmed.strip_prefix('=') {
                let cmp_val = parse_criteria_value(rest.trim());
                return Criteria::Equal(cmp_val);
            }
            // Check for wildcards
            if trimmed.contains('*') || trimmed.contains('?') {
                return Criteria::Wildcard(trimmed.to_string());
            }
            // Plain value comparison
            Criteria::Equal(parse_criteria_value(trimmed))
        }
        Value::Number(n) => Criteria::Equal(Value::Number(*n)),
        Value::Bool(b) => Criteria::Equal(Value::Bool(*b)),
        _ => Criteria::Equal(val.clone()),
    }
}

fn parse_criteria_value(s: &str) -> Value {
    if let Ok(n) = s.parse::<f64>() {
        Value::Number(n)
    } else {
        Value::String(s.to_string())
    }
}

fn matches_criteria(val: &Value, criteria: &Criteria) -> bool {
    match criteria {
        Criteria::Equal(cmp) => values_equal_criteria(val, cmp),
        Criteria::NotEqual(cmp) => !values_equal_criteria(val, cmp),
        Criteria::GreaterThan(n) => {
            if let Ok(v) = val.to_number() { v > *n } else { false }
        }
        Criteria::GreaterEqual(n) => {
            if let Ok(v) = val.to_number() { v >= *n } else { false }
        }
        Criteria::LessThan(n) => {
            if let Ok(v) = val.to_number() { v < *n } else { false }
        }
        Criteria::LessEqual(n) => {
            if let Ok(v) = val.to_number() { v <= *n } else { false }
        }
        Criteria::Wildcard(pattern) => {
            if let Ok(s) = val.to_string_val() {
                wildcard_match(&s.to_lowercase(), &pattern.to_lowercase())
            } else {
                false
            }
        }
    }
}

fn values_equal_criteria(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Number(x), Value::Number(y)) => (x - y).abs() < f64::EPSILON,
        (Value::String(x), Value::String(y)) => x.eq_ignore_ascii_case(y),
        (Value::Bool(x), Value::Bool(y)) => x == y,
        (Value::Empty, Value::Empty) => true,
        _ => false,
    }
}

/// Simple wildcard matching: * matches any sequence, ? matches any single char.
fn wildcard_match(text: &str, pattern: &str) -> bool {
    let t: Vec<char> = text.chars().collect();
    let p: Vec<char> = pattern.chars().collect();
    let (tlen, plen) = (t.len(), p.len());

    // DP approach
    let mut dp = vec![vec![false; plen + 1]; tlen + 1];
    dp[0][0] = true;

    // Handle leading *
    for j in 1..=plen {
        if p[j - 1] == '*' {
            dp[0][j] = dp[0][j - 1];
        }
    }

    for i in 1..=tlen {
        for j in 1..=plen {
            if p[j - 1] == '*' {
                dp[i][j] = dp[i][j - 1] || dp[i - 1][j];
            } else if p[j - 1] == '?' || p[j - 1] == t[i - 1] {
                dp[i][j] = dp[i - 1][j - 1];
            }
        }
    }

    dp[tlen][plen]
}

fn flatten_values(val: &Value) -> Vec<Value> {
    match val {
        Value::Array(rows) => {
            let mut flat = Vec::new();
            for row in rows {
                for cell in row {
                    flat.push(cell.clone());
                }
            }
            flat
        }
        other => vec![other.clone()],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn num(n: f64) -> Value {
        Value::Number(n)
    }
    fn s(val: &str) -> Value {
        Value::String(val.to_string())
    }

    fn nums_array(values: &[f64]) -> Value {
        Value::Array(vec![values.iter().map(|&n| num(n)).collect()])
    }

    #[test]
    fn test_stdev() {
        let data = nums_array(&[2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0]);
        if let Value::Number(result) = fn_stdev(&[data]) {
            // mean=5, sum_sq_dev=32, var=32/7≈4.571, stdev≈2.138
            assert!((result - 2.138).abs() < 0.01);
        } else {
            panic!("Expected number");
        }
    }

    #[test]
    fn test_stdev_insufficient_data() {
        assert_eq!(fn_stdev(&[num(5.0)]), Value::Error(ErrorKind::Div0));
    }

    #[test]
    fn test_var() {
        let data = nums_array(&[2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0]);
        if let Value::Number(result) = fn_var(&[data]) {
            // mean=5, sum_sq_dev=32, sample_var=32/7≈4.571
            assert!((result - 4.571).abs() < 0.01);
        } else {
            panic!("Expected number");
        }
    }

    #[test]
    fn test_median_odd() {
        let data = nums_array(&[1.0, 3.0, 5.0, 7.0, 9.0]);
        assert_eq!(fn_median(&[data]), num(5.0));
    }

    #[test]
    fn test_median_even() {
        let data = nums_array(&[1.0, 3.0, 5.0, 7.0]);
        assert_eq!(fn_median(&[data]), num(4.0));
    }

    #[test]
    fn test_percentile() {
        let data = nums_array(&[1.0, 2.0, 3.0, 4.0]);
        // 0th percentile = min
        assert_eq!(fn_percentile(&[data.clone(), num(0.0)]), num(1.0));
        // 100th percentile = max
        assert_eq!(fn_percentile(&[data.clone(), num(1.0)]), num(4.0));
        // 50th percentile = median
        assert_eq!(fn_percentile(&[data, num(0.5)]), num(2.5));
    }

    #[test]
    fn test_percentile_out_of_range() {
        let data = nums_array(&[1.0, 2.0, 3.0]);
        assert_eq!(
            fn_percentile(&[data.clone(), num(1.5)]),
            Value::Error(ErrorKind::Num)
        );
        assert_eq!(
            fn_percentile(&[data, num(-0.1)]),
            Value::Error(ErrorKind::Num)
        );
    }

    #[test]
    fn test_rank_descending() {
        let data = nums_array(&[3.0, 1.0, 4.0, 1.0, 5.0]);
        // Rank of 5 in descending order = 1
        assert_eq!(fn_rank(&[num(5.0), data.clone()]), num(1.0));
        // Rank of 3 in descending order = 3
        assert_eq!(fn_rank(&[num(3.0), data]), num(3.0));
    }

    #[test]
    fn test_rank_ascending() {
        let data = nums_array(&[3.0, 1.0, 4.0, 1.0, 5.0]);
        // Rank of 1 in ascending order = 1
        assert_eq!(fn_rank(&[num(1.0), data.clone(), num(1.0)]), num(1.0));
        // Rank of 5 in ascending order = 5
        assert_eq!(fn_rank(&[num(5.0), data, num(1.0)]), num(5.0));
    }

    #[test]
    fn test_countif() {
        let range = nums_array(&[1.0, 2.0, 3.0, 2.0, 1.0]);
        assert_eq!(fn_countif(&[range.clone(), num(2.0)]), num(2.0));
        assert_eq!(fn_countif(&[range.clone(), s(">1")]), num(3.0));
        assert_eq!(fn_countif(&[range, s(">=2")]), num(3.0));
    }

    #[test]
    fn test_countif_wildcard() {
        let range = Value::Array(vec![vec![s("apple"), s("banana"), s("apricot"), s("cherry")]]);
        assert_eq!(fn_countif(&[range, s("ap*")]), num(2.0));
    }

    #[test]
    fn test_sumif() {
        let range = nums_array(&[1.0, 2.0, 3.0, 4.0, 5.0]);
        // Sum values > 3
        assert_eq!(fn_sumif(&[range, s(">3")]), num(9.0));
    }

    #[test]
    fn test_sumif_with_sum_range() {
        let criteria_range = Value::Array(vec![vec![s("A"), s("B"), s("A"), s("B")]]);
        let sum_range = nums_array(&[10.0, 20.0, 30.0, 40.0]);
        assert_eq!(
            fn_sumif(&[criteria_range, s("A"), sum_range]),
            num(40.0)
        );
    }

    #[test]
    fn test_wildcard_match() {
        assert!(wildcard_match("hello", "hel*"));
        assert!(wildcard_match("hello", "h?llo"));
        assert!(wildcard_match("hello", "*llo"));
        assert!(!wildcard_match("hello", "h?lo"));
        assert!(wildcard_match("", "*"));
        assert!(!wildcard_match("", "?"));
    }
}
