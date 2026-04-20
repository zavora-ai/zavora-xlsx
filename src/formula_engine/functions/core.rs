//! Core Excel functions: SUM, AVERAGE, COUNT, COUNTA, MIN, MAX, IF, VLOOKUP, INDEX, MATCH.

use crate::formula_engine::evaluator::{ErrorKind, Value};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------



// ---------------------------------------------------------------------------
// SUM
// ---------------------------------------------------------------------------

/// SUM(number1, [number2], ...)
///
/// Sums all numeric values. In ranges, strings and bools are skipped.
/// Direct bool args are coerced (TRUE=1). Direct string args that parse as
/// numbers are included; non-numeric strings produce #VALUE!.
pub fn fn_sum(args: &[Value]) -> Value {
    let mut total = 0.0;
    for arg in args {
        match arg {
            Value::Number(n) => total += n,
            Value::Array(rows) => {
                for row in rows {
                    for cell in row {
                        match cell {
                            Value::Number(n) => total += n,
                            Value::Error(e) => return Value::Error(*e),
                            _ => {} // skip strings, bools, empty in ranges
                        }
                    }
                }
            }
            Value::Bool(b) => total += if *b { 1.0 } else { 0.0 },
            Value::Empty => {}
            Value::String(s) => {
                match s.parse::<f64>() {
                    Ok(n) => total += n,
                    Err(_) => return Value::Error(ErrorKind::Value),
                }
            }
            Value::Error(e) => return Value::Error(*e),
        }
    }
    Value::Number(total)
}

// ---------------------------------------------------------------------------
// AVERAGE
// ---------------------------------------------------------------------------

/// AVERAGE(number1, [number2], ...)
///
/// Returns the arithmetic mean of numeric values. Non-numeric values in
/// ranges are skipped. Returns #DIV/0! if no numeric values are found.
pub fn fn_average(args: &[Value]) -> Value {
    let mut sum = 0.0;
    let mut count = 0usize;
    for arg in args {
        match arg {
            Value::Number(n) => { sum += n; count += 1; }
            Value::Array(rows) => {
                for row in rows {
                    for cell in row {
                        match cell {
                            Value::Number(n) => { sum += n; count += 1; }
                            Value::Error(e) => return Value::Error(*e),
                            _ => {}
                        }
                    }
                }
            }
            Value::Bool(b) => { sum += if *b { 1.0 } else { 0.0 }; count += 1; }
            Value::Empty => {}
            Value::String(s) => {
                match s.parse::<f64>() {
                    Ok(n) => { sum += n; count += 1; }
                    Err(_) => return Value::Error(ErrorKind::Value),
                }
            }
            Value::Error(e) => return Value::Error(*e),
        }
    }
    if count == 0 {
        Value::Error(ErrorKind::Div0)
    } else {
        Value::Number(sum / count as f64)
    }
}

// ---------------------------------------------------------------------------
// COUNT
// ---------------------------------------------------------------------------

/// COUNT(value1, [value2], ...)
///
/// Counts the number of cells/arguments that contain numeric values.
/// Bools in ranges are NOT counted. Direct bool args ARE counted.
pub fn fn_count(args: &[Value]) -> Value {
    let mut count = 0usize;
    for arg in args {
        match arg {
            Value::Number(_) => count += 1,
            Value::Array(rows) => {
                for row in rows {
                    for cell in row {
                        match cell {
                            Value::Number(_) => count += 1,
                            Value::Error(e) => return Value::Error(*e),
                            _ => {}
                        }
                    }
                }
            }
            Value::Bool(_) => count += 1, // direct bool arg counts
            Value::String(s) => {
                if s.parse::<f64>().is_ok() {
                    count += 1;
                }
            }
            Value::Error(e) => return Value::Error(*e),
            Value::Empty => {}
        }
    }
    Value::Number(count as f64)
}

// ---------------------------------------------------------------------------
// COUNTA
// ---------------------------------------------------------------------------

/// COUNTA(value1, [value2], ...)
///
/// Counts the number of non-empty cells/arguments.
pub fn fn_counta(args: &[Value]) -> Value {
    let mut count = 0usize;
    for arg in args {
        match arg {
            Value::Empty => {}
            Value::Array(rows) => {
                for row in rows {
                    for cell in row {
                        match cell {
                            Value::Empty => {}
                            Value::Error(e) => return Value::Error(*e),
                            _ => count += 1,
                        }
                    }
                }
            }
            Value::Error(e) => return Value::Error(*e),
            _ => count += 1,
        }
    }
    Value::Number(count as f64)
}

// ---------------------------------------------------------------------------
// MIN
// ---------------------------------------------------------------------------

