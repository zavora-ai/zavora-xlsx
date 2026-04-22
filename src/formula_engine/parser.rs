//! Pratt parser: converts a token stream into an AST.
//!
//! Handles operator precedence naturally via binding powers.

use crate::formula_engine::ast::AstNode;
use crate::formula_engine::token::{FormulaError, Op, Token};

/// Parse a token stream into an AST.
pub fn parse(tokens: &[Token]) -> Result<AstNode, FormulaError> {
    let mut parser = Parser { tokens, pos: 0 };
    let node = parser.parse_expr(0)?;
    if parser.pos < parser.tokens.len() {
        return Err(FormulaError {
            position: parser.pos,
            message: format!("Unexpected token: {:?}", parser.tokens[parser.pos]),
        });
    }
    Ok(node)
}

struct Parser<'a> {
    tokens: &'a [Token],
    pos: usize,
}

impl<'a> Parser<'a> {
    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn advance(&mut self) -> Option<&Token> {
        let tok = self.tokens.get(self.pos);
        if tok.is_some() {
            self.pos += 1;
        }
        tok
    }

    fn expect_token(&mut self, expected: &Token) -> Result<(), FormulaError> {
        let pos_before = self.pos;
        match self.advance() {
            Some(t) if t == expected => Ok(()),
            Some(t) => {
                let t_dbg = format!("{t:?}");
                Err(FormulaError {
                    position: pos_before,
                    message: format!("Expected {expected:?}, got {t_dbg}"),
                })
            }
            None => Err(FormulaError {
                position: pos_before,
                message: format!("Expected {expected:?}, got end of input"),
            }),
        }
    }

    /// Parse an expression with the given minimum binding power.
    fn parse_expr(&mut self, min_bp: u8) -> Result<AstNode, FormulaError> {
        let mut lhs = self.parse_prefix()?;

        while let Some(tok) = self.peek() {
            // Postfix: percent operator
            if matches!(tok, Token::Operator(Op::Percent)) {
                self.advance();
                lhs = AstNode::UnaryOp {
                    op: Op::Percent,
                    operand: Box::new(lhs),
                };
                continue;
            }

            // Colon is a special infix operator for ranges
            if matches!(tok, Token::Colon) {
                let bp = 14; // highest precedence for range
                if bp < min_bp {
                    break;
                }
                self.advance();
                let rhs = self.parse_expr(bp + 1)?;
                lhs = AstNode::Range {
                    start: Box::new(lhs),
                    end: Box::new(rhs),
                };
                continue;
            }

            // Infix operators
            if let Token::Operator(op) = tok {
                let op = *op;
                let (l_bp, r_bp) = infix_binding_power(op);
                if l_bp < min_bp {
                    break;
                }
                self.advance();
                let rhs = self.parse_expr(r_bp)?;
                lhs = AstNode::BinaryOp {
                    op,
                    left: Box::new(lhs),
                    right: Box::new(rhs),
                };
                continue;
            }

            break;
        }

        Ok(lhs)
    }

