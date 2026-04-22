//! Text functions: CONCATENATE, CONCAT, LEFT, RIGHT, MID, LEN, TRIM, UPPER, LOWER, SUBSTITUTE.

use crate::formula_engine::evaluator::{ErrorKind, Value};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Coerce a Value to a string for text functions.
/// Numbers are coerced to strings, errors propagate.
fn coerce_to_string(val: &Value) -> Result<String, ErrorKind> {
    val.to_string_val()
}

// ---------------------------------------------------------------------------
// CONCATENATE / CONCAT
// ---------------------------------------------------------------------------

/// CONCATENATE(`text1`, \[`text2`\], ...)
/// Joins multiple text strings into one.
pub fn fn_concatenate(args: &[Value]) -> Value {
    let mut result = String::new();
    for arg in args {
        match arg {
            Value::Array(rows) => {
                for row in rows {
                    for cell in row {
                        match coerce_to_string(cell) {
                            Ok(s) => result.push_str(&s),
                            Err(e) => return Value::Error(e),
                        }
                    }
                }
            }
            other => match coerce_to_string(other) {
                Ok(s) => result.push_str(&s),
                Err(e) => return Value::Error(e),
            },
        }
    }
    Value::String(result)
}

/// CONCAT is an alias for CONCATENATE.
pub fn fn_concat(args: &[Value]) -> Value {
    fn_concatenate(args)
}

// ---------------------------------------------------------------------------
// LEFT
// ---------------------------------------------------------------------------

/// LEFT(`text`, \[`num_chars`\])
/// Returns the leftmost characters. Default num_chars = 1.
pub fn fn_left(args: &[Value]) -> Value {
    if args.is_empty() || args.len() > 2 {
        return Value::Error(ErrorKind::Value);
    }
    let text = match coerce_to_string(&args[0]) {
        Ok(s) => s,
        Err(e) => return Value::Error(e),
    };
    let num_chars = if args.len() == 2 {
        match args[1].to_number() {
            Ok(n) => {
                if n < 0.0 {
                    return Value::Error(ErrorKind::Value);
                }
                n as usize
            }
            Err(e) => return Value::Error(e),
        }
    } else {
        1
    };
    let chars: String = text.chars().take(num_chars).collect();
    Value::String(chars)
}

// ---------------------------------------------------------------------------
// RIGHT
// ---------------------------------------------------------------------------

/// RIGHT(`text`, \[`num_chars`\])
/// Returns the rightmost characters. Default num_chars = 1.
pub fn fn_right(args: &[Value]) -> Value {
    if args.is_empty() || args.len() > 2 {
        return Value::Error(ErrorKind::Value);
    }
    let text = match coerce_to_string(&args[0]) {
        Ok(s) => s,
        Err(e) => return Value::Error(e),
    };
    let num_chars = if args.len() == 2 {
        match args[1].to_number() {
            Ok(n) => {
                if n < 0.0 {
                    return Value::Error(ErrorKind::Value);
                }
                n as usize
            }
            Err(e) => return Value::Error(e),
        }
    } else {
        1
    };
    let char_count = text.chars().count();
    let skip = char_count.saturating_sub(num_chars);
    let chars: String = text.chars().skip(skip).collect();
    Value::String(chars)
}

// ---------------------------------------------------------------------------
// MID
// ---------------------------------------------------------------------------

/// MID(text, start_num, num_chars)
/// Returns characters from the middle of a text string. start_num is 1-based.
pub fn fn_mid(args: &[Value]) -> Value {
    if args.len() != 3 {
        return Value::Error(ErrorKind::Value);
    }
    let text = match coerce_to_string(&args[0]) {
        Ok(s) => s,
        Err(e) => return Value::Error(e),
    };
    let start_num = match args[1].to_number() {
        Ok(n) => {
            if n < 1.0 {
                return Value::Error(ErrorKind::Value);
            }
            n as usize
        }
        Err(e) => return Value::Error(e),
    };
    let num_chars = match args[2].to_number() {
        Ok(n) => {
            if n < 0.0 {
                return Value::Error(ErrorKind::Value);
            }
            n as usize
        }
        Err(e) => return Value::Error(e),
    };
    let chars: String = text.chars().skip(start_num - 1).take(num_chars).collect();
    Value::String(chars)
}

