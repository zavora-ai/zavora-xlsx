//! Tests for core formula functions: SUM, AVERAGE, COUNT, COUNTA, MIN, MAX,
//! IF, VLOOKUP, INDEX, MATCH.

use zavora_xlsx::formula_engine::evaluator::{evaluate, ErrorKind, SimpleContext, Value};
use zavora_xlsx::formula_engine::{parse, tokenize};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn eval(formula: &str) -> Value {
    let tokens = tokenize(formula).unwrap();
    let ast = parse(&tokens).unwrap();
    let ctx = SimpleContext::new(1);
    evaluate(&ast, &ctx, 0)
}

fn eval_ctx(formula: &str, ctx: &SimpleContext) -> Value {
    let tokens = tokenize(formula).unwrap();
    let ast = parse(&tokens).unwrap();
    evaluate(&ast, ctx, 0)
}

/// Build a context with A1:A5 = [1, 2, 3, 4, 5].
fn numeric_column() -> SimpleContext {
    let mut ctx = SimpleContext::new(1);
    for i in 0..5u32 {
        ctx.set(0, i, 0, Value::Number((i + 1) as f64));
    }
    ctx
}

/// Build a context with mixed types in A1:A5.
fn mixed_column() -> SimpleContext {
    let mut ctx = SimpleContext::new(1);
    ctx.set(0, 0, 0, Value::Number(10.0));
    ctx.set(0, 1, 0, Value::String("hello".into()));
    ctx.set(0, 2, 0, Value::Bool(true));
    ctx.set(0, 3, 0, Value::Number(20.0));
    // A5 is empty
    ctx
}

// ===========================================================================
// SUM tests
// ===========================================================================

#[test]
fn test_sum_basic_range() {
    let ctx = numeric_column();
    assert_eq!(eval_ctx("SUM(A1:A5)", &ctx), Value::Number(15.0));
}

#[test]
fn test_sum_single_value() {
    assert_eq!(eval("SUM(42)"), Value::Number(42.0));
}

#[test]
fn test_sum_multiple_args() {
    assert_eq!(eval("SUM(1,2,3)"), Value::Number(6.0));
}

#[test]
fn test_sum_empty_range() {
    let ctx = SimpleContext::new(1);
    assert_eq!(eval_ctx("SUM(A1:A3)", &ctx), Value::Number(0.0));
}

#[test]
fn test_sum_skips_strings_in_range() {
    let ctx = mixed_column();
    // A1=10, A2="hello"(skip), A3=TRUE(skip in range), A4=20, A5=empty(skip)
    assert_eq!(eval_ctx("SUM(A1:A5)", &ctx), Value::Number(30.0));
}

#[test]
fn test_sum_error_propagation() {
    let mut ctx = SimpleContext::new(1);
    ctx.set(0, 0, 0, Value::Number(1.0));
    ctx.set(0, 1, 0, Value::Error(ErrorKind::Div0));
    assert_eq!(eval_ctx("SUM(A1:A2)", &ctx), Value::Error(ErrorKind::Div0));
}

#[test]
fn test_sum_direct_bool_arg() {
    assert_eq!(eval("SUM(1,TRUE,3)"), Value::Number(5.0));
}

// ===========================================================================
// AVERAGE tests
// ===========================================================================

#[test]
fn test_average_basic() {
    let ctx = numeric_column();
    assert_eq!(eval_ctx("AVERAGE(A1:A5)", &ctx), Value::Number(3.0));
}

#[test]
fn test_average_single_value() {
    assert_eq!(eval("AVERAGE(7)"), Value::Number(7.0));
}

#[test]
fn test_average_empty_returns_div0() {
    let ctx = SimpleContext::new(1);
    assert_eq!(eval_ctx("AVERAGE(A1:A3)", &ctx), Value::Error(ErrorKind::Div0));
}

