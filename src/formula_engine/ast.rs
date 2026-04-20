//! Abstract syntax tree for Excel formulas.

use crate::formula_engine::token::Op;

/// A node in the formula AST.
#[derive(Debug, Clone, PartialEq)]
pub enum AstNode {
    /// Numeric literal.
    Number(f64),
    /// String literal.
    String(String),
    /// Boolean literal.
    Bool(bool),
    /// Error literal (e.g. `#VALUE!`).
    Error(String),
    /// A1-style cell reference.
    CellRef {
        col: u16,
        row: u32,
        abs_col: bool,
        abs_row: bool,
    },
    /// R1C1-style cell reference.
    R1C1Ref {
        row: i32,
        col: i32,
        row_relative: bool,
        col_relative: bool,
    },
    /// Range (two refs joined by colon).
    Range {
        start: Box<AstNode>,
        end: Box<AstNode>,
    },
    /// Sheet-qualified reference.
    SheetRef {
        sheet: String,
        inner: Box<AstNode>,
    },
    /// Structured table reference.
    StructuredRef {
        table: String,
        column: String,
    },
    /// Binary operation.
    BinaryOp {
        op: Op,
        left: Box<AstNode>,
        right: Box<AstNode>,
    },
    /// Unary operation (negation, percent).
    UnaryOp {
        op: Op,
        operand: Box<AstNode>,
    },
    /// Function call.
    FunctionCall {
        name: String,
        args: Vec<AstNode>,
    },
    /// Array literal `{1,2;3,4}`.
    Array {
        rows: Vec<Vec<AstNode>>,
    },
}