/// MIN(number1, [number2], ...)
///
/// Returns the smallest numeric value. Non-numeric values in ranges are
/// skipped. Returns 0 if no numeric values are found.
pub fn fn_min(args: &[Value]) -> Value {
    let mut min: Option<f64> = None;
    for arg in args {
        match arg {
            Value::Number(n) => {
                min = Some(min.map_or(*n, |m: f64| m.min(*n)));
            }
            Value::Array(rows) => {
                for row in rows {
                    for cell in row {
                        match cell {
                            Value::Number(n) => {
                                min = Some(min.map_or(*n, |m: f64| m.min(*n)));
                            }
                            Value::Error(e) => return Value::Error(*e),
                            _ => {}
                        }
                    }
                }
            }
            Value::Bool(b) => {
                let n = if *b { 1.0 } else { 0.0 };
                min = Some(min.map_or(n, |m: f64| m.min(n)));
            }
            Value::Empty => {}
            Value::String(s) => {
                match s.parse::<f64>() {
                    Ok(n) => min = Some(min.map_or(n, |m: f64| m.min(n))),
                    Err(_) => return Value::Error(ErrorKind::Value),
                }
            }
            Value::Error(e) => return Value::Error(*e),
        }
    }
    Value::Number(min.unwrap_or(0.0))
}

// ---------------------------------------------------------------------------
// MAX
// ---------------------------------------------------------------------------

/// MAX(number1, [number2], ...)
///
/// Returns the largest numeric value. Non-numeric values in ranges are
/// skipped. Returns 0 if no numeric values are found.
pub fn fn_max(args: &[Value]) -> Value {
    let mut max: Option<f64> = None;
    for arg in args {
        match arg {
            Value::Number(n) => {
                max = Some(max.map_or(*n, |m: f64| m.max(*n)));
            }
            Value::Array(rows) => {
                for row in rows {
                    for cell in row {
                        match cell {
                            Value::Number(n) => {
                                max = Some(max.map_or(*n, |m: f64| m.max(*n)));
                            }
                            Value::Error(e) => return Value::Error(*e),
                            _ => {}
                        }
                    }
                }
            }
            Value::Bool(b) => {
                let n = if *b { 1.0 } else { 0.0 };
                max = Some(max.map_or(n, |m: f64| m.max(n)));
            }
            Value::Empty => {}
            Value::String(s) => {
                match s.parse::<f64>() {
                    Ok(n) => max = Some(max.map_or(n, |m: f64| m.max(n))),
                    Err(_) => return Value::Error(ErrorKind::Value),
                }
            }
            Value::Error(e) => return Value::Error(*e),
        }
    }
    Value::Number(max.unwrap_or(0.0))
}

// ---------------------------------------------------------------------------
// IF
// ---------------------------------------------------------------------------

/// IF(logical_test, value_if_true, [value_if_false])
///
/// With 2 args: returns `value_if_true` when condition is truthy, FALSE otherwise.
/// With 3 args: returns `value_if_true` or `value_if_false`.
pub fn fn_if(args: &[Value]) -> Value {
    if args.len() < 2 || args.len() > 3 {
        return Value::Error(ErrorKind::Value);
    }
    let condition = match args[0].to_bool() {
        Ok(b) => b,
        Err(e) => return Value::Error(e),
    };
    if condition {
        args[1].clone()
    } else if args.len() == 3 {
        args[2].clone()
    } else {
        Value::Bool(false)
    }
}

// ---------------------------------------------------------------------------
// VLOOKUP
// ---------------------------------------------------------------------------

/// VLOOKUP(lookup_value, table_array, col_index_num, [range_lookup])
///
/// - `range_lookup` = FALSE (or 0): exact match, linear search
/// - `range_lookup` = TRUE (or 1, or omitted): approximate match, binary search
///   (assumes first column is sorted ascending)
pub fn fn_vlookup(args: &[Value]) -> Value {
    if args.len() < 3 || args.len() > 4 {
        return Value::Error(ErrorKind::Value);
    }

    let lookup_val = &args[0];
    if let Value::Error(e) = lookup_val {
        return Value::Error(*e);
    }

    let table = match &args[1] {
        Value::Array(rows) => rows,
        _ => return Value::Error(ErrorKind::Value),
    };

    let col_index = match args[2].to_number() {
        Ok(n) => n as usize,
        Err(e) => return Value::Error(e),
    };

    if col_index < 1 {
        return Value::Error(ErrorKind::Value);
    }

    // Check col_index is within table width
    if let Some(first_row) = table.first() {
        if col_index > first_row.len() {
            return Value::Error(ErrorKind::Ref);
        }
    } else {
        return Value::Error(ErrorKind::Na);
    }

    let range_lookup = if args.len() == 4 {
        match args[3].to_bool() {
            Ok(b) => b,
            Err(e) => return Value::Error(e),
        }
    } else {
        true // default: approximate match
    };

    if range_lookup {
        // Approximate match: binary search on first column (sorted ascending)
        vlookup_approximate(lookup_val, table, col_index)
    } else {
        // Exact match: linear search
        vlookup_exact(lookup_val, table, col_index)
    }
}

