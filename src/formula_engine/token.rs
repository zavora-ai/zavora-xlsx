//! Formula tokenizer: converts a formula string into a sequence of tokens.
//!
//! Supports A1-style references, R1C1-style references, structured table
//! references, function calls, operators, literals, and array constants.

use std::fmt;

/// Arithmetic and comparison operators.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Op {
    Add,
    Sub,
    Mul,
    Div,
    Pow,
    Concat,  // &
    Eq,      // =
    Ne,      // <>
    Lt,      // <
    Gt,      // >
    Le,      // <=
    Ge,      // >=
    Percent, // %
}

impl fmt::Display for Op {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Op::Add => "+",
            Op::Sub => "-",
            Op::Mul => "*",
            Op::Div => "/",
            Op::Pow => "^",
            Op::Concat => "&",
            Op::Eq => "=",
            Op::Ne => "<>",
            Op::Lt => "<",
            Op::Gt => ">",
            Op::Le => "<=",
            Op::Ge => ">=",
            Op::Percent => "%",
        };
        write!(f, "{s}")
    }
}

/// A token produced by the formula tokenizer.
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    /// A1-style cell reference (e.g. `A1`, `$B$2`).
    CellRef {
        col: u16,
        row: u32,
        abs_col: bool,
        abs_row: bool,
    },
    /// R1C1-style cell reference (e.g. `R1C1`, `R[-1]C[2]`).
    R1C1Ref {
        row: i32,
        col: i32,
        row_relative: bool,
        col_relative: bool,
    },
    /// Sheet-qualified reference (e.g. `Sheet1!A1`).
    SheetRef {
        sheet: String,
        inner: Box<Token>,
    },
    /// Structured table reference (e.g. `Table1[Column]`).
    StructuredRef {
        table: String,
        column: String,
    },
    /// Numeric literal.
    Number(f64),
    /// String literal (without surrounding quotes).
    StringLiteral(String),
    /// Boolean literal.
    Bool(bool),
    /// Error literal (e.g. `#VALUE!`, `#REF!`).
    Error(String),
    /// Function name (followed by open paren).
    Function(String),
    /// Operator.
    Operator(Op),
    /// Opening parenthesis.
    OpenParen,
    /// Closing parenthesis.
    CloseParen,
    /// Comma (argument separator).
    Comma,
    /// Colon (range operator).
    Colon,
    /// Array open brace `{`.
    ArrayOpen,
    /// Array close brace `}`.
    ArrayClose,
    /// Semicolon (array row separator).
    Semicolon,
}

/// Error returned by the formula tokenizer.
#[derive(Debug, Clone, PartialEq)]
pub struct FormulaError {
    pub position: usize,
    pub message: String,
}

impl fmt::Display for FormulaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Formula error at position {}: {}", self.position, self.message)
    }
}

impl std::error::Error for FormulaError {}

