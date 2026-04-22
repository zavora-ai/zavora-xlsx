//! Formula dependency graph: tracks cell-to-cell dependencies for
//! topological evaluation ordering and cycle detection.

use std::collections::{HashMap, HashSet, VecDeque};

use crate::formula_engine::ast::AstNode;

/// A cell address uniquely identifying a cell across sheets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CellAddr {
    pub sheet: usize,
    pub row: u32,
    pub col: u16,
}

/// Directed graph of cell dependencies.
///
/// An edge from A → B means "A depends on B" (A's formula references B).
#[derive(Debug, Clone)]
pub struct DependencyGraph {
    /// cell → set of cells it depends on (its precedents)
    edges: HashMap<CellAddr, HashSet<CellAddr>>,
    /// cell → set of cells that depend on it (its dependents)
    reverse: HashMap<CellAddr, HashSet<CellAddr>>,
}

impl Default for DependencyGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl DependencyGraph {
    /// Create an empty dependency graph.
    pub fn new() -> Self {
        Self {
            edges: HashMap::new(),
            reverse: HashMap::new(),
        }
    }

    /// Add a dependency: `cell` depends on `depends_on`.
    pub fn add_dependency(&mut self, cell: CellAddr, depends_on: CellAddr) {
        self.edges.entry(cell).or_default().insert(depends_on);
        self.reverse.entry(depends_on).or_default().insert(cell);
    }

    /// Build the graph from a set of (cell_addr, ast) pairs.
    ///
    /// Extracts all cell references from each AST and records them as dependencies.
    pub fn build_from_formulas(formulas: &[(CellAddr, &AstNode)]) -> Self {
        let mut graph = Self::new();
        for &(cell, ast) in formulas {
            let refs = extract_cell_refs(ast, cell.sheet);
            for dep in refs {
                graph.add_dependency(cell, dep);
            }
        }
        graph
    }

    /// Return a topological sort order (Kahn's algorithm).
    ///
    /// Returns cells in evaluation order: cells with no dependencies first.
    /// Returns `Err` with the cycle members if a cycle exists.
    pub fn topological_order(&self) -> Result<Vec<CellAddr>, Vec<CellAddr>> {
        // Collect all nodes
        let mut all_nodes: HashSet<CellAddr> = HashSet::new();
        for (&cell, deps) in &self.edges {
            all_nodes.insert(cell);
            for &dep in deps {
                all_nodes.insert(dep);
            }
        }
        for &cell in self.reverse.keys() {
            all_nodes.insert(cell);
        }

        // Compute in-degree (number of precedents each cell has)
        let mut in_degree: HashMap<CellAddr, usize> = HashMap::new();
        for &node in &all_nodes {
            in_degree.insert(node, self.edges.get(&node).map_or(0, |s| s.len()));
        }

        // Start with nodes that have no dependencies
        let mut queue: VecDeque<CellAddr> = VecDeque::new();
        for (&node, &deg) in &in_degree {
            if deg == 0 {
                queue.push_back(node);
            }
        }

        let mut order = Vec::new();
        while let Some(node) = queue.pop_front() {
            order.push(node);
            // For each cell that depends on this node, decrement its in-degree
            if let Some(dependents) = self.reverse.get(&node) {
                for &dep in dependents {
                    if let Some(deg) = in_degree.get_mut(&dep) {
                        *deg -= 1;
                        if *deg == 0 {
                            queue.push_back(dep);
                        }
                    }
                }
            }
        }

        if order.len() < all_nodes.len() {
            // Cycle detected — find nodes not in the order
            let ordered_set: HashSet<_> = order.iter().copied().collect();
            let cycle_members: Vec<CellAddr> = all_nodes
                .into_iter()
                .filter(|n| !ordered_set.contains(n))
                .collect();
            Err(cycle_members)
        } else {
            Ok(order)
        }
    }

    /// Detect cycles using DFS. Returns `Some(cycle)` if a cycle exists.
    pub fn detect_cycle(&self) -> Option<Vec<CellAddr>> {
        self.topological_order().err()
    }

