//! Formula engine: tokenizer, parser, dependency graph, and evaluator.
//!
//! This module provides the infrastructure for parsing, analyzing, and
//! evaluating Excel formula expressions.

pub mod ast;
pub mod dependency;
pub mod evaluator;
pub mod functions;
pub mod parser;
pub mod printer;
pub mod recalc;
pub mod token;

pub use ast::AstNode;
pub use dependency::{CellAddr, DependencyGraph};
pub use evaluator::{
    ArrayResult, CellContext, ErrorKind, SimpleContext, SpillResult, Value, evaluate,
    evaluate_array, evaluate_dynamic_array,
};
pub use parser::parse;
pub use printer::print_formula;
pub use recalc::{
    CircularRefError, MutableCellContext, MutableSimpleContext, is_volatile, recalculate,
};
pub use token::{Op, Token, tokenize};