/// Tokenize a formula string into a sequence of tokens.
///
/// The input should NOT include the leading `=` sign.
///
/// # Examples
/// ```
/// use zavora_xlsx::formula_engine::token::{tokenize, Token, Op};
/// let tokens = tokenize("A1+B2").unwrap();
/// assert_eq!(tokens.len(), 3);
/// ```
pub fn tokenize(formula: &str) -> Result<Vec<Token>, FormulaError> {
    let mut tokens = Vec::new();
    let chars: Vec<char> = formula.chars().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        let ch = chars[i];

        // Skip whitespace
        if ch.is_ascii_whitespace() {
            i += 1;
            continue;
        }

        // String literal
        if ch == '"' {
            i += 1;
            let mut s = String::new();
            while i < len {
                if chars[i] == '"' {
                    if i + 1 < len && chars[i + 1] == '"' {
                        s.push('"');
                        i += 2;
                    } else {
                        i += 1;
                        break;
                    }
                } else {
                    s.push(chars[i]);
                    i += 1;
                }
            }
            tokens.push(Token::StringLiteral(s));
            continue;
        }

        // Error literals (#VALUE!, #REF!, etc.)
        if ch == '#' {
            let err_start = i;
            i += 1;
            while i < len && (chars[i].is_ascii_alphanumeric() || chars[i] == '/' || chars[i] == '!') {
                i += 1;
            }
            let err_str = chars[err_start..i].iter().collect::<String>();
            tokens.push(Token::Error(err_str));
            continue;
        }

        // Array braces
        if ch == '{' {
            tokens.push(Token::ArrayOpen);
            i += 1;
            continue;
        }
        if ch == '}' {
            tokens.push(Token::ArrayClose);
            i += 1;
            continue;
        }

        // Parentheses
        if ch == '(' {
            tokens.push(Token::OpenParen);
            i += 1;
            continue;
        }
        if ch == ')' {
            tokens.push(Token::CloseParen);
            i += 1;
            continue;
        }

        // Comma
        if ch == ',' {
            tokens.push(Token::Comma);
            i += 1;
            continue;
        }

        // Semicolon
        if ch == ';' {
            tokens.push(Token::Semicolon);
            i += 1;
            continue;
        }

        // Colon (range operator)
        if ch == ':' {
            tokens.push(Token::Colon);
            i += 1;
            continue;
        }

        // Operators
        if ch == '+' { tokens.push(Token::Operator(Op::Add)); i += 1; continue; }
        if ch == '-' { tokens.push(Token::Operator(Op::Sub)); i += 1; continue; }
        if ch == '*' { tokens.push(Token::Operator(Op::Mul)); i += 1; continue; }
        if ch == '/' { tokens.push(Token::Operator(Op::Div)); i += 1; continue; }
        if ch == '^' { tokens.push(Token::Operator(Op::Pow)); i += 1; continue; }
        if ch == '&' { tokens.push(Token::Operator(Op::Concat)); i += 1; continue; }
        if ch == '%' { tokens.push(Token::Operator(Op::Percent)); i += 1; continue; }
        if ch == '=' { tokens.push(Token::Operator(Op::Eq)); i += 1; continue; }
        if ch == '<' {
            if i + 1 < len && chars[i + 1] == '>' {
                tokens.push(Token::Operator(Op::Ne));
                i += 2;
            } else if i + 1 < len && chars[i + 1] == '=' {
                tokens.push(Token::Operator(Op::Le));
                i += 2;
            } else {
                tokens.push(Token::Operator(Op::Lt));
                i += 1;
            }
            continue;
        }
        if ch == '>' {
            if i + 1 < len && chars[i + 1] == '=' {
                tokens.push(Token::Operator(Op::Ge));
                i += 2;
            } else {
                tokens.push(Token::Operator(Op::Gt));
                i += 1;
            }
            continue;
        }

        // Numbers
        if ch.is_ascii_digit() || (ch == '.' && i + 1 < len && chars[i + 1].is_ascii_digit()) {
            let start = i;
            while i < len && chars[i].is_ascii_digit() {
                i += 1;
            }
            if i < len && chars[i] == '.' {
                i += 1;
                while i < len && chars[i].is_ascii_digit() {
                    i += 1;
                }
            }
            // Scientific notation
            if i < len && (chars[i] == 'e' || chars[i] == 'E') {
                i += 1;
                if i < len && (chars[i] == '+' || chars[i] == '-') {
                    i += 1;
                }
                while i < len && chars[i].is_ascii_digit() {
                    i += 1;
                }
            }
            let num_str: String = chars[start..i].iter().collect();
            let val: f64 = num_str.parse().map_err(|_| FormulaError {
                position: start,
                message: format!("Invalid number: {num_str}"),
            })?;
            tokens.push(Token::Number(val));
            continue;
        }

        // Quoted sheet name: 'Sheet Name'!A1
        if ch == '\'' {
            let start = i;
            i += 1;
            let mut sheet_name = String::new();
            while i < len && chars[i] != '\'' {
                sheet_name.push(chars[i]);
                i += 1;
            }
            if i < len { i += 1; } // skip closing quote
            // Expect '!'
            if i < len && chars[i] == '!' {
                i += 1;
                // Parse the inner reference
                let inner_token = parse_cell_or_range(&chars, &mut i)?;
                tokens.push(Token::SheetRef {
                    sheet: sheet_name,
                    inner: Box::new(inner_token),
                });
            } else {
                return Err(FormulaError {
                    position: start,
                    message: "Expected '!' after quoted sheet name".into(),
                });
            }
            continue;
        }

        // Identifiers: cell refs, function names, booleans, R1C1 refs, sheet refs
        if ch.is_ascii_alphabetic() || ch == '_' || ch == '$' {
            let token = parse_identifier(&chars, &mut i, &tokens)?;
            tokens.push(token);
            continue;
        }

        return Err(FormulaError {
            position: i,
            message: format!("Unexpected character: '{ch}'"),
        });
    }

    Ok(tokens)
}