    /// Return all cells that transitively depend on the given cell.
    ///
    /// Useful for incremental recalculation: when `cell` changes, all
    /// returned cells need to be re-evaluated.
    pub fn dependents_of(&self, cell: CellAddr) -> Vec<CellAddr> {
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        queue.push_back(cell);
        visited.insert(cell);

        while let Some(current) = queue.pop_front() {
            if let Some(deps) = self.reverse.get(&current) {
                for &dep in deps {
                    if visited.insert(dep) {
                        queue.push_back(dep);
                    }
                }
            }
        }

        visited.remove(&cell);
        visited.into_iter().collect()
    }

    /// Get the direct precedents (cells referenced by) a given cell.
    pub fn precedents_of(&self, cell: CellAddr) -> Vec<CellAddr> {
        self.edges
            .get(&cell)
            .map_or_else(Vec::new, |s| s.iter().copied().collect())
    }
}

/// Extract all cell references from an AST node, resolving them to CellAddr.
///
/// `default_sheet` is used when a reference has no explicit sheet qualifier.
fn extract_cell_refs(node: &AstNode, default_sheet: usize) -> Vec<CellAddr> {
    let mut refs = Vec::new();
    collect_refs(node, default_sheet, &mut refs);
    refs
}

fn collect_refs(node: &AstNode, default_sheet: usize, refs: &mut Vec<CellAddr>) {
    match node {
        AstNode::CellRef { col, row, .. } => {
            refs.push(CellAddr {
                sheet: default_sheet,
                row: *row,
                col: *col,
            });
        }
        AstNode::SheetRef { sheet: _, inner } => {
            // For now, we can't resolve sheet names to indices without a workbook.
            // We'll use default_sheet. A full implementation would resolve the name.
            collect_refs(inner, default_sheet, refs);
        }
        AstNode::Range { start, end } => {
            // For a range like A1:B5, we add all cells in the range
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
                for r in min_r..=max_r {
                    for c in min_c..=max_c {
                        refs.push(CellAddr {
                            sheet: default_sheet,
                            row: r,
                            col: c,
                        });
                    }
                }
            } else {
                collect_refs(start, default_sheet, refs);
                collect_refs(end, default_sheet, refs);
            }
        }
        AstNode::BinaryOp { left, right, .. } => {
            collect_refs(left, default_sheet, refs);
            collect_refs(right, default_sheet, refs);
        }
        AstNode::UnaryOp { operand, .. } => {
            collect_refs(operand, default_sheet, refs);
        }
        AstNode::FunctionCall { args, .. } => {
            for arg in args {
                collect_refs(arg, default_sheet, refs);
            }
        }
        AstNode::Array { rows } => {
            for row in rows {
                for cell in row {
                    collect_refs(cell, default_sheet, refs);
                }
            }
        }
        _ => {}
    }
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
    fn test_linear_chain() {
        // A1 = 1 (no formula)
        // B1 = A1 + 1
        // C1 = B1 + 1
        let b1_ast = parse_ast("A1+1");
        let c1_ast = parse_ast("B1+1");

        let formulas = vec![
            (addr(0, 0, 1), &b1_ast), // B1
            (addr(0, 0, 2), &c1_ast), // C1
        ];

        let graph = DependencyGraph::build_from_formulas(&formulas);
        let order = graph.topological_order().unwrap();

        // A1 should come before B1, B1 before C1
        let pos_a1 = order.iter().position(|a| *a == addr(0, 0, 0));
        let pos_b1 = order.iter().position(|a| *a == addr(0, 0, 1));
        let pos_c1 = order.iter().position(|a| *a == addr(0, 0, 2));

        assert!(pos_a1.unwrap() < pos_b1.unwrap());
        assert!(pos_b1.unwrap() < pos_c1.unwrap());
    }

    #[test]
    fn test_diamond_dependency() {
        // A1 = 1
        // B1 = A1 * 2
        // C1 = A1 * 3
        // D1 = B1 + C1
        let b1_ast = parse_ast("A1*2");
        let c1_ast = parse_ast("A1*3");
        let d1_ast = parse_ast("B1+C1");

        let formulas = vec![
            (addr(0, 0, 1), &b1_ast),
            (addr(0, 0, 2), &c1_ast),
            (addr(0, 0, 3), &d1_ast),
        ];

        let graph = DependencyGraph::build_from_formulas(&formulas);
        let order = graph.topological_order().unwrap();

        let pos_a1 = order.iter().position(|a| *a == addr(0, 0, 0)).unwrap();
        let pos_b1 = order.iter().position(|a| *a == addr(0, 0, 1)).unwrap();
        let pos_c1 = order.iter().position(|a| *a == addr(0, 0, 2)).unwrap();
        let pos_d1 = order.iter().position(|a| *a == addr(0, 0, 3)).unwrap();

        assert!(pos_a1 < pos_b1);
        assert!(pos_a1 < pos_c1);
        assert!(pos_b1 < pos_d1);
        assert!(pos_c1 < pos_d1);
    }

    #[test]
    fn test_cycle_detection() {
        // A1 = B1 + 1
        // B1 = A1 + 1  (circular!)
        let a1_ast = parse_ast("B1+1");
        let b1_ast = parse_ast("A1+1");

        let formulas = vec![(addr(0, 0, 0), &a1_ast), (addr(0, 0, 1), &b1_ast)];

        let graph = DependencyGraph::build_from_formulas(&formulas);
        let cycle = graph.detect_cycle();
        assert!(cycle.is_some());
        let cycle_cells = cycle.unwrap();
        assert!(cycle_cells.contains(&addr(0, 0, 0)));
        assert!(cycle_cells.contains(&addr(0, 0, 1)));
    }

    #[test]
    fn test_dependents_of() {
        // A1 = 1
        // B1 = A1 + 1
        // C1 = A1 + 2
        // D1 = B1 + C1
        let b1_ast = parse_ast("A1+1");
        let c1_ast = parse_ast("A1+2");
        let d1_ast = parse_ast("B1+C1");

        let formulas = vec![
            (addr(0, 0, 1), &b1_ast),
            (addr(0, 0, 2), &c1_ast),
            (addr(0, 0, 3), &d1_ast),
        ];

        let graph = DependencyGraph::build_from_formulas(&formulas);

        // If A1 changes, B1, C1, and D1 all need recalculation
        let deps = graph.dependents_of(addr(0, 0, 0));
        assert!(deps.contains(&addr(0, 0, 1))); // B1
        assert!(deps.contains(&addr(0, 0, 2))); // C1
        assert!(deps.contains(&addr(0, 0, 3))); // D1

        // If B1 changes, only D1 needs recalculation
        let deps_b = graph.dependents_of(addr(0, 0, 1));
        assert!(deps_b.contains(&addr(0, 0, 3))); // D1
        assert!(!deps_b.contains(&addr(0, 0, 0))); // not A1
    }

    #[test]
    fn test_range_dependency() {
        // A1 = SUM(B1:B3)
        let a1_ast = parse_ast("SUM(B1:B3)");

        let formulas = vec![(addr(0, 0, 0), &a1_ast)];

        let graph = DependencyGraph::build_from_formulas(&formulas);

        // A1 should depend on B1, B2, B3
        let precedents = graph.precedents_of(addr(0, 0, 0));
        assert!(precedents.contains(&addr(0, 0, 1))); // B1
        assert!(precedents.contains(&addr(0, 1, 1))); // B2
        assert!(precedents.contains(&addr(0, 2, 1))); // B3
    }

    #[test]
    fn test_no_cycle_empty_graph() {
        let graph = DependencyGraph::new();
        assert!(graph.detect_cycle().is_none());
        assert_eq!(graph.topological_order().unwrap().len(), 0);
    }
}