// ---------------------------------------------------------------------------
// LEN
// ---------------------------------------------------------------------------

/// LEN(text)
/// Returns the number of characters in a text string.
pub fn fn_len(args: &[Value]) -> Value {
    if args.len() != 1 {
        return Value::Error(ErrorKind::Value);
    }
    let text = match coerce_to_string(&args[0]) {
        Ok(s) => s,
        Err(e) => return Value::Error(e),
    };
    Value::Number(text.chars().count() as f64)
}

// ---------------------------------------------------------------------------
// TRIM
// ---------------------------------------------------------------------------

/// TRIM(text)
/// Removes leading, trailing, and extra internal spaces (collapses to single space).
pub fn fn_trim(args: &[Value]) -> Value {
    if args.len() != 1 {
        return Value::Error(ErrorKind::Value);
    }
    let text = match coerce_to_string(&args[0]) {
        Ok(s) => s,
        Err(e) => return Value::Error(e),
    };
    // Excel TRIM removes leading/trailing spaces and collapses internal runs to single space
    let trimmed: String = text.split_whitespace().collect::<Vec<&str>>().join(" ");
    Value::String(trimmed)
}

// ---------------------------------------------------------------------------
// UPPER
// ---------------------------------------------------------------------------

/// UPPER(text)
pub fn fn_upper(args: &[Value]) -> Value {
    if args.len() != 1 {
        return Value::Error(ErrorKind::Value);
    }
    let text = match coerce_to_string(&args[0]) {
        Ok(s) => s,
        Err(e) => return Value::Error(e),
    };
    Value::String(text.to_uppercase())
}

// ---------------------------------------------------------------------------
// LOWER
// ---------------------------------------------------------------------------

/// LOWER(text)
pub fn fn_lower(args: &[Value]) -> Value {
    if args.len() != 1 {
        return Value::Error(ErrorKind::Value);
    }
    let text = match coerce_to_string(&args[0]) {
        Ok(s) => s,
        Err(e) => return Value::Error(e),
    };
    Value::String(text.to_lowercase())
}

// ---------------------------------------------------------------------------
// SUBSTITUTE
// ---------------------------------------------------------------------------