/// Parse an identifier starting at position `i`. This handles:
/// - Cell references (A1, $B$2)
/// - R1C1 references (R1C1, R[-1]C[2])
/// - Function names (SUM, IF)
/// - Boolean literals (TRUE, FALSE)
/// - Sheet references (Sheet1!A1)
/// - Structured table references (Table1[Column])
fn parse_identifier(chars: &[char], i: &mut usize, _prev_tokens: &[Token]) -> Result<Token, FormulaError> {
    let len = chars.len();

    // Collect the identifier (letters, digits, underscores, dots, $)
    let mut ident = String::new();
    while *i < len && (chars[*i].is_ascii_alphanumeric() || chars[*i] == '_' || chars[*i] == '$' || chars[*i] == '.') {
        ident.push(chars[*i]);
        *i += 1;
    }

    // Check for R1C1-style reference (must check before structured ref since R[-1]C[2] has brackets)
    // R1C1 refs start with R followed by digits or [
    if ident.len() >= 1 {
        let upper = ident.to_uppercase();
        // Check if this looks like an R1C1 ref: starts with R, possibly followed by digits
        // and the next char might be C or [
        if upper.starts_with('R') && (upper.len() > 1 || (*i < len && (chars[*i] == '[' || chars[*i].is_ascii_digit()))) {
            // Try to parse as R1C1 including any bracket portions
            let saved_i = *i;
            let mut full_ref = ident.clone();
            // Consume remaining R1C1 parts (brackets and C portion)
            while *i < len && (chars[*i] == '[' || chars[*i] == ']' || chars[*i] == '-' || chars[*i] == '+' || chars[*i].is_ascii_digit() || chars[*i].is_ascii_alphabetic()) {
                full_ref.push(chars[*i]);
                *i += 1;
            }
            if let Some(token) = try_parse_r1c1(&full_ref, 0) {
                return Ok(token);
            }
            // Not a valid R1C1 ref, restore position
            *i = saved_i;
        }
    }

    // Check for sheet reference: Ident!CellRef
    if *i < len && chars[*i] == '!' {
        let sheet_name = ident;
        *i += 1; // skip '!'
        let inner = parse_cell_or_range(chars, i)?;
        return Ok(Token::SheetRef {
            sheet: sheet_name,
            inner: Box::new(inner),
        });
    }

    // Check for structured table reference: Table1[Column]
    if *i < len && chars[*i] == '[' {
        let table_name = ident;
        *i += 1; // skip '['
        let mut col_name = String::new();
        while *i < len && chars[*i] != ']' {
            col_name.push(chars[*i]);
            *i += 1;
        }
        if *i < len { *i += 1; } // skip ']'
        return Ok(Token::StructuredRef {
            table: table_name,
            column: col_name,
        });
    }

    // Check for function call: IDENT(
    if *i < len && chars[*i] == '(' {
        return Ok(Token::Function(ident.to_uppercase()));
    }

    // Check for boolean
    let upper = ident.to_uppercase();
    if upper == "TRUE" {
        return Ok(Token::Bool(true));
    }
    if upper == "FALSE" {
        return Ok(Token::Bool(false));
    }

    // Try to parse as cell reference
    if let Some(token) = try_parse_cell_ref(&ident) {
        return Ok(token);
    }

    // If nothing else matches, treat as a function name (without parens)
    // or a named range
    Ok(Token::CellRef {
        col: 0,
        row: 0,
        abs_col: false,
        abs_row: false,
    })
}

