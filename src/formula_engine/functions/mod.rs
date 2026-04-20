//! Function registry for the formula engine.
//!
//! Dispatches function calls by name to their implementations.

pub mod core;
pub mod math;
pub mod text;
pub mod date;
pub mod lookup;
pub mod statistical;
pub mod logical;

use crate::formula_engine::evaluator::{ErrorKind, Value};

/// Evaluate a function by name with the given arguments.
///
/// Returns `Value::Error(ErrorKind::Name)` if the function is not recognized.
pub fn call_function(name: &str, args: &[Value]) -> Value {
    match name {
        // Aggregate functions
        "SUM" => core::fn_sum(args),
        "AVERAGE" => core::fn_average(args),
        "COUNT" => core::fn_count(args),
        "COUNTA" => core::fn_counta(args),
        "MIN" => core::fn_min(args),
        "MAX" => core::fn_max(args),

        // Logical (core)
        "IF" => core::fn_if(args),

        // Lookup (core)
        "VLOOKUP" => core::fn_vlookup(args),
        "INDEX" => core::fn_index(args),
        "MATCH" => core::fn_match(args),

        // Math functions
        "ROUND" => math::fn_round(args),
        "ROUNDUP" => math::fn_roundup(args),
        "ROUNDDOWN" => math::fn_rounddown(args),
        "ABS" => math::fn_abs(args),
        "MOD" => math::fn_mod(args),
        "POWER" => math::fn_power(args),
        "SQRT" => math::fn_sqrt(args),
        "LOG" => math::fn_log(args),
        "LN" => math::fn_ln(args),
        "EXP" => math::fn_exp(args),
        "PI" => math::fn_pi(args),

        // Text functions
        "CONCATENATE" => text::fn_concatenate(args),
        "CONCAT" => text::fn_concat(args),
        "LEFT" => text::fn_left(args),
        "RIGHT" => text::fn_right(args),
        "MID" => text::fn_mid(args),
        "LEN" => text::fn_len(args),
        "TRIM" => text::fn_trim(args),
        "UPPER" => text::fn_upper(args),
        "LOWER" => text::fn_lower(args),
        "SUBSTITUTE" => text::fn_substitute(args),

        // Date functions
        "TODAY" => date::fn_today(args),
        "NOW" => date::fn_now(args),
        "DATE" => date::fn_date(args),
        "YEAR" => date::fn_year(args),
        "MONTH" => date::fn_month(args),
        "DAY" => date::fn_day(args),
        "EDATE" => date::fn_edate(args),
        "EOMONTH" => date::fn_eomonth(args),
        "NETWORKDAYS" => date::fn_networkdays(args),

        // Lookup functions
        "HLOOKUP" => lookup::fn_hlookup(args),
        "XLOOKUP" => lookup::fn_xlookup(args),
        "INDIRECT" => lookup::fn_indirect(args),
        "OFFSET" => lookup::fn_offset(args),

        // Statistical functions
        "STDEV" => statistical::fn_stdev(args),
        "VAR" => statistical::fn_var(args),
        "MEDIAN" => statistical::fn_median(args),
        "PERCENTILE" => statistical::fn_percentile(args),
        "RANK" => statistical::fn_rank(args),
        "COUNTIF" => statistical::fn_countif(args),
        "SUMIF" => statistical::fn_sumif(args),

        // Logical functions
        "AND" => logical::fn_and(args),
        "OR" => logical::fn_or(args),
        "NOT" => logical::fn_not(args),
        "IFERROR" => logical::fn_iferror(args),
        "IFNA" => logical::fn_ifna(args),
        "SWITCH" => logical::fn_switch(args),
        "IFS" => logical::fn_ifs(args),

        _ => Value::Error(ErrorKind::Name),
    }
}
