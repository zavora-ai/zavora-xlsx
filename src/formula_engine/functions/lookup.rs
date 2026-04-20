//! Lookup functions: HLOOKUP, XLOOKUP, INDIRECT, OFFSET.
//!
//! MATCH is already implemented in core.rs and re-exported via the function registry.
//! INDIRECT and OFFSET require runtime context for full implementation;
//! they return Value::Error(ErrorKind::Value) as stubs.

use crate::formula_engine::evaluator::{ErrorKind, Value};

// ---------------------------------------------------------------------------
// Comparison helpers (reused from core patterns)
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// HLOOKUP
// ---------------------------------------------------------------------------

/// HLOOKUP(lookup_value, table_array, row_index_num, [range_lookup])
///
/// Searches for a value in the first row of a table and returns a value in the
/// same column from a specified row.
pub fn fn_hlookup(args: &[Value]) -> Value {
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

    let row_index = match args[2].to_number() {
        Ok(n) => n as usize,
        Err(e) => return Value::Error(e),
    };

    if row_index < 1 || table.is_empty() {
        return Value::Error(ErrorKind::Value);
    }
    if row_index > table.len() {
        return Value::Error(ErrorKind::Ref);
    }

    let range_lookup = if args.len() == 4 {
        match args[3].to_bool() {
            Ok(b) => b,
            Err(e) => return Value::Error(e),
        }
    } else {
        true
    };

    let first_row = &table[0];

    if range_lookup {
        // Approximate match: find largest value <= lookup_value in first row
        let mut best_col: Option<usize> = None;
        for (i, cell) in first_row.iter().enumerate() {
            match compare_values(cell, lookup_val) {
                Some(std::cmp::Ordering::Less | std::cmp::Ordering::Equal) => {
                    best_col = Some(i);
                }
                Some(std::cmp::Ordering::Greater) => break,
                None => continue,
            }
        }
        match best_col {
            Some(col) => {
                let target_row = &table[row_index - 1];
                if col < target_row.len() {
                    target_row[col].clone()
                } else {
                    Value::Error(ErrorKind::Ref)
                }
            }
            None => Value::Error(ErrorKind::Na),
        }
    } else {
        // Exact match: linear search in first row
        for (i, cell) in first_row.iter().enumerate() {
            if values_equal(lookup_val, cell) {
                let target_row = &table[row_index - 1];
                return if i < target_row.len() {
                    target_row[i].clone()
                } else {
                    Value::Error(ErrorKind::Ref)
                };
            }
        }
        Value::Error(ErrorKind::Na)
    }
}

// ---------------------------------------------------------------------------
// XLOOKUP
// ---------------------------------------------------------------------------