/// Try to parse a string as an A1-style cell reference.
fn try_parse_cell_ref(s: &str) -> Option<Token> {
    let bytes = s.as_bytes();
    let len = bytes.len();
    let mut pos = 0;

    let abs_col = if pos < len && bytes[pos] == b'$' { pos += 1; true } else { false };

    // Column letters
    let col_start = pos;
    while pos < len && bytes[pos].is_ascii_alphabetic() {
        pos += 1;
    }
    if pos == col_start { return None; }
    let col_str = &s[col_start..pos];

    let abs_row = if pos < len && bytes[pos] == b'$' { pos += 1; true } else { false };

    // Row digits
    let row_start = pos;
    while pos < len && bytes[pos].is_ascii_digit() {
        pos += 1;
    }
    if pos == row_start || pos != len { return None; }
    let row_str = &s[row_start..pos];

    let col = col_letters_to_index(col_str)?;
    let row: u32 = row_str.parse().ok()?;
    if row == 0 { return None; }

    Some(Token::CellRef {
        col,
        row: row - 1, // 0-based
        abs_col,
        abs_row,
    })
}

/// Convert column letters to 0-based index: A=0, B=1, ..., Z=25, AA=26.
fn col_letters_to_index(s: &str) -> Option<u16> {
    let mut result: u16 = 0;
    for &b in s.as_bytes() {
        let c = b.to_ascii_uppercase();
        if !c.is_ascii_alphabetic() { return None; }
        result = result.checked_mul(26)?.checked_add((c - b'A') as u16 + 1)?;
    }
    Some(result - 1) // 0-based
}

/// Try to parse an R1C1-style reference like R1C1, R[-1]C[2], RC.
fn try_parse_r1c1(s: &str, _start: usize) -> Option<Token> {
    let upper = s.to_uppercase();
    let bytes = upper.as_bytes();
    let len = bytes.len();
    if len == 0 || bytes[0] != b'R' { return None; }

    let mut pos = 1;
    let (row, row_relative) = parse_r1c1_component(bytes, &mut pos)?;

    if pos >= len || bytes[pos] != b'C' { return None; }
    pos += 1;
    let (col, col_relative) = parse_r1c1_component(bytes, &mut pos)?;

    if pos != len { return None; }

    Some(Token::R1C1Ref { row, col, row_relative, col_relative })
}

/// Parse one component of an R1C1 ref: either `[n]` (relative) or `n` (absolute) or empty (relative 0).
fn parse_r1c1_component(bytes: &[u8], pos: &mut usize) -> Option<(i32, bool)> {
    let len = bytes.len();
    if *pos >= len {
        return Some((0, true)); // e.g. just "R" or "C" alone means relative 0
    }
    if bytes[*pos] == b'[' {
        *pos += 1;
        let start = *pos;
        if *pos < len && (bytes[*pos] == b'-' || bytes[*pos] == b'+') { *pos += 1; }
        while *pos < len && bytes[*pos].is_ascii_digit() { *pos += 1; }
        if *pos >= len || bytes[*pos] != b']' { return None; }
        let num_str: String = bytes[start..*pos].iter().map(|&b| b as char).collect();
        *pos += 1; // skip ']'
        let val: i32 = num_str.parse().ok()?;
        Some((val, true))
    } else if bytes[*pos].is_ascii_digit() {
        let start = *pos;
        while *pos < len && bytes[*pos].is_ascii_digit() { *pos += 1; }
        let num_str: String = bytes[start..*pos].iter().map(|&b| b as char).collect();
        let val: i32 = num_str.parse().ok()?;
        Some((val, false))
    } else {
        Some((0, true))
    }
}

