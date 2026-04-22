//! Formula evaluator: walks the AST and computes a result value.
//!
//! The evaluator uses a `CellContext` trait to resolve cell references,
//! allowing it to work with any backing store.

use crate::formula_engine::ast::AstNode;
use crate::formula_engine::token::Op;

/// The result of evaluating a formula expression.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Number(f64),
    String(String),
    Bool(bool),
    Error(ErrorKind),
    Empty,
    /// A range of values (for function arguments).
    Array(Vec<Vec<Value>>),
}

/// Formula error types matching Excel's error values.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorKind {
    Value, // #VALUE!
    Ref,   // #REF!
    Div0,  // #DIV/0!
    Name,  // #NAME?
    Num,   // #NUM!
    Na,    // #N/A
    Null,  // #NULL!
    Spill, // #SPILL!
}

impl std::fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorKind::Value => write!(f, "#VALUE!"),
            ErrorKind::Ref => write!(f, "#REF!"),
            ErrorKind::Div0 => write!(f, "#DIV/0!"),
            ErrorKind::Name => write!(f, "#NAME?"),
            ErrorKind::Num => write!(f, "#NUM!"),
            ErrorKind::Na => write!(f, "#N/A"),
            ErrorKind::Null => write!(f, "#NULL!"),
            ErrorKind::Spill => write!(f, "#SPILL!"),
        }
    }
}

impl Value {
    /// Coerce to a number for arithmetic. Empty → 0, Bool → 0/1, String → parse or error.
    pub fn to_number(&self) -> Result<f64, ErrorKind> {
        match self {
            Value::Number(n) => Ok(*n),
            Value::Bool(b) => Ok(if *b { 1.0 } else { 0.0 }),
            Value::Empty => Ok(0.0),
            Value::String(s) => s.parse::<f64>().map_err(|_| ErrorKind::Value),
            Value::Error(e) => Err(*e),
            Value::Array(_) => Err(ErrorKind::Value),
        }
    }

    /// Coerce to a string for concatenation.
    pub fn to_string_val(&self) -> Result<String, ErrorKind> {
        match self {
            Value::String(s) => Ok(s.clone()),
            Value::Number(n) => Ok(format_number(*n)),
            Value::Bool(b) => Ok(if *b { "TRUE".into() } else { "FALSE".into() }),
            Value::Empty => Ok(String::new()),
            Value::Error(e) => Err(*e),
            Value::Array(_) => Err(ErrorKind::Value),
        }
    }

    /// Coerce to bool for logical operations.
    pub fn to_bool(&self) -> Result<bool, ErrorKind> {
        match self {
            Value::Bool(b) => Ok(*b),
            Value::Number(n) => Ok(*n != 0.0),
            Value::Empty => Ok(false),
            Value::String(_) => Err(ErrorKind::Value),
            Value::Error(e) => Err(*e),
            Value::Array(_) => Err(ErrorKind::Value),
        }
    }
}

fn format_number(n: f64) -> String {
    if n == n.trunc() && n.abs() < 1e15 {
        format!("{}", n as i64)
    } else {
        format!("{n}")
    }
}

/// Trait for resolving cell references during evaluation.
pub trait CellContext {
    /// Get the value of a cell. Returns `Value::Empty` if the cell is blank.
    fn get_cell(&self, sheet: usize, row: u32, col: u16) -> Value;
}

/// A simple in-memory context for testing.
pub struct SimpleContext {
    /// cells[sheet][(row, col)] = value
    pub cells: Vec<std::collections::HashMap<(u32, u16), Value>>,
}

impl SimpleContext {
    pub fn new(num_sheets: usize) -> Self {
        Self {
            cells: vec![std::collections::HashMap::new(); num_sheets],
        }
    }

    pub fn set(&mut self, sheet: usize, row: u32, col: u16, val: Value) {
        self.cells[sheet].insert((row, col), val);
    }
}

impl CellContext for SimpleContext {
    fn get_cell(&self, sheet: usize, row: u32, col: u16) -> Value {
        self.cells
            .get(sheet)
            .and_then(|s| s.get(&(row, col)))
            .cloned()
            .unwrap_or(Value::Empty)
    }
}

