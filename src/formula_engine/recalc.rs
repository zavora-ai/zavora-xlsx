//! Recalculation engine: integrates dependency graph, cycle detection,
//! and volatile function handling into a single `recalculate()` flow.

use std::collections::HashSet;

use crate::formula_engine::ast::AstNode;
use crate::formula_engine::dependency::{CellAddr, DependencyGraph};
use crate::formula_engine::evaluator::{CellContext, Value, evaluate};

/// Set of function names that are volatile (must be re-evaluated every time).
const VOLATILE_FUNCTIONS: &[&str] = &["RAND", "RANDBETWEEN", "NOW", "TODAY", "INDIRECT"];

/// Error returned when recalculation encounters a circular reference.
#[derive(Debug, Clone, PartialEq)]
pub struct CircularRefError {
    /// The cells involved in the cycle.
    pub cells: Vec<CellAddr>,
}

impl std::fmt::Display for CircularRefError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Circular reference detected involving cells: ")?;
        for (i, cell) in self.cells.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "sheet{}!R{}C{}", cell.sheet, cell.row + 1, cell.col + 1)?;
        }
        Ok(())
    }
}

impl std::error::Error for CircularRefError {}

/// A mutable cell context that supports writing computed values back.
pub trait MutableCellContext: CellContext {
    /// Set the value of a cell after evaluation.
    fn set_cell(&mut self, sheet: usize, row: u32, col: u16, val: Value);
}

/// A simple mutable context for testing.
pub struct MutableSimpleContext {
    pub cells: Vec<std::collections::HashMap<(u32, u16), Value>>,
}

impl MutableSimpleContext {
    pub fn new(num_sheets: usize) -> Self {
        Self {
            cells: vec![std::collections::HashMap::new(); num_sheets],
        }
    }

    pub fn set(&mut self, sheet: usize, row: u32, col: u16, val: Value) {
        self.cells[sheet].insert((row, col), val);
    }

    pub fn get(&self, sheet: usize, row: u32, col: u16) -> Value {
        self.cells
            .get(sheet)
            .and_then(|s| s.get(&(row, col)))
            .cloned()
            .unwrap_or(Value::Empty)
    }
}

impl CellContext for MutableSimpleContext {
    fn get_cell(&self, sheet: usize, row: u32, col: u16) -> Value {
        self.get(sheet, row, col)
    }
}

impl MutableCellContext for MutableSimpleContext {
    fn set_cell(&mut self, sheet: usize, row: u32, col: u16, val: Value) {
        self.set(sheet, row, col, val);
    }
}

/// Recalculate all formula cells in dependency order.
///
/// 1. Builds the dependency graph from the provided formulas.
/// 2. Detects circular references — returns error if found.
/// 3. Computes topological order.
/// 4. Evaluates each cell in order, writing results back to the context.
/// 5. Re-evaluates volatile cells and their dependents.
///
/// Returns the number of cells evaluated.
pub fn recalculate(
    formulas: &[(CellAddr, AstNode)],
    ctx: &mut dyn MutableCellContext,
) -> Result<usize, CircularRefError> {
    // Build refs for the dependency graph
    let refs: Vec<(CellAddr, &AstNode)> = formulas.iter().map(|(addr, ast)| (*addr, ast)).collect();
    let graph = DependencyGraph::build_from_formulas(&refs);

    // Check for cycles
    let order = graph
        .topological_order()
        .map_err(|cycle_cells| CircularRefError { cells: cycle_cells })?;

    // Identify volatile cells
    let volatile_cells: HashSet<CellAddr> = formulas
        .iter()
        .filter(|(_, ast)| contains_volatile(ast))
        .map(|(addr, _)| *addr)
        .collect();

    // Build a lookup from CellAddr to AST
    let formula_map: std::collections::HashMap<CellAddr, &AstNode> =
        formulas.iter().map(|(addr, ast)| (*addr, ast)).collect();

    // Evaluate in topological order
    let mut eval_count = 0;
    for cell in &order {
        if let Some(ast) = formula_map.get(cell) {
            let val = evaluate(ast, ctx, cell.sheet);
            ctx.set_cell(cell.sheet, cell.row, cell.col, val);
            eval_count += 1;
        }
    }

    // Re-evaluate volatile cells and their dependents
    if !volatile_cells.is_empty() {
        let mut to_reevaluate: HashSet<CellAddr> = volatile_cells.clone();
        for &vcell in &volatile_cells {
            for dep in graph.dependents_of(vcell) {
                to_reevaluate.insert(dep);
            }
        }

        // Re-evaluate in topological order (only the affected cells)
        for cell in &order {
            if to_reevaluate.contains(cell)
                && let Some(ast) = formula_map.get(cell)
            {
                let val = evaluate(ast, ctx, cell.sheet);
                ctx.set_cell(cell.sheet, cell.row, cell.col, val);
                eval_count += 1;
            }
        }
    }

    Ok(eval_count)
}