/// Exact match VLOOKUP: linear search through first column.
fn vlookup_exact(lookup_val: &Value, table: &[Vec<Value>], col_index: usize) -> Value {
    for row in table {
        if row.is_empty() {
            continue;
        }
        if values_equal(lookup_val, &row[0]) {
            return if col_index - 1 < row.len() {
                row[col_index - 1].clone()
            } else {
                Value::Error(ErrorKind::Ref)
            };
        }
    }
    Value::Error(ErrorKind::Na)
}

/// Approximate match VLOOKUP: find the largest value in the first column
/// that is less than or equal to the lookup value.
/// Assumes the first column is sorted in ascending order.
fn vlookup_approximate(lookup_val: &Value, table: &[Vec<Value>], col_index: usize) -> Value {
    let mut best_row: Option<usize> = None;

    for (i, row) in table.iter().enumerate() {
        if row.is_empty() {
            continue;
        }
        let cell = &row[0];
        match compare_values(cell, lookup_val) {
            Some(std::cmp::Ordering::Less | std::cmp::Ordering::Equal) => {
                best_row = Some(i);
            }
            Some(std::cmp::Ordering::Greater) => {
                // Since sorted, no need to continue
                break;
            }
            None => {
                // Incompatible types, skip
                continue;
            }
        }
    }

    match best_row {
        Some(i) => {
            let row = &table[i];
            if col_index - 1 < row.len() {
                row[col_index - 1].clone()
            } else {
                Value::Error(ErrorKind::Ref)
            }
        }
        None => Value::Error(ErrorKind::Na),
    }
}

// ---------------------------------------------------------------------------
// INDEX
// ---------------------------------------------------------------------------

/// INDEX(array, row_num, [col_num])
///
/// Returns the value at the specified position in an array or range.
/// Row and column numbers are 1-based.
/// If row_num is 0 and array is a single column, returns the whole column (as array).
/// If col_num is 0 and array is a single row, returns the whole row (as array).
pub fn fn_index(args: &[Value]) -> Value {
    if args.is_empty() || args.len() > 3 {
        return Value::Error(ErrorKind::Value);
    }

    let array = match &args[0] {
        Value::Array(rows) => rows,
        // Single value: treat as 1x1 array
        other => {
            if args.len() == 1 {
                return other.clone();
            }
            let row_num = match args.get(1).unwrap_or(&Value::Number(1.0)).to_number() {
                Ok(n) => n as usize,
                Err(e) => return Value::Error(e),
            };
            if row_num <= 1 {
                return other.clone();
            }
            return Value::Error(ErrorKind::Ref);
        }
    };

    if array.is_empty() {
        return Value::Error(ErrorKind::Ref);
    }

    let row_num = if args.len() >= 2 {
        match args[1].to_number() {
            Ok(n) => n as usize,
            Err(e) => return Value::Error(e),
        }
    } else {
        1
    };

    let col_num = if args.len() >= 3 {
        match args[2].to_number() {
            Ok(n) => n as usize,
            Err(e) => return Value::Error(e),
        }
    } else {
        // If array is a single column, default col to 1
        // If array is a single row, default col to 1
        1
    };

    // row_num=0 means return entire column
    if row_num == 0 {
        if col_num == 0 {
            return Value::Error(ErrorKind::Value);
        }
        if col_num > array[0].len() {
            return Value::Error(ErrorKind::Ref);
        }
        let col: Vec<Vec<Value>> = array
            .iter()
            .map(|r| {
                vec![if col_num - 1 < r.len() {
                    r[col_num - 1].clone()
                } else {
                    Value::Empty
                }]
            })
            .collect();
        return Value::Array(col);
    }

    // col_num=0 means return entire row
    if col_num == 0 {
        if row_num > array.len() {
            return Value::Error(ErrorKind::Ref);
        }
        return Value::Array(vec![array[row_num - 1].clone()]);
    }

    // Normal case: return single cell
    if row_num > array.len() {
        return Value::Error(ErrorKind::Ref);
    }
    let row = &array[row_num - 1];
    if col_num > row.len() {
        return Value::Error(ErrorKind::Ref);
    }
    row[col_num - 1].clone()
}