/// Evaluate an AST node in the given context.
///
/// `current_sheet` is the sheet index of the cell being evaluated.
pub fn evaluate(node: &AstNode, ctx: &dyn CellContext, current_sheet: usize) -> Value {
    match node {
        AstNode::Number(n) => Value::Number(*n),
        AstNode::String(s) => Value::String(s.clone()),
        AstNode::Bool(b) => Value::Bool(*b),
        AstNode::Error(e) => Value::Error(parse_error_kind(e)),

        AstNode::CellRef { col, row, .. } => ctx.get_cell(current_sheet, *row, *col),

        AstNode::SheetRef { inner, .. } => {
            // For now, evaluate in current sheet (full impl would resolve sheet name)
            evaluate(inner, ctx, current_sheet)
        }

        AstNode::Range { start, end } => {
            // Expand range to array of values
            if let (
                AstNode::CellRef {
                    col: c1, row: r1, ..
                },
                AstNode::CellRef {
                    col: c2, row: r2, ..
                },
            ) = (start.as_ref(), end.as_ref())
            {
                let min_r = (*r1).min(*r2);
                let max_r = (*r1).max(*r2);
                let min_c = (*c1).min(*c2);
                let max_c = (*c1).max(*c2);
                let mut rows = Vec::new();
                for r in min_r..=max_r {
                    let mut row = Vec::new();
                    for c in min_c..=max_c {
                        row.push(ctx.get_cell(current_sheet, r, c));
                    }
                    rows.push(row);
                }
                Value::Array(rows)
            } else {
                Value::Error(ErrorKind::Ref)
            }
        }

        AstNode::UnaryOp { op, operand } => {
            let val = evaluate(operand, ctx, current_sheet);
            match op {
                Op::Sub => match val.to_number() {
                    Ok(n) => Value::Number(-n),
                    Err(e) => Value::Error(e),
                },
                Op::Percent => match val.to_number() {
                    Ok(n) => Value::Number(n / 100.0),
                    Err(e) => Value::Error(e),
                },
                _ => Value::Error(ErrorKind::Value),
            }
        }

        AstNode::BinaryOp { op, left, right } => {
            eval_binary_op(*op, left, right, ctx, current_sheet)
        }

        AstNode::FunctionCall { name, args } => {
            // Evaluate args and delegate to function registry
            // For now, just handle basic cases
            let eval_args: Vec<Value> = args
                .iter()
                .map(|a| evaluate(a, ctx, current_sheet))
                .collect();
            eval_function(name, &eval_args)
        }

        AstNode::Array { rows } => {
            let eval_rows: Vec<Vec<Value>> = rows
                .iter()
                .map(|row| {
                    row.iter()
                        .map(|cell| evaluate(cell, ctx, current_sheet))
                        .collect()
                })
                .collect();
            Value::Array(eval_rows)
        }

        _ => Value::Error(ErrorKind::Value),
    }
}

fn eval_binary_op(
    op: Op,
    left: &AstNode,
    right: &AstNode,
    ctx: &dyn CellContext,
    sheet: usize,
) -> Value {
    let lval = evaluate(left, ctx, sheet);
    let rval = evaluate(right, ctx, sheet);

    // Propagate errors
    if let Value::Error(e) = &lval {
        return Value::Error(*e);
    }
    if let Value::Error(e) = &rval {
        return Value::Error(*e);
    }

    match op {
        Op::Add | Op::Sub | Op::Mul | Op::Div | Op::Pow => {
            let l = match lval.to_number() {
                Ok(n) => n,
                Err(e) => return Value::Error(e),
            };
            let r = match rval.to_number() {
                Ok(n) => n,
                Err(e) => return Value::Error(e),
            };
            let result = match op {
                Op::Add => l + r,
                Op::Sub => l - r,
                Op::Mul => l * r,
                Op::Div => {
                    if r == 0.0 {
                        return Value::Error(ErrorKind::Div0);
                    }
                    l / r
                }
                Op::Pow => l.powf(r),
                _ => unreachable!(),
            };
            Value::Number(result)
        }
        Op::Concat => {
            let l = match lval.to_string_val() {
                Ok(s) => s,
                Err(e) => return Value::Error(e),
            };
            let r = match rval.to_string_val() {
                Ok(s) => s,
                Err(e) => return Value::Error(e),
            };
            Value::String(format!("{l}{r}"))
        }
        Op::Eq | Op::Ne | Op::Lt | Op::Gt | Op::Le | Op::Ge => eval_comparison(op, &lval, &rval),
        Op::Percent => Value::Error(ErrorKind::Value), // shouldn't appear as binary
    }
}