#[test]
fn test_average_skips_strings_in_range() {
    let ctx = mixed_column();
    // Only A1=10 and A4=20 are numeric in the range
    assert_eq!(eval_ctx("AVERAGE(A1:A5)", &ctx), Value::Number(15.0));
}

// ===========================================================================
// COUNT tests
// ===========================================================================

#[test]
fn test_count_numeric_range() {
    let ctx = numeric_column();
    assert_eq!(eval_ctx("COUNT(A1:A5)", &ctx), Value::Number(5.0));
}

#[test]
fn test_count_mixed_range() {
    let ctx = mixed_column();
    // A1=10(yes), A2="hello"(no), A3=TRUE(no in range), A4=20(yes), A5=empty(no)
    assert_eq!(eval_ctx("COUNT(A1:A5)", &ctx), Value::Number(2.0));
}

#[test]
fn test_count_empty_range() {
    let ctx = SimpleContext::new(1);
    assert_eq!(eval_ctx("COUNT(A1:A3)", &ctx), Value::Number(0.0));
}

#[test]
fn test_count_direct_bool_counts() {
    assert_eq!(eval("COUNT(1,TRUE,3)"), Value::Number(3.0));
}

// ===========================================================================
// COUNTA tests
// ===========================================================================

#[test]
fn test_counta_mixed_range() {
    let ctx = mixed_column();
    // A1=10, A2="hello", A3=TRUE, A4=20 → 4 non-empty
    assert_eq!(eval_ctx("COUNTA(A1:A5)", &ctx), Value::Number(4.0));
}

#[test]
fn test_counta_all_empty() {
    let ctx = SimpleContext::new(1);
    assert_eq!(eval_ctx("COUNTA(A1:A3)", &ctx), Value::Number(0.0));
}

#[test]
fn test_counta_counts_strings() {
    let mut ctx = SimpleContext::new(1);
    ctx.set(0, 0, 0, Value::String("".into())); // empty string is non-empty
    assert_eq!(eval_ctx("COUNTA(A1)", &ctx), Value::Number(1.0));
}

// ===========================================================================
// MIN tests
// ===========================================================================

#[test]
fn test_min_basic() {
    let ctx = numeric_column();
    assert_eq!(eval_ctx("MIN(A1:A5)", &ctx), Value::Number(1.0));
}

#[test]
fn test_min_with_negatives() {
    assert_eq!(eval("MIN(5,-3,0,2)"), Value::Number(-3.0));
}

#[test]
fn test_min_empty_returns_zero() {
    let ctx = SimpleContext::new(1);
    assert_eq!(eval_ctx("MIN(A1:A3)", &ctx), Value::Number(0.0));
}

#[test]
fn test_min_skips_strings_in_range() {
    let ctx = mixed_column();
    assert_eq!(eval_ctx("MIN(A1:A5)", &ctx), Value::Number(10.0));
}

// ===========================================================================
// MAX tests
// ===========================================================================

#[test]
fn test_max_basic() {
    let ctx = numeric_column();
    assert_eq!(eval_ctx("MAX(A1:A5)", &ctx), Value::Number(5.0));
}

#[test]
fn test_max_with_negatives() {
    assert_eq!(eval("MAX(-5,-3,-1)"), Value::Number(-1.0));
}

#[test]
fn test_max_empty_returns_zero() {
    let ctx = SimpleContext::new(1);
    assert_eq!(eval_ctx("MAX(A1:A3)", &ctx), Value::Number(0.0));
}

// ===========================================================================
// IF tests
// ===========================================================================

#[test]
fn test_if_true_branch() {
    assert_eq!(eval("IF(TRUE,1,2)"), Value::Number(1.0));
}

#[test]
fn test_if_false_branch() {
    assert_eq!(eval("IF(FALSE,1,2)"), Value::Number(2.0));
}

#[test]
fn test_if_two_args_true() {
    assert_eq!(eval("IF(TRUE,42)"), Value::Number(42.0));
}

#[test]
fn test_if_two_args_false() {
    assert_eq!(eval("IF(FALSE,42)"), Value::Bool(false));
}