/// Parse a cell reference or range at the current position (used after `!`).
fn parse_cell_or_range(chars: &[char], i: &mut usize) -> Result<Token, FormulaError> {
    let start = *i;
    let len = chars.len();

    // Collect the reference text
    let mut ref_text = String::new();
    while *i < len && (chars[*i].is_ascii_alphanumeric() || chars[*i] == '$') {
        ref_text.push(chars[*i]);
        *i += 1;
    }

    if let Some(token) = try_parse_cell_ref(&ref_text) {
        return Ok(token);
    }

    Err(FormulaError {
        position: start,
        message: format!("Invalid cell reference: {ref_text}"),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_addition() {
        let tokens = tokenize("A1+B2").unwrap();
        assert_eq!(tokens, vec![
            Token::CellRef { col: 0, row: 0, abs_col: false, abs_row: false },
            Token::Operator(Op::Add),
            Token::CellRef { col: 1, row: 1, abs_col: false, abs_row: false },
        ]);
    }

    #[test]
    fn test_absolute_refs() {
        let tokens = tokenize("$A$1+$B2+C$3").unwrap();
        assert_eq!(tokens[0], Token::CellRef { col: 0, row: 0, abs_col: true, abs_row: true });
        assert_eq!(tokens[2], Token::CellRef { col: 1, row: 1, abs_col: true, abs_row: false });
        assert_eq!(tokens[4], Token::CellRef { col: 2, row: 2, abs_col: false, abs_row: true });
    }

    #[test]
    fn test_function_call() {
        let tokens = tokenize("SUM(A1:A10)").unwrap();
        assert_eq!(tokens, vec![
            Token::Function("SUM".into()),
            Token::OpenParen,
            Token::CellRef { col: 0, row: 0, abs_col: false, abs_row: false },
            Token::Colon,
            Token::CellRef { col: 0, row: 9, abs_col: false, abs_row: false },
            Token::CloseParen,
        ]);
    }

    #[test]
    fn test_nested_functions() {
        let tokens = tokenize("IF(A1>0,SUM(B1:B5),0)").unwrap();
        assert_eq!(tokens[0], Token::Function("IF".into()));
        assert_eq!(tokens[1], Token::OpenParen);
        assert_eq!(tokens[3], Token::Operator(Op::Gt));
        assert_eq!(tokens[4], Token::Number(0.0));
    }

    #[test]
    fn test_string_literal() {
        let tokens = tokenize(r#""Hello, World!""#).unwrap();
        assert_eq!(tokens, vec![Token::StringLiteral("Hello, World!".into())]);
    }

    #[test]
    fn test_string_with_escaped_quotes() {
        let tokens = tokenize(r#""He said ""hi""!""#).unwrap();
        assert_eq!(tokens, vec![Token::StringLiteral(r#"He said "hi"!"#.into())]);
    }

    #[test]
    fn test_boolean_literals() {
        let tokens = tokenize("TRUE+FALSE").unwrap();
        assert_eq!(tokens, vec![
            Token::Bool(true),
            Token::Operator(Op::Add),
            Token::Bool(false),
        ]);
    }

    #[test]
    fn test_error_literal() {
        let tokens = tokenize("#VALUE!+#REF!").unwrap();
        assert_eq!(tokens[0], Token::Error("#VALUE!".into()));
        assert_eq!(tokens[2], Token::Error("#REF!".into()));
    }

    #[test]
    fn test_comparison_operators() {
        let tokens = tokenize("A1<>B1").unwrap();
        assert_eq!(tokens[1], Token::Operator(Op::Ne));

        let tokens = tokenize("A1<=B1").unwrap();
        assert_eq!(tokens[1], Token::Operator(Op::Le));

        let tokens = tokenize("A1>=B1").unwrap();
        assert_eq!(tokens[1], Token::Operator(Op::Ge));
    }

    #[test]
    fn test_sheet_reference() {
        let tokens = tokenize("Sheet1!A1").unwrap();
        assert_eq!(tokens, vec![
            Token::SheetRef {
                sheet: "Sheet1".into(),
                inner: Box::new(Token::CellRef { col: 0, row: 0, abs_col: false, abs_row: false }),
            },
        ]);
    }

    #[test]
    fn test_quoted_sheet_reference() {
        let tokens = tokenize("'My Sheet'!B5").unwrap();
        assert_eq!(tokens, vec![
            Token::SheetRef {
                sheet: "My Sheet".into(),
                inner: Box::new(Token::CellRef { col: 1, row: 4, abs_col: false, abs_row: false }),
            },
        ]);
    }

    #[test]
    fn test_structured_ref() {
        let tokens = tokenize("Table1[Revenue]").unwrap();
        assert_eq!(tokens, vec![
            Token::StructuredRef { table: "Table1".into(), column: "Revenue".into() },
        ]);
    }

    #[test]
    fn test_r1c1_absolute() {
        let tokens = tokenize("R1C1").unwrap();
        assert_eq!(tokens, vec![
            Token::R1C1Ref { row: 1, col: 1, row_relative: false, col_relative: false },
        ]);
    }

    #[test]
    fn test_r1c1_relative() {
        let tokens = tokenize("R[-1]C[2]").unwrap();
        assert_eq!(tokens, vec![
            Token::R1C1Ref { row: -1, col: 2, row_relative: true, col_relative: true },
        ]);
    }

    #[test]
    fn test_number_scientific() {
        let tokens = tokenize("1.5E3+2e-1").unwrap();
        assert_eq!(tokens[0], Token::Number(1500.0));
        assert_eq!(tokens[2], Token::Number(0.2));
    }

    #[test]
    fn test_array_literal() {
        let tokens = tokenize("{1,2;3,4}").unwrap();
        assert_eq!(tokens, vec![
            Token::ArrayOpen,
            Token::Number(1.0),
            Token::Comma,
            Token::Number(2.0),
            Token::Semicolon,
            Token::Number(3.0),
            Token::Comma,
            Token::Number(4.0),
            Token::ArrayClose,
        ]);
    }

    #[test]
    fn test_complex_formula() {
        let tokens = tokenize("IF(AND(A1>0,B1<100),A1*0.1,0)").unwrap();
        assert_eq!(tokens[0], Token::Function("IF".into()));
        assert_eq!(tokens[2], Token::Function("AND".into()));
    }

    #[test]
    fn test_concat_operator() {
        let tokens = tokenize(r#""Hello"&" "&"World""#).unwrap();
        assert_eq!(tokens.len(), 5);
        assert_eq!(tokens[1], Token::Operator(Op::Concat));
        assert_eq!(tokens[3], Token::Operator(Op::Concat));
    }

    #[test]
    fn test_percent_operator() {
        let tokens = tokenize("50%").unwrap();
        assert_eq!(tokens, vec![Token::Number(50.0), Token::Operator(Op::Percent)]);
    }

    #[test]
    fn test_multi_letter_column() {
        let tokens = tokenize("AA1+AZ100").unwrap();
        assert_eq!(tokens[0], Token::CellRef { col: 26, row: 0, abs_col: false, abs_row: false });
        assert_eq!(tokens[2], Token::CellRef { col: 51, row: 99, abs_col: false, abs_row: false });
    }
}
