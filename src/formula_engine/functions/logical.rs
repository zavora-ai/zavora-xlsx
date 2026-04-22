//! Logical functions: AND, OR, NOT, IFERROR, IFNA, SWITCH, IFS.

use crate::formula_engine::evaluator::{ErrorKind, Value};

// ---------------------------------------------------------------------------
// AND
// ---------------------------------------------------------------------------

/// AND(`logical1`, \[`logical2`\], ...)
/// Returns TRUE if all arguments are TRUE.
pub fn fn_and(args: &[Value]) -> Value {
    if args.is_empty() {
        return Value::Error(ErrorKind::Value);
    }
    for arg in args {
        match arg {
            Value::Array(rows) => {
                for row in rows {
                    for cell in row {
                        match cell.to_bool() {
                            Ok(false) => return Value::Bool(false),
                            Ok(true) => {}
                            Err(e) => return Value::Error(e),
                        }
                    }
                }
            }
            Value::Error(e) => return Value::Error(*e),
            other => match other.to_bool() {
                Ok(false) => return Value::Bool(false),
                Ok(true) => {}
                Err(e) => return Value::Error(e),
            },
        }
    }
    Value::Bool(true)
}

// ---------------------------------------------------------------------------
// OR
// ---------------------------------------------------------------------------

/// OR(`logical1`, \[`logical2`\], ...)
/// Returns TRUE if any argument is TRUE.
pub fn fn_or(args: &[Value]) -> Value {
    if args.is_empty() {
        return Value::Error(ErrorKind::Value);
    }
    for arg in args {
        match arg {
            Value::Array(rows) => {
                for row in rows {
                    for cell in row {
                        match cell.to_bool() {
                            Ok(true) => return Value::Bool(true),
                            Ok(false) => {}
                            Err(e) => return Value::Error(e),
                        }
                    }
                }
            }
            Value::Error(e) => return Value::Error(*e),
            other => match other.to_bool() {
                Ok(true) => return Value::Bool(true),
                Ok(false) => {}
                Err(e) => return Value::Error(e),
            },
        }
    }
    Value::Bool(false)
}

// ---------------------------------------------------------------------------
// NOT
// ---------------------------------------------------------------------------

/// NOT(logical)
/// Reverses the logic of its argument.
pub fn fn_not(args: &[Value]) -> Value {
    if args.len() != 1 {
        return Value::Error(ErrorKind::Value);
    }
    match args[0].to_bool() {
        Ok(b) => Value::Bool(!b),
        Err(e) => Value::Error(e),
    }
}

// ---------------------------------------------------------------------------
// IFERROR
// ---------------------------------------------------------------------------

/// IFERROR(value, value_if_error)
/// Returns value_if_error if the first argument is any error, otherwise returns the value.
pub fn fn_iferror(args: &[Value]) -> Value {
    if args.len() != 2 {
        return Value::Error(ErrorKind::Value);
    }
    match &args[0] {
        Value::Error(_) => args[1].clone(),
        other => other.clone(),
    }
}

// ---------------------------------------------------------------------------
// IFNA
// ---------------------------------------------------------------------------

/// IFNA(value, value_if_na)
/// Returns value_if_na if the first argument is #N/A, otherwise returns the value.
pub fn fn_ifna(args: &[Value]) -> Value {
    if args.len() != 2 {
        return Value::Error(ErrorKind::Value);
    }
    match &args[0] {
        Value::Error(ErrorKind::Na) => args[1].clone(),
        other => other.clone(),
    }
}

// ---------------------------------------------------------------------------
// SWITCH
// ---------------------------------------------------------------------------

/// SWITCH(`expression`, `value1`, `result1`, \[`value2`, `result2`\], ..., \[`default`\])
/// Evaluates an expression against a list of values and returns the result
/// corresponding to the first matching value. If no match, returns default or #N/A.
pub fn fn_switch(args: &[Value]) -> Value {
    if args.len() < 3 {
        return Value::Error(ErrorKind::Value);
    }

    let expression = &args[0];
    if let Value::Error(e) = expression {
        return Value::Error(*e);
    }

    // Pairs: (value, result), with optional trailing default
    let pairs = &args[1..];
    let mut i = 0;
    while i + 1 < pairs.len() {
        if values_equal(expression, &pairs[i]) {
            return pairs[i + 1].clone();
        }
        i += 2;
    }

    // If there's an odd trailing argument, it's the default
    if i < pairs.len() {
        pairs[i].clone()
    } else {
        Value::Error(ErrorKind::Na)
    }
}

// ---------------------------------------------------------------------------
// IFS
// ---------------------------------------------------------------------------

