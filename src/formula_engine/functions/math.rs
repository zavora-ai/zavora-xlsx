//! Math functions: ROUND, ROUNDUP, ROUNDDOWN, ABS, MOD, POWER, SQRT, LOG, LN, EXP, PI.

use crate::formula_engine::evaluator::{ErrorKind, Value};

/// ROUND(number, num_digits)
pub fn fn_round(args: &[Value]) -> Value {
    if args.len() != 2 {
        return Value::Error(ErrorKind::Value);
    }
    let n = match args[0].to_number() {
        Ok(v) => v,
        Err(e) => return Value::Error(e),
    };
    let digits = match args[1].to_number() {
        Ok(v) => v as i32,
        Err(e) => return Value::Error(e),
    };
    Value::Number(round_half_away(n, digits))
}

/// ROUNDUP(number, num_digits)
pub fn fn_roundup(args: &[Value]) -> Value {
    if args.len() != 2 {
        return Value::Error(ErrorKind::Value);
    }
    let n = match args[0].to_number() {
        Ok(v) => v,
        Err(e) => return Value::Error(e),
    };
    let digits = match args[1].to_number() {
        Ok(v) => v as i32,
        Err(e) => return Value::Error(e),
    };
    let factor = 10_f64.powi(digits);
    let result = if n >= 0.0 {
        (n * factor).ceil() / factor
    } else {
        (n * factor).floor() / factor
    };
    Value::Number(result)
}

/// ROUNDDOWN(number, num_digits)
pub fn fn_rounddown(args: &[Value]) -> Value {
    if args.len() != 2 {
        return Value::Error(ErrorKind::Value);
    }
    let n = match args[0].to_number() {
        Ok(v) => v,
        Err(e) => return Value::Error(e),
    };
    let digits = match args[1].to_number() {
        Ok(v) => v as i32,
        Err(e) => return Value::Error(e),
    };
    let factor = 10_f64.powi(digits);
    let result = if n >= 0.0 {
        (n * factor).floor() / factor
    } else {
        (n * factor).ceil() / factor
    };
    Value::Number(result)
}

/// ABS(number)
pub fn fn_abs(args: &[Value]) -> Value {
    if args.len() != 1 {
        return Value::Error(ErrorKind::Value);
    }
    match args[0].to_number() {
        Ok(n) => Value::Number(n.abs()),
        Err(e) => Value::Error(e),
    }
}

/// MOD(number, divisor)
pub fn fn_mod(args: &[Value]) -> Value {
    if args.len() != 2 {
        return Value::Error(ErrorKind::Value);
    }
    let n = match args[0].to_number() {
        Ok(v) => v,
        Err(e) => return Value::Error(e),
    };
    let d = match args[1].to_number() {
        Ok(v) => v,
        Err(e) => return Value::Error(e),
    };
    if d == 0.0 {
        return Value::Error(ErrorKind::Div0);
    }
    // Excel MOD: result has the same sign as the divisor
    let result = n - d * (n / d).floor();
    Value::Number(result)
}

/// POWER(number, power)
pub fn fn_power(args: &[Value]) -> Value {
    if args.len() != 2 {
        return Value::Error(ErrorKind::Value);
    }
    let base = match args[0].to_number() {
        Ok(v) => v,
        Err(e) => return Value::Error(e),
    };
    let exp = match args[1].to_number() {
        Ok(v) => v,
        Err(e) => return Value::Error(e),
    };
    let result = base.powf(exp);
    if result.is_nan() || result.is_infinite() {
        Value::Error(ErrorKind::Num)
    } else {
        Value::Number(result)
    }
}

/// SQRT(number)
pub fn fn_sqrt(args: &[Value]) -> Value {
    if args.len() != 1 {
        return Value::Error(ErrorKind::Value);
    }
    let n = match args[0].to_number() {
        Ok(v) => v,
        Err(e) => return Value::Error(e),
    };
    if n < 0.0 {
        Value::Error(ErrorKind::Num)
    } else {
        Value::Number(n.sqrt())
    }
}

/// LOG(`number`, \[`base`\])
/// Default base is 10.
pub fn fn_log(args: &[Value]) -> Value {
    if args.is_empty() || args.len() > 2 {
        return Value::Error(ErrorKind::Value);
    }
    let n = match args[0].to_number() {
        Ok(v) => v,
        Err(e) => return Value::Error(e),
    };
    if n <= 0.0 {
        return Value::Error(ErrorKind::Num);
    }
    let base = if args.len() == 2 {
        match args[1].to_number() {
            Ok(v) => v,
            Err(e) => return Value::Error(e),
        }
    } else {
        10.0
    };
    if base <= 0.0 || base == 1.0 {
        return Value::Error(ErrorKind::Num);
    }
    Value::Number(n.ln() / base.ln())
}

/// LN(number)
pub fn fn_ln(args: &[Value]) -> Value {
    if args.len() != 1 {
        return Value::Error(ErrorKind::Value);
    }
    let n = match args[0].to_number() {
        Ok(v) => v,
        Err(e) => return Value::Error(e),
    };
    if n <= 0.0 {
        Value::Error(ErrorKind::Num)
    } else {
        Value::Number(n.ln())
    }
}

/// EXP(number)
pub fn fn_exp(args: &[Value]) -> Value {
    if args.len() != 1 {
        return Value::Error(ErrorKind::Value);
    }
    match args[0].to_number() {
        Ok(n) => Value::Number(n.exp()),
        Err(e) => Value::Error(e),
    }
}