/// SUBSTITUTE(`text`, `old_text`, `new_text`, \[`instance_num`\])
/// Replaces occurrences of old_text with new_text.
/// If instance_num is provided, only that occurrence is replaced (1-based).
pub fn fn_substitute(args: &[Value]) -> Value {
    if args.len() < 3 || args.len() > 4 {
        return Value::Error(ErrorKind::Value);
    }
    let text = match coerce_to_string(&args[0]) {
        Ok(s) => s,
        Err(e) => return Value::Error(e),
    };
    let old_text = match coerce_to_string(&args[1]) {
        Ok(s) => s,
        Err(e) => return Value::Error(e),
    };
    let new_text = match coerce_to_string(&args[2]) {
        Ok(s) => s,
        Err(e) => return Value::Error(e),
    };

    if old_text.is_empty() {
        return Value::String(text);
    }

    if args.len() == 4 {
        let instance_num = match args[3].to_number() {
            Ok(n) => {
                if n < 1.0 {
                    return Value::Error(ErrorKind::Value);
                }
                n as usize
            }
            Err(e) => return Value::Error(e),
        };
        // Replace only the Nth occurrence
        let mut count = 0usize;
        let mut result = String::new();
        let mut remaining = text.as_str();
        while let Some(pos) = remaining.find(&old_text) {
            count += 1;
            if count == instance_num {
                result.push_str(&remaining[..pos]);
                result.push_str(&new_text);
                result.push_str(&remaining[pos + old_text.len()..]);
                return Value::String(result);
            }
            result.push_str(&remaining[..pos + old_text.len()]);
            remaining = &remaining[pos + old_text.len()..];
        }
        result.push_str(remaining);
        Value::String(result)
    } else {
        // Replace all occurrences
        Value::String(text.replace(&old_text, &new_text))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(val: &str) -> Value {
        Value::String(val.to_string())
    }
    fn num(n: f64) -> Value {
        Value::Number(n)
    }

    #[test]
    fn test_concatenate() {
        assert_eq!(
            fn_concatenate(&[s("Hello"), s(" "), s("World")]),
            s("Hello World")
        );
        // Number coercion
        assert_eq!(fn_concatenate(&[s("Value: "), num(42.0)]), s("Value: 42"));
    }

    #[test]
    fn test_left() {
        assert_eq!(fn_left(&[s("Hello")]), s("H"));
        assert_eq!(fn_left(&[s("Hello"), num(3.0)]), s("Hel"));
        // Overflow: request more chars than available
        assert_eq!(fn_left(&[s("Hi"), num(10.0)]), s("Hi"));
        assert_eq!(
            fn_left(&[s("Hello"), num(-1.0)]),
            Value::Error(ErrorKind::Value)
        );
    }

    #[test]
    fn test_right() {
        assert_eq!(fn_right(&[s("Hello")]), s("o"));
        assert_eq!(fn_right(&[s("Hello"), num(3.0)]), s("llo"));
        assert_eq!(fn_right(&[s("Hi"), num(10.0)]), s("Hi"));
    }

    #[test]
    fn test_mid() {
        assert_eq!(fn_mid(&[s("Hello World"), num(7.0), num(5.0)]), s("World"));
        assert_eq!(fn_mid(&[s("Hello"), num(1.0), num(3.0)]), s("Hel"));
        // Overflow: request more chars than available from start position
        assert_eq!(fn_mid(&[s("Hi"), num(1.0), num(10.0)]), s("Hi"));
        assert_eq!(
            fn_mid(&[s("Hi"), num(0.0), num(1.0)]),
            Value::Error(ErrorKind::Value)
        );
    }

    #[test]
    fn test_len() {
        assert_eq!(fn_len(&[s("Hello")]), num(5.0));
        assert_eq!(fn_len(&[s("")]), num(0.0));
        // Number coercion
        assert_eq!(fn_len(&[num(123.0)]), num(3.0));
    }

    #[test]
    fn test_trim() {
        assert_eq!(fn_trim(&[s("  Hello   World  ")]), s("Hello World"));
        assert_eq!(fn_trim(&[s("NoSpaces")]), s("NoSpaces"));
    }

    #[test]
    fn test_upper_lower() {
        assert_eq!(fn_upper(&[s("hello")]), s("HELLO"));
        assert_eq!(fn_lower(&[s("HELLO")]), s("hello"));
    }

    #[test]
    fn test_substitute() {
        assert_eq!(
            fn_substitute(&[s("Hello World"), s("World"), s("Rust")]),
            s("Hello Rust")
        );
        // Replace specific instance
        assert_eq!(
            fn_substitute(&[s("aaa"), s("a"), s("b"), num(2.0)]),
            s("aba")
        );
        // Replace all
        assert_eq!(fn_substitute(&[s("aaa"), s("a"), s("b")]), s("bbb"));
        // Empty old_text returns original
        assert_eq!(fn_substitute(&[s("Hello"), s(""), s("X")]), s("Hello"));
    }

    #[test]
    fn test_number_coercion() {
        // Numbers should be coerced to strings in text functions
        assert_eq!(fn_upper(&[num(123.0)]), s("123"));
        assert_eq!(fn_len(&[num(3.14)]), num(4.0));
        assert_eq!(fn_left(&[num(12345.0), num(3.0)]), s("123"));
    }

    #[test]
    fn test_error_propagation() {
        let err = Value::Error(ErrorKind::Ref);
        assert_eq!(fn_len(&[err.clone()]), Value::Error(ErrorKind::Ref));
        assert_eq!(fn_upper(&[err.clone()]), Value::Error(ErrorKind::Ref));
        assert_eq!(fn_left(&[err, num(1.0)]), Value::Error(ErrorKind::Ref));
    }
}