/// IFS(logical_test1, value_if_true1, [logical_test2, value_if_true2], ...)
/// Checks multiple conditions and returns the value corresponding to the first TRUE condition.
pub fn fn_ifs(args: &[Value]) -> Value {
    if args.len() < 2 || !args.len().is_multiple_of(2) {
        return Value::Error(ErrorKind::Value);
    }

    let mut i = 0;
    while i < args.len() {
        match &args[i] {
            Value::Error(e) => return Value::Error(*e),
            condition => match condition.to_bool() {
                Ok(true) => return args[i + 1].clone(),
                Ok(false) => {}
                Err(e) => return Value::Error(e),
            },
        }
        i += 2;
    }

    Value::Error(ErrorKind::Na)
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn values_equal(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Number(x), Value::Number(y)) => (x - y).abs() < f64::EPSILON,
        (Value::String(x), Value::String(y)) => x.eq_ignore_ascii_case(y),
        (Value::Bool(x), Value::Bool(y)) => x == y,
        (Value::Empty, Value::Empty) => true,
        _ => false,
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

    #[test]
    fn test_and() {
        assert_eq!(
            fn_and(&[Value::Bool(true), Value::Bool(true)]),
            Value::Bool(true)
        );
        assert_eq!(
            fn_and(&[Value::Bool(true), Value::Bool(false)]),
            Value::Bool(false)
        );
        assert_eq!(fn_and(&[Value::Bool(false)]), Value::Bool(false));
        // Numbers: 0 = false, non-zero = true
        assert_eq!(fn_and(&[num(1.0), num(2.0)]), Value::Bool(true));
        assert_eq!(fn_and(&[num(1.0), num(0.0)]), Value::Bool(false));
    }

    #[test]
    fn test_or() {
        assert_eq!(
            fn_or(&[Value::Bool(false), Value::Bool(true)]),
            Value::Bool(true)
        );
        assert_eq!(
            fn_or(&[Value::Bool(false), Value::Bool(false)]),
            Value::Bool(false)
        );
        assert_eq!(fn_or(&[num(0.0), num(1.0)]), Value::Bool(true));
    }

    #[test]
    fn test_not() {
        assert_eq!(fn_not(&[Value::Bool(true)]), Value::Bool(false));
        assert_eq!(fn_not(&[Value::Bool(false)]), Value::Bool(true));
        assert_eq!(fn_not(&[num(0.0)]), Value::Bool(true));
        assert_eq!(fn_not(&[num(1.0)]), Value::Bool(false));
    }

    #[test]
    fn test_iferror() {
        assert_eq!(fn_iferror(&[num(1.0), s("error")]), num(1.0));
        assert_eq!(
            fn_iferror(&[Value::Error(ErrorKind::Div0), s("error")]),
            s("error")
        );
        assert_eq!(
            fn_iferror(&[Value::Error(ErrorKind::Na), s("error")]),
            s("error")
        );
    }

    #[test]
    fn test_ifna() {
        assert_eq!(fn_ifna(&[num(1.0), s("not found")]), num(1.0));
        assert_eq!(
            fn_ifna(&[Value::Error(ErrorKind::Na), s("not found")]),
            s("not found")
        );
        // Non-NA errors pass through
        assert_eq!(
            fn_ifna(&[Value::Error(ErrorKind::Div0), s("not found")]),
            Value::Error(ErrorKind::Div0)
        );
    }

    #[test]
    fn test_switch() {
        // SWITCH(2, 1, "one", 2, "two", 3, "three")
        assert_eq!(
            fn_switch(&[
                num(2.0),
                num(1.0),
                s("one"),
                num(2.0),
                s("two"),
                num(3.0),
                s("three")
            ]),
            s("two")
        );
        // No match, with default
        assert_eq!(
            fn_switch(&[
                num(4.0),
                num(1.0),
                s("one"),
                num(2.0),
                s("two"),
                s("default")
            ]),
            s("default")
        );
        // No match, no default
        assert_eq!(
            fn_switch(&[num(4.0), num(1.0), s("one"), num(2.0), s("two")]),
            Value::Error(ErrorKind::Na)
        );
    }

    #[test]
    fn test_ifs() {
        // IFS(FALSE, "a", TRUE, "b")
        assert_eq!(
            fn_ifs(&[Value::Bool(false), s("a"), Value::Bool(true), s("b")]),
            s("b")
        );
        // First true wins
        assert_eq!(
            fn_ifs(&[
                Value::Bool(true),
                s("first"),
                Value::Bool(true),
                s("second")
            ]),
            s("first")
        );
        // No match
        assert_eq!(
            fn_ifs(&[Value::Bool(false), s("a"), Value::Bool(false), s("b")]),
            Value::Error(ErrorKind::Na)
        );
    }

    #[test]
    fn test_error_propagation() {
        let err = Value::Error(ErrorKind::Ref);
        assert_eq!(fn_and(&[err.clone()]), Value::Error(ErrorKind::Ref));
        assert_eq!(fn_or(&[err.clone()]), Value::Error(ErrorKind::Ref));
        assert_eq!(fn_not(&[err.clone()]), Value::Error(ErrorKind::Ref));
    }

    #[test]
    fn test_and_or_with_arrays() {
        let arr = Value::Array(vec![vec![Value::Bool(true), Value::Bool(false)]]);
        assert_eq!(fn_and(&[arr.clone()]), Value::Bool(false));
        assert_eq!(fn_or(&[arr]), Value::Bool(true));
    }

    #[test]
    fn test_switch_with_strings() {
        assert_eq!(
            fn_switch(&[s("b"), s("a"), num(1.0), s("b"), num(2.0), s("c"), num(3.0)]),
            num(2.0)
        );
    }

    #[test]
    fn test_ifs_with_numbers() {
        // Numbers as conditions: 0 = false, non-zero = true
        assert_eq!(fn_ifs(&[num(0.0), s("zero"), num(1.0), s("one")]), s("one"));
    }
}
