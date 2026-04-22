# zavora-xlsx Roadmap

A prioritized roadmap for reaching feature parity with the Excel OOXML universe.
Items are grouped by theme and ordered by user impact.

## What's Implemented (v0.1.0)

| Area | Status |
|------|--------|
| Create / save / edit workbooks | ✅ |
| Cell types: string, number, bool, formula, date, rich text, error | ✅ |
| Formatting: font, fill, borders (incl. diagonal), alignment, number format, patterns, lock/unlock | ✅ |
| Charts: 9 types, combo, secondary axis, data labels, trendlines, data table, pivot charts | ✅ |
| Tables with autofilter, total row, 20+ styles | ✅ |
| Conditional formatting: 11 rule types with DXF | ✅ |
| Data validation: 7 rule types | ✅ |
| Images: PNG + JPEG with scaling | ✅ |
| Sparklines: line, column, win/loss | ✅ |
| Pivot tables with calculated fields, layout modes, styles | ✅ |
| Treemap charts (ChartEx) | ✅ |
| Print settings, page breaks, repeat rows/cols, headers/footers | ✅ |
| Sheet/workbook protection with password | ✅ |
| Insert/remove rows & columns with formula shift | ✅ |
| Comments, hyperlinks, merge cells | ✅ |
| Row/column grouping (outline), hidden rows/cols | ✅ |
| Freeze panes, zoom, gridlines, headings, RTL, tab color | ✅ |
| Defined names, document properties | ✅ |
| Streaming write mode (constant memory) | ✅ |
| Edit mode with VBA/macro passthrough | ✅ |
| Read: cells, formulas, merges, widths, heights, freeze, visibility, images | ✅ |
| Autofit column widths | ✅ |

---

## Phase 1 — Read Parity

Reading is currently cell-focused. These additions make the reader useful for analytics and migration tools.

| # | Feature | Description |
|---|---------|-------------|
| 1.1 | Read formatting | Parse xf index back to font, fill, border, alignment, number format |
| 1.2 | Read conditional formatting | Parse CF rules from sheet XML |
| 1.3 | Read data validation | Parse validation rules from sheet XML |
| 1.4 | Read charts | Parse chart XML into `Chart` structs |
| 1.5 | Read tables | Parse table parts into `Table` structs |
| 1.6 | Read hyperlinks | Parse hyperlink relationships |
| 1.7 | Read comments / notes | Parse comment XML (legacy + threaded) |
| 1.8 | Read sparklines | Parse sparkline groups from extLst |
| 1.9 | Read defined names with scope | Sheet-scoped vs workbook-scoped names |
| 1.10 | Read print settings | Margins, orientation, headers/footers, page breaks |
| 1.11 | Read sheet protection | Parse protection attributes |
| 1.12 | Read row/column grouping | Outline levels from row/col elements |

## Phase 2 — Formula Engine

Currently formulas are stored as strings. A calculation engine unlocks offline report generation.

| # | Feature | Description |
|---|---------|-------------|
| 2.1 | Formula parser | Tokenize Excel formula syntax (A1, R1C1, structured refs) |
| 2.2 | Dependency graph | Build cell dependency DAG for calculation order |
| 2.3 | Core functions | SUM, AVERAGE, COUNT, MIN, MAX, IF, VLOOKUP, INDEX/MATCH |
| 2.4 | Math functions | ROUND, ABS, MOD, POWER, SQRT, LOG, LN, EXP, PI |
| 2.5 | Text functions | CONCATENATE, LEFT, RIGHT, MID, LEN, TRIM, UPPER, LOWER, SUBSTITUTE |
| 2.6 | Date functions | TODAY, NOW, DATE, YEAR, MONTH, DAY, EDATE, EOMONTH, NETWORKDAYS |
| 2.7 | Lookup functions | HLOOKUP, XLOOKUP, MATCH, INDIRECT, OFFSET |
| 2.8 | Statistical functions | STDEV, VAR, MEDIAN, PERCENTILE, RANK, COUNTIF, SUMIF |
| 2.9 | Logical functions | AND, OR, NOT, IFERROR, IFNA, SWITCH, IFS |
| 2.10 | Array formula evaluation | CSE and dynamic array spill |
| 2.11 | Circular reference detection | Detect and report cycles |
| 2.12 | Volatile function handling | RAND, NOW, TODAY, INDIRECT recalc behavior |

## Phase 3 — Chart Completeness

| # | Feature | Description |
|---|---------|-------------|
| 3.1 | Bubble charts | XY with bubble size dimension |
| 3.2 | Waterfall charts | ChartEx waterfall (Excel 2016+) |
| 3.3 | Funnel charts | ChartEx funnel (Excel 2016+) |
| 3.4 | Sunburst charts | ChartEx hierarchical (Excel 2016+) |
| 3.5 | Histogram / Pareto | ChartEx statistical charts |
| 3.6 | Box & Whisker | ChartEx statistical charts |
| 3.7 | Map charts | Geographic data visualization |
| 3.8 | 3D chart variants | 3D column, bar, line, pie, area |
| 3.9 | Surface charts | 3D surface and wireframe |
| 3.10 | Chart style themes | Apply built-in chart styles (Style 1–48) |
| 3.11 | Axis formatting | Number format, font, tick marks, gridline styles |
| 3.12 | Plot area formatting | Fill, border, gradient for plot area |
| 3.13 | Series formatting | Line width, dash style, fill patterns, gradient fills |
| 3.14 | Error bars | Standard error, percentage, fixed value, custom |
| 3.15 | Drop lines / high-low lines | Line chart accessories |
| 3.16 | Chart on own sheet | Chart sheets (not embedded in worksheet) |

