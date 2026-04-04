# zavora-xlsx — Unified High-Performance Rust Excel Engine

## Vision

A single Rust crate that reads, writes, and edits xlsx files with:
- **Calamine-class read performance** (streaming XML, SIMD parsing, zero-copy)
- **rust_xlsxwriter-class write features** (charts, tables, conditional formatting, sparklines, images)
- **umya-spreadsheet-class edit capability** (open → modify → save with fidelity)
- **One coordinate system, one API, one mental model**

No other Rust crate does all three well. zavora-xlsx will.

## License

MIT OR Apache-2.0 (dual-licensed, standard Rust ecosystem practice).

All three reference engines are open source:
- calamine: MIT/Apache-2.0
- rust_xlsxwriter: MIT/Apache-2.0  
- umya-spreadsheet: MIT

We study their approaches and build our own implementation. We don't copy code verbatim — we design from the patterns and improve.

---

## Table of Contents

1. [OOXML File Format](#1-ooxml-file-format)
2. [Public API](#2-public-api)
3. [Internal Architecture](#3-internal-architecture)
4. [Module Breakdown](#4-module-breakdown)
5. [Performance Strategy](#5-performance-strategy)
6. [Implementation Phases](#6-implementation-phases)
7. [Dependencies](#7-dependencies)
8. [Design Decisions Log](#8-design-decisions-log)

---

## 1. OOXML File Format

An `.xlsx` file is a ZIP archive following the Office Open XML (ECMA-376) standard.

### 1.1 Zip Structure

```
[Content_Types].xml              ← maps paths/extensions to MIME types
_rels/.rels                      ← root relationships
docProps/
  app.xml                        ← application properties (company, version)
  core.xml                       ← Dublin Core metadata (author, title, dates)
  custom.xml                     ← custom key-value properties
xl/
  workbook.xml                   ← sheet list, defined names, workbook settings
  sharedStrings.xml              ← deduplicated string table
  styles.xml                     ← fonts, fills, borders, number formats, xf records
  theme/theme1.xml               ← color scheme, font scheme
  _rels/workbook.xml.rels        ← workbook → sheet/style/SST/theme relationships
  worksheets/
    sheet1.xml                   ← cell data, merge cells, conditional formatting
    sheet2.xml
    _rels/
      sheet1.xml.rels            ← sheet → drawing/table/comment relationships
  drawings/
    drawing1.xml                 ← drawing container (anchors for charts/images)
    _rels/drawing1.xml.rels      ← drawing → chart/image relationships
  charts/
    chart1.xml                   ← chart definition (type, series, axes)
  tables/
    table1.xml                   ← table definition (columns, style, autofilter)
  media/
    image1.png                   ← embedded image binary data
  printerSettings/
    printerSettings1.bin         ← binary printer settings
```

### 1.2 Key XML Schemas

| File | Root Element | Critical Attributes/Children |
|------|-------------|------------------------------|
| workbook.xml | `<workbook>` | `<sheets><sheet name="" sheetId="" r:id=""/>`, `<definedNames>`, `<workbookPr date1904="">` |
| sharedStrings.xml | `<sst count="" uniqueCount="">` | `<si><t>text</t></si>` or `<si><r><rPr/><t/></r></si>` (rich text) |
| styles.xml | `<styleSheet>` | `<numFmts>`, `<fonts>`, `<fills>`, `<borders>`, `<cellXfs>` (the xf index table) |
| sheet.xml | `<worksheet>` | `<dimension>`, `<sheetViews>`, `<cols>`, `<sheetData>`, `<mergeCells>`, `<conditionalFormatting>`, `<dataValidations>`, `<drawing>`, `<tableParts>` |
| drawing.xml | `<wsDr>` | `<twoCellAnchor>`, `<oneCellAnchor>` containing `<graphicFrame>` (charts) or `<pic>` (images) |

### 1.3 Relationship System

Every XML file can have a `.rels` companion in a `_rels/` subdirectory. Relationships map `rId` strings to target paths and relationship types (URIs).

Key relationship types:
- `http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet`
- `http://schemas.openxmlformats.org/officeDocument/2006/relationships/sharedStrings`
- `http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles`
- `http://schemas.openxmlformats.org/officeDocument/2006/relationships/theme`
- `http://schemas.openxmlformats.org/officeDocument/2006/relationships/drawing`
- `http://schemas.openxmlformats.org/officeDocument/2006/relationships/chart`
- `http://schemas.openxmlformats.org/officeDocument/2006/relationships/table`
- `http://schemas.openxmlformats.org/officeDocument/2006/relationships/image`

### 1.4 Cell XML Format

```xml
<row r="1" spans="1:5">
  <c r="A1" s="0" t="s">     <!-- s=style index, t=type -->
    <v>0</v>                   <!-- value: SST index for t="s", number for t="n" -->
  </c>
  <c r="B1" s="2" t="n">
    <v>42.5</v>
  </c>
  <c r="C1" s="0" t="b">
    <v>1</v>                   <!-- 1=true, 0=false -->
  </c>
  <c r="D1" s="0">
    <f>SUM(A1:C1)</f>          <!-- formula -->
    <v>43.5</v>                <!-- cached result -->
  </c>
</row>
```

Cell type attribute (`t`):
- `s` — shared string (value is index into SST)
- `n` or absent — number
- `b` — boolean
- `e` — error
- `str` — inline string (formula result)
- `d` — ISO 8601 date string
- `inlineStr` — inline string with `<is><t>` child

### 1.5 Style Index Resolution

A cell's `s` attribute indexes into `<cellXfs>` in styles.xml. Each `<xf>` record references:
- `fontId` → index into `<fonts>`
- `fillId` → index into `<fills>`
- `borderId` → index into `<borders>`
- `numFmtId` → builtin ID or index into `<numFmts>`
- `applyFont`, `applyFill`, etc. — flags indicating which parts are active

Date detection: Numbers with date-like number formats (IDs 14-22, 45-47, or custom formats containing `d/m/y/h/s` tokens) are dates. The number is an Excel serial date (days since 1900-01-01 or 1904-01-01).


---

## 2. Public API

### 2.1 Design Principles

1. **One coordinate system**: 0-based `(row: u32, col: u16)` everywhere. Row 0 = Excel row 1, col 0 = column A. Matches rust_xlsxwriter. Type aliases: `type RowNum = u32; type ColNum = u16;`
2. **Three modes, one type**: `Workbook` handles create, open-for-edit, and open-readonly. Mode determines what operations are available (enforced at runtime, not compile time, for API simplicity).
3. **Builder-pattern formatting**: `Format::new().bold().font_size(14.0).font_color("#FF0000")`
4. **Generic write via trait**: `worksheet.write(row, col, value)` accepts any `impl IntoExcelData`
5. **Chainable worksheet methods**: Most methods return `Result<&mut Worksheet, Error>` for chaining
6. **A1 notation accepted everywhere**: Any method taking `(row, col)` also has an `_a1` variant or accepts `impl Into<CellRef>`

### 2.2 Core Types

```rust
// ── Workbook ──────────────────────────────────────────────────

/// The top-level Excel workbook.
pub struct Workbook { /* internal */ }

impl Workbook {
    /// Create a new empty workbook with one sheet ("Sheet1").
    pub fn new() -> Workbook;

    /// Open an existing xlsx file for editing.
    /// Reads structure eagerly, worksheet data lazily.
    pub fn open(path: impl AsRef<Path>) -> Result<Workbook, Error>;

    /// Open an existing xlsx file in read-only mode.
    /// Fastest path — streaming reads, no edit capability.
    pub fn open_readonly(path: impl AsRef<Path>) -> Result<Workbook, Error>;

    /// Save to a file path. Creates or overwrites.
    pub fn save(&mut self, path: impl AsRef<Path>) -> Result<(), Error>;

    /// Save to an in-memory buffer.
    pub fn save_to_buffer(&mut self) -> Result<Vec<u8>, Error>;

    // ── Sheet management ──
    pub fn add_worksheet(&mut self) -> &mut Worksheet;
    pub fn add_worksheet_with_name(&mut self, name: &str) -> Result<&mut Worksheet, Error>;
    pub fn worksheet(&mut self, index: usize) -> Result<&mut Worksheet, Error>;
    pub fn worksheet_by_name(&mut self, name: &str) -> Result<&mut Worksheet, Error>;
    pub fn sheet_names(&self) -> Vec<&str>;
    pub fn sheet_count(&self) -> usize;
    pub fn remove_worksheet(&mut self, index: usize) -> Result<(), Error>;
    pub fn rename_worksheet(&mut self, index: usize, name: &str) -> Result<(), Error>;

    // ── Properties ──
    pub fn set_properties(&mut self, props: &DocProperties) -> &mut Workbook;

    // ── Defined names ──
    pub fn define_name(&mut self, name: &str, formula: &str) -> Result<&mut Workbook, Error>;
}

// ── Worksheet ─────────────────────────────────────────────────

pub struct Worksheet { /* internal */ }

impl Worksheet {
    // ── Writing ──
    pub fn write(&mut self, row: RowNum, col: ColNum, data: impl IntoExcelData) -> Result<&mut Self, Error>;
    pub fn write_with_format(&mut self, row: RowNum, col: ColNum, data: impl IntoExcelData, format: &Format) -> Result<&mut Self, Error>;
    pub fn write_row(&mut self, row: RowNum, col: ColNum, data: impl IntoIterator<Item = impl IntoExcelData>) -> Result<&mut Self, Error>;
    pub fn write_column(&mut self, row: RowNum, col: ColNum, data: impl IntoIterator<Item = impl IntoExcelData>) -> Result<&mut Self, Error>;
    pub fn write_formula(&mut self, row: RowNum, col: ColNum, formula: &str) -> Result<&mut Self, Error>;
    pub fn write_blank(&mut self, row: RowNum, col: ColNum, format: &Format) -> Result<&mut Self, Error>;

    // ── Reading ──
    pub fn read_cell(&self, row: RowNum, col: ColNum) -> CellValue;
    pub fn read_cell_a1(&self, reference: &str) -> Result<CellValue, Error>;
    pub fn used_range(&self) -> Option<(RowNum, ColNum, RowNum, ColNum)>;
    pub fn rows(&self) -> RowIterator<'_>;
    pub fn rows_range(&self, start_row: RowNum, end_row: RowNum) -> RowIterator<'_>;

    // ── Formatting ──
    pub fn set_cell_format(&mut self, row: RowNum, col: ColNum, format: &Format) -> Result<&mut Self, Error>;
    pub fn set_range_format(&mut self, first_row: RowNum, first_col: ColNum, last_row: RowNum, last_col: ColNum, format: &Format) -> Result<&mut Self, Error>;
    pub fn merge_range(&mut self, first_row: RowNum, first_col: ColNum, last_row: RowNum, last_col: ColNum, text: &str, format: &Format) -> Result<&mut Self, Error>;

    // ── Layout ──
    pub fn set_column_width(&mut self, col: ColNum, width: f64) -> Result<&mut Self, Error>;
    pub fn set_row_height(&mut self, row: RowNum, height: f64) -> Result<&mut Self, Error>;
    pub fn set_freeze_panes(&mut self, row: RowNum, col: ColNum) -> Result<&mut Self, Error>;
    pub fn autofit(&mut self) -> &mut Self;

    // ── Features ──
    pub fn insert_chart(&mut self, row: RowNum, col: ColNum, chart: &Chart) -> Result<&mut Self, Error>;
    pub fn insert_image(&mut self, row: RowNum, col: ColNum, image: &Image) -> Result<&mut Self, Error>;
    pub fn add_table(&mut self, first_row: RowNum, first_col: ColNum, last_row: RowNum, last_col: ColNum, table: &Table) -> Result<&mut Self, Error>;
    pub fn add_conditional_format(&mut self, first_row: RowNum, first_col: ColNum, last_row: RowNum, last_col: ColNum, cf: &impl ConditionalFormat) -> Result<&mut Self, Error>;
    pub fn add_data_validation(&mut self, first_row: RowNum, first_col: ColNum, last_row: RowNum, last_col: ColNum, dv: &DataValidation) -> Result<&mut Self, Error>;
    pub fn add_sparkline(&mut self, row: RowNum, col: ColNum, sparkline: &Sparkline) -> Result<&mut Self, Error>;

    // ── Sheet properties ──
    pub fn set_name(&mut self, name: &str) -> Result<&mut Self, Error>;
    pub fn name(&self) -> &str;
    pub fn set_tab_color(&mut self, color: impl IntoColor) -> &mut Self;
    pub fn set_hidden(&mut self) -> &mut Self;
    pub fn protect(&mut self) -> &mut Self;
    pub fn protect_with_password(&mut self, password: &str) -> &mut Self;
}
```

### 2.3 Cell Values

```rust
/// What you get back when reading a cell.
pub enum CellValue {
    Empty,
    String(String),
    Number(f64),
    Integer(i64),
    Bool(bool),
    DateTime(ExcelDateTime),
    Error(CellError),
    Formula { formula: String, cached_value: Box<CellValue> },
}

/// Trait for types that can be written to a cell.
pub trait IntoExcelData {
    fn write(self, ws: &mut Worksheet, row: RowNum, col: ColNum) -> Result<&mut Worksheet, Error>;
    fn write_with_format(self, ws: &mut Worksheet, row: RowNum, col: ColNum, fmt: &Format) -> Result<&mut Worksheet, Error>;
}

// Implemented for: &str, String, f32, f64, i8..i64, u8..u64, bool,
// ExcelDateTime, Formula, Option<T: IntoExcelData>
```

### 2.4 Format

```rust
/// Cell formatting. Uses builder pattern. Implements Hash + Eq for deduplication.
pub struct Format { /* internal */ }

impl Format {
    pub fn new() -> Format;

    // Font
    pub fn bold(self) -> Self;
    pub fn italic(self) -> Self;
    pub fn underline(self, style: Underline) -> Self;
    pub fn strikethrough(self) -> Self;
    pub fn font_size(self, size: f64) -> Self;
    pub fn font_name(self, name: &str) -> Self;
    pub fn font_color(self, color: impl IntoColor) -> Self;

    // Fill
    pub fn background_color(self, color: impl IntoColor) -> Self;
    pub fn pattern(self, pattern: Pattern) -> Self;
    pub fn foreground_color(self, color: impl IntoColor) -> Self;

    // Borders
    pub fn border(self, style: BorderStyle) -> Self;
    pub fn border_color(self, color: impl IntoColor) -> Self;
    pub fn border_top(self, style: BorderStyle) -> Self;
    pub fn border_bottom(self, style: BorderStyle) -> Self;
    pub fn border_left(self, style: BorderStyle) -> Self;
    pub fn border_right(self, style: BorderStyle) -> Self;

    // Alignment
    pub fn align(self, align: Align) -> Self;
    pub fn text_wrap(self) -> Self;
    pub fn indent(self, level: u8) -> Self;
    pub fn rotation(self, angle: i16) -> Self;
    pub fn shrink(self) -> Self;

    // Number format
    pub fn num_format(self, format: &str) -> Self;
}

/// Anything that can become a color.
pub trait IntoColor {
    fn into_color(self) -> Color;
}
// Implemented for: &str ("#FF0000"), u32 (0xFF0000), Color enum, (u8,u8,u8)

pub enum Color {
    Rgb(u8, u8, u8),
    Theme(u8, f64),  // theme index + tint
    Named(NamedColor),
    Auto,
}
```

### 2.5 Charts

```rust
pub struct Chart { /* internal */ }

impl Chart {
    pub fn new(chart_type: ChartType) -> Chart;

    pub fn add_series(&mut self) -> &mut ChartSeries;
    pub fn title(&mut self) -> &mut ChartTitle;
    pub fn x_axis(&mut self) -> &mut ChartAxis;
    pub fn y_axis(&mut self) -> &mut ChartAxis;
    pub fn legend(&mut self) -> &mut ChartLegend;
    pub fn set_width(&mut self, width: u32) -> &mut Self;
    pub fn set_height(&mut self, height: u32) -> &mut Self;
    pub fn set_style(&mut self, style: u8) -> &mut Self;
}

pub struct ChartSeries { /* internal */ }
impl ChartSeries {
    pub fn set_values(&mut self, range: &str) -> &mut Self;      // "Sheet1!$B$1:$B$5"
    pub fn set_categories(&mut self, range: &str) -> &mut Self;
    pub fn set_name(&mut self, name: &str) -> &mut Self;
    pub fn set_format(&mut self, format: &ChartFormat) -> &mut Self;
}

pub enum ChartType {
    Bar, Column, Line, Pie, Scatter, Area, Doughnut, Radar, Stock,
}
```

### 2.6 Tables, Images, Conditional Formatting, Data Validation, Sparklines

```rust
// ── Table ──
pub struct Table { /* internal */ }
impl Table {
    pub fn new() -> Table;
    pub fn set_columns(self, columns: &[TableColumn]) -> Self;
    pub fn set_style(self, style: TableStyle) -> Self;
    pub fn set_total_row(self, enable: bool) -> Self;
    pub fn set_autofilter(self, enable: bool) -> Self;
}

// ── Image ──
pub struct Image { /* internal */ }
impl Image {
    pub fn new(path: impl AsRef<Path>) -> Result<Image, Error>;
    pub fn from_buffer(data: &[u8]) -> Result<Image, Error>;
    pub fn set_width(self, px: u32) -> Self;
    pub fn set_height(self, px: u32) -> Self;
}

// ── Conditional Formatting (trait-based like rust_xlsxwriter) ──
pub trait ConditionalFormat { /* internal */ }

pub struct ConditionalFormatCell { /* ... */ }
pub struct ConditionalFormat2ColorScale { /* ... */ }
pub struct ConditionalFormat3ColorScale { /* ... */ }
pub struct ConditionalFormatDataBar { /* ... */ }
pub struct ConditionalFormatIconSet { /* ... */ }

// ── Data Validation ──
pub struct DataValidation { /* internal */ }
impl DataValidation {
    pub fn new() -> Self;
    pub fn allow_list_strings(self, values: &[&str]) -> Result<Self, Error>;
    pub fn allow_whole_number(self, rule: ValidationRule<i32>) -> Self;
    pub fn allow_decimal(self, rule: ValidationRule<f64>) -> Self;
    pub fn allow_date(self, rule: ValidationRule<ExcelDateTime>) -> Self;
    pub fn allow_text_length(self, rule: ValidationRule<u32>) -> Self;
    pub fn allow_custom(self, formula: &str) -> Self;
    pub fn set_input_message(self, title: &str, message: &str) -> Self;
    pub fn set_error_message(self, title: &str, message: &str) -> Self;
}

// ── Sparkline ──
pub struct Sparkline { /* internal */ }
impl Sparkline {
    pub fn new() -> Self;
    pub fn set_range(self, range: &str) -> Self;
    pub fn set_type(self, sparkline_type: SparklineType) -> Self;
    pub fn set_color(self, color: impl IntoColor) -> Self;
}
```


---

## 3. Internal Architecture

### 3.1 Layer Diagram

```
┌─────────────────────────────────────────────────────────────┐
│                     PUBLIC API (lib.rs)                       │
│  Workbook, Worksheet, Format, Chart, Table, Image, etc.      │
├─────────────────────────────────────────────────────────────┤
│                     MODEL LAYER                              │
│  WorkbookModel, SheetData, CellStore, StyleRegistry,         │
│  SharedStringTable, FormatDedup, RelationshipManager         │
├──────────────────────┬──────────────────────────────────────┤
│    READER            │           WRITER                      │
│  XlsxReader          │  XlsxWriter                           │
│  SheetReader         │  SheetWriter                          │
│  (streaming cells)   │  (ordered XML gen)                    │
│  StyleParser         │  StyleWriter                          │
│  SstParser           │  SstWriter                            │
│  RelParser           │  RelWriter                            │
├──────────────────────┴──────────────────────────────────────┤
│                     XML LAYER                                │
│  XmlReader (quick-xml streaming + buffer reuse)              │
│  XmlWriter (Cursor<Vec<u8>> + write!() macros)               │
│  Attribute helpers, namespace handling, escaping              │
├─────────────────────────────────────────────────────────────┤
│                     ZIP LAYER                                │
│  ZipReader (zip crate, lazy decompression)                   │
│  ZipWriter (zip crate, deflate compression)                  │
│  Case-insensitive path cache                                 │
└─────────────────────────────────────────────────────────────┘
```

### 3.2 Workbook Modes

```rust
enum WorkbookMode {
    /// New file. Write-only. Uses direct XML generation.
    Create,
    /// Opened existing file. Read + write.
    /// Lazy-loads sheets. Unmodified sheets pass through as raw bytes.
    Edit {
        source_zip: ZipArchive<BufReader<File>>,
        zip_path_cache: HashMap<String, String>,
        raw_sheets: HashMap<usize, RawSheetData>,
    },
    /// Read-only. Streaming reads. No modification allowed.
    ReadOnly {
        source_zip: ZipArchive<BufReader<File>>,
        zip_path_cache: HashMap<String, String>,
    },
}
```

### 3.3 Cell Storage (Hybrid Design)

The key insight: reading and writing have different optimal storage.

```rust
/// Cell storage for a worksheet.
enum CellStore {
    /// For Create mode: ordered map for deterministic XML output.
    /// Inspired by rust_xlsxwriter.
    Ordered(BTreeMap<RowNum, BTreeMap<ColNum, CellData>>),

    /// For Edit mode: hash map for O(1) random access + dual sorted indices.
    /// Inspired by umya-spreadsheet.
    Indexed {
        cells: HashMap<(RowNum, ColNum), CellData>,
        row_order: BTreeSet<(RowNum, ColNum)>,  // for row-major iteration
        dirty: bool,  // track if any cells were modified
    },

    /// For ReadOnly mode: no storage. Cells are streamed on demand.
    Streaming,
}
```

### 3.4 Style System (Indexed + Dedup)

Combines the best of both approaches:
- **Public API**: Users work with `Format` objects (like rust_xlsxwriter)
- **Internal storage**: Formats are deduplicated via Hash+Eq and stored in a registry with indices (like xlsx's actual xf system)
- **Per-cell storage**: Only the xf index (u32), not the full format (unlike umya's per-cell Style clone)

```rust
struct StyleRegistry {
    fonts: Vec<FontData>,
    fills: Vec<FillData>,
    borders: Vec<BorderData>,
    num_formats: Vec<NumFormatData>,
    xf_records: Vec<XfRecord>,  // the cellXfs entries

    // Dedup maps (Hash → index)
    font_map: HashMap<FontData, usize>,
    fill_map: HashMap<FillData, usize>,
    border_map: HashMap<BorderData, usize>,
    xf_map: HashMap<XfRecord, u32>,
}

struct XfRecord {
    font_id: usize,
    fill_id: usize,
    border_id: usize,
    num_fmt_id: u16,
    alignment: Option<AlignmentData>,
    protection: Option<ProtectionData>,
}

/// What's stored per cell — just the value + format index.
struct CellData {
    value: CellType,
    xf_index: u32,  // index into StyleRegistry.xf_records
}

enum CellType {
    Empty,
    Number(f64),
    String(StringRef),     // index into SharedStringTable
    InlineString(String),  // for constant-memory mode
    Bool(bool),
    Formula { text: String, cached: Option<Box<CellType>> },
    DateTime(f64),         // serial date, distinguished by xf_index format
    Error(CellError),
    RichString(Vec<RichStringSegment>),
}
```

### 3.5 Shared String Table

```rust
struct SharedStringTable {
    strings: Vec<Arc<str>>,                    // index → string
    index_map: HashMap<Arc<str>, u32>,         // string → index (dedup)
    count: u32,                                // total references
    unique_count: u32,                         // unique strings
}

impl SharedStringTable {
    /// Get or insert a string, returning its index.
    fn intern(&mut self, s: &str) -> u32;

    /// Look up a string by index.
    fn get(&self, index: u32) -> Option<&str>;
}
```

### 3.6 Relationship Manager

```rust
struct RelationshipManager {
    next_id: u32,
    relationships: Vec<Relationship>,
}

struct Relationship {
    id: String,        // "rId1", "rId2", ...
    rel_type: String,  // the full URI
    target: String,    // relative path
}

impl RelationshipManager {
    fn add(&mut self, rel_type: &str, target: &str) -> String; // returns rId
    fn find_by_type(&self, rel_type: &str) -> Option<&Relationship>;
}
```

### 3.7 Raw Passthrough (Edit Mode)

When a workbook is opened for editing, sheets that are never accessed are stored as raw bytes and written back verbatim. This preserves features we don't yet support.

```rust
struct RawSheetData {
    worksheet_xml: Vec<u8>,
    relationships: Vec<u8>,
    associated_files: Vec<(String, Vec<u8>)>,  // (path, content)
}
```

A sheet transitions from raw → deserialized when:
- `worksheet()` or `worksheet_by_name()` is called (mutable access)
- Any read method is called on it

Unmodified deserialized sheets are re-serialized. Modified sheets are written from the model. Raw sheets are written as-is.


---

## 4. Module Breakdown

### 4.1 Crate Structure

```
zavora-xlsx/
├── Cargo.toml
├── src/
│   ├── lib.rs                  ← public re-exports, crate docs
│   │
│   ├── workbook.rs             ← Workbook struct, open/create/save
│   ├── worksheet.rs            ← Worksheet struct, read/write/format
│   ├── format.rs               ← Format builder, Color, enums
│   ├── cell.rs                 ← CellValue, CellType, IntoExcelData trait
│   ├── datetime.rs             ← ExcelDateTime, serial date conversion
│   ├── error.rs                ← Error enum, Result type alias
│   ├── utility.rs              ← A1 parsing, column name conversion
│   │
│   ├── model/
│   │   ├── mod.rs
│   │   ├── cell_store.rs       ← CellStore enum (Ordered/Indexed/Streaming)
│   │   ├── style_registry.rs   ← StyleRegistry, format dedup, xf records
│   │   ├── shared_strings.rs   ← SharedStringTable
│   │   ├── relationships.rs    ← RelationshipManager
│   │   ├── content_types.rs    ← ContentTypes manager
│   │   └── properties.rs       ← DocProperties
│   │
│   ├── reader/
│   │   ├── mod.rs
│   │   ├── xlsx_reader.rs      ← Top-level: open zip, parse workbook.xml
│   │   ├── sheet_reader.rs     ← Streaming cell reader (calamine-style)
│   │   ├── style_parser.rs     ← Parse styles.xml → StyleRegistry
│   │   ├── sst_parser.rs       ← Parse sharedStrings.xml → SharedStringTable
│   │   ├── rel_parser.rs       ← Parse .rels files
│   │   ├── drawing_reader.rs   ← Parse drawings, charts, images
│   │   └── table_reader.rs     ← Parse table definitions
│   │
│   ├── writer/
│   │   ├── mod.rs
│   │   ├── xlsx_writer.rs      ← Top-level: assemble zip, write all parts
│   │   ├── sheet_writer.rs     ← Write worksheet XML from CellStore
│   │   ├── style_writer.rs     ← Write styles.xml from StyleRegistry
│   │   ├── sst_writer.rs       ← Write sharedStrings.xml
│   │   ├── rel_writer.rs       ← Write .rels files
│   │   ├── drawing_writer.rs   ← Write drawings, chart refs, image refs
│   │   ├── chart_writer.rs     ← Write chart XML
│   │   ├── table_writer.rs     ← Write table XML
│   │   └── content_types_writer.rs
│   │
│   ├── xml/
│   │   ├── mod.rs
│   │   ├── xml_reader.rs       ← quick-xml wrapper, buffer management
│   │   └── xml_writer.rs       ← Cursor<Vec<u8>> writer, escape helpers
│   │
│   ├── zip/
│   │   ├── mod.rs
│   │   ├── zip_reader.rs       ← ZipArchive wrapper, path cache
│   │   └── zip_writer.rs       ← ZipWriter wrapper
│   │
│   └── features/
│       ├── mod.rs
│       ├── chart.rs            ← Chart, ChartSeries, ChartAxis, etc.
│       ├── table.rs            ← Table, TableColumn
│       ├── image.rs            ← Image (PNG/JPEG parsing, dimensions)
│       ├── conditional.rs      ← ConditionalFormat* types
│       ├── validation.rs       ← DataValidation
│       ├── sparkline.rs        ← Sparkline
│       ├── merge.rs            ← Merge cell tracking
│       ├── hyperlink.rs        ← Hyperlink support
│       ├── comment.rs          ← Cell comments
│       └── protection.rs       ← Sheet/workbook protection
│
├── tests/
│   ├── read_tests.rs           ← Round-trip read tests with fixture files
│   ├── write_tests.rs          ← Write + verify with calamine
│   ├── edit_tests.rs           ← Open → modify → save → verify
│   ├── format_tests.rs         ← Style dedup, format application
│   ├── chart_tests.rs
│   ├── table_tests.rs
│   ├── performance_tests.rs    ← Benchmarks
│   └── fixtures/               ← Test xlsx files
│
└── benches/
    ├── read_bench.rs           ← Compare with calamine
    └── write_bench.rs          ← Compare with rust_xlsxwriter
```

### 4.2 Key Module Responsibilities

| Module | Reads From | Writes To | Key Pattern |
|--------|-----------|-----------|-------------|
| `sheet_reader.rs` | `xl/worksheets/sheet*.xml` | `CellStore::Indexed` | Streaming XML, buffer reuse, SIMD parsing |
| `sheet_writer.rs` | `CellStore` | `xl/worksheets/sheet*.xml` | BTreeMap ordered iteration, `write!()` macros |
| `style_parser.rs` | `xl/styles.xml` | `StyleRegistry` | Two-pass: numFmts first, then cellXfs |
| `style_writer.rs` | `StyleRegistry` | `xl/styles.xml` | Indexed output matching Excel's expected order |
| `sst_parser.rs` | `xl/sharedStrings.xml` | `SharedStringTable` | Pre-allocate from uniqueCount, handle rich text |
| `sst_writer.rs` | `SharedStringTable` | `xl/sharedStrings.xml` | Sequential write of all unique strings |
| `chart_writer.rs` | `Chart` model | `xl/charts/chart*.xml` | One XML file per chart, relationship linking |
| `xlsx_writer.rs` | All models | ZIP archive | Orchestrates file order, parallel sheet assembly |


---

## 5. Performance Strategy

### 5.1 Techniques Adopted from Reference Engines

| Technique | Source | Where Applied |
|-----------|--------|---------------|
| Streaming XML (no DOM) | calamine | All reading |
| Buffer reuse (`Vec::clear()` not reallocate) | calamine | `sheet_reader.rs`, `sst_parser.rs` |
| SIMD numeric parsing (`atoi_simd`, `fast_float2`) | calamine | Cell value parsing |
| Zero-copy shared strings (`&str` borrows) | calamine | Read-only mode cell iteration |
| Pre-allocation from XML hints | calamine | SST (`uniqueCount`), sheet (`<dimension>`) |
| Case-insensitive zip path cache | calamine | `zip_reader.rs` |
| Relaxed XML parser config | calamine | All XML reading |
| Parallel worksheet assembly | rust_xlsxwriter | `xlsx_writer.rs` via `thread::scope` |
| Format dedup via Hash+Eq | rust_xlsxwriter | `style_registry.rs` |
| `Cursor<Vec<u8>>` XML buffers | rust_xlsxwriter | All XML writing |
| `write!()` macros for hot paths | rust_xlsxwriter | Cell XML generation |
| Lazy sheet deserialization | umya-spreadsheet | Edit mode |
| Raw passthrough for unmodified sheets | umya-spreadsheet | Edit mode save |
| Constant-memory write mode | rust_xlsxwriter | Optional flush-as-you-go |

### 5.2 New Optimizations

1. **Columnar read mode**: For analytics use cases, read a single column without parsing entire rows. Skip cells outside the target column range during streaming.

2. **Memory-mapped zip**: For very large files, use `memmap2` to memory-map the zip file instead of buffered reads. The zip central directory is already in memory; this extends to entry data.

3. **String interning pool**: In edit mode, use a global string pool (`Arc<str>`) so identical strings across sheets share one allocation.

4. **Batch format application**: When applying the same format to a range, compute the xf_index once and apply it to all cells, rather than dedup-checking per cell.

5. **Sparse row output**: When writing, skip entirely empty rows in `<sheetData>` rather than emitting empty `<row>` elements.

### 5.3 Benchmarking Targets

| Operation | Target | Baseline (current best) |
|-----------|--------|------------------------|
| Read 100K rows × 10 cols | < 200ms | calamine ~150ms |
| Write 100K rows × 10 cols | < 500ms | rust_xlsxwriter ~400ms |
| Open → read 1 cell → close | < 50ms | calamine ~30ms |
| Open → edit 1 cell → save | < 1s | umya ~800ms |
| Format dedup (10K unique) | < 10ms | rust_xlsxwriter ~5ms |

---

## 6. Implementation Phases

### Phase 1: Core Read + Write (MVP)
**Goal**: Read and create xlsx files with cell data and basic formatting.

- [ ] Zip layer (read + write)
- [ ] XML layer (streaming reader + buffer writer)
- [ ] Shared string table (parse + generate)
- [ ] Style registry (parse + generate, basic: font, fill, border, number format)
- [ ] Cell reading (streaming, all types: string, number, bool, formula, date, error)
- [ ] Cell writing (all types, ordered output)
- [ ] Workbook.new() + save()
- [ ] Workbook.open_readonly() + read cells
- [ ] Format builder (bold, italic, font size/color, background, borders, number format, alignment)
- [ ] Merge cells
- [ ] Column width, row height, freeze panes
- [ ] A1 notation parsing + conversion
- [ ] Date detection and ExcelDateTime
- [ ] Content types + relationships (read + write)
- [ ] Integration tests: write → read round-trip

### Phase 2: Edit Mode
**Goal**: Open existing files, modify, save with fidelity.

- [ ] Workbook.open() (edit mode)
- [ ] Lazy sheet deserialization
- [ ] Raw passthrough for unmodified sheets
- [ ] CellStore::Indexed with dirty tracking
- [ ] Insert/remove rows and columns
- [ ] Formula coordinate adjustment
- [ ] Sheet management (add, remove, rename, reorder)
- [ ] Defined names
- [ ] Document properties
- [ ] Integration tests: open → edit → save → verify

### Phase 3: Rich Features
**Goal**: Charts, tables, images, conditional formatting.

- [ ] Charts (bar, column, line, pie, scatter, area, doughnut, radar)
- [ ] Chart series, axes, titles, legends
- [ ] Tables (columns, styles, autofilter, totals row)
- [ ] Images (PNG, JPEG — parse dimensions, embed in zip)
- [ ] Drawing anchors (one-cell, two-cell)
- [ ] Conditional formatting (cell value, 2/3-color scale, data bar, icon set)
- [ ] Data validation (list, number, date, text length, custom formula)
- [ ] Sparklines
- [ ] Hyperlinks
- [ ] Comments

### Phase 4: Advanced + Polish
**Goal**: Production-ready, feature-complete.

- [ ] Constant-memory write mode (flush-as-you-go)
- [ ] Parallel worksheet assembly
- [ ] Autofit column widths
- [ ] Rich text (multiple fonts in one cell)
- [ ] Sheet protection with password
- [ ] Workbook protection
- [ ] Print settings (page setup, margins, headers/footers, page breaks)
- [ ] VBA macro preservation (read → passthrough on save)
- [ ] Password-protected file detection
- [ ] xlsm format support
- [ ] Comprehensive error messages
- [ ] Documentation + examples
- [ ] Publish to crates.io

---

## 7. Dependencies

```toml
[package]
name = "zavora-xlsx"
version = "0.1.0"
edition = "2021"
license = "MIT OR Apache-2.0"
description = "High-performance Excel xlsx reader, writer, and editor"
repository = "https://github.com/user/zavora-xlsx"
keywords = ["excel", "xlsx", "spreadsheet", "ooxml"]
categories = ["parser-implementations", "encoding"]

[dependencies]
quick-xml = { version = "0.37", features = ["encoding"] }
zip = { version = "2", default-features = false, features = ["deflate"] }
atoi_simd = "0.16"
fast_float2 = "0.2"

[dev-dependencies]
calamine = "0.34"          # for verification in tests
tempfile = "3"
criterion = "0.5"

[[bench]]
name = "read_bench"
harness = false

[[bench]]
name = "write_bench"
harness = false
```

Minimal dependency philosophy:
- `quick-xml` — streaming XML (the only sane choice for Rust)
- `zip` — zip archive handling (mature, well-maintained)
- `atoi_simd` + `fast_float2` — SIMD-accelerated numeric parsing (calamine's secret sauce)
- No `serde` in the core crate (avoid the compile time tax; add as optional feature later)
- No `chrono` or `time` in core (ExcelDateTime is self-contained; add interop as optional features)

Optional features (future):
```toml
[features]
default = []
serde = ["dep:serde"]       # Serialize/Deserialize for CellValue, Format, etc.
chrono = ["dep:chrono"]     # ExcelDateTime ↔ chrono::NaiveDateTime conversion
time = ["dep:time"]         # ExcelDateTime ↔ time::OffsetDateTime conversion
```

---

## 8. Design Decisions Log

### D1: 0-based (row, col) coordinates
**Decision**: Use 0-based `(row: u32, col: u16)` everywhere.
**Rationale**: Matches rust_xlsxwriter, natural for Rust array indexing, avoids umya's confusing (col, row) order. `u16` for columns because Excel max is 16384 (fits u16). `u32` for rows because Excel max is 1,048,576.
**Tradeoff**: Users must remember to subtract 1 from Excel's 1-based display. Mitigated by providing A1 notation helpers.

### D2: Runtime mode checking, not compile-time
**Decision**: One `Workbook` type with runtime mode (Create/Edit/ReadOnly).
**Rationale**: Simpler API. Users don't need to learn `ReadOnlyWorkbook` vs `EditableWorkbook` vs `NewWorkbook`. Attempting an unsupported operation returns `Err(Error::ReadOnly)`.
**Tradeoff**: Errors caught at runtime instead of compile time. Acceptable because the mode is set once at open time and rarely changes.

### D3: Format dedup with xf_index per cell
**Decision**: Store only a `u32` xf_index per cell, not a full Style object.
**Rationale**: umya's per-cell `Box<Style>` wastes memory (each Style is ~200 bytes). With 1M cells, that's 200MB just for styles. An xf_index is 4 bytes. The StyleRegistry deduplicates identical formats.
**Tradeoff**: Reading a cell's format requires a registry lookup. Acceptable — it's O(1).

### D4: Lazy deserialization with raw passthrough
**Decision**: In edit mode, store raw XML bytes for sheets until accessed. Write unmodified sheets as raw bytes.
**Rationale**: umya's best feature. Opening a 50-sheet workbook to edit 1 sheet shouldn't parse all 50. And saving shouldn't re-serialize 49 unchanged sheets (which could lose unsupported features).
**Tradeoff**: More complex save logic (must handle both raw and model paths).

### D5: Streaming reader for read-only mode
**Decision**: In read-only mode, cells are streamed via an iterator, not loaded into memory.
**Rationale**: calamine's approach. Reading a 1GB file shouldn't require 1GB of RAM.
**Tradeoff**: Can't random-access cells in read-only mode (must iterate). Users who need random access should use edit mode.

### D6: No serde in core
**Decision**: Core crate has zero serde dependency. Available as optional feature.
**Rationale**: serde adds ~15s to clean compile time. Most xlsx operations don't need serialization. Users who need it can opt in.
**Tradeoff**: Can't derive Serialize/Deserialize on public types by default.

### D7: Self-contained ExcelDateTime
**Decision**: Own datetime type with conversion methods, no chrono/time dependency.
**Rationale**: Excel dates are just f64 serial numbers. The conversion math is ~50 lines. No need to pull in a full datetime library. Optional features provide interop.
**Tradeoff**: Users must convert to/from their preferred datetime library. Mitigated by optional feature flags.

### D8: Hybrid cell storage
**Decision**: Different CellStore backends for different modes (BTreeMap for create, HashMap+BTreeSet for edit, streaming for read-only).
**Rationale**: No single data structure is optimal for all three modes. BTreeMap gives ordered iteration for writing. HashMap gives O(1) access for editing. Streaming gives minimal memory for reading.
**Tradeoff**: More internal complexity. But the CellStore enum encapsulates it — the Worksheet API is the same regardless of backend.

### D9: Build our own, don't wrap
**Decision**: Implement xlsx reading/writing from scratch using quick-xml and zip, rather than wrapping calamine/rust_xlsxwriter/umya.
**Rationale**: Wrapping creates the same multi-engine problems the MCP server had. A unified implementation means one coordinate system, one style model, one error type. We study the reference engines' approaches and implement our own versions.
**Tradeoff**: Much more initial work. But the result is a coherent, maintainable codebase.