fn eval_comparison(op: Op, lval: &Value, rval: &Value) -> Value {
    // Compare numbers to numbers, strings to strings (case-insensitive)
    match (lval, rval) {
        (Value::Number(l), Value::Number(r)) => Value::Bool(compare_op(op, l.partial_cmp(r))),
        (Value::String(l), Value::String(r)) => {
            let cmp = l.to_lowercase().cmp(&r.to_lowercase());
            Value::Bool(compare_op(op, Some(cmp)))
        }
        (Value::Bool(l), Value::Bool(r)) => Value::Bool(compare_op(op, l.cmp(r).into())),
        // Mixed types: numbers < strings < booleans in Excel's comparison
        (Value::Number(_), Value::String(_)) => Value::Bool(matches!(op, Op::Lt | Op::Le | Op::Ne)),
        (Value::String(_), Value::Number(_)) => Value::Bool(matches!(op, Op::Gt | Op::Ge | Op::Ne)),
        (Value::Empty, Value::Number(n)) => Value::Bool(compare_op(op, 0.0_f64.partial_cmp(n))),
        (Value::Number(n), Value::Empty) => Value::Bool(compare_op(op, n.partial_cmp(&0.0))),
        (Value::Empty, Value::String(s)) => Value::Bool(compare_op(op, "".cmp(s.as_str()).into())),
        (Value::String(s), Value::Empty) => Value::Bool(compare_op(op, Some(s.as_str().cmp("")))),
        (Value::Empty, Value::Empty) => Value::Bool(matches!(op, Op::Eq | Op::Le | Op::Ge)),
        _ => Value::Error(ErrorKind::Value),
    }
}

fn compare_op(op: Op, ord: Option<std::cmp::Ordering>) -> bool {
    use std::cmp::Ordering::*;
    match (op, ord) {
        (Op::Eq, Some(Equal)) => true,
        (Op::Ne, Some(o)) if o != Equal => true,
        (Op::Lt, Some(Less)) => true,
        (Op::Gt, Some(Greater)) => true,
        (Op::Le, Some(Less | Equal)) => true,
        (Op::Ge, Some(Greater | Equal)) => true,
        _ => false,
    }
}

fn parse_error_kind(s: &str) -> ErrorKind {
    match s {
        "#VALUE!" => ErrorKind::Value,
        "#REF!" => ErrorKind::Ref,
        "#DIV/0!" => ErrorKind::Div0,
        "#NAME?" => ErrorKind::Name,
        "#NUM!" => ErrorKind::Num,
        "#N/A" => ErrorKind::Na,
        "#NULL!" => ErrorKind::Null,
        "#SPILL!" => ErrorKind::Spill,
        _ => ErrorKind::Value,
    }
}

/// Dispatch a function call to the function registry.
fn eval_function(name: &str, args: &[Value]) -> Value {
    crate::formula_engine::functions::call_function(name, args)
}

// ---------------------------------------------------------------------------
// CSE Array Formula Evaluation
// ---------------------------------------------------------------------------

/// Result of evaluating a CSE (Ctrl+Shift+Enter) array formula.
#[derive(Debug, Clone, PartialEq)]
pub struct ArrayResult {
    /// The values to fill into the target range, row-major.
    pub values: Vec<Vec<Value>>,
    /// Number of rows in the result.
    pub rows: usize,
    /// Number of columns in the result.
    pub cols: usize,
}