/// XLOOKUP(lookup_value, lookup_array, return_array, [if_not_found], [match_mode], [search_mode])
///
/// match_mode: 0 = exact (default), -1 = exact or next smaller, 1 = exact or next larger, 2 = wildcard
/// search_mode: 1 = first-to-last (default), -1 = last-to-first, 2 = binary asc, -2 = binary desc
pub fn fn_xlookup(args: &[Value]) -> Value {
    if args.len() < 3 || args.len() > 6 {
        return Value::Error(ErrorKind::Value);
    }

    let lookup_val = &args[0];
    if let Value::Error(e) = lookup_val {
        return Value::Error(*e);
    }

    let lookup_array = flatten_array(&args[1]);
    let return_array = flatten_array(&args[2]);

    let if_not_found = args.get(3);

    let match_mode = if args.len() >= 5 {
        match args[4].to_number() {
            Ok(n) => n as i32,
            Err(e) => return Value::Error(e),
        }
    } else {
        0
    };

    let search_mode = if args.len() >= 6 {
        match args[5].to_number() {
            Ok(n) => n as i32,
            Err(e) => return Value::Error(e),
        }
    } else {
        1
    };

    // Build search indices
    let indices: Vec<usize> = match search_mode {
        1 | 2 => (0..lookup_array.len()).collect(),
        -1 | -2 => (0..lookup_array.len()).rev().collect(),
        _ => return Value::Error(ErrorKind::Value),
    };

    match match_mode {
        0 => {
            // Exact match
            for &i in &indices {
                if values_equal(lookup_val, &lookup_array[i]) {
                    return get_return_value(&return_array, i);
                }
            }
        }
        -1 => {
            // Exact match or next smaller
            let mut best: Option<usize> = None;
            for (i, cell) in lookup_array.iter().enumerate() {
                match compare_values(cell, lookup_val) {
                    Some(std::cmp::Ordering::Equal) => {
                        return get_return_value(&return_array, i);
                    }
                    Some(std::cmp::Ordering::Less) => {
                        if best.is_none()
                            || compare_values(cell, &lookup_array[best.unwrap()])
                                == Some(std::cmp::Ordering::Greater)
                        {
                            best = Some(i);
                        }
                    }
                    _ => {}
                }
            }
            if let Some(i) = best {
                return get_return_value(&return_array, i);
            }
        }
        1 => {
            // Exact match or next larger
            let mut best: Option<usize> = None;
            for (i, cell) in lookup_array.iter().enumerate() {
                match compare_values(cell, lookup_val) {
                    Some(std::cmp::Ordering::Equal) => {
                        return get_return_value(&return_array, i);
                    }
                    Some(std::cmp::Ordering::Greater) => {
                        if best.is_none()
                            || compare_values(cell, &lookup_array[best.unwrap()])
                                == Some(std::cmp::Ordering::Less)
                        {
                            best = Some(i);
                        }
                    }
                    _ => {}
                }
            }
            if let Some(i) = best {
                return get_return_value(&return_array, i);
            }
        }
        2 => {
            // Wildcard match (simplified: treat as exact for now)
            for &i in &indices {
                if values_equal(lookup_val, &lookup_array[i]) {
                    return get_return_value(&return_array, i);
                }
            }
        }
        _ => return Value::Error(ErrorKind::Value),
    }

    // Not found
    match if_not_found {
        Some(val) => val.clone(),
        None => Value::Error(ErrorKind::Na),
    }
}

