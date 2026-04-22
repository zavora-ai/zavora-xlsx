//! CSV/TSV export with configurable options.

use crate::cell::CellValue;
use crate::worksheet::Worksheet;

/// Options for CSV export.
#[derive(Debug, Clone)]
pub struct CsvOptions {
    /// Field delimiter byte (default: `b','`).
    pub delimiter: u8,
    /// Quote character byte (default: `b'"'`).
    pub quote: u8,
    /// Line ending string (default: `"\r\n"`).
    pub line_ending: String,
    /// Date format string (default: ISO 8601 `"yyyy-mm-dd"`).
    pub date_format: String,
}

impl Default for CsvOptions {
    fn default() -> Self {
        Self {
            delimiter: b',',
            quote: b'"',
            line_ending: "\r\n".to_string(),
            date_format: "yyyy-mm-dd".to_string(),
        }
    }
}

impl CsvOptions {
    /// Create default CSV options (comma-delimited).
    pub fn new() -> Self {
        Self::default()
    }

    /// Create TSV options (tab-delimited).
    pub fn tsv() -> Self {
        Self {
            delimiter: b'\t',
            ..Self::default()
        }
    }

    /// Set the delimiter.
    pub fn delimiter(mut self, d: u8) -> Self {
        self.delimiter = d;
        self
    }

    /// Set the quote character.
    pub fn quote(mut self, q: u8) -> Self {
        self.quote = q;
        self
    }

    /// Set the line ending.
    pub fn line_ending(mut self, le: &str) -> Self {
        self.line_ending = le.to_string();
        self
    }

    /// Set the date format.
    pub fn date_format(mut self, fmt: &str) -> Self {
        self.date_format = fmt.to_string();
        self
    }
}

impl Worksheet {
    /// Export the worksheet to a CSV string using the given options.
    pub fn to_csv_string(&self, options: &CsvOptions) -> String {
        let range = match self.used_range() {
            Some(r) => r,
            None => return String::new(),
        };
        let mut result = String::new();
        let delim = options.delimiter as char;
        let _quote = options.quote as char;

        for r in range.0..=range.2 {
            let mut first = true;
            for c in range.1..=range.3 {
                if !first {
                    result.push(delim);
                }
                first = false;
                let val = self.read_cell(r, c);
                let s = format_cell_value(&val, &options.date_format);
                let escaped = escape_csv_field(&s, options.delimiter, options.quote);
                result.push_str(&escaped);
            }
            result.push_str(&options.line_ending);
        }
        result
    }

    /// Export the worksheet to a CSV file using the given options.
    pub fn to_csv_file(
        &self,
        path: impl AsRef<std::path::Path>,
        options: &CsvOptions,
    ) -> crate::Result<()> {
        let content = self.to_csv_string(options);
        std::fs::write(path, content.as_bytes())?;
        Ok(())
    }
}

/// Format a cell value as a string for CSV output.
fn format_cell_value(val: &CellValue, _date_format: &str) -> String {
    match val {
        CellValue::Empty => String::new(),
        CellValue::String(s) => s.clone(),
        CellValue::Number(n) => format!("{n}"),
        CellValue::Bool(b) => {
            if *b {
                "TRUE".into()
            } else {
                "FALSE".into()
            }
        }
        CellValue::DateTime(dt) => dt.to_iso_string(),
        CellValue::Error(e) => e.clone(),
        CellValue::Formula { cached_value, .. } => match cached_value.as_ref() {
            CellValue::Number(n) => format!("{n}"),
            CellValue::String(s) => s.clone(),
            CellValue::Bool(b) => {
                if *b {
                    "TRUE".into()
                } else {
                    "FALSE".into()
                }
            }
            _ => String::new(),
        },
        CellValue::RichText(rt) => rt.plain_text(),
    }
}

/// Escape a CSV field: wrap in quotes if it contains the delimiter, quote char, or newlines.
fn escape_csv_field(field: &str, delimiter: u8, quote: u8) -> String {
    let delim_char = delimiter as char;
    let quote_char = quote as char;

    if field.contains(delim_char)
        || field.contains(quote_char)
        || field.contains('\n')
        || field.contains('\r')
    {
        let mut escaped = String::with_capacity(field.len() + 2);
        escaped.push(quote_char);
        for ch in field.chars() {
            if ch == quote_char {
                escaped.push(quote_char);
            }
            escaped.push(ch);
        }
        escaped.push(quote_char);
        escaped
    } else {
        field.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_csv_field_plain() {
        assert_eq!(escape_csv_field("hello", b',', b'"'), "hello");
    }

    #[test]
    fn test_escape_csv_field_with_comma() {
        assert_eq!(
            escape_csv_field("hello,world", b',', b'"'),
            "\"hello,world\""
        );
    }

    #[test]
    fn test_escape_csv_field_with_quote() {
        assert_eq!(
            escape_csv_field("say \"hi\"", b',', b'"'),
            "\"say \"\"hi\"\"\""
        );
    }

    #[test]
    fn test_escape_csv_field_with_newline() {
        assert_eq!(
            escape_csv_field("line1\nline2", b',', b'"'),
            "\"line1\nline2\""
        );
    }

    #[test]
    fn test_csv_options_default() {
        let opts = CsvOptions::default();
        assert_eq!(opts.delimiter, b',');
        assert_eq!(opts.quote, b'"');
        assert_eq!(opts.line_ending, "\r\n");
    }

    #[test]
    fn test_csv_options_tsv() {
        let opts = CsvOptions::tsv();
        assert_eq!(opts.delimiter, b'\t');
    }
}
