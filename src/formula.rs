/// Adjust cell references in a formula when rows/columns are inserted or removed.
///
/// `delta_row` > 0 means rows inserted, < 0 means rows removed.
/// `delta_col` > 0 means columns inserted, < 0 means columns removed.
/// References at or after the threshold are shifted.
use crate::utility::{ColNum, RowNum};

pub fn adjust_formula(
    formula: &str,
    at_row: Option<RowNum>,    // row threshold (0-based)
    delta_row: i64,
    at_col: Option<ColNum>,    // col threshold (0-based)
    delta_col: i64,
) -> String {
    let mut result = String::with_capacity(formula.len());
    let bytes = formula.as_bytes();
    let mut i = 0;

    while i < bytes.len() {
        let b = bytes[i];

        // Skip quoted strings
        if b == b'"' {
            result.push('"');
            i += 1;
            while i < bytes.len() && bytes[i] != b'"' {
                result.push(bytes[i] as char);
                i += 1;
            }
            if i < bytes.len() { result.push('"'); i += 1; }
            continue;
        }

        // Try to parse a cell reference (letter+ digit+)
        if b.is_ascii_alphabetic() || b == b'$' {
            let _start = i;
            if let Some((_ref_str, end, row, col, abs_row, abs_col)) = try_parse_ref(bytes, i) {
                let new_row = if let Some(at) = at_row {
                    if row >= at { (row as i64 + delta_row).max(0) as RowNum } else { row }
                } else { row };
                let new_col = if let Some(at) = at_col {
                    if col >= at { (col as i64 + delta_col).max(0) as ColNum } else { col }
                } else { col };

                // Rebuild reference
                if abs_col { result.push('$'); }
                let col_str = crate::utility::col_to_letter(new_col);
                result.push_str(&col_str);
                if abs_row { result.push('$'); }
                result.push_str(&(new_row + 1).to_string());
                i = end;
                continue;
            }
        }

        result.push(b as char);
        i += 1;
    }
    result
}

/// Try to parse a cell reference starting at position `start`.
/// Returns (original_str, end_pos, row_0based, col_0based, abs_row, abs_col).
fn try_parse_ref(bytes: &[u8], start: usize) -> Option<(String, usize, RowNum, ColNum, bool, bool)> {
    let mut i = start;

    // Optional $ before column
    let abs_col = i < bytes.len() && bytes[i] == b'$';
    if abs_col { i += 1; }

    // Column letters (at least one)
    let col_start = i;
    while i < bytes.len() && bytes[i].is_ascii_alphabetic() { i += 1; }
    if i == col_start { return None; }
    let col_str = std::str::from_utf8(&bytes[col_start..i]).ok()?;

    // Optional $ before row
    let abs_row = i < bytes.len() && bytes[i] == b'$';
    if abs_row { i += 1; }

    // Row digits (at least one)
    let row_start = i;
    while i < bytes.len() && bytes[i].is_ascii_digit() { i += 1; }
    if i == row_start { return None; }

    // Make sure this isn't part of a longer identifier (e.g. SUM, IF)
    if i < bytes.len() && (bytes[i].is_ascii_alphabetic() || bytes[i] == b'_') {
        return None;
    }

    let row_str = std::str::from_utf8(&bytes[row_start..i]).ok()?;
    let row_1: u32 = row_str.parse().ok()?;
    if row_1 == 0 { return None; }

    let col = crate::utility::col_from_letter(col_str).ok()?;
    let orig = std::str::from_utf8(&bytes[start..i]).ok()?.to_string();

    Some((orig, i, row_1 - 1, col, abs_row, abs_col))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_row_shifts_refs() {
        // Insert 1 row at row 1 (0-based): A2 → A3, A1 stays
        assert_eq!(adjust_formula("A1+A2", Some(1), 1, None, 0), "A1+A3");
        assert_eq!(adjust_formula("SUM(A1:A5)", Some(1), 1, None, 0), "SUM(A1:A6)");
    }

    #[test]
    fn delete_row_shifts_refs() {
        assert_eq!(adjust_formula("A3+A1", Some(1), -1, None, 0), "A2+A1");
    }

    #[test]
    fn insert_col_shifts_refs() {
        // Insert 1 col at col B (1): B1 → C1, A1 stays
        assert_eq!(adjust_formula("A1+B1", None, 0, Some(1), 1), "A1+C1");
    }

    #[test]
    fn absolute_refs_shifted() {
        assert_eq!(adjust_formula("$A$2+$B$1", Some(1), 1, None, 0), "$A$3+$B$1");
    }

    #[test]
    fn quoted_strings_preserved() {
        assert_eq!(adjust_formula("\"A1\"+A1", Some(0), 1, None, 0), "\"A1\"+A2");
    }

    #[test]
    fn function_names_not_shifted() {
        // SUM should not be treated as a cell ref
        assert_eq!(adjust_formula("SUM(A1)", Some(0), 1, None, 0), "SUM(A2)");
    }
}
