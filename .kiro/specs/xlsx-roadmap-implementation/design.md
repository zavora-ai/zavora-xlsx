# Design Document — zavora-xlsx Roadmap Implementation

## Overview

This design covers the architecture for extending zavora-xlsx from its current v0.1.0 baseline to full Excel OOXML feature parity across 10 phases. The design addresses read parity (Phase 1), a formula engine (Phase 2), chart completeness (Phase 3), advanced formatting (Phase 4), data and interactivity (Phase 5), streaming and performance (Phase 6), file format variants (Phase 7), document features (Phase 8), security and compliance (Phase 9), and ecosystem integrations (Phase 10).

The crate currently has ~6.6K lines of Rust with 4 dependencies (`quick-xml`, `zip`, `atoi_simd`, `fast-float2`), a modular structure (`reader/`, `writer/`, `features/`, `worksheet/`, `workbook/`, `model/`, `xml/`, `zip/`), and supports reading cells, formulas, merges, widths, heights, freeze panes, visibility, and images. The writer supports full formatting, 9 chart types, tables, conditional formatting, data validation, images, sparklines, pivot tables, and treemaps. A streaming write mode and edit mode with VBA passthrough exist.

### Design Principles

1. **Zero-copy where possible**: Use borrowed slices and avoid allocations in hot paths (parsing, streaming)
2. **Feature-gated dependencies**: New heavy dependencies (crypto, serde, PyO3, napi-rs) are behind Cargo feature flags
3. **Backward compatibility**: All existing public APIs remain unchanged; new capabilities are additive
4. **Fail gracefully**: Unknown or unsupported XML elements are skipped, not errored — matching the existing reader pattern
5. **Minimal dependency surface**: Prefer implementing small algorithms in-crate over adding dependencies

---

## Architecture

### High-Level Module Map

