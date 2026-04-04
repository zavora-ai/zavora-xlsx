/// 0-based row index. Row 0 = Excel row 1.
pub type RowNum = u32;
/// 0-based column index. Col 0 = column A. Max 16383 (XFD).
pub type ColNum = u16;

const MAX_COL: u32 = 16383;
const MAX_ROW: u32 = 1_048_575;

/// Convert column letters (e.g. "A", "AA", "XFD") to 0-based index.
pub fn col_from_letter(s: &str) -> crate::Result<ColNum> {
    if s.is_empty() {
        return Err(crate::Error::InvalidCellRef("empty column".into()));
    }
    let mut n: u32 = 0;
    for b in s.bytes() {
        if !b.is_ascii_alphabetic() {
            return Err(crate::Error::InvalidCellRef(format!("invalid column char in '{s}'")));
        }
        n = n * 26 + (b.to_ascii_uppercase() - b'A') as u32 + 1;
    }
    let idx = n - 1;
    if idx > MAX_COL {
        return Err(crate::Error::InvalidCellRef(format!("column '{s}' exceeds XFD")));
    }
    Ok(idx as ColNum)
}

/// Convert 0-based column index to letters (e.g. 0 → "A", 26 → "AA").
pub fn col_to_letter(col: ColNum) -> String {
    let mut result = Vec::new();
    let mut n = col as u32 + 1;
    while n > 0 {
        n -= 1;
        result.push(b'A' + (n % 26) as u8);
        n /= 26;
    }
    result.reverse();
    String::from_utf8(result).unwrap()
}

/// Format (row, col) as A1 notation.
pub fn to_a1(row: RowNum, col: ColNum) -> String {
    format!("{}{}", col_to_letter(col), row + 1)
}

/// Parse "B3" → (row=2, col=1).
pub fn parse_cell_ref(s: &str) -> crate::Result<(RowNum, ColNum)> {
    let s = s.trim();
    // Strip all $ signs (absolute reference markers)
    let clean: String = s.chars().filter(|&c| c != '$').collect();
    let s = &clean;
    let split = s.find(|c: char| c.is_ascii_digit())
        .ok_or_else(|| crate::Error::InvalidCellRef(format!("no row in '{s}'")))?;
    if split == 0 {
        return Err(crate::Error::InvalidCellRef(format!("no column in '{s}'")));
    }
    let col_str = &s[..split];
    let row_str = &s[split..];
    let col = col_from_letter(col_str)?;
    let row_1: u32 = row_str.parse()
        .map_err(|_| crate::Error::InvalidCellRef(format!("bad row '{row_str}'")))?;
    if row_1 == 0 || row_1 - 1 > MAX_ROW {
        return Err(crate::Error::InvalidCellRef(format!("row {row_1} out of range")));
    }
    Ok((row_1 - 1, col))
}

/// Parse "A1:C3" → (start_row, start_col, end_row, end_col).
pub fn parse_range_ref(s: &str) -> crate::Result<(RowNum, ColNum, RowNum, ColNum)> {
    let parts: Vec<&str> = s.split(':').collect();
    if parts.len() != 2 {
        return Err(crate::Error::InvalidRange(format!("expected ':' in '{s}'")));
    }
    let (r1, c1) = parse_cell_ref(parts[0])?;
    let (r2, c2) = parse_cell_ref(parts[1])?;
    Ok((r1, c1, r2, c2))
}

/// Parse column+row from raw bytes of an `r` attribute (e.g. b"B3").
/// Returns (row_0based, col_0based). Optimized for hot path.
pub fn parse_cell_attr(bytes: &[u8]) -> Option<(RowNum, ColNum)> {
    let mut col: u32 = 0;
    let mut i = 0;
    while i < bytes.len() && bytes[i].is_ascii_alphabetic() {
        col = col * 26 + (bytes[i].to_ascii_uppercase() - b'A') as u32 + 1;
        i += 1;
    }
    if i == 0 || i == bytes.len() { return None; }
    let row: u32 = atoi_simd::parse(&bytes[i..]).ok()?;
    if row == 0 { return None; }
    Some((row - 1, (col - 1) as ColNum))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn col_roundtrip() {
        assert_eq!(col_from_letter("A").unwrap(), 0);
        assert_eq!(col_from_letter("Z").unwrap(), 25);
        assert_eq!(col_from_letter("AA").unwrap(), 26);
        assert_eq!(col_from_letter("AZ").unwrap(), 51);
        assert_eq!(col_from_letter("XFD").unwrap(), 16383);
        for i in 0..=16383u16 {
            assert_eq!(col_from_letter(&col_to_letter(i)).unwrap(), i);
        }
    }

    #[test]
    fn parse_a1() {
        assert_eq!(parse_cell_ref("A1").unwrap(), (0, 0));
        assert_eq!(parse_cell_ref("B3").unwrap(), (2, 1));
        assert_eq!(parse_cell_ref("$C$5").unwrap(), (4, 2));
        assert_eq!(parse_cell_ref("XFD1048576").unwrap(), (1_048_575, 16383));
    }

    #[test]
    fn parse_a1_errors() {
        assert!(parse_cell_ref("").is_err());
        assert!(parse_cell_ref("A0").is_err());
        assert!(parse_cell_ref("123").is_err());
        assert!(parse_cell_ref("XFDA1").is_err()); // exceeds max col
    }

    #[test]
    fn parse_range() {
        assert_eq!(parse_range_ref("A1:C3").unwrap(), (0, 0, 2, 2));
    }

    #[test]
    fn to_a1_roundtrip() {
        assert_eq!(to_a1(0, 0), "A1");
        assert_eq!(to_a1(2, 1), "B3");
    }

    #[test]
    fn parse_cell_attr_fast() {
        assert_eq!(parse_cell_attr(b"A1"), Some((0, 0)));
        assert_eq!(parse_cell_attr(b"C10"), Some((9, 2)));
        assert_eq!(parse_cell_attr(b""), None);
        assert_eq!(parse_cell_attr(b"123"), None);
    }
}
