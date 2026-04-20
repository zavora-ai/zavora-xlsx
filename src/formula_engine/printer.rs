//! AST → formula string printer for round-trip support.

use crate::formula_engine::ast::AstNode;
use crate::formula_engine::token::Op;

/// Convert an AST node back into a formula string.
pub fn print_formula(ast: &AstNode) -> String {
    let mut out = String::new();
    write_node(ast, &mut out);
    out
}

fn write_node(node: &AstNode, out: &mut String) {
    match node {
        AstNode::Number(n) => {
            if *n == n.trunc() && n.abs() < 1e15 {
                out.push_str(&format!("{}", *n as i64));
            } else {
                out.push_str(&format!("{n}"));
            }
        }
        AstNode::String(s) => {
            out.push('"');
            for ch in s.chars() {
                if ch == '"' { out.push('"'); }
                out.push(ch);
            }
            out.push('"');
        }
        AstNode::Bool(b) => out.push_str(if *b { "TRUE" } else { "FALSE" }),
        AstNode::Error(e) => out.push_str(e),
        AstNode::CellRef { col, row, abs_col, abs_row } => {
            if *abs_col { out.push('$'); }
            out.push_str(&col_to_letters(*col));
            if *abs_row { out.push('$'); }
            out.push_str(&(row + 1).to_string());
        }
        AstNode::R1C1Ref { row, col, row_relative, col_relative } => {
            out.push('R');
            if *row_relative {
                if *row != 0 { out.push_str(&format!("[{row}]")); }
            } else {
                out.push_str(&row.to_string());
            }
            out.push('C');
            if *col_relative {
                if *col != 0 { out.push_str(&format!("[{col}]")); }
            } else {
                out.push_str(&col.to_string());
            }
        }
        AstNode::Range { start, end } => {
            write_node(start, out);
            out.push(':');
            write_node(end, out);
        }
        AstNode::SheetRef { sheet, inner } => {
            if sheet.contains(' ') || sheet.contains('\'') {
                out.push('\'');
                out.push_str(sheet);
                out.push('\'');
            } else {
                out.push_str(sheet);
            }
            out.push('!');
            write_node(inner, out);
        }
        AstNode::StructuredRef { table, column } => {
            out.push_str(table);
            out.push('[');
            out.push_str(column);
            out.push(']');
        }
        AstNode::BinaryOp { op, left, right } => {
            let needs_parens_left = needs_parens(left, *op, true);
            let needs_parens_right = needs_parens(right, *op, false);
            if needs_parens_left { out.push('('); }
            write_node(left, out);
            if needs_parens_left { out.push(')'); }
            out.push_str(&op.to_string());
            if needs_parens_right { out.push('('); }
            write_node(right, out);
            if needs_parens_right { out.push(')'); }
        }
        AstNode::UnaryOp { op, operand } => {
            out.push_str(&op.to_string());
            let needs = matches!(operand.as_ref(), AstNode::BinaryOp { .. });
            if needs { out.push('('); }
            write_node(operand, out);
            if needs { out.push(')'); }
        }
        AstNode::FunctionCall { name, args } => {
            out.push_str(name);
            out.push('(');
            for (i, arg) in args.iter().enumerate() {
                if i > 0 { out.push(','); }
                write_node(arg, out);
            }
            out.push(')');
        }
        AstNode::Array { rows } => {
            out.push('{');
            for (ri, row) in rows.iter().enumerate() {
                if ri > 0 { out.push(';'); }
                for (ci, cell) in row.iter().enumerate() {
                    if ci > 0 { out.push(','); }
                    write_node(cell, out);
                }
            }
            out.push('}');
        }
    }
}

/// Convert 0-based column index to letters: 0→A, 25→Z, 26→AA.
fn col_to_letters(col: u16) -> String {
    let mut result = String::new();
    let mut n = col as u32 + 1;
    while n > 0 {
        n -= 1;
        result.insert(0, (b'A' + (n % 26) as u8) as char);
        n /= 26;
    }
    result
}

/// Determine if a child node needs parentheses given the parent operator.
fn needs_parens(child: &AstNode, parent_op: Op, is_left: bool) -> bool {
    if let AstNode::BinaryOp { op: child_op, .. } = child {
        let parent_prec = op_precedence(parent_op);
        let child_prec = op_precedence(*child_op);
        if child_prec < parent_prec {
            return true;
        }
        // Same precedence on right side needs parens for left-associative ops
        if child_prec == parent_prec && !is_left && parent_op != Op::Pow {
            return true;
        }
    }
    false
}

fn op_precedence(op: Op) -> u8 {
    match op {
        Op::Eq | Op::Ne | Op::Lt | Op::Gt | Op::Le | Op::Ge => 1,
        Op::Concat => 2,
        Op::Add | Op::Sub => 3,
        Op::Mul | Op::Div => 4,
        Op::Pow => 5,
        Op::Percent => 6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::formula_engine::parser::parse;
    use crate::formula_engine::token::tokenize;

    fn roundtrip(formula: &str) -> String {
        let tokens = tokenize(formula).unwrap();
        let ast = parse(&tokens).unwrap();
        print_formula(&ast)
    }

    #[test]
    fn test_roundtrip_simple() {
        assert_eq!(roundtrip("1+2"), "1+2");
    }

    #[test]
    fn test_roundtrip_precedence() {
        assert_eq!(roundtrip("1+2*3"), "1+2*3");
        assert_eq!(roundtrip("(1+2)*3"), "(1+2)*3");
    }

    #[test]
    fn test_roundtrip_function() {
        assert_eq!(roundtrip("SUM(A1:A10)"), "SUM(A1:A10)");
    }

    #[test]
    fn test_roundtrip_string() {
        assert_eq!(roundtrip(r#""Hello""#), r#""Hello""#);
    }

    #[test]
    fn test_roundtrip_sheet_ref() {
        assert_eq!(roundtrip("Sheet1!A1"), "Sheet1!A1");
    }

    #[test]
    fn test_roundtrip_absolute() {
        assert_eq!(roundtrip("$A$1"), "$A$1");
    }

    #[test]
    fn test_roundtrip_array() {
        assert_eq!(roundtrip("{1,2;3,4}"), "{1,2;3,4}");
    }

    #[test]
    fn test_roundtrip_nested() {
        assert_eq!(roundtrip("IF(A1>0,SUM(B1:B5),0)"), "IF(A1>0,SUM(B1:B5),0)");
    }

    #[test]
    fn test_parse_print_parse_equivalence() {
        let formulas = ["1+2*3", "SUM(A1:A10)", "IF(A1>0,B1,C1)", "$A$1+B2"];
        for f in formulas {
            let tokens1 = tokenize(f).unwrap();
            let ast1 = parse(&tokens1).unwrap();
            let printed = print_formula(&ast1);
            let tokens2 = tokenize(&printed).unwrap();
            let ast2 = parse(&tokens2).unwrap();
            assert_eq!(ast1, ast2, "Round-trip failed for: {f}");
        }
    }
}