/// Evaluate a formula as a CSE array formula over a target range.
///
/// The formula is evaluated once, and if it produces an array result,
/// the values are distributed across the target range. If the result
/// is a scalar, it's replicated across all cells.
pub fn evaluate_array(
    node: &AstNode,
    ctx: &dyn CellContext,
    current_sheet: usize,
    target_rows: usize,
    target_cols: usize,
) -> ArrayResult {
    let result = evaluate(node, ctx, current_sheet);

    match result {
        Value::Array(rows) => {
            let mut output = Vec::with_capacity(target_rows);
            for r in 0..target_rows {
                let mut row = Vec::with_capacity(target_cols);
                for c in 0..target_cols {
                    let val = rows
                        .get(r)
                        .and_then(|row| row.get(c))
                        .cloned()
                        .unwrap_or(Value::Error(ErrorKind::Na));
                    row.push(val);
                }
                output.push(row);
            }
            ArrayResult {
                values: output,
                rows: target_rows,
                cols: target_cols,
            }
        }
        scalar => {
            // Replicate scalar across the entire target range
            let output = vec![vec![scalar; target_cols]; target_rows];
            ArrayResult {
                values: output,
                rows: target_rows,
                cols: target_cols,
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Dynamic Array Spill
// ---------------------------------------------------------------------------

/// Result of computing a dynamic array spill range.
#[derive(Debug, Clone, PartialEq)]
pub enum SpillResult {
    /// The formula produced values that can spill into the given range.
    Ok(ArrayResult),
    /// The spill range overlaps with non-empty cells.
    SpillError,
}

/// Evaluate a dynamic array formula and compute its spill range.
///
/// Checks that the spill range (starting at `origin_row`, `origin_col`)
/// doesn't overlap with existing non-empty cells. Returns `SpillError`
/// if it does.
pub fn evaluate_dynamic_array(
    node: &AstNode,
    ctx: &dyn CellContext,
    current_sheet: usize,
    origin_row: u32,
    origin_col: u16,
) -> SpillResult {
    let result = evaluate(node, ctx, current_sheet);

    match result {
        Value::Array(rows) => {
            let num_rows = rows.len();
            let num_cols = rows.first().map_or(0, |r| r.len());

            if num_rows == 0 || num_cols == 0 {
                return SpillResult::Ok(ArrayResult {
                    values: vec![],
                    rows: 0,
                    cols: 0,
                });
            }

            // Check for spill conflicts (skip the origin cell itself)
            for r in 0..num_rows {
                for c in 0..num_cols {
                    if r == 0 && c == 0 {
                        continue;
                    } // origin cell is the formula cell
                    let cell_row = origin_row + r as u32;
                    let cell_col = origin_col + c as u16;
                    let existing = ctx.get_cell(current_sheet, cell_row, cell_col);
                    if !matches!(existing, Value::Empty) {
                        return SpillResult::SpillError;
                    }
                }
            }

            SpillResult::Ok(ArrayResult {
                values: rows,
                rows: num_rows,
                cols: num_cols,
            })
        }
        scalar => {
            // Single value, no spill needed
            SpillResult::Ok(ArrayResult {
                values: vec![vec![scalar]],
                rows: 1,
                cols: 1,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::formula_engine::{parse, tokenize};

    fn eval(formula: &str) -> Value {
        let tokens = tokenize(formula).unwrap();
        let ast = parse(&tokens).unwrap();
        let ctx = SimpleContext::new(1);
        evaluate(&ast, &ctx, 0)
    }

    fn eval_with_ctx(formula: &str, ctx: &SimpleContext) -> Value {
        let tokens = tokenize(formula).unwrap();
        let ast = parse(&tokens).unwrap();
        evaluate(&ast, ctx, 0)
    }

    #[test]
    fn test_arithmetic() {
        assert_eq!(eval("1+2"), Value::Number(3.0));
        assert_eq!(eval("10-3"), Value::Number(7.0));
        assert_eq!(eval("4*5"), Value::Number(20.0));
        assert_eq!(eval("10/4"), Value::Number(2.5));
        assert_eq!(eval("2^3"), Value::Number(8.0));
    }

    #[test]
    fn test_precedence() {
        assert_eq!(eval("2+3*4"), Value::Number(14.0));
        assert_eq!(eval("(2+3)*4"), Value::Number(20.0));
    }

    #[test]
    fn test_unary_minus() {
        assert_eq!(eval("-5"), Value::Number(-5.0));
        assert_eq!(eval("-2+3"), Value::Number(1.0));
    }

    #[test]
    fn test_division_by_zero() {
        assert_eq!(eval("1/0"), Value::Error(ErrorKind::Div0));
    }

    #[test]
    fn test_comparison() {
        assert_eq!(eval("1=1"), Value::Bool(true));
        assert_eq!(eval("1=2"), Value::Bool(false));
        assert_eq!(eval("3>2"), Value::Bool(true));
        assert_eq!(eval("3<2"), Value::Bool(false));
        assert_eq!(eval("3>=3"), Value::Bool(true));
        assert_eq!(eval("3<=2"), Value::Bool(false));
        assert_eq!(eval("1<>2"), Value::Bool(true));
    }

    #[test]
    fn test_string_concat() {
        assert_eq!(
            eval(r#""Hello"&" World""#),
            Value::String("Hello World".into())
        );
    }

    #[test]
    fn test_cell_reference() {
        let mut ctx = SimpleContext::new(1);
        ctx.set(0, 0, 0, Value::Number(10.0)); // A1 = 10
        ctx.set(0, 0, 1, Value::Number(20.0)); // B1 = 20

        assert_eq!(eval_with_ctx("A1+B1", &ctx), Value::Number(30.0));
    }

    #[test]
    fn test_sum_range() {
        let mut ctx = SimpleContext::new(1);
        ctx.set(0, 0, 0, Value::Number(1.0)); // A1
        ctx.set(0, 1, 0, Value::Number(2.0)); // A2
        ctx.set(0, 2, 0, Value::Number(3.0)); // A3

        assert_eq!(eval_with_ctx("SUM(A1:A3)", &ctx), Value::Number(6.0));
    }

    #[test]
    fn test_empty_cell_is_zero() {
        let ctx = SimpleContext::new(1);
        assert_eq!(eval_with_ctx("A1+1", &ctx), Value::Number(1.0));
    }

    #[test]
    fn test_percent() {
        assert_eq!(eval("50%"), Value::Number(0.5));
    }

    #[test]
    fn test_error_propagation() {
        assert_eq!(eval("#VALUE!+1"), Value::Error(ErrorKind::Value));
    }

    // ── CSE Array Formula Tests ──

    #[test]
    fn test_cse_array_scalar_replication() {
        let ctx = SimpleContext::new(1);
        let tokens = tokenize("42").unwrap();
        let ast = parse(&tokens).unwrap();
        let result = evaluate_array(&ast, &ctx, 0, 3, 2);
        assert_eq!(result.rows, 3);
        assert_eq!(result.cols, 2);
        assert_eq!(result.values[0][0], Value::Number(42.0));
        assert_eq!(result.values[2][1], Value::Number(42.0));
    }

    #[test]
    fn test_cse_array_from_range() {
        let mut ctx = SimpleContext::new(1);
        ctx.set(0, 0, 0, Value::Number(1.0));
        ctx.set(0, 1, 0, Value::Number(2.0));
        ctx.set(0, 2, 0, Value::Number(3.0));

        let tokens = tokenize("A1:A3").unwrap();
        let ast = parse(&tokens).unwrap();
        let result = evaluate_array(&ast, &ctx, 0, 3, 1);
        assert_eq!(result.rows, 3);
        assert_eq!(result.cols, 1);
        assert_eq!(result.values[0][0], Value::Number(1.0));
        assert_eq!(result.values[1][0], Value::Number(2.0));
        assert_eq!(result.values[2][0], Value::Number(3.0));
    }

    #[test]
    fn test_cse_array_overflow_fills_na() {
        let mut ctx = SimpleContext::new(1);
        ctx.set(0, 0, 0, Value::Number(1.0));
        ctx.set(0, 1, 0, Value::Number(2.0));

        let tokens = tokenize("A1:A2").unwrap();
        let ast = parse(&tokens).unwrap();
        // Target is 4 rows but source only has 2
        let result = evaluate_array(&ast, &ctx, 0, 4, 1);
        assert_eq!(result.values[0][0], Value::Number(1.0));
        assert_eq!(result.values[1][0], Value::Number(2.0));
        assert_eq!(result.values[2][0], Value::Error(ErrorKind::Na));
        assert_eq!(result.values[3][0], Value::Error(ErrorKind::Na));
    }

    // ── Dynamic Array Spill Tests ──

    #[test]
    fn test_dynamic_spill_ok() {
        let mut ctx = SimpleContext::new(1);
        ctx.set(0, 0, 0, Value::Number(1.0));
        ctx.set(0, 1, 0, Value::Number(2.0));
        ctx.set(0, 2, 0, Value::Number(3.0));

        let tokens = tokenize("A1:A3").unwrap();
        let ast = parse(&tokens).unwrap();
        // Spill starting at B1 (row=0, col=1) — B2 and B3 should be empty
        let result = evaluate_dynamic_array(&ast, &ctx, 0, 0, 1);
        match result {
            SpillResult::Ok(arr) => {
                assert_eq!(arr.rows, 3);
                assert_eq!(arr.cols, 1);
            }
            SpillResult::SpillError => panic!("Expected Ok, got SpillError"),
        }
    }

    #[test]
    fn test_dynamic_spill_conflict() {
        let mut ctx = SimpleContext::new(1);
        ctx.set(0, 0, 0, Value::Number(1.0));
        ctx.set(0, 1, 0, Value::Number(2.0));
        ctx.set(0, 2, 0, Value::Number(3.0));
        // Put something in the spill range
        ctx.set(0, 1, 1, Value::String("blocker".into()));

        let tokens = tokenize("A1:A3").unwrap();
        let ast = parse(&tokens).unwrap();
        // Spill starting at B1 — B2 has a value, should conflict
        let result = evaluate_dynamic_array(&ast, &ctx, 0, 0, 1);
        assert_eq!(result, SpillResult::SpillError);
    }

    #[test]
    fn test_dynamic_spill_scalar() {
        let ctx = SimpleContext::new(1);
        let tokens = tokenize("42").unwrap();
        let ast = parse(&tokens).unwrap();
        let result = evaluate_dynamic_array(&ast, &ctx, 0, 0, 0);
        match result {
            SpillResult::Ok(arr) => {
                assert_eq!(arr.rows, 1);
                assert_eq!(arr.cols, 1);
                assert_eq!(arr.values[0][0], Value::Number(42.0));
            }
            SpillResult::SpillError => panic!("Scalar should not spill-error"),
        }
    }
}