fn flatten_array(val: &Value) -> Vec<Value> {
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

fn get_return_value(return_array: &[Value], index: usize) -> Value {
    if index < return_array.len() {
        return_array[index].clone()
    } else {
        Value::Error(ErrorKind::Ref)
    }
}

// ---------------------------------------------------------------------------
// INDIRECT
// ---------------------------------------------------------------------------

/// INDIRECT(ref_text, [a1])
///
/// Converts a text string to a cell reference. This requires runtime context
/// to resolve the reference, so for now it returns #VALUE!.
/// Full implementation would parse the string and resolve via CellContext.
pub fn fn_indirect(args: &[Value]) -> Value {
    if args.is_empty() || args.len() > 2 {
        return Value::Error(ErrorKind::Value);
    }
    // TODO: Full implementation requires runtime CellContext to resolve string references.
    // For now, return #REF! as we cannot resolve without context.
    Value::Error(ErrorKind::Ref)
}

// ---------------------------------------------------------------------------
// OFFSET
// ---------------------------------------------------------------------------

/// OFFSET(reference, rows, cols, [height], [width])
///
/// Returns a reference shifted by the specified rows and columns.
/// This requires runtime context for full implementation.
pub fn fn_offset(args: &[Value]) -> Value {
    if args.len() < 3 || args.len() > 5 {
        return Value::Error(ErrorKind::Value);
    }
    // TODO: Full implementation requires runtime CellContext to shift references.
    // For now, return #REF! as we cannot resolve without context.
    Value::Error(ErrorKind::Ref)
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

    fn make_table() -> Value {
        // 3x3 table:
        //   A    B    C
        //   1    2    3
        //   4    5    6
        Value::Array(vec![
            vec![s("A"), s("B"), s("C")],
            vec![num(1.0), num(2.0), num(3.0)],
            vec![num(4.0), num(5.0), num(6.0)],
        ])
    }

    #[test]
    fn test_hlookup_exact() {
        let table = make_table();
        // Look up "B" in first row, return from row 2
        assert_eq!(
            fn_hlookup(&[s("B"), table.clone(), num(2.0), Value::Bool(false)]),
            num(2.0)
        );
        // Look up "C" in first row, return from row 3
        assert_eq!(
            fn_hlookup(&[s("C"), table.clone(), num(3.0), Value::Bool(false)]),
            num(6.0)
        );
        // Not found
        assert_eq!(
            fn_hlookup(&[s("D"), table, num(2.0), Value::Bool(false)]),
            Value::Error(ErrorKind::Na)
        );
    }

    #[test]
    fn test_hlookup_approximate() {
        // Numeric first row, sorted ascending
        let table = Value::Array(vec![
            vec![num(10.0), num(20.0), num(30.0)],
            vec![s("ten"), s("twenty"), s("thirty")],
        ]);
        // 25 → closest ≤ is 20
        assert_eq!(
            fn_hlookup(&[num(25.0), table.clone(), num(2.0)]),
            s("twenty")
        );
        // 30 → exact match
        assert_eq!(
            fn_hlookup(&[num(30.0), table, num(2.0)]),
            s("thirty")
        );
    }

    #[test]
    fn test_xlookup_exact() {
        let lookup = Value::Array(vec![vec![s("A"), s("B"), s("C")]]);
        let returns = Value::Array(vec![vec![num(1.0), num(2.0), num(3.0)]]);
        assert_eq!(
            fn_xlookup(&[s("B"), lookup.clone(), returns.clone()]),
            num(2.0)
        );
        // Not found, no if_not_found
        assert_eq!(
            fn_xlookup(&[s("D"), lookup.clone(), returns.clone()]),
            Value::Error(ErrorKind::Na)
        );
        // Not found, with if_not_found
        assert_eq!(
            fn_xlookup(&[s("D"), lookup, returns, s("N/A")]),
            s("N/A")
        );
    }

    #[test]
    fn test_xlookup_next_smaller() {
        let lookup = Value::Array(vec![vec![num(10.0), num(20.0), num(30.0)]]);
        let returns = Value::Array(vec![vec![s("ten"), s("twenty"), s("thirty")]]);
        // match_mode = -1: exact or next smaller
        assert_eq!(
            fn_xlookup(&[num(25.0), lookup, returns, Value::Error(ErrorKind::Na), num(-1.0)]),
            s("twenty")
        );
    }

    #[test]
    fn test_xlookup_next_larger() {
        let lookup = Value::Array(vec![vec![num(10.0), num(20.0), num(30.0)]]);
        let returns = Value::Array(vec![vec![s("ten"), s("twenty"), s("thirty")]]);
        // match_mode = 1: exact or next larger
        assert_eq!(
            fn_xlookup(&[num(15.0), lookup, returns, Value::Error(ErrorKind::Na), num(1.0)]),
            s("twenty")
        );
    }

    #[test]
    fn test_indirect_stub() {
        // INDIRECT returns #REF! as a stub
        assert_eq!(fn_indirect(&[s("A1")]), Value::Error(ErrorKind::Ref));
    }

    #[test]
    fn test_offset_stub() {
        // OFFSET returns #REF! as a stub
        assert_eq!(
            fn_offset(&[num(1.0), num(0.0), num(0.0)]),
            Value::Error(ErrorKind::Ref)
        );
    }

    #[test]
    fn test_hlookup_row_index_out_of_range() {
        let table = make_table();
        assert_eq!(
            fn_hlookup(&[s("A"), table, num(5.0), Value::Bool(false)]),
            Value::Error(ErrorKind::Ref)
        );
    }
}