#[test]
fn test_if_numeric_condition() {
    // Non-zero is truthy
    assert_eq!(eval("IF(1,10,20)"), Value::Number(10.0));
    assert_eq!(eval("IF(0,10,20)"), Value::Number(20.0));
}

#[test]
fn test_if_string_condition_error() {
    // Strings can't be coerced to bool
    assert_eq!(eval("IF(\"yes\",1,2)"), Value::Error(ErrorKind::Value));
}

#[test]
fn test_if_with_comparison() {
    let mut ctx = SimpleContext::new(1);
    ctx.set(0, 0, 0, Value::Number(10.0));
    assert_eq!(eval_ctx("IF(A1>5,\"big\",\"small\")", &ctx), Value::String("big".into()));
    ctx.set(0, 0, 0, Value::Number(3.0));
    assert_eq!(eval_ctx("IF(A1>5,\"big\",\"small\")", &ctx), Value::String("small".into()));
}

// ===========================================================================
// VLOOKUP tests
// ===========================================================================

/// Build a lookup table in A1:C4:
///   1  "Apple"   0.50
///   2  "Banana"  0.30
///   3  "Cherry"  0.75
///   4  "Date"    1.00
fn vlookup_ctx() -> SimpleContext {
    let mut ctx = SimpleContext::new(1);
    let data = [
        (1.0, "Apple", 0.50),
        (2.0, "Banana", 0.30),
        (3.0, "Cherry", 0.75),
        (4.0, "Date", 1.00),
    ];
    for (r, (id, name, price)) in data.iter().enumerate() {
        ctx.set(0, r as u32, 0, Value::Number(*id));
        ctx.set(0, r as u32, 1, Value::String(name.to_string()));
        ctx.set(0, r as u32, 2, Value::Number(*price));
    }
    ctx
}

#[test]
fn test_vlookup_exact_match() {
    let ctx = vlookup_ctx();
    // VLOOKUP(2, A1:C4, 2, FALSE) → "Banana"
    assert_eq!(
        eval_ctx("VLOOKUP(2,A1:C4,2,FALSE)", &ctx),
        Value::String("Banana".into())
    );
}

#[test]
fn test_vlookup_exact_match_not_found() {
    let ctx = vlookup_ctx();
    assert_eq!(
        eval_ctx("VLOOKUP(5,A1:C4,2,FALSE)", &ctx),
        Value::Error(ErrorKind::Na)
    );
}

#[test]
fn test_vlookup_approximate_match() {
    let ctx = vlookup_ctx();
    // VLOOKUP(2.5, A1:C4, 2, TRUE) → "Banana" (largest ≤ 2.5 is 2)
    assert_eq!(
        eval_ctx("VLOOKUP(2.5,A1:C4,2,TRUE)", &ctx),
        Value::String("Banana".into())
    );
}

#[test]
fn test_vlookup_approximate_match_exact_hit() {
    let ctx = vlookup_ctx();
    assert_eq!(
        eval_ctx("VLOOKUP(3,A1:C4,3,TRUE)", &ctx),
        Value::Number(0.75)
    );
}

#[test]
fn test_vlookup_col_index_out_of_range() {
    let ctx = vlookup_ctx();
    assert_eq!(
        eval_ctx("VLOOKUP(1,A1:C4,5,FALSE)", &ctx),
        Value::Error(ErrorKind::Ref)
    );
}

#[test]
fn test_vlookup_col_index_zero() {
    let ctx = vlookup_ctx();
    assert_eq!(
        eval_ctx("VLOOKUP(1,A1:C4,0,FALSE)", &ctx),
        Value::Error(ErrorKind::Value)
    );
}

#[test]
fn test_vlookup_default_approximate() {
    let ctx = vlookup_ctx();
    // Omitting range_lookup defaults to TRUE (approximate)
    assert_eq!(
        eval_ctx("VLOOKUP(3,A1:C4,2)", &ctx),
        Value::String("Cherry".into())
    );
}

