use clap::{Parser, Subcommand};
use zavora_xlsx::{CsvOptions, Workbook};

/// Represents a detected cell value type from CSV parsing.
#[derive(Debug, PartialEq)]
enum CellValueType {
    Number(f64),
    Bool(bool),
    String(String),
}

/// Parse a CSV line into fields, handling:
/// - Fields separated by the delimiter
/// - Quoted fields (wrapped in double quotes)
/// - Escaped quotes within quoted fields (doubled quotes "")
/// - Fields with embedded delimiters inside quotes
fn parse_csv_line(line: &str, delimiter: u8) -> Vec<String> {
    let delim = delimiter as char;
    let mut fields = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    let mut chars = line.chars().peekable();

    while let Some(c) = chars.next() {
        if in_quotes {
            if c == '"' {
                // Check for escaped quote (doubled "")
                if chars.peek() == Some(&'"') {
                    current.push('"');
                    chars.next(); // consume the second quote
                } else {
                    // End of quoted field
                    in_quotes = false;
                }
            } else {
                current.push(c);
            }
        } else if c == '"' && current.is_empty() {
            // Start of a quoted field
            in_quotes = true;
        } else if c == delim {
            fields.push(current.clone());
            current.clear();
        } else {
            current.push(c);
        }
    }

    // Push the last field
    fields.push(current);

    fields
}

/// Detect the type of a CSV field value.
/// Tries f64 parse first, then checks TRUE/FALSE case-insensitive, then falls back to String.
fn detect_cell_value(field: &str) -> CellValueType {
    if let Ok(n) = field.parse::<f64>() {
        return CellValueType::Number(n);
    }
    match field.to_uppercase().as_str() {
        "TRUE" => CellValueType::Bool(true),
        "FALSE" => CellValueType::Bool(false),
        _ => CellValueType::String(field.to_string()),
    }
}

#[derive(Parser)]
#[command(
    name = "zavora-xlsx",
    version,
    about = "Inspect, export, and convert Excel xlsx files"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Inspect an Excel file: show sheet names, dimensions, and metadata
    Inspect {
        /// Path to the xlsx file
        file: String,
    },
    /// Export a worksheet to CSV or TSV
    Export {
        /// Path to the xlsx file
        file: String,
        /// Sheet name to export (default: first sheet)
        #[arg(long)]
        sheet: Option<String>,
        /// Zero-based sheet index to export
        #[arg(long)]
        sheet_index: Option<usize>,
        /// Output file path (default: stdout)
        #[arg(short, long)]
        output: Option<String>,
        /// Field delimiter character (default: comma)
        #[arg(long)]
        delimiter: Option<char>,
        /// Use tab as delimiter (TSV mode)
        #[arg(long)]
        tsv: bool,
        /// Date format string
        #[arg(long)]
        date_format: Option<String>,
    },
    /// Convert between xlsx, xlsm, and csv formats
    Convert {
        /// Path to the input file
        file: String,
        /// Target format: csv, xlsx, xlsm
        #[arg(long)]
        format: String,
        /// Sheet name (for single-sheet csv export)
        #[arg(long)]
        sheet: Option<String>,
        /// Output file or directory path
        #[arg(short, long)]
        output: Option<String>,
        /// Field delimiter for CSV operations
        #[arg(long)]
        delimiter: Option<char>,
    },
}