/// PI()
pub fn fn_pi(args: &[Value]) -> Value {
    if !args.is_empty() {
        return Value::Error(ErrorKind::Value);
    }
    Value::Number(std::f64::consts::PI)
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Round half away from zero (Excel's rounding behavior).
fn round_half_away(n: f64, digits: i32) -> f64 {
    let factor = 10_f64.powi(digits);
    let shifted = n * factor;
    // Use round which does round-half-to-even in Rust, but we want half-away-from-zero
    let rounded = if shifted >= 0.0 {
        (shifted + 0.5).floor()
    } else {
        (shifted - 0.5).ceil()
    };
    rounded / factor
}

#[cfg(test)]
mod tests {
    use super::*;

    fn num(n: f64) -> Value {
        Value::Number(n)
    }

    #[test]
    fn test_round() {
        assert_eq!(fn_round(&[num(2.15), num(1.0)]), num(2.2));
        assert_eq!(fn_round(&[num(2.149), num(1.0)]), num(2.1));
        assert_eq!(fn_round(&[num(-1.475), num(2.0)]), num(-1.48));
        assert_eq!(fn_round(&[num(21.5), num(-1.0)]), num(20.0));
        assert_eq!(fn_round(&[num(25.0), num(-1.0)]), num(30.0));
    }

    #[test]
    fn test_roundup() {
        assert_eq!(fn_roundup(&[num(3.2), num(0.0)]), num(4.0));
        assert_eq!(fn_roundup(&[num(76.9), num(0.0)]), num(77.0));
        assert_eq!(fn_roundup(&[num(4.14159), num(3.0)]), num(4.142));
        assert_eq!(fn_roundup(&[num(-4.14159), num(1.0)]), num(-4.2));
    }

    #[test]
    fn test_rounddown() {
        assert_eq!(fn_rounddown(&[num(3.2), num(0.0)]), num(3.0));
        assert_eq!(fn_rounddown(&[num(76.9), num(0.0)]), num(76.0));
        assert_eq!(fn_rounddown(&[num(4.14159), num(3.0)]), num(4.141));
        assert_eq!(fn_rounddown(&[num(-4.14159), num(1.0)]), num(-4.1));
    }

    #[test]
    fn test_abs() {
        assert_eq!(fn_abs(&[num(-5.0)]), num(5.0));
        assert_eq!(fn_abs(&[num(5.0)]), num(5.0));
        assert_eq!(fn_abs(&[num(0.0)]), num(0.0));
    }

    #[test]
    fn test_mod() {
        assert_eq!(fn_mod(&[num(3.0), num(2.0)]), num(1.0));
        assert_eq!(fn_mod(&[num(-3.0), num(2.0)]), num(1.0));
        assert_eq!(fn_mod(&[num(3.0), num(-2.0)]), num(-1.0));
        assert_eq!(fn_mod(&[num(5.0), num(0.0)]), Value::Error(ErrorKind::Div0));
    }

    #[test]
    fn test_power() {
        assert_eq!(fn_power(&[num(2.0), num(3.0)]), num(8.0));
        assert_eq!(fn_power(&[num(4.0), num(0.5)]), num(2.0));
        assert_eq!(fn_power(&[num(0.0), num(0.0)]), num(1.0));
    }

    #[test]
    fn test_sqrt() {
        assert_eq!(fn_sqrt(&[num(4.0)]), num(2.0));
        assert_eq!(fn_sqrt(&[num(0.0)]), num(0.0));
        assert_eq!(fn_sqrt(&[num(-1.0)]), Value::Error(ErrorKind::Num));
    }

    #[test]
    fn test_log() {
        assert_eq!(fn_log(&[num(100.0)]), num(2.0));
        assert_eq!(fn_log(&[num(8.0), num(2.0)]), num(3.0));
        assert_eq!(fn_log(&[num(0.0)]), Value::Error(ErrorKind::Num));
        assert_eq!(fn_log(&[num(-1.0)]), Value::Error(ErrorKind::Num));
    }

    #[test]
    fn test_ln() {
        let result = fn_ln(&[num(std::f64::consts::E)]);
        if let Value::Number(n) = result {
            assert!((n - 1.0).abs() < 1e-10);
        } else {
            panic!("Expected number");
        }
        assert_eq!(fn_ln(&[num(0.0)]), Value::Error(ErrorKind::Num));
        assert_eq!(fn_ln(&[num(-1.0)]), Value::Error(ErrorKind::Num));
    }

    #[test]
    fn test_exp() {
        assert_eq!(fn_exp(&[num(0.0)]), num(1.0));
        let result = fn_exp(&[num(1.0)]);
        if let Value::Number(n) = result {
            assert!((n - std::f64::consts::E).abs() < 1e-10);
        } else {
            panic!("Expected number");
        }
    }

    #[test]
    fn test_pi() {
        assert_eq!(fn_pi(&[]), Value::Number(std::f64::consts::PI));
        assert_eq!(fn_pi(&[num(1.0)]), Value::Error(ErrorKind::Value));
    }

    #[test]
    fn test_error_propagation() {
        let err = Value::Error(ErrorKind::Ref);
        assert_eq!(
            fn_abs(std::slice::from_ref(&err)),
            Value::Error(ErrorKind::Ref)
        );
        assert_eq!(
            fn_sqrt(std::slice::from_ref(&err)),
            Value::Error(ErrorKind::Ref)
        );
        assert_eq!(fn_round(&[err, num(1.0)]), Value::Error(ErrorKind::Ref));
    }
}