```
src/
├── cell.rs                  # CellValue, CellType, IntoExcelData
├── datetime.rs              # ExcelDateTime
├── error.rs                 # Error enum (extended with new variants)
├── format.rs                # Format builder (extended: gradients, themes)
├── formula.rs               # Current: adjust_formula only
├── lib.rs                   # Public re-exports
├── properties.rs            # DocProperties
├── streaming.rs             # StreamingWorkbook (extended: charts, images, CF)
├── utility.rs               # Cell ref parsing, col_to_letter
│
├── model/
│   ├── shared_strings.rs    # SharedStringTable
│   └── style_registry.rs    # StyleRegistry (extended: gradient, theme)
│
├── reader/
│   ├── mod.rs
│   ├── xlsx_reader.rs       # XlsxData, read_xlsx (extended: more parts)
│   ├── sheet_reader.rs      # RawCell, SheetMeta (extended: CF, validation, hyperlinks, etc.)
│   ├── sst_parser.rs        # SST parsing
│   ├── style_parser.rs      # ParsedStyles (extended: full font/fill/border/alignment)
│   ├── rel_parser.rs        # Relationship parsing
│   ├── chart_reader.rs      # [NEW] Parse chart XML → Chart structs
│   ├── table_reader.rs      # [NEW] Parse table XML → Table structs
│   ├── cf_reader.rs         # [NEW] Parse CF rules from sheet XML
│   ├── validation_reader.rs # [NEW] Parse data validation rules
│   ├── sparkline_reader.rs  # [NEW] Parse sparkline groups from extLst
│   ├── comment_reader.rs    # [NEW] Parse threaded comments
│   └── streaming_reader.rs  # [NEW] SAX-style row iterator
│
├── writer/                  # (extended with new chart types, shapes)
│   ├── chart_writer.rs      # Extended: bubble, 3D, surface
│   ├── chartex_writer.rs    # Extended: waterfall, funnel, sunburst, histogram, box-whisker
│   └── ...
│
├── features/
│   ├── chart.rs             # Extended: Bubble, 3D, Surface, Map chart types
│   ├── conditional.rs       # Extended: gradient data bars
│   ├── image.rs             # Extended: alt text
│   ├── pivot.rs             # Extended: grouping, calculated items
│   ├── sparkline.rs
│   ├── table.rs             # Extended: custom styles, alt text
│   ├── treemap.rs
│   ├── validation.rs
│   ├── slicer.rs            # [NEW] Slicer definitions
│   ├── timeline.rs          # [NEW] Timeline definitions
│   └── shape.rs             # [NEW] Drawing shapes
│
├── formula_engine/          # [NEW] Phase 2 — Formula Engine
│   ├── mod.rs               # Public API: evaluate, recalculate
│   ├── token.rs             # Tokenizer: formula string → Token stream
│   ├── ast.rs               # AST node types
│   ├── parser.rs            # Pratt parser: Token stream → AST
│   ├── printer.rs           # AST → formula string (for round-trip)
│   ├── dependency.rs        # DependencyGraph, topological sort, cycle detection
│   ├── evaluator.rs         # AST evaluator with cell context
│   └── functions/
│       ├── mod.rs           # Function registry
│       ├── core.rs          # SUM, AVERAGE, COUNT, IF, VLOOKUP, INDEX, MATCH
│       ├── math.rs          # ROUND, ABS, MOD, POWER, SQRT, LOG, etc.
│       ├── text.rs          # CONCATENATE, LEFT, RIGHT, MID, LEN, TRIM, etc.
│       ├── date.rs          # TODAY, NOW, DATE, YEAR, MONTH, DAY, etc.
│       ├── lookup.rs        # HLOOKUP, XLOOKUP, MATCH, INDIRECT, OFFSET
│       ├── statistical.rs   # STDEV, VAR, MEDIAN, COUNTIF, SUMIF
│       └── logical.rs       # AND, OR, NOT, IFERROR, IFNA, SWITCH, IFS
│
├── formats/                 # [NEW] Phase 7 — File Format Variants
│   ├── mod.rs
│   ├── xlsb_reader.rs       # XLSB binary record parser
│   ├── xls_reader.rs        # BIFF8 OLE2 parser
│   ├── ods.rs               # ODS read/write
│   ├── csv_export.rs        # CSV/TSV export
│   └── strict_ooxml.rs      # Strict namespace mapping
│
├── crypto/                  # [NEW] Phase 9 — Encryption
│   ├── mod.rs
│   ├── standard.rs          # ECMA-376 Standard Encryption
│   └── agile.rs             # ECMA-376 Agile Encryption
│
├── serde_support/           # [NEW] Phase 10 — Serde Integration
│   ├── mod.rs
│   ├── ser.rs               # Serialize structs → rows
│   └── de.rs                # Deserialize rows → structs
│
├── workbook/
│   ├── mod.rs               # Extended: named ranges CRUD, custom properties
│   ├── save.rs
│   └── xml.rs
│
├── worksheet/
│   ├── mod.rs               # Extended: shapes, form controls
│   ├── features.rs
│   ├── layout.rs
│   ├── ops.rs
│   ├── types.rs             # Extended: SheetProtection granularity
│   └── write.rs
│
├── xml/
│   ├── xml_reader.rs
│   └── xml_writer.rs
│
└── zip/
    ├── zip_reader.rs
    └── zip_writer.rs
```

### Dependency Strategy

New dependencies are gated behind Cargo feature flags to keep the default build minimal:

| Feature Flag | Dependencies Added | Purpose |
|---|---|---|
| `formula` | (none — pure Rust) | Formula engine |
| `crypto` | `aes`, `sha2`, `hmac`, `cbc`, `pbkdf2` | File encryption/decryption |
| `serde` | `serde`, `serde_derive` | Struct ↔ row mapping |
| `derive` | `syn`, `quote`, `proc-macro2` | `#[derive(ExcelRow)]` proc macro |
| `async` | `tokio` or `async-std` (optional) | Async file I/O |
| `python` | `pyo3` | Python bindings |
| `nodejs` | `napi`, `napi-derive` | Node.js bindings |
| `cffi` | (none — `#[no_mangle] extern "C"`) | C FFI |

```toml
[features]
default = []
formula = []
crypto = ["dep:aes", "dep:sha2", "dep:hmac", "dep:cbc", "dep:pbkdf2"]
serde = ["dep:serde", "dep:serde_derive"]
derive = ["dep:zavora-xlsx-derive"]
async-tokio = ["dep:tokio"]
```