fn main() {
    let cli = Cli::parse();
    let result = match cli.command {
        Commands::Inspect { ref file } => cmd_inspect(file),
        Commands::Export {
            ref file,
            ref sheet,
            sheet_index,
            ref output,
            delimiter,
            tsv,
            ref date_format,
        } => cmd_export(
            file,
            sheet.as_deref(),
            sheet_index,
            output.as_deref(),
            delimiter,
            tsv,
            date_format.as_deref(),
        ),
        Commands::Convert {
            ref file,
            ref format,
            ref sheet,
            ref output,
            delimiter,
        } => cmd_convert(file, format, sheet.as_deref(), output.as_deref(), delimiter),
    };
    if let Err(e) = result {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}

fn cmd_inspect(file: &str) -> Result<(), Box<dyn std::error::Error>> {
    let wb = Workbook::open_readonly(file)?;
    let names = wb.sheet_names();

    for (i, name) in names.iter().enumerate() {
        let ws = wb.worksheet_ref(i)?;
        let (rows, cols) = match ws.used_range() {
            Some((min_row, min_col, max_row, max_col)) => {
                let row_count = max_row - min_row + 1;
                let col_count = max_col - min_col + 1;
                (row_count as usize, col_count as usize)
            }
            None => (0, 0),
        };
        println!("  {name}: {rows} rows x {cols} cols");
    }

    println!("Sheets: {}", wb.sheet_count());

    let props = wb.properties();
    if let Some(ref v) = props.title {
        println!("Title: {v}");
    }
    if let Some(ref v) = props.author {
        println!("Author: {v}");
    }
    if let Some(ref v) = props.subject {
        println!("Subject: {v}");
    }
    if let Some(ref v) = props.keywords {
        println!("Keywords: {v}");
    }
    if let Some(ref v) = props.category {
        println!("Category: {v}");
    }
    if let Some(ref v) = props.company {
        println!("Company: {v}");
    }

    Ok(())
}

fn cmd_export(
    file: &str,
    sheet: Option<&str>,
    sheet_index: Option<usize>,
    output: Option<&str>,
    delimiter: Option<char>,
    tsv: bool,
    date_format: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    let wb = Workbook::open_readonly(file)?;

    // Resolve the target sheet index
    let idx = if let Some(name) = sheet {
        let names = wb.sheet_names();
        names
            .iter()
            .position(|n| *n == name)
            .ok_or_else(|| format!("sheet '{}' not found in workbook", name))?
    } else if let Some(i) = sheet_index {
        if i >= wb.sheet_count() {
            return Err(format!(
                "sheet index {} out of bounds (workbook has {} sheets)",
                i,
                wb.sheet_count()
            )
            .into());
        }
        i
    } else {
        0
    };

    // Build CsvOptions from flags
    let mut options = CsvOptions::new();
    if tsv {
        options = options.delimiter(b'\t');
    }
    if let Some(d) = delimiter {
        options = options.delimiter(d as u8);
    }
    if let Some(fmt) = date_format {
        options = options.date_format(fmt);
    }

    let ws = wb.worksheet_ref(idx)?;

    if let Some(path) = output {
        ws.to_csv_file(path, &options)?;
    } else {
        let csv = ws.to_csv_string(&options);
        print!("{}", csv);
    }

    Ok(())
}

fn cmd_convert(
    file: &str,
    format: &str,
    sheet: Option<&str>,
    output: Option<&str>,
    delimiter: Option<char>,
) -> Result<(), Box<dyn std::error::Error>> {
    let input_path = std::path::Path::new(file);
    let input_ext = input_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    match format {
        "csv" => {
            // xlsx/xlsm → csv
            if input_ext != "xlsx" && input_ext != "xlsm" {
                return Err(format!(
                    "cannot convert .{} to csv; expected .xlsx or .xlsm input",
                    input_ext
                )
                .into());
            }

            let wb = Workbook::open_readonly(file)?;

            // Build CsvOptions
            let mut options = CsvOptions::new();
            if let Some(d) = delimiter {
                options = options.delimiter(d as u8);
            }

            // Determine output directory
            let out_dir = match output {
                Some(p) => std::path::PathBuf::from(p),
                None => std::path::PathBuf::from("."),
            };
            std::fs::create_dir_all(&out_dir)?;

            if let Some(name) = sheet {
                // Export a single sheet
                let names = wb.sheet_names();
                let idx = names
                    .iter()
                    .position(|n| *n == name)
                    .ok_or_else(|| format!("sheet '{}' not found in workbook", name))?;
                let ws = wb.worksheet_ref(idx)?;
                let csv_path = out_dir.join(format!("{}.csv", name));
                ws.to_csv_file(&csv_path, &options)?;
            } else {
                // Export all sheets
                let names = wb.sheet_names();
                for (i, name) in names.iter().enumerate() {
                    let ws = wb.worksheet_ref(i)?;
                    let csv_path = out_dir.join(format!("{}.csv", name));
                    ws.to_csv_file(&csv_path, &options)?;
                }
            }

            Ok(())
        }
        "xlsx" => {
            // csv → xlsx
            if input_ext != "csv" {
                return Err(
                    format!("cannot convert .{} to xlsx; expected .csv input", input_ext).into(),
                );
            }

            let delim = delimiter.map(|d| d as u8).unwrap_or(b',');

            // Derive output filename
            let output_path = match output {
                Some(p) => std::path::PathBuf::from(p),
                None => input_path.with_extension("xlsx"),
            };

            // Read CSV file line-by-line
            let file_handle = std::fs::File::open(file)?;
            let reader = std::io::BufReader::new(file_handle);

            let mut wb = Workbook::new();
            let ws = wb.worksheet(0)?;

            use std::io::BufRead;
            for (row_idx, line_result) in reader.lines().enumerate() {
                let line = line_result?;
                let fields = parse_csv_line(&line, delim);
                for (col_idx, field) in fields.iter().enumerate() {
                    match detect_cell_value(field) {
                        CellValueType::Number(n) => {
                            ws.write(row_idx as u32, col_idx as u16, n)?;
                        }
                        CellValueType::Bool(b) => {
                            ws.write(row_idx as u32, col_idx as u16, b)?;
                        }
                        CellValueType::String(s) => {
                            ws.write(row_idx as u32, col_idx as u16, s.as_str())?;
                        }
                    }
                }
            }

            wb.save(&output_path)?;

            Ok(())
        }
        "xlsm" => {
            // xlsx → xlsm
            if input_ext != "xlsx" {
                return Err(format!(
                    "cannot convert .{} to xlsm; expected .xlsx input",
                    input_ext
                )
                .into());
            }

            let mut wb = Workbook::open(file)?;

            // Check if the workbook contains VBA project data
            let vba_data = wb
                .passthrough_entries()
                .iter()
                .find(|(name, _)| name.eq_ignore_ascii_case("xl/vbaProject.bin"))
                .map(|(_, data)| data.clone());

            match vba_data {
                Some(vba_bytes) => {
                    let output_path = match output {
                        Some(p) => std::path::PathBuf::from(p),
                        None => input_path.with_extension("xlsm"),
                    };
                    wb.save_as_xlsm(&output_path, &vba_bytes)?;
                    Ok(())
                }
                None => Err(
                    "cannot convert xlsx to xlsm: the input file does not contain VBA project data. \
                     xlsx\u{2192}xlsm conversion requires an existing VBA project."
                        .into(),
                ),
            }
        }
        _ => Err(format!("unsupported target format: {}", format).into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- detect_cell_value tests ---

    #[test]
    fn detect_integer() {
        assert_eq!(detect_cell_value("42"), CellValueType::Number(42.0));
    }

    #[test]
    fn detect_float() {
        assert_eq!(detect_cell_value("3.14"), CellValueType::Number(3.14));
    }

    #[test]
    fn detect_negative_number() {
        assert_eq!(detect_cell_value("-7.5"), CellValueType::Number(-7.5));
    }

    #[test]
    fn detect_true_lowercase() {
        assert_eq!(detect_cell_value("true"), CellValueType::Bool(true));
    }

    #[test]
    fn detect_true_uppercase() {
        assert_eq!(detect_cell_value("TRUE"), CellValueType::Bool(true));
    }

    #[test]
    fn detect_true_mixed_case() {
        assert_eq!(detect_cell_value("True"), CellValueType::Bool(true));
    }

    #[test]
    fn detect_false_lowercase() {
        assert_eq!(detect_cell_value("false"), CellValueType::Bool(false));
    }

    #[test]
    fn detect_false_uppercase() {
        assert_eq!(detect_cell_value("FALSE"), CellValueType::Bool(false));
    }

    #[test]
    fn detect_string() {
        assert_eq!(
            detect_cell_value("hello"),
            CellValueType::String("hello".to_string())
        );
    }

    #[test]
    fn detect_empty_string() {
        assert_eq!(detect_cell_value(""), CellValueType::String("".to_string()));
    }

    // --- parse_csv_line tests ---

    #[test]
    fn parse_simple_fields() {
        assert_eq!(parse_csv_line("a,b,c", b','), vec!["a", "b", "c"]);
    }

    #[test]
    fn parse_empty_fields() {
        assert_eq!(parse_csv_line("a,,c", b','), vec!["a", "", "c"]);
    }

    #[test]
    fn parse_quoted_field() {
        assert_eq!(
            parse_csv_line(r#""hello",world"#, b','),
            vec!["hello", "world"]
        );
    }

    #[test]
    fn parse_quoted_field_with_delimiter() {
        assert_eq!(parse_csv_line(r#""a,b",c"#, b','), vec!["a,b", "c"]);
    }

    #[test]
    fn parse_escaped_quotes() {
        assert_eq!(
            parse_csv_line(r#""say ""hi""",done"#, b','),
            vec![r#"say "hi""#, "done"]
        );
    }

    #[test]
    fn parse_tab_delimiter() {
        assert_eq!(parse_csv_line("a\tb\tc", b'\t'), vec!["a", "b", "c"]);
    }

    #[test]
    fn parse_single_field() {
        assert_eq!(parse_csv_line("only", b','), vec!["only"]);
    }

    #[test]
    fn parse_empty_line() {
        assert_eq!(parse_csv_line("", b','), vec![""]);
    }

    #[test]
    fn parse_quoted_empty_field() {
        assert_eq!(parse_csv_line(r#""",b"#, b','), vec!["", "b"]);
    }

    #[test]
    fn parse_custom_delimiter() {
        assert_eq!(parse_csv_line("a|b|c", b'|'), vec!["a", "b", "c"]);
    }
}