## Phase 4 — Advanced Formatting

| # | Feature | Description |
|---|---------|-------------|
| 4.1 | Gradient fills | Two-color and multi-stop gradients |
| 4.2 | Theme colors | Access theme color palette (accent1–6, dk1, lt1, etc.) |
| 4.3 | Conditional format: gradient data bars | Gradient fill data bars vs solid |
| 4.4 | Cell styles | Named styles (Normal, Heading 1, Currency, etc.) |
| 4.5 | Table style customization | Custom table styles beyond built-in |
| 4.6 | Phonetic text (furigana) | East Asian phonetic guide runs |
| 4.7 | Text effects | Shadow, outline, emboss, engrave on font |

## Phase 5 — Data & Interactivity

| # | Feature | Description |
|---|---------|-------------|
| 5.1 | Slicer | Visual filter controls for tables and pivot tables |
| 5.2 | Timeline | Date-based slicer for pivot tables |
| 5.3 | Named ranges CRUD | Create, update, delete named ranges programmatically |
| 5.4 | External data connections | Query tables, ODBC/OLEDB connection strings |
| 5.5 | Power Query (M) | Preserve/roundtrip Power Query definitions |
| 5.6 | Pivot table cache from external | Pivot cache from external data source |
| 5.7 | Pivot table grouping | Date grouping, numeric ranges |
| 5.8 | Pivot table calculated items | Calculated items (not just fields) |
| 5.9 | Sort state | Preserve/set sort order on columns |
| 5.10 | Advanced autofilter | Custom filters (top 10, date filters, color filters) |

## Phase 6 — Streaming & Performance

| # | Feature | Description |
|---|---------|-------------|
| 6.1 | Streaming read | SAX-style row-by-row reader for large files |
| 6.2 | Streaming write: charts/images | Support charts and images in streaming mode |
| 6.3 | Streaming write: conditional formatting | CF rules in streaming mode |
| 6.4 | Parallel sheet writing | Write multiple sheets concurrently |
| 6.5 | Memory-mapped reading | mmap for large file random access |
| 6.6 | Incremental save | Only rewrite dirty sheets in edit mode |
| 6.7 | Shared string deduplication tuning | Threshold for inline vs shared strings |

## Phase 7 — File Format Variants

| # | Feature | Description |
|---|---------|-------------|
| 7.1 | XLSM write | Save as macro-enabled workbook (not just passthrough) |
| 7.2 | XLTX / XLTM | Template file format support |
| 7.3 | XLSB read | Binary Excel format reading |
| 7.4 | XLS read (BIFF8) | Legacy .xls format reading |
| 7.5 | CSV / TSV export | Export sheets to delimited text |
| 7.6 | ODS read/write | OpenDocument Spreadsheet interop |
| 7.7 | Strict OOXML | ISO 29500 strict namespace support |

## Phase 8 — Document Features

| # | Feature | Description |
|---|---------|-------------|
| 8.1 | Threaded comments | Modern Excel threaded comment model |
| 8.2 | Form controls | Checkboxes, dropdowns, buttons, spinners |
| 8.3 | ActiveX controls | Embedded ActiveX objects |
| 8.4 | OLE objects | Embedded documents (Word, PDF, etc.) |
| 8.5 | Drawing shapes | Rectangles, arrows, callouts, text boxes |
| 8.6 | SmartArt | Preserve/roundtrip SmartArt diagrams |
| 8.7 | Equation objects | OMML equation preservation |
| 8.8 | Digital signatures | Sign and verify workbook signatures |
| 8.9 | Custom XML parts | Custom XML data bindings |
| 8.10 | Metadata / custom properties | Custom document properties |

## Phase 9 — Security & Compliance

| # | Feature | Description |
|---|---------|-------------|
| 9.1 | File encryption | Read/write password-encrypted xlsx (Standard/Agile) |
| 9.2 | Sheet protection granularity | Per-feature protection flags (sort, filter, pivot, etc.) |
| 9.3 | VBA project signing | Sign VBA macros |
| 9.4 | Information Rights Management | IRM metadata preservation |
| 9.5 | Accessibility metadata | Alt text for charts, images, tables |

## Phase 10 — Ecosystem

| # | Feature | Description |
|---|---------|-------------|
| 10.1 | `serde` integration | Serialize/deserialize structs to/from rows |
| 10.2 | `#[derive(ExcelRow)]` | Proc macro for typed row mapping |
| 10.3 | Async I/O | `tokio::fs` / `async-std` file operations |
| 10.4 | WASM target | Compile to WebAssembly for browser use |
| 10.5 | Python bindings (PyO3) | `pip install zavora-xlsx` |
| 10.6 | Node.js bindings (napi-rs) | `npm install zavora-xlsx` |
| 10.7 | C FFI | C-compatible API for cross-language use |
| 10.8 | CLI tool | Command-line xlsx inspection and conversion |

---

## Priority Recommendation

For a crate launch, the highest-impact items are:

1. **Phase 1 (Read Parity)** — Most users need to read existing files fully, not just cell values
2. **Phase 10.1–10.2 (serde)** — Ergonomic typed access is the #1 feature request in the Rust xlsx ecosystem
3. **Phase 6.1 (Streaming read)** — Large file support for the read path
4. **Phase 3.1–3.6 (Chart completeness)** — Modern Excel chart types
5. **Phase 9.1 (Encryption)** — Many enterprise files are encrypted

Everything else is incremental value on a solid foundation.