### Cross-Cutting Concerns

**Error Handling**: The existing `Error` enum is extended with new variants:

```rust
pub enum Error {
    // ... existing variants ...
    FormulaError(FormulaError),    // Parse/eval errors
    CryptoError(String),           // Encryption/decryption failures
    Authentication,                // Wrong password
    SerdeError(String),            // Serde mapping errors
    UnsupportedFormat(String),     // XLSB/XLS/ODS feature gaps
    CircularReference(Vec<String>),// Cycle in formula graph
}
```

**Thread Safety**: The `StyleRegistry` and `SharedStringTable` gain `Arc<Mutex<_>>` wrappers for parallel sheet writing (Phase 6.4). The existing single-threaded path remains unchanged — the mutex is only acquired when the parallel API is used.

---

## Components and Interfaces

### Phase 1: Reader Extensions

The current reader parses cells, formulas, merges, widths, heights, freeze panes, visibility, and images. Each new reader component follows the same pattern: a standalone parsing function that takes `&[u8]` (raw XML) and returns a parsed struct.

#### Style Parser Extension

The current `ParsedStyles` only stores `xf_num_fmt_ids`. It is extended to store the full style chain:

```rust
pub struct ParsedStyles {
    pub num_formats: Vec<(u16, String)>,
    pub xf_num_fmt_ids: Vec<u16>,
    // NEW fields:
    pub fonts: Vec<ParsedFont>,
    pub fills: Vec<ParsedFill>,
    pub borders: Vec<ParsedBorder>,
    pub xf_records: Vec<XfRecord>,  // full xf: fontId, fillId, borderId, alignment
    pub dxf_records: Vec<DxfRecord>, // differential formats for CF
}

pub struct XfRecord {
    pub font_id: usize,
    pub fill_id: usize,
    pub border_id: usize,
    pub num_fmt_id: u16,
    pub alignment: ParsedAlignment,
}
```

A `resolve_format(xf_index: usize) -> Format` method reconstructs a `Format` struct from the parsed components. This is used by the sheet reader to populate cell formats.

#### Sheet Reader Extension

`SheetMeta` is extended to capture all sheet-level features in a single pass:

```rust
pub struct SheetMeta {
    // existing
    pub merge_ranges: Vec<(RowNum, ColNum, RowNum, ColNum)>,
    pub col_widths: Vec<(ColNum, f64)>,
    pub row_heights: Vec<(RowNum, f64)>,
    pub freeze_row: RowNum,
    pub freeze_col: ColNum,
    pub drawing_rid: Option<String>,
    pub legacy_drawing_rid: Option<String>,
    // NEW
    pub conditional_formats: Vec<ParsedCfRule>,
    pub validations: Vec<DataValidation>,
    pub hyperlinks: Vec<ParsedHyperlink>,
    pub sparkline_groups: Vec<ParsedSparklineGroup>,
    pub print_settings: Option<PrintSettings>,
    pub protection: Option<SheetProtection>,
    pub row_outline_levels: BTreeMap<RowNum, u8>,
    pub col_outline_levels: BTreeMap<ColNum, u8>,
}
```

All new elements are parsed in the existing single-pass loop through the sheet XML. Elements before `<sheetData>` (cols, pane, sheetProtection, conditionalFormatting) and after `<sheetData>` (mergeCells, hyperlinks, dataValidations, drawing, extLst for sparklines) are captured in their respective scan phases.

#### Chart Reader

Charts are stored in separate XML parts (`xl/charts/chart{N}.xml`). The reader:
1. Parses drawing relationships from `xl/drawings/drawing{N}.xml` to find chart part paths
2. Parses each chart XML into a `Chart` struct, handling both `c:chart` and `cx:chart` namespaces
3. Returns charts associated with their anchor positions

```rust
pub fn read_chart(data: &[u8]) -> Result<Chart> { ... }
pub fn read_chartex(data: &[u8]) -> Result<TreemapChart> { ... }
```

#### Table Reader

Tables are stored in `xl/tables/table{N}.xml`. The reader parses table XML into `Table` structs:

```rust
pub fn read_table(data: &[u8]) -> Result<Table> { ... }
```

### Phase 2: Formula Engine