    /// Parse a prefix expression (atoms, unary operators, parenthesized exprs).
    fn parse_prefix(&mut self) -> Result<AstNode, FormulaError> {
        let pos_before = self.pos;
        let tok = self
            .advance()
            .ok_or_else(|| FormulaError {
                position: pos_before,
                message: "Unexpected end of formula".into(),
            })?
            .clone();

        match tok {
            Token::Number(n) => Ok(AstNode::Number(n)),
            Token::StringLiteral(s) => Ok(AstNode::String(s)),
            Token::Bool(b) => Ok(AstNode::Bool(b)),
            Token::Error(e) => Ok(AstNode::Error(e)),

            Token::CellRef {
                col,
                row,
                abs_col,
                abs_row,
            } => Ok(AstNode::CellRef {
                col,
                row,
                abs_col,
                abs_row,
            }),

            Token::R1C1Ref {
                row,
                col,
                row_relative,
                col_relative,
            } => Ok(AstNode::R1C1Ref {
                row,
                col,
                row_relative,
                col_relative,
            }),

            Token::SheetRef { sheet, inner } => {
                let inner_node = token_to_ast_atom(&inner)?;
                Ok(AstNode::SheetRef {
                    sheet,
                    inner: Box::new(inner_node),
                })
            }

            Token::StructuredRef { table, column } => Ok(AstNode::StructuredRef { table, column }),

            Token::Function(name) => {
                self.expect_token(&Token::OpenParen)?;
                let mut args = Vec::new();
                if self.peek() != Some(&Token::CloseParen) {
                    args.push(self.parse_expr(0)?);
                    while self.peek() == Some(&Token::Comma) {
                        self.advance();
                        args.push(self.parse_expr(0)?);
                    }
                }
                self.expect_token(&Token::CloseParen)?;
                Ok(AstNode::FunctionCall { name, args })
            }

            Token::OpenParen => {
                let expr = self.parse_expr(0)?;
                self.expect_token(&Token::CloseParen)?;
                Ok(expr)
            }

            // Unary minus/plus
            Token::Operator(Op::Sub) => {
                let operand = self.parse_expr(12)?; // high precedence for unary
                Ok(AstNode::UnaryOp {
                    op: Op::Sub,
                    operand: Box::new(operand),
                })
            }
            Token::Operator(Op::Add) => {
                // Unary plus is a no-op, just parse the operand
                self.parse_expr(12)
            }

            // Array literal
            Token::ArrayOpen => {
                let mut rows = Vec::new();
                let mut current_row = Vec::new();
                if self.peek() != Some(&Token::ArrayClose) {
                    current_row.push(self.parse_expr(0)?);
                    loop {
                        match self.peek() {
                            Some(Token::Comma) => {
                                self.advance();
                                current_row.push(self.parse_expr(0)?);
                            }
                            Some(Token::Semicolon) => {
                                self.advance();
                                rows.push(current_row);
                                current_row = vec![self.parse_expr(0)?];
                            }
                            _ => break,
                        }
                    }
                }
                rows.push(current_row);
                self.expect_token(&Token::ArrayClose)?;
                Ok(AstNode::Array { rows })
            }

            _ => Err(FormulaError {
                position: self.pos - 1,
                message: format!("Unexpected token in expression: {tok:?}"),
            }),
        }
    }
}

/// Convert a token (from SheetRef inner) to an AST atom.
fn token_to_ast_atom(token: &Token) -> Result<AstNode, FormulaError> {
    match token {
        Token::CellRef {
            col,
            row,
            abs_col,
            abs_row,
        } => Ok(AstNode::CellRef {
            col: *col,
            row: *row,
            abs_col: *abs_col,
            abs_row: *abs_row,
        }),
        Token::R1C1Ref {
            row,
            col,
            row_relative,
            col_relative,
        } => Ok(AstNode::R1C1Ref {
            row: *row,
            col: *col,
            row_relative: *row_relative,
            col_relative: *col_relative,
        }),
        _ => Err(FormulaError {
            position: 0,
            message: format!("Expected cell reference in sheet ref, got {token:?}"),
        }),
    }
}