/// Check if an AST contains any volatile function calls.
fn contains_volatile(node: &AstNode) -> bool {
    match node {
        AstNode::FunctionCall { name, args } => {
            if VOLATILE_FUNCTIONS.contains(&name.as_str()) {
                return true;
            }
            args.iter().any(contains_volatile)
        }
        AstNode::BinaryOp { left, right, .. } => {
            contains_volatile(left) || contains_volatile(right)
        }
        AstNode::UnaryOp { operand, .. } => contains_volatile(operand),
        AstNode::Range { start, end } => contains_volatile(start) || contains_volatile(end),
        AstNode::SheetRef { inner, .. } => contains_volatile(inner),
        AstNode::Array { rows } => rows.iter().any(|row| row.iter().any(contains_volatile)),
        _ => false,
    }
}

/// Check if a function name is volatile.
pub fn is_volatile(name: &str) -> bool {
    VOLATILE_FUNCTIONS.contains(&name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::formula_engine::{parse, tokenize};

    fn addr(sheet: usize, row: u32, col: u16) -> CellAddr {
        CellAddr { sheet, row, col }
    }

    fn parse_ast(formula: &str) -> AstNode {
        let tokens = tokenize(formula).unwrap();
        parse(&tokens).unwrap()
    }

    #[test]
    fn test_recalculate_linear_chain() {
        // A1 = 10 (constant, not a formula)
        // B1 = A1 + 1
        // C1 = B1 * 2
        let mut ctx = MutableSimpleContext::new(1);
        ctx.set(0, 0, 0, Value::Number(10.0)); // A1 = 10

        let formulas = vec![
            (addr(0, 0, 1), parse_ast("A1+1")), // B1
            (addr(0, 0, 2), parse_ast("B1*2")), // C1
        ];

        let count = recalculate(&formulas, &mut ctx).unwrap();
        assert_eq!(count, 2);
        assert_eq!(ctx.get(0, 0, 1), Value::Number(11.0)); // B1 = 10+1
        assert_eq!(ctx.get(0, 0, 2), Value::Number(22.0)); // C1 = 11*2
    }

    #[test]
    fn test_recalculate_circular_reference() {
        // A1 = B1 + 1
        // B1 = A1 + 1
        let mut ctx = MutableSimpleContext::new(1);

        let formulas = vec![
            (addr(0, 0, 0), parse_ast("B1+1")),
            (addr(0, 0, 1), parse_ast("A1+1")),
        ];

        let result = recalculate(&formulas, &mut ctx);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.cells.contains(&addr(0, 0, 0)));
        assert!(err.cells.contains(&addr(0, 0, 1)));
    }

    #[test]
    fn test_recalculate_volatile_function() {
        // A1 = TODAY() — volatile
        // B1 = A1 + 1 — depends on volatile
        let mut ctx = MutableSimpleContext::new(1);

        let formulas = vec![
            (addr(0, 0, 0), parse_ast("TODAY()")),
            (addr(0, 0, 1), parse_ast("A1+1")),
        ];

        let count = recalculate(&formulas, &mut ctx).unwrap();
        // Both cells evaluated initially, then both re-evaluated (volatile + dependent)
        assert_eq!(count, 4);

        // TODAY() returns 45292.0 in test mode
        assert_eq!(ctx.get(0, 0, 0), Value::Number(45292.0));
        assert_eq!(ctx.get(0, 0, 1), Value::Number(45293.0));
    }

    #[test]
    fn test_contains_volatile() {
        assert!(contains_volatile(&parse_ast("TODAY()")));
        assert!(contains_volatile(&parse_ast("NOW()+1")));
        assert!(contains_volatile(&parse_ast("A1+RAND()")));
        assert!(!contains_volatile(&parse_ast("SUM(A1:A5)")));
        assert!(!contains_volatile(&parse_ast("1+2")));
    }

    #[test]
    fn test_is_volatile() {
        assert!(is_volatile("RAND"));
        assert!(is_volatile("RANDBETWEEN"));
        assert!(is_volatile("NOW"));
        assert!(is_volatile("TODAY"));
        assert!(is_volatile("INDIRECT"));
        assert!(!is_volatile("SUM"));
        assert!(!is_volatile("IF"));
    }

    #[test]
    fn test_recalculate_diamond() {
        // A1 = 5
        // B1 = A1 * 2
        // C1 = A1 * 3
        // D1 = B1 + C1
        let mut ctx = MutableSimpleContext::new(1);
        ctx.set(0, 0, 0, Value::Number(5.0));

        let formulas = vec![
            (addr(0, 0, 1), parse_ast("A1*2")),
            (addr(0, 0, 2), parse_ast("A1*3")),
            (addr(0, 0, 3), parse_ast("B1+C1")),
        ];

        recalculate(&formulas, &mut ctx).unwrap();
        assert_eq!(ctx.get(0, 0, 1), Value::Number(10.0));
        assert_eq!(ctx.get(0, 0, 2), Value::Number(15.0));
        assert_eq!(ctx.get(0, 0, 3), Value::Number(25.0));
    }
}