The formula engine is a self-contained module (`src/formula_engine/`) with no external dependencies.

#### Tokenizer

Converts a formula string into a flat token stream:

```rust
pub enum Token {
    CellRef { col: ColNum, row: RowNum, abs_col: bool, abs_row: bool },
    RangeRef { start: CellRef, end: CellRef },
    SheetRef { sheet: String, reference: Box<Token> },
    StructuredRef { table: String, column: String },
    Number(f64),
    StringLiteral(String),
    Bool(bool),
    Error(String),          // #VALUE!, #REF!, etc.
    Function(String),       // SUM, IF, etc.
    Operator(Op),           // +, -, *, /, ^, &, =, <>, <, >, <=, >=
    OpenParen,
    CloseParen,
    Comma,
    Colon,                  // range operator
    ArrayOpen,              // {
    ArrayClose,             // }
    Semicolon,              // array row separator
}

pub fn tokenize(formula: &str) -> Result<Vec<Token>, FormulaError> { ... }
```

#### Parser (Pratt Parsing)

A Pratt parser converts the token stream into an AST. Pratt parsing handles operator precedence naturally and is well-suited for Excel's expression grammar:

```rust
pub enum AstNode {
    Number(f64),
    String(String),
    Bool(bool),
    Error(String),
    CellRef { col: ColNum, row: RowNum, abs_col: bool, abs_row: bool },
    Range { start: Box<AstNode>, end: Box<AstNode> },
    SheetRef { sheet: String, inner: Box<AstNode> },
    BinaryOp { op: Op, left: Box<AstNode>, right: Box<AstNode> },
    UnaryOp { op: Op, operand: Box<AstNode> },
    FunctionCall { name: String, args: Vec<AstNode> },
    Array { rows: Vec<Vec<AstNode>> },
}

pub fn parse(tokens: &[Token]) -> Result<AstNode, FormulaError> { ... }
```

#### Printer (AST → String)

Converts an AST back to a formula string. This enables the round-trip property: `parse(print(parse(s))) == parse(s)`.

```rust
pub fn print_formula(ast: &AstNode) -> String { ... }
```

#### Dependency Graph

```rust
pub struct DependencyGraph {
    edges: HashMap<CellAddr, Vec<CellAddr>>,  // cell → cells it depends on
    reverse: HashMap<CellAddr, Vec<CellAddr>>, // cell → cells that depend on it
}

impl DependencyGraph {
    pub fn build(workbook: &Workbook) -> Result<Self, Error> { ... }
    pub fn topological_order(&self) -> Result<Vec<CellAddr>, Error> { ... }
    pub fn detect_cycles(&self) -> Option<Vec<CellAddr>> { ... }
    pub fn dependents_of(&self, cell: CellAddr) -> Vec<CellAddr> { ... }
}
```

Cycle detection uses Tarjan's algorithm. Topological sort uses Kahn's algorithm.

#### Evaluator

```rust
pub struct Evaluator<'a> {
    workbook: &'a Workbook,
    volatile_cells: HashSet<CellAddr>,
}

impl<'a> Evaluator<'a> {
    pub fn evaluate(&self, ast: &AstNode, context: &CellContext) -> Result<CellValue, FormulaError> { ... }
    pub fn recalculate(&mut self) -> Result<(), Error> { ... }
}
```

Functions are registered in a `HashMap<&str, fn(&[CellValue]) -> Result<CellValue, FormulaError>>` dispatch table.

### Phase 3: Chart Engine Extensions

New chart types are added to the existing `ChartType` enum and `ChartEx` system:

```rust
pub enum ChartType {
    Bar, Column, Line, Pie, Scatter, Area, Doughnut, Radar, Stock,
    // NEW
    Bubble,
    Column3D, Bar3D, Line3D, Pie3D, Area3D,
    Surface, WireframeSurface,
    Map,
}

pub enum ChartExType {
    Treemap,
    // NEW
    Waterfall, Funnel, Sunburst, Histogram, Pareto, BoxWhisker,
}
```

Bubble charts extend `ChartSeries` with a `bubble_sizes: Option<String>` field for the size dimension reference.

3D charts add a `View3D` struct to `Chart`:

```rust
pub struct View3D {
    pub rot_x: i16,      // elevation
    pub rot_y: i16,      // rotation
    pub perspective: u8,
    pub right_angle_axes: bool,
}
```

Chart style themes (1–48) are stored as a `style: Option<u8>` on `Chart` and serialized as `<c:style val="N"/>`.

Axis formatting, plot area formatting, and series formatting extend the existing `Chart` and `ChartSeries` structs with optional formatting fields.

### Phase 6: Streaming Reader

The streaming reader provides a SAX-style iterator over rows with constant memory:

```rust
pub struct StreamingReader<R: Read + Seek> {
    zip: ZipReader<R>,
    sst: SharedStringTable,
    styles: ParsedStyles,
    sheet_index: usize,
    xml_reader: Option<XmlReaderInner<BufReader<ZipFile<'_>>>>,
    current_row: Option<StreamingRow>,
}

pub struct StreamingRow {
    pub row_index: RowNum,
    pub cells: Vec<StreamingCell>,
}

pub struct StreamingCell {
    pub col: ColNum,
    pub value: CellValue,
    pub xf_index: u32,
}

impl<R: Read + Seek> Iterator for StreamingReader<R> {
    type Item = Result<StreamingRow>;
}
```

The SST is loaded upfront (it's typically small relative to sheet data). The sheet XML is streamed through the `quick-xml` SAX reader without buffering the entire document.

### Phase 7: File Format Variants

#### XLSB Reader

XLSB files use the same ZIP container but replace XML parts with binary record streams. Each record has a type ID and length prefix. The reader:
1. Detects XLSB via content types (`application/vnd.ms-excel.sheet.binary.macroEnabled.main+xml`)
2. Reads binary records sequentially, mapping record types to cell values
3. Reuses the existing `CellValue` and `SheetMeta` types

#### XLS Reader (BIFF8)

XLS files use OLE2 compound document format. The reader:
1. Parses the OLE2 header and directory to find the Workbook stream
2. Reads BIFF8 records (BOF, SHEET, SST, LABELSST, NUMBER, FORMULA, etc.)
3. Maps to the same `CellValue` types

Both XLSB and XLS readers are behind feature flags to avoid bloating the default build.

#### ODS Read/Write

ODS uses a ZIP container with `content.xml` containing `<table:table>` elements. The reader maps ODS cell types to `CellValue`. The writer serializes the workbook model to ODS XML.

### Phase 9: Encryption

ECMA-376 defines two encryption schemes:

1. **Standard Encryption**: AES-128-CBC with SHA-1 key derivation
2. **Agile Encryption**: AES-256-CBC with SHA-512 and HMAC integrity verification

The encryption module wraps the OLE2 compound document that contains the encrypted package:

```rust
pub fn decrypt_standard(ole2_data: &[u8], password: &str) -> Result<Vec<u8>> { ... }
pub fn decrypt_agile(ole2_data: &[u8], password: &str) -> Result<Vec<u8>> { ... }
pub fn encrypt_agile(xlsx_data: &[u8], password: &str) -> Result<Vec<u8>> { ... }
```

The `Workbook::open` path detects encrypted files by checking for the OLE2 magic bytes (`D0 CF 11 E0`) instead of the ZIP local file header (`PK`).

### Phase 10: Serde Integration

#### Serialization (Struct → Rows)

```rust
impl Workbook {
    pub fn write_rows<T: Serialize>(&mut self, sheet: usize, data: &[T]) -> Result<()> { ... }
}
```

Uses a custom `Serializer` that maps struct fields to columns. Field names become headers (row 0). Each struct instance becomes a row.

#### Deserialization (Rows → Struct)

```rust
impl Workbook {
    pub fn read_rows<T: DeserializeOwned>(&self, sheet: usize) -> Result<Vec<T>> { ... }
}
```

Uses header row to map column positions to struct fields. A custom `Deserializer` reads cell values and coerces types.

#### Derive Macro

A separate crate `zavora-xlsx-derive` provides `#[derive(ExcelRow)]`:

```rust
#[derive(ExcelRow)]
struct Invoice {
    #[excel(header = "Invoice #")]
    id: u32,
    #[excel(header = "Amount", format = "#,##0.00")]
    amount: f64,
    #[excel(header = "Due Date", format = "yyyy-mm-dd")]
    due_date: Option<String>,
}
```

### Language Bindings

All bindings wrap the same Rust core. Each binding crate lives in a separate workspace member:

```
zavora-xlsx/           # core crate
zavora-xlsx-python/    # PyO3 bindings
zavora-xlsx-node/      # napi-rs bindings
zavora-xlsx-derive/    # proc macro crate
```

The C FFI uses opaque pointer handles:

```c
typedef struct ZavoraWorkbook ZavoraWorkbook;
ZavoraWorkbook* zavora_workbook_new(void);
int zavora_worksheet_write_string(ZavoraWorkbook* wb, uint32_t sheet, uint32_t row, uint16_t col, const char* text);
int zavora_workbook_save(ZavoraWorkbook* wb, const char* path);
void zavora_workbook_free(ZavoraWorkbook* wb);
const char* zavora_last_error(void);
```

WASM compilation targets `wasm32-unknown-unknown` with `#[cfg(target_arch = "wasm32")]` guards on file system operations. Only buffer-based APIs are exposed.

---

## Data Models

### Extended CellValue

No changes to the existing `CellValue` enum. The formula engine operates on `CellValue` directly.

### Formula Engine Types

```rust
pub struct CellAddr {
    pub sheet: usize,
    pub row: RowNum,
    pub col: ColNum,
}

pub enum FormulaError {
    Parse { position: usize, message: String },
    Eval(String),
    CircularRef(Vec<CellAddr>),
    DivByZero,
    Value,
    Ref,
    Name,
    Num,
    Na,
    Spill,
}
```

### Extended Format (Gradients and Themes)

```rust
pub struct Format {
    // ... existing fields ...
    // NEW
    pub(crate) gradient: Option<GradientFill>,
    pub(crate) theme_color: Option<ThemeColor>,
}

pub struct GradientFill {
    pub angle: f64,
    pub stops: Vec<GradientStop>,
}

pub struct GradientStop {
    pub position: f64,  // 0.0 to 1.0
    pub color: [u8; 3],
}

pub struct ThemeColor {
    pub index: ThemeColorIndex,
    pub tint: f64,  // -1.0 to 1.0
}

pub enum ThemeColorIndex {
    Dark1, Light1, Dark2, Light2,
    Accent1, Accent2, Accent3, Accent4, Accent5, Accent6,
}
```

### Slicer and Timeline

```rust
pub struct Slicer {
    pub name: String,
    pub caption: String,
    pub source_name: String,  // table or pivot table column
    pub row: RowNum,
    pub col: ColNum,
    pub width: u32,
    pub height: u32,
    pub style: String,
}

pub struct Timeline {
    pub name: String,
    pub caption: String,
    pub source_name: String,  // pivot table date field
    pub row: RowNum,
    pub col: ColNum,
    pub level: TimelineLevel,
}

pub enum TimelineLevel { Years, Quarters, Months, Days }
```

### Drawing Shapes

```rust
pub struct Shape {
    pub shape_type: ShapeType,
    pub row: RowNum,
    pub col: ColNum,
    pub width: u32,
    pub height: u32,
    pub text: Option<String>,
    pub fill_color: Option<[u8; 3]>,
    pub outline_color: Option<[u8; 3]>,
    pub outline_width: Option<f64>,
}

pub enum ShapeType {
    Rectangle, RoundedRectangle, Arrow, Callout, TextBox,
}
```

### Encryption Metadata

```rust
pub struct EncryptionInfo {
    pub scheme: EncryptionScheme,
    pub key_bits: u16,
    pub hash_algorithm: String,
    pub salt: Vec<u8>,
    pub encrypted_verifier: Vec<u8>,
    pub encrypted_verifier_hash: Vec<u8>,
}

pub enum EncryptionScheme { Standard, Agile }
```

### CSV Export Options

```rust
pub struct CsvOptions {
    pub delimiter: u8,        // default: b','
    pub quote: u8,            // default: b'"'
    pub line_ending: String,  // default: "\r\n"
    pub date_format: String,  // default: ISO 8601
}
```