// ===========================================================================
// INDEX tests
// ===========================================================================

#[test]
fn test_index_basic() {
    let ctx = numeric_column();
    // INDEX(A1:A5, 3) → 3
    assert_eq!(eval_ctx("INDEX(A1:A5,3)", &ctx), Value::Number(3.0));
}

#[test]
fn test_index_2d() {
    let ctx = vlookup_ctx();
    // INDEX(A1:C4, 2, 2) → "Banana"
    assert_eq!(
        eval_ctx("INDEX(A1:C4,2,2)", &ctx),
        Value::String("Banana".into())
    );
}

#[test]
fn test_index_out_of_bounds() {
    let ctx = numeric_column();
    assert_eq!(eval_ctx("INDEX(A1:A5,10)", &ctx), Value::Error(ErrorKind::Ref));
}

#[test]
fn test_index_row_zero_returns_column() {
    let ctx = vlookup_ctx();
    // INDEX(A1:C4, 0, 2) → entire column 2 as array
    let result = eval_ctx("INDEX(A1:C4,0,2)", &ctx);
    match result {
        Value::Array(rows) => {
            assert_eq!(rows.len(), 4);
            assert_eq!(rows[0][0], Value::String("Apple".into()));
            assert_eq!(rows[1][0], Value::String("Banana".into()));
        }
        other => panic!("Expected Array, got {:?}", other),
    }
}

// ===========================================================================
// MATCH tests
// ===========================================================================

#[test]
fn test_match_exact() {
    let ctx = numeric_column();
    // MATCH(3, A1:A5, 0) → 3
    assert_eq!(eval_ctx("MATCH(3,A1:A5,0)", &ctx), Value::Number(3.0));
}

#[test]
fn test_match_exact_not_found() {
    let ctx = numeric_column();
    assert_eq!(eval_ctx("MATCH(6,A1:A5,0)", &ctx), Value::Error(ErrorKind::Na));
}

#[test]
fn test_match_ascending_default() {
    let ctx = numeric_column();
    // MATCH(3.5, A1:A5) → 3 (default match_type=1, largest ≤ 3.5)
    assert_eq!(eval_ctx("MATCH(3.5,A1:A5)", &ctx), Value::Number(3.0));
}

#[test]
fn test_match_descending() {
    let mut ctx = SimpleContext::new(1);
    // Descending: 5, 4, 3, 2, 1
    for i in 0..5u32 {
        ctx.set(0, i, 0, Value::Number((5 - i) as f64));
    }
    // MATCH(3.5, A1:A5, -1) → smallest ≥ 3.5 is 4 at position 2
    assert_eq!(eval_ctx("MATCH(3.5,A1:A5,-1)", &ctx), Value::Number(2.0));
}

#[test]
fn test_match_string_exact() {
    let mut ctx = SimpleContext::new(1);
    ctx.set(0, 0, 0, Value::String("Apple".into()));
    ctx.set(0, 1, 0, Value::String("Banana".into()));
    ctx.set(0, 2, 0, Value::String("Cherry".into()));
    // Case-insensitive match
    assert_eq!(
        eval_ctx("MATCH(\"banana\",A1:A3,0)", &ctx),
        Value::Number(2.0)
    );
}

// ===========================================================================
// INDEX + MATCH combination
// ===========================================================================

#[test]
fn test_index_match_combination() {
    let ctx = vlookup_ctx();
    // INDEX(B1:B4, MATCH(3, A1:A4, 0)) → "Cherry"
    assert_eq!(
        eval_ctx("INDEX(B1:B4,MATCH(3,A1:A4,0))", &ctx),
        Value::String("Cherry".into())
    );
}

// ===========================================================================
// Unknown function
// ===========================================================================

#[test]
fn test_unknown_function_returns_name_error() {
    assert_eq!(eval("FOOBAR(1,2)"), Value::Error(ErrorKind::Name));
}