// ---------------------------------------------------------------------------
// MATCH
// ---------------------------------------------------------------------------

/// MATCH(lookup_value, lookup_array, [match_type])
///
/// Returns the 1-based position of a value in a one-dimensional range.
///
/// - `match_type` = 1 (default): finds the largest value ≤ lookup_value
///   (lookup_array must be sorted ascending)
/// - `match_type` = 0: finds the first exact match
/// - `match_type` = -1: finds the smallest value ≥ lookup_value
///   (lookup_array must be sorted descending)
pub fn fn_match(args: &[Value]) -> Value {
    if args.is_empty() || args.len() > 3 {
        return Value::Error(ErrorKind::Value);
    }

    let lookup_val = &args[0];
    if let Value::Error(e) = lookup_val {
        return Value::Error(*e);
    }

    // Extract the lookup array as a flat list of values
    let lookup_array: Vec<&Value> = match &args[1] {
        Value::Array(rows) => {
            // Flatten: could be a row vector or column vector
            let mut flat = Vec::new();
            for row in rows {
                for cell in row {
                    flat.push(cell);
                }
            }
            flat
        }
        other => vec![other],
    };

    let match_type = if args.len() >= 3 {
        match args[2].to_number() {
            Ok(n) => {
                if n > 0.0 { 1 }
                else if n < 0.0 { -1 }
                else { 0 }
            }
            Err(e) => return Value::Error(e),
        }
    } else {
        1 // default
    };

    match match_type {
        0 => {
            // Exact match: linear search
            for (i, cell) in lookup_array.iter().enumerate() {
                if values_equal(lookup_val, cell) {
                    return Value::Number((i + 1) as f64);
                }
            }
            Value::Error(ErrorKind::Na)
        }
        1 => {
            // Largest value <= lookup_value (sorted ascending)
            let mut best: Option<usize> = None;
            for (i, cell) in lookup_array.iter().enumerate() {
                match compare_values(cell, lookup_val) {
                    Some(std::cmp::Ordering::Less | std::cmp::Ordering::Equal) => {
                        best = Some(i);
                    }
                    Some(std::cmp::Ordering::Greater) => break,
                    None => continue,
                }
            }
            match best {
                Some(i) => Value::Number((i + 1) as f64),
                None => Value::Error(ErrorKind::Na),
            }
        }
        -1 => {
            // Smallest value >= lookup_value (sorted descending)
            let mut best: Option<usize> = None;
            for (i, cell) in lookup_array.iter().enumerate() {
                match compare_values(cell, lookup_val) {
                    Some(std::cmp::Ordering::Greater | std::cmp::Ordering::Equal) => {
                        best = Some(i);
                    }
                    Some(std::cmp::Ordering::Less) => break,
                    None => continue,
                }
            }
            match best {
                Some(i) => Value::Number((i + 1) as f64),
                None => Value::Error(ErrorKind::Na),
            }
        }
        _ => Value::Error(ErrorKind::Value),
    }
}

// ---------------------------------------------------------------------------
// Comparison helpers
// ---------------------------------------------------------------------------

/// Check if two values are equal for lookup purposes.
/// String comparison is case-insensitive (Excel behavior).
fn values_equal(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Number(x), Value::Number(y)) => (x - y).abs() < f64::EPSILON,
        (Value::String(x), Value::String(y)) => x.eq_ignore_ascii_case(y),
        (Value::Bool(x), Value::Bool(y)) => x == y,
        (Value::Empty, Value::Empty) => true,
        (Value::Number(n), Value::Empty) | (Value::Empty, Value::Number(n)) => *n == 0.0,
        (Value::String(s), Value::Empty) | (Value::Empty, Value::String(s)) => s.is_empty(),
        _ => false,
    }
}

/// Compare two values for ordering (used in approximate match).
/// Returns `None` if the types are incompatible.
fn compare_values(a: &Value, b: &Value) -> Option<std::cmp::Ordering> {
    match (a, b) {
        (Value::Number(x), Value::Number(y)) => x.partial_cmp(y),
        (Value::String(x), Value::String(y)) => {
            Some(x.to_lowercase().cmp(&y.to_lowercase()))
        }
        (Value::Empty, Value::Number(n)) => 0.0_f64.partial_cmp(n),
        (Value::Number(n), Value::Empty) => n.partial_cmp(&0.0),
        (Value::Empty, Value::String(s)) => Some("".cmp(s.as_str())),
        (Value::String(s), Value::Empty) => Some(s.as_str().cmp("")),
        (Value::Empty, Value::Empty) => Some(std::cmp::Ordering::Equal),
        _ => None,
    }
}