/// Return (left binding power, right binding power) for infix operators.
///
/// Higher numbers bind tighter. Right-associative operators have r_bp = l_bp.
fn infix_binding_power(op: Op) -> (u8, u8) {
    match op {
        Op::Eq | Op::Ne | Op::Lt | Op::Gt | Op::Le | Op::Ge => (2, 3),
        Op::Concat => (4, 5),
        Op::Add | Op::Sub => (6, 7),
        Op::Mul | Op::Div => (8, 9),
        Op::Pow => (11, 10),     // right-associative
        Op::Percent => (13, 14), // shouldn't be used as infix
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::formula_engine::token::tokenize;

    fn parse_formula(s: &str) -> AstNode {
        let tokens = tokenize(s).unwrap();
        parse(&tokens).unwrap()
    }

    #[test]
    fn test_simple_number() {
        assert_eq!(parse_formula("42"), AstNode::Number(42.0));
    }

    #[test]
    fn test_addition() {
        let ast = parse_formula("1+2");
        assert_eq!(
            ast,
            AstNode::BinaryOp {
                op: Op::Add,
                left: Box::new(AstNode::Number(1.0)),
                right: Box::new(AstNode::Number(2.0)),
            }
        );
    }

    #[test]
    fn test_precedence_mul_over_add() {
        // 1+2*3 should parse as 1+(2*3)
        let ast = parse_formula("1+2*3");
        match ast {
            AstNode::BinaryOp {
                op: Op::Add,
                left,
                right,
            } => {
                assert_eq!(*left, AstNode::Number(1.0));
                match *right {
                    AstNode::BinaryOp { op: Op::Mul, .. } => {}
                    _ => panic!("Expected Mul on right"),
                }
            }
            _ => panic!("Expected Add at top"),
        }
    }

    #[test]
    fn test_power_right_associative() {
        // 2^3^4 should parse as 2^(3^4)
        let ast = parse_formula("2^3^4");
        match ast {
            AstNode::BinaryOp {
                op: Op::Pow,
                left,
                right,
            } => {
                assert_eq!(*left, AstNode::Number(2.0));
                match *right {
                    AstNode::BinaryOp { op: Op::Pow, .. } => {}
                    _ => panic!("Expected Pow on right"),
                }
            }
            _ => panic!("Expected Pow at top"),
        }
    }

    #[test]
    fn test_unary_minus() {
        let ast = parse_formula("-5");
        assert_eq!(
            ast,
            AstNode::UnaryOp {
                op: Op::Sub,
                operand: Box::new(AstNode::Number(5.0)),
            }
        );
    }

    #[test]
    fn test_parentheses() {
        // (1+2)*3
        let ast = parse_formula("(1+2)*3");
        match ast {
            AstNode::BinaryOp {
                op: Op::Mul,
                left,
                right,
            } => {
                match *left {
                    AstNode::BinaryOp { op: Op::Add, .. } => {}
                    _ => panic!("Expected Add in parens"),
                }
                assert_eq!(*right, AstNode::Number(3.0));
            }
            _ => panic!("Expected Mul at top"),
        }
    }

    #[test]
    fn test_function_call() {
        let ast = parse_formula("SUM(1,2,3)");
        match ast {
            AstNode::FunctionCall { name, args } => {
                assert_eq!(name, "SUM");
                assert_eq!(args.len(), 3);
            }
            _ => panic!("Expected FunctionCall"),
        }
    }

    #[test]
    fn test_nested_functions() {
        let ast = parse_formula("IF(A1>0,SUM(B1:B5),0)");
        match ast {
            AstNode::FunctionCall { name, args } => {
                assert_eq!(name, "IF");
                assert_eq!(args.len(), 3);
                // Second arg should be a SUM function call
                match &args[1] {
                    AstNode::FunctionCall { name, .. } => assert_eq!(name, "SUM"),
                    _ => panic!("Expected SUM as second arg"),
                }
            }
            _ => panic!("Expected IF"),
        }
    }

    #[test]
    fn test_range() {
        let ast = parse_formula("A1:B5");
        match ast {
            AstNode::Range { start, end } => {
                assert_eq!(
                    *start,
                    AstNode::CellRef {
                        col: 0,
                        row: 0,
                        abs_col: false,
                        abs_row: false
                    }
                );
                assert_eq!(
                    *end,
                    AstNode::CellRef {
                        col: 1,
                        row: 4,
                        abs_col: false,
                        abs_row: false
                    }
                );
            }
            _ => panic!("Expected Range"),
        }
    }

    #[test]
    fn test_array_literal() {
        let ast = parse_formula("{1,2;3,4}");
        match ast {
            AstNode::Array { rows } => {
                assert_eq!(rows.len(), 2);
                assert_eq!(rows[0].len(), 2);
                assert_eq!(rows[1].len(), 2);
            }
            _ => panic!("Expected Array"),
        }
    }

    #[test]
    fn test_comparison() {
        let ast = parse_formula("A1>=10");
        match ast {
            AstNode::BinaryOp { op: Op::Ge, .. } => {}
            _ => panic!("Expected Ge comparison"),
        }
    }

    #[test]
    fn test_concat() {
        let ast = parse_formula(r#""Hello"&" World""#);
        match ast {
            AstNode::BinaryOp {
                op: Op::Concat,
                left,
                right,
            } => {
                assert_eq!(*left, AstNode::String("Hello".into()));
                assert_eq!(*right, AstNode::String(" World".into()));
            }
            _ => panic!("Expected Concat"),
        }
    }

    #[test]
    fn test_sheet_ref() {
        let ast = parse_formula("Sheet1!A1");
        match ast {
            AstNode::SheetRef { sheet, inner } => {
                assert_eq!(sheet, "Sheet1");
                assert_eq!(
                    *inner,
                    AstNode::CellRef {
                        col: 0,
                        row: 0,
                        abs_col: false,
                        abs_row: false
                    }
                );
            }
            _ => panic!("Expected SheetRef"),
        }
    }
}
