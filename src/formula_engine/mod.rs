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

pub use token::{tokenize, Op, Token};
pub use ast::AstNode;
pub use parser::parse;
pub use printer::print_formula;
pub use dependency::{CellAddr, DependencyGraph};
pub use evaluator::{evaluate, evaluate_array, evaluate_dynamic_array, ArrayResult, SpillResult, CellContext, ErrorKind, SimpleContext, Value};
pub use recalc::{recalculate, CircularRefError, MutableCellContext, MutableSimpleContext, is_volatile};
